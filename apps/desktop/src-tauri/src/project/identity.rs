use std::{
    ffi::OsString,
    fs, io,
    os::unix::{ffi::OsStringExt, fs::MetadataExt},
    path::{Component, Path, PathBuf},
};

use super::types::{DirectoryAccessibilityState, GitSummary};

const MAX_PATH_BYTES: usize = 4096;
const MAX_GIT_POINTER_BYTES: u64 = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitIdentity {
    pub worktree_root: PathBuf,
    pub common_dir: PathBuf,
    pub is_linked_worktree: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectoryIdentity {
    pub selected_path: PathBuf,
    pub resolved_path: PathBuf,
    pub selected_display_path: String,
    pub resolved_display_path: String,
    pub accessibility: DirectoryAccessibilityState,
    pub device_id: u64,
    pub inode: u64,
    pub filesystem_type: Option<String>,
    pub mount_id: Option<u64>,
    pub git: Option<GitIdentity>,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

impl DirectoryIdentity {
    pub(crate) fn git_summary(&self) -> GitSummary {
        GitSummary {
            is_repository: self.git.is_some(),
            is_linked_worktree: self.git.as_ref().is_some_and(|git| git.is_linked_worktree),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DirectoryInspectionError {
    pub state: DirectoryAccessibilityState,
}

pub(crate) fn inspect_directory(
    selected_path: &Path,
) -> Result<DirectoryIdentity, DirectoryInspectionError> {
    validate_selected_path(selected_path)?;

    fs::symlink_metadata(selected_path).map_err(map_io_error)?;
    let resolved_path = fs::canonicalize(selected_path).map_err(map_io_error)?;
    validate_selected_path(&resolved_path)?;
    let metadata = fs::metadata(&resolved_path).map_err(map_io_error)?;
    if !metadata.is_dir() {
        return Err(DirectoryInspectionError {
            state: DirectoryAccessibilityState::MissingOrMoved,
        });
    }

    let git = match inspect_git_identity(&resolved_path) {
        Ok(git) => git,
        Err(()) => {
            return Err(DirectoryInspectionError {
                state: DirectoryAccessibilityState::GitInvalid,
            });
        }
    };
    let mount = find_mount_identity(&resolved_path);
    let accessibility = if mount.is_none() {
        DirectoryAccessibilityState::VerificationUnknown
    } else if metadata.mode() & 0o222 == 0 || mount.as_ref().is_some_and(|mount| mount.read_only) {
        DirectoryAccessibilityState::ConnectedReadOnly
    } else {
        DirectoryAccessibilityState::ConnectedAccessible
    };

    Ok(DirectoryIdentity {
        selected_path: selected_path.to_path_buf(),
        resolved_path: resolved_path.clone(),
        selected_display_path: display_path(selected_path),
        resolved_display_path: display_path(&resolved_path),
        accessibility,
        device_id: metadata.dev(),
        inode: metadata.ino(),
        filesystem_type: mount.as_ref().map(|mount| mount.filesystem_type.clone()),
        mount_id: mount.as_ref().map(|mount| mount.id),
        git,
        has_agents_guidance: resolved_path.join("AGENTS.md").is_file(),
        has_codex_config: resolved_path.join(".codex").is_dir(),
    })
}

pub(crate) fn display_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if path == home {
            return "~".to_owned();
        }
        if let Ok(relative) = path.strip_prefix(&home) {
            return format!("~/{}", relative.to_string_lossy());
        }
    }
    path.to_string_lossy().into_owned()
}

pub(crate) fn disconnected_state(filesystem_type: Option<&str>) -> DirectoryAccessibilityState {
    let filesystem = filesystem_type.unwrap_or_default().to_ascii_lowercase();
    if ["nfs", "nfs4", "cifs", "smb3", "sshfs"]
        .iter()
        .any(|value| filesystem == *value)
        || filesystem.starts_with("fuse.sshfs")
    {
        DirectoryAccessibilityState::NetworkUnavailable
    } else if ["vfat", "exfat", "ntfs", "iso9660", "udf"]
        .iter()
        .any(|value| filesystem == *value)
    {
        DirectoryAccessibilityState::RemovableDisconnected
    } else {
        DirectoryAccessibilityState::MissingOrMoved
    }
}

fn validate_selected_path(path: &Path) -> Result<(), DirectoryInspectionError> {
    let valid_components = path
        .components()
        .all(|component| matches!(component, Component::RootDir | Component::Normal(_)));
    let path_text = path
        .to_str()
        .filter(|value| value.len() <= MAX_PATH_BYTES && !value.chars().any(char::is_control));
    if !path.is_absolute() || !valid_components || path_text.is_none() {
        return Err(DirectoryInspectionError {
            state: DirectoryAccessibilityState::VerificationUnknown,
        });
    }
    Ok(())
}

fn map_io_error(error: io::Error) -> DirectoryInspectionError {
    let state = match error.kind() {
        io::ErrorKind::NotFound => DirectoryAccessibilityState::MissingOrMoved,
        io::ErrorKind::PermissionDenied => DirectoryAccessibilityState::PermissionDenied,
        _ => DirectoryAccessibilityState::VerificationUnknown,
    };
    DirectoryInspectionError { state }
}

fn inspect_git_identity(root: &Path) -> Result<Option<GitIdentity>, ()> {
    for ancestor in root.ancestors() {
        let marker = ancestor.join(".git");
        let marker_metadata = match fs::symlink_metadata(&marker) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return Err(()),
        };

        if marker_metadata.is_dir() || marker.is_dir() {
            return Ok(Some(GitIdentity {
                worktree_root: fs::canonicalize(ancestor).map_err(|_| ())?,
                common_dir: fs::canonicalize(marker).map_err(|_| ())?,
                is_linked_worktree: false,
            }));
        }
        if !marker_metadata.is_file() {
            return Err(());
        }

        let pointer = read_small_text_file(&marker)?;
        let git_dir_value = pointer.strip_prefix("gitdir: ").ok_or(())?.trim();
        if git_dir_value.is_empty() || git_dir_value.chars().any(char::is_control) {
            return Err(());
        }
        let git_dir = Path::new(git_dir_value);
        let git_dir = if git_dir.is_absolute() {
            git_dir.to_path_buf()
        } else {
            ancestor.join(git_dir)
        };
        let git_dir = fs::canonicalize(git_dir).map_err(|_| ())?;
        if !git_dir.is_dir() {
            return Err(());
        }

        let common_pointer = git_dir.join("commondir");
        let common_dir = if common_pointer.is_file() {
            let value = read_small_text_file(&common_pointer)?;
            let value = value.trim();
            if value.is_empty() || value.chars().any(char::is_control) {
                return Err(());
            }
            let path = Path::new(value);
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                git_dir.join(path)
            };
            fs::canonicalize(path).map_err(|_| ())?
        } else {
            git_dir.clone()
        };

        return Ok(Some(GitIdentity {
            worktree_root: fs::canonicalize(ancestor).map_err(|_| ())?,
            common_dir,
            is_linked_worktree: true,
        }));
    }
    Ok(None)
}

fn read_small_text_file(path: &Path) -> Result<String, ()> {
    let metadata = fs::metadata(path).map_err(|_| ())?;
    if metadata.len() > MAX_GIT_POINTER_BYTES {
        return Err(());
    }
    fs::read_to_string(path).map_err(|_| ())
}

#[derive(Clone, Debug)]
struct MountIdentity {
    id: u64,
    mount_point: PathBuf,
    filesystem_type: String,
    read_only: bool,
}

fn find_mount_identity(path: &Path) -> Option<MountIdentity> {
    let mount_info = fs::read_to_string("/proc/self/mountinfo").ok()?;
    mount_info
        .lines()
        .filter_map(parse_mount_line)
        .filter(|mount| path == mount.mount_point || path.starts_with(&mount.mount_point))
        .max_by_key(|mount| mount.mount_point.as_os_str().len())
}

fn parse_mount_line(line: &str) -> Option<MountIdentity> {
    let (left, right) = line.split_once(" - ")?;
    let left_fields: Vec<_> = left.split_ascii_whitespace().collect();
    let right_fields: Vec<_> = right.split_ascii_whitespace().collect();
    Some(MountIdentity {
        id: left_fields.first()?.parse().ok()?,
        mount_point: PathBuf::from(OsString::from_vec(decode_mount_field(left_fields.get(4)?)?)),
        filesystem_type: right_fields.first()?.to_string(),
        read_only: mount_options_are_read_only(left_fields.get(5)?)
            || mount_options_are_read_only(right_fields.get(2)?),
    })
}

fn mount_options_are_read_only(options: &str) -> bool {
    options.split(',').any(|option| option == "ro")
}

fn decode_mount_field(value: &str) -> Option<Vec<u8>> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            let octal = bytes.get(index + 1..index + 4)?;
            if !octal.iter().all(|byte| matches!(byte, b'0'..=b'7')) {
                return None;
            }
            let value = (octal[0] - b'0') * 64 + (octal[1] - b'0') * 8 + (octal[2] - b'0');
            decoded.push(value);
            index += 4;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        fs,
        os::unix::{ffi::OsStringExt, fs::symlink},
    };

    use uuid::Uuid;

    use super::{
        decode_mount_field, inspect_directory, parse_mount_line, DirectoryAccessibilityState,
    };

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temporary directory must be created");
        path
    }

    #[test]
    fn preserves_selected_symlink_and_resolves_the_execution_target() {
        let root = temporary_directory("identity");
        let target = root.join("target");
        let selected = root.join("selected");
        fs::create_dir(&target).expect("target must be created");
        symlink(&target, &selected).expect("symlink must be created");

        let identity = inspect_directory(&selected).expect("symlink must be inspectable");

        assert_eq!(identity.selected_path, selected);
        assert_eq!(identity.resolved_path, target);
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn rejects_a_resolved_path_that_cannot_be_stored_exactly() {
        let root = temporary_directory("non-utf8-target");
        let target = root.join(OsString::from_vec(b"target-\xff".to_vec()));
        let selected = root.join("selected");
        fs::create_dir(&target).expect("non-UTF-8 target must be created");
        symlink(&target, &selected).expect("symlink must be created");

        let error = inspect_directory(&selected)
            .expect_err("a lossy resolved path must not enter exact metadata");

        assert_eq!(
            error.state,
            DirectoryAccessibilityState::VerificationUnknown
        );
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn decodes_mountinfo_escapes_without_exposing_mount_sources() {
        assert_eq!(
            decode_mount_field("/media/a\\040space\\134name"),
            Some(b"/media/a space\\name".to_vec())
        );
    }

    #[test]
    fn recognizes_read_only_mount_options() {
        let mount = parse_mount_line(
            "36 25 0:32 / /mnt/project ro,nosuid - ext4 /dev/example rw,errors=remount-ro",
        )
        .expect("mount fixture must parse");

        assert!(mount.read_only);
        assert_eq!(mount.filesystem_type, "ext4");
    }

    #[test]
    fn recognizes_linked_worktree_identity_without_running_git() {
        let root = temporary_directory("worktree");
        let worktree = root.join("worktree");
        let common = root.join("repository.git");
        let git_dir = common.join("worktrees/worktree");
        fs::create_dir(&worktree).expect("worktree must be created");
        fs::create_dir_all(&git_dir).expect("linked git directory must be created");
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", git_dir.display()),
        )
        .expect("git pointer must be written");
        fs::write(git_dir.join("commondir"), "../..\n")
            .expect("common directory pointer must be written");

        let identity = inspect_directory(&worktree).expect("worktree must be inspectable");
        let git = identity.git.expect("worktree must have git identity");

        assert!(git.is_linked_worktree);
        assert_eq!(git.worktree_root, worktree);
        assert_eq!(git.common_dir, common);
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn rejects_an_invalid_git_pointer() {
        let directory = temporary_directory("invalid-git");
        fs::write(directory.join(".git"), "not-a-git-pointer\n")
            .expect("invalid pointer must be written");

        let error = inspect_directory(&directory).expect_err("invalid git metadata must fail");

        assert_eq!(error.state, DirectoryAccessibilityState::GitInvalid);
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }
}
