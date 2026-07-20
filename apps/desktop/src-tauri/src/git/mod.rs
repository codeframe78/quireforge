mod mutation;
pub mod types;

use std::{
    path::{Component, Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use tokio::{io::AsyncReadExt, process::Command, time::timeout};

use crate::project::{ProjectExecutionError, ProjectReviewRoot, ProjectService};
use mutation::MutationCoordinator;
use types::{
    GitBranchSummary, GitChangeKind, GitDiagnosticCode, GitDiffArea, GitDiffKind, GitDiffLine,
    GitDiffLineKind, GitDiffRequest, GitDiffSnapshot, GitDiffState, GitFileChange,
    GitMutationConfirmRequest, GitMutationPreviewRequest, GitMutationPreviewSnapshot,
    GitMutationResultSnapshot, GitOpenFileRequest, GitRecoveryRequest, GitWorkspaceSnapshot,
    GitWorkspaceState, GIT_SCHEMA_VERSION,
};

const GIT_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_STATUS_BYTES: usize = 512 * 1024;
const MAX_DIFF_BYTES: usize = 256 * 1024;
const MAX_STDERR_BYTES: usize = 8 * 1024;
const MAX_CHANGES: usize = 512;
const MAX_DIFF_LINES: usize = 1_500;
const MAX_LINE_CHARACTERS: usize = 4_096;

#[derive(Default)]
pub struct GitService {
    mutations: MutationCoordinator,
}

#[derive(Debug)]
enum GitRunError {
    Unavailable,
    Failed,
    TooLarge,
    TimedOut,
}

struct GitOutput {
    stdout: Vec<u8>,
    success: bool,
    code: Option<i32>,
}

impl GitService {
    pub async fn status(
        &self,
        project_id: String,
        projects: &ProjectService,
    ) -> GitWorkspaceSnapshot {
        let root = match projects.review_root(&project_id) {
            Ok(root) => root,
            Err(error) => {
                return GitWorkspaceSnapshot::unavailable(
                    Some(project_id),
                    map_project_error(error),
                );
            }
        };
        workspace_from_root(project_id, &root).await
    }

    pub async fn diff(
        &self,
        request: GitDiffRequest,
        projects: &ProjectService,
    ) -> GitDiffSnapshot {
        if !valid_relative_path(&request.path) {
            return GitDiffSnapshot::unavailable(request, GitDiagnosticCode::InvalidPath);
        }
        let root = match projects.review_root(&request.project_id) {
            Ok(root) => root,
            Err(error) => {
                return GitDiffSnapshot::unavailable(request, map_project_error(error));
            }
        };
        let status = match inspect_status(&root.attached_root, &root.worktree_root).await {
            Ok(status) => status,
            Err(error) => {
                return GitDiffSnapshot::unavailable(
                    request,
                    map_run_error(error, GitDiagnosticCode::GitFailed),
                );
            }
        };
        let Some(change) = status.1.iter().find(|change| change.path == request.path) else {
            return GitDiffSnapshot::unavailable(request, GitDiagnosticCode::InvalidPath);
        };
        let area_available = match request.area {
            GitDiffArea::Staged => change.staged.is_some(),
            GitDiffArea::Worktree => change.worktree.is_some(),
        };
        if !change.reviewable || !area_available {
            return GitDiffSnapshot::unavailable(request, GitDiagnosticCode::DiffUnavailable);
        }

        match inspect_diff(&root.attached_root, &request, change).await {
            Ok((kind, lines, truncated)) => GitDiffSnapshot {
                schema_version: GIT_SCHEMA_VERSION,
                state: GitDiffState::Ready,
                project_id: request.project_id,
                path: request.path,
                area: request.area,
                kind: Some(kind),
                lines,
                truncated,
                diagnostic_code: None,
            },
            Err(error) => GitDiffSnapshot::unavailable(
                request,
                map_run_error(error, GitDiagnosticCode::DiffUnavailable),
            ),
        }
    }

    pub async fn review_file(
        &self,
        request: GitOpenFileRequest,
        projects: &ProjectService,
    ) -> Result<PathBuf, GitDiagnosticCode> {
        if !valid_relative_path(&request.path) {
            return Err(GitDiagnosticCode::InvalidPath);
        }
        let root = projects
            .review_root(&request.project_id)
            .map_err(map_project_error)?;
        let (_, changes, _) = inspect_status(&root.attached_root, &root.worktree_root)
            .await
            .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
        let Some(change) = changes.iter().find(|change| change.path == request.path) else {
            return Err(GitDiagnosticCode::InvalidPath);
        };
        if !change.reviewable
            || change.worktree == Some(GitChangeKind::Deleted)
            || (change.staged == Some(GitChangeKind::Deleted) && change.worktree.is_none())
        {
            return Err(GitDiagnosticCode::DiffUnavailable);
        }
        let candidate = root.attached_root.join(&request.path);
        let metadata = candidate
            .symlink_metadata()
            .map_err(|_| GitDiagnosticCode::DiffUnavailable)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(GitDiagnosticCode::InvalidPath);
        }
        let resolved = candidate
            .canonicalize()
            .map_err(|_| GitDiagnosticCode::DiffUnavailable)?;
        if !resolved.starts_with(&root.attached_root) || !resolved.is_file() {
            return Err(GitDiagnosticCode::InvalidPath);
        }
        Ok(resolved)
    }

    pub async fn preview_mutation(
        &self,
        request: GitMutationPreviewRequest,
        projects: &ProjectService,
    ) -> GitMutationPreviewSnapshot {
        self.mutations.preview(request, projects).await
    }

    pub async fn confirm_mutation(
        &self,
        request: GitMutationConfirmRequest,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        self.mutations.confirm(request, projects).await
    }

    pub async fn recover_mutation(
        &self,
        request: GitRecoveryRequest,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        self.mutations.recover(request, projects).await
    }
}

async fn workspace_from_root(project_id: String, root: &ProjectReviewRoot) -> GitWorkspaceSnapshot {
    match inspect_status(&root.attached_root, &root.worktree_root).await {
        Ok((branch, changes, truncated)) => GitWorkspaceSnapshot {
            schema_version: GIT_SCHEMA_VERSION,
            state: if changes.is_empty() {
                GitWorkspaceState::Clean
            } else {
                GitWorkspaceState::Ready
            },
            project_id: Some(project_id),
            branch: Some(branch),
            changes,
            truncated,
            diagnostic_code: None,
        },
        Err(error) => GitWorkspaceSnapshot::unavailable(
            Some(project_id),
            map_run_error(error, GitDiagnosticCode::GitFailed),
        ),
    }
}

async fn inspect_status(
    cwd: &Path,
    worktree_root: &Path,
) -> Result<(GitBranchSummary, Vec<GitFileChange>, bool), GitRunError> {
    let scope = cwd
        .strip_prefix(worktree_root)
        .map_err(|_| GitRunError::Failed)?;
    let scope = scope.to_str().ok_or(GitRunError::Failed)?;
    if !scope.is_empty() && !valid_relative_path(scope) {
        return Err(GitRunError::Failed);
    }
    let output = run_git(
        cwd,
        &[
            "status",
            "--porcelain=v2",
            "--branch",
            "-z",
            "--untracked-files=all",
            "--",
            ".",
        ],
        MAX_STATUS_BYTES,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    parse_status(&output.stdout, scope)
}

async fn inspect_diff(
    cwd: &Path,
    request: &GitDiffRequest,
    change: &GitFileChange,
) -> Result<(GitDiffKind, Vec<GitDiffLine>, bool), GitRunError> {
    if request.area == GitDiffArea::Worktree
        && change.worktree != Some(GitChangeKind::Deleted)
        && !safe_worktree_file(cwd, &request.path)
    {
        return Err(GitRunError::Failed);
    }
    let output = if request.area == GitDiffArea::Worktree
        && change.worktree == Some(GitChangeKind::Untracked)
    {
        run_git(
            cwd,
            &[
                "diff",
                "--no-index",
                "--no-ext-diff",
                "--no-textconv",
                "--unified=3",
                "--",
                "/dev/null",
                &request.path,
            ],
            MAX_DIFF_BYTES,
        )
        .await?
    } else {
        let mut arguments = vec!["diff", "--no-ext-diff", "--no-textconv", "--unified=3"];
        if request.area == GitDiffArea::Staged {
            arguments.push("--cached");
        }
        arguments.extend(["--", request.path.as_str()]);
        run_git(cwd, &arguments, MAX_DIFF_BYTES).await?
    };
    let accepted = output.success
        || (request.area == GitDiffArea::Worktree
            && change.worktree == Some(GitChangeKind::Untracked)
            && output.code == Some(1));
    if !accepted {
        return Err(GitRunError::Failed);
    }
    parse_diff(&output.stdout)
}

async fn run_git(cwd: &Path, arguments: &[&str], limit: usize) -> Result<GitOutput, GitRunError> {
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", "/nonexistent")
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .arg("--no-pager")
        .args(["-c", "core.quotepath=false"])
        .args(["-c", "color.ui=false"])
        .args(["-c", "core.fsmonitor=false"])
        .args(["-c", "core.untrackedCache=false"])
        .args(["-c", "status.relativePaths=true"])
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| GitRunError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(GitRunError::Unavailable)?;
    let stderr = child.stderr.take().ok_or(GitRunError::Unavailable)?;
    let result = timeout(GIT_TIMEOUT, async {
        let (stdout, stderr, status) = tokio::join!(
            read_bounded(stdout, limit),
            read_bounded(stderr, MAX_STDERR_BYTES),
            child.wait(),
        );
        (stdout, stderr, status)
    })
    .await;
    let (stdout, stderr, status) = match result {
        Ok(result) => result,
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(GitRunError::TimedOut);
        }
    };
    let stdout = stdout?;
    let _ = stderr?;
    let status = status.map_err(|_| GitRunError::Failed)?;
    Ok(GitOutput {
        stdout,
        success: status.success(),
        code: status.code(),
    })
}

async fn read_bounded(
    reader: impl tokio::io::AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, GitRunError> {
    let mut bytes = Vec::with_capacity(limit.min(16 * 1024));
    reader
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| GitRunError::Failed)?;
    if bytes.len() > limit {
        return Err(GitRunError::TooLarge);
    }
    Ok(bytes)
}

fn parse_status(
    bytes: &[u8],
    scope: &str,
) -> Result<(GitBranchSummary, Vec<GitFileChange>, bool), GitRunError> {
    let records: Vec<&[u8]> = bytes.split(|byte| *byte == 0).collect();
    let mut branch = GitBranchSummary {
        head: None,
        upstream: None,
        ahead: 0,
        behind: 0,
        detached: false,
    };
    let mut changes = Vec::new();
    let mut truncated = false;
    let mut index = 0;

    while index < records.len() {
        let record = records[index];
        index += 1;
        if record.is_empty() {
            continue;
        }
        let Ok(text) = std::str::from_utf8(record) else {
            truncated = true;
            continue;
        };
        if let Some(value) = text.strip_prefix("# branch.head ") {
            if value == "(detached)" {
                branch.detached = true;
            } else {
                branch.head = safe_label(value, 256);
            }
            continue;
        }
        if let Some(value) = text.strip_prefix("# branch.upstream ") {
            branch.upstream = safe_label(value, 256);
            continue;
        }
        if let Some(value) = text.strip_prefix("# branch.ab ") {
            for count in value.split_whitespace() {
                if let Some(ahead) = count.strip_prefix('+') {
                    branch.ahead = ahead.parse().unwrap_or(0);
                } else if let Some(behind) = count.strip_prefix('-') {
                    branch.behind = behind.parse().unwrap_or(0);
                }
            }
            continue;
        }

        let parsed = if text.starts_with("1 ") {
            parse_ordinary_change(text, scope)
        } else if text.starts_with("2 ") {
            let previous = records.get(index).copied();
            if previous.is_some() {
                index += 1;
            }
            parse_renamed_change(text, previous, scope)
        } else if text.starts_with("u ") {
            parse_unmerged_change(text, scope)
        } else if let Some(path) = text.strip_prefix("? ") {
            scoped_status_path(path, scope).map(|path| GitFileChange {
                path,
                previous_path: None,
                staged: None,
                worktree: Some(GitChangeKind::Untracked),
                conflict: false,
                submodule: false,
                reviewable: true,
            })
        } else {
            None
        };
        if let Some(change) = parsed {
            if changes.len() == MAX_CHANGES {
                truncated = true;
                break;
            }
            changes.push(change);
        } else if !text.starts_with('!') && !text.starts_with('#') {
            truncated = true;
        }
    }

    changes.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((branch, changes, truncated))
}

fn parse_ordinary_change(record: &str, scope: &str) -> Option<GitFileChange> {
    let fields: Vec<&str> = record.splitn(9, ' ').collect();
    if fields.len() != 9 {
        return None;
    }
    change_from_fields(fields[1], fields[2], fields[8], None, false, scope)
}

fn parse_renamed_change(
    record: &str,
    previous: Option<&[u8]>,
    scope: &str,
) -> Option<GitFileChange> {
    let fields: Vec<&str> = record.splitn(10, ' ').collect();
    if fields.len() != 10 {
        return None;
    }
    let previous = previous
        .and_then(|value| std::str::from_utf8(value).ok())
        .and_then(|value| scoped_status_path(value, scope));
    change_from_fields(fields[1], fields[2], fields[9], previous, false, scope)
}

fn parse_unmerged_change(record: &str, scope: &str) -> Option<GitFileChange> {
    let fields: Vec<&str> = record.splitn(11, ' ').collect();
    if fields.len() != 11 {
        return None;
    }
    change_from_fields(fields[1], fields[2], fields[10], None, true, scope)
}

fn change_from_fields(
    xy: &str,
    submodule: &str,
    path: &str,
    previous_path: Option<String>,
    conflict: bool,
    scope: &str,
) -> Option<GitFileChange> {
    let mut status = xy.chars();
    let staged = change_kind(status.next()?);
    let worktree = change_kind(status.next()?);
    if staged.is_none() && worktree.is_none() {
        return None;
    }
    if (matches!(staged, Some(GitChangeKind::Renamed | GitChangeKind::Copied))
        || matches!(
            worktree,
            Some(GitChangeKind::Renamed | GitChangeKind::Copied)
        ))
        && previous_path.is_none()
    {
        return None;
    }
    let path = scoped_status_path(path, scope)?;
    let conflict = conflict
        || staged == Some(GitChangeKind::Unmerged)
        || worktree == Some(GitChangeKind::Unmerged);
    let submodule = submodule.starts_with('S');
    Some(GitFileChange {
        path,
        previous_path,
        staged,
        worktree,
        conflict,
        submodule,
        reviewable: !conflict && !submodule,
    })
}

fn change_kind(value: char) -> Option<GitChangeKind> {
    match value {
        '.' => None,
        'M' => Some(GitChangeKind::Modified),
        'A' => Some(GitChangeKind::Added),
        'D' => Some(GitChangeKind::Deleted),
        'R' => Some(GitChangeKind::Renamed),
        'C' => Some(GitChangeKind::Copied),
        'T' => Some(GitChangeKind::TypeChanged),
        'U' => Some(GitChangeKind::Unmerged),
        '?' => Some(GitChangeKind::Untracked),
        _ => None,
    }
}

fn parse_diff(bytes: &[u8]) -> Result<(GitDiffKind, Vec<GitDiffLine>, bool), GitRunError> {
    let text = std::str::from_utf8(bytes).map_err(|_| GitRunError::Failed)?;
    if text
        .lines()
        .any(|line| line.starts_with("Binary files ") || line.starts_with("GIT binary patch"))
    {
        return Ok((GitDiffKind::Binary, Vec::new(), false));
    }

    let mut lines = Vec::new();
    let mut old_line = 0_u32;
    let mut new_line = 0_u32;
    let mut truncated = false;
    for line in text.lines() {
        if line.starts_with("diff --git ")
            || line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("new file mode ")
            || line.starts_with("deleted file mode ")
            || line.starts_with("similarity index ")
            || line.starts_with("rename from ")
            || line.starts_with("rename to ")
        {
            continue;
        }
        let parsed = if line.starts_with("@@ ") {
            if let Some((old, new)) = parse_hunk_lines(line) {
                old_line = old;
                new_line = new;
            }
            Some(GitDiffLine {
                kind: GitDiffLineKind::Hunk,
                old_line: None,
                new_line: None,
                text: sanitize_line(line),
            })
        } else if let Some(text) = line.strip_prefix('+') {
            let current = new_line;
            new_line = new_line.saturating_add(1);
            Some(GitDiffLine {
                kind: GitDiffLineKind::Addition,
                old_line: None,
                new_line: Some(current),
                text: sanitize_line(text),
            })
        } else if let Some(text) = line.strip_prefix('-') {
            let current = old_line;
            old_line = old_line.saturating_add(1);
            Some(GitDiffLine {
                kind: GitDiffLineKind::Deletion,
                old_line: Some(current),
                new_line: None,
                text: sanitize_line(text),
            })
        } else if let Some(text) = line.strip_prefix(' ') {
            let previous = old_line;
            let current = new_line;
            old_line = old_line.saturating_add(1);
            new_line = new_line.saturating_add(1);
            Some(GitDiffLine {
                kind: GitDiffLineKind::Context,
                old_line: Some(previous),
                new_line: Some(current),
                text: sanitize_line(text),
            })
        } else {
            None
        };
        if let Some(line) = parsed {
            if lines.len() == MAX_DIFF_LINES {
                truncated = true;
                break;
            }
            lines.push(line);
        }
    }
    Ok((GitDiffKind::Text, lines, truncated))
}

fn parse_hunk_lines(line: &str) -> Option<(u32, u32)> {
    let mut fields = line.split_whitespace();
    if fields.next()? != "@@" {
        return None;
    }
    let old = fields
        .next()?
        .strip_prefix('-')?
        .split(',')
        .next()?
        .parse()
        .ok()?;
    let new = fields
        .next()?
        .strip_prefix('+')?
        .split(',')
        .next()?
        .parse()
        .ok()?;
    Some((old, new))
}

fn scoped_status_path(value: &str, scope: &str) -> Option<String> {
    if !valid_relative_path(value) {
        return None;
    }
    if scope.is_empty() {
        return Some(value.to_owned());
    }
    value
        .strip_prefix(scope)
        .and_then(|value| value.strip_prefix('/'))
        .filter(|value| valid_relative_path(value))
        .map(str::to_owned)
}

fn safe_worktree_file(cwd: &Path, relative: &str) -> bool {
    let candidate = cwd.join(relative);
    let Ok(metadata) = candidate.symlink_metadata() else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return false;
    }
    candidate
        .canonicalize()
        .is_ok_and(|resolved| resolved.starts_with(cwd))
}

fn valid_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4_096
        && !value.contains('\\')
        && !value.chars().any(unsafe_display_character)
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn safe_label(value: &str, max: usize) -> Option<String> {
    if value.is_empty() || value.chars().any(unsafe_display_character) {
        return None;
    }
    Some(value.chars().take(max).collect())
}

fn unsafe_display_character(character: char) -> bool {
    character.is_control() || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn sanitize_line(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            (!character.is_control() || *character == '\t')
                && !matches!(*character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(MAX_LINE_CHARACTERS)
        .collect()
}

fn map_project_error(error: ProjectExecutionError) -> GitDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            GitDiagnosticCode::ProjectNotFound
        }
        ProjectExecutionError::IdentityChanged => GitDiagnosticCode::IdentityChanged,
        ProjectExecutionError::NotRepository => GitDiagnosticCode::NotRepository,
        ProjectExecutionError::NotWritable => GitDiagnosticCode::ReadOnly,
        ProjectExecutionError::ProjectBusy => GitDiagnosticCode::ProjectBusy,
        ProjectExecutionError::MetadataUnavailable
        | ProjectExecutionError::DirectoryUnavailable => GitDiagnosticCode::DirectoryUnavailable,
    }
}

fn map_run_error(error: GitRunError, fallback: GitDiagnosticCode) -> GitDiagnosticCode {
    match error {
        GitRunError::Unavailable => GitDiagnosticCode::GitUnavailable,
        GitRunError::TooLarge => GitDiagnosticCode::OutputTooLarge,
        GitRunError::Failed | GitRunError::TimedOut => fallback,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, process::Command as StdCommand};

    use serde_json::Value;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn unavailable_snapshots_match_the_shared_frontend_fixtures() {
        let workspace_fixture: Value =
            serde_json::from_str(include_str!("../../../fixtures/git-workspace.json"))
                .expect("workspace fixture must be valid JSON");
        let diff_fixture: Value =
            serde_json::from_str(include_str!("../../../fixtures/git-diff.json"))
                .expect("diff fixture must be valid JSON");
        let workspace = GitWorkspaceSnapshot::unavailable(None, GitDiagnosticCode::ProjectNotFound);
        let diff = GitDiffSnapshot::unavailable(
            GitDiffRequest {
                project_id: "018f0000-0000-7000-8000-000000000001".to_owned(),
                path: "README.md".to_owned(),
                area: GitDiffArea::Worktree,
            },
            GitDiagnosticCode::DiffUnavailable,
        );

        assert_eq!(serde_json::to_value(workspace).unwrap(), workspace_fixture);
        assert_eq!(serde_json::to_value(diff).unwrap(), diff_fixture);
    }

    #[test]
    fn parses_bounded_porcelain_v2_without_exposing_object_ids() {
        let input = [
            b"# branch.oid abcdef\0# branch.head feature/review\0# branch.upstream origin/feature/review\0# branch.ab +2 -1\0".as_slice(),
            b"1 M. N... 100644 100644 100644 aaa bbb README.md\0? src/new.ts\0".as_slice(),
        ]
        .concat();
        let (branch, changes, truncated) = parse_status(&input, "").expect("status must parse");
        assert_eq!(branch.head.as_deref(), Some("feature/review"));
        assert_eq!(branch.upstream.as_deref(), Some("origin/feature/review"));
        assert_eq!((branch.ahead, branch.behind), (2, 1));
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].path, "README.md");
        assert_eq!(changes[0].staged, Some(GitChangeKind::Modified));
        assert_eq!(changes[1].worktree, Some(GitChangeKind::Untracked));
        assert!(!truncated);
    }

    #[test]
    fn parses_renames_and_rejects_escaping_or_control_paths() {
        let input = b"2 R. N... 100644 100644 100644 aaa bbb R100 src/new.rs\0src/old.rs\0? ../outside\0? bad\nname\0";
        let (_, changes, truncated) = parse_status(input, "").expect("status must parse");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "src/new.rs");
        assert_eq!(changes[0].previous_path.as_deref(), Some("src/old.rs"));
        assert!(truncated);
    }

    #[test]
    fn rejects_deceptive_paths_and_labels_at_the_native_boundary() {
        assert!(!valid_relative_path("src\\outside.rs"));
        assert!(!valid_relative_path("src/\u{202e}rs.exe"));
        assert_eq!(safe_label("feature/\u{2066}hidden", 256), None);
    }

    #[test]
    fn normalizes_diff_lines_and_strips_directional_controls() {
        let input = b"diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,2 +1,2 @@\n-old\n+new\xE2\x80\xAE\n context\n";
        let (kind, lines, truncated) = parse_diff(input).expect("diff must parse");
        assert_eq!(kind, GitDiffKind::Text);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[1].old_line, Some(1));
        assert_eq!(lines[2].new_line, Some(1));
        assert_eq!(lines[2].text, "new");
        assert!(!truncated);
    }

    #[test]
    fn recognizes_binary_diff_without_forwarding_git_paths() {
        let input = b"diff --git a/image.png b/image.png\nBinary files a/image.png and b/image.png differ\n";
        let (kind, lines, truncated) = parse_diff(input).expect("diff must parse");
        assert_eq!(kind, GitDiffKind::Binary);
        assert!(lines.is_empty());
        assert!(!truncated);
    }

    #[tokio::test]
    async fn fixed_git_commands_review_an_untracked_text_file() {
        let root = std::env::temp_dir().join(format!("quireforge-git-{}", Uuid::now_v7()));
        fs::create_dir(&root).expect("temporary repository directory must be created");
        let initialized = StdCommand::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git must start for the repository test");
        assert!(initialized.success());
        fs::write(root.join("note.txt"), "safe review\n")
            .expect("temporary review file must be written");

        let (_, changes, truncated) = inspect_status(&root, &root)
            .await
            .expect("status must succeed");
        assert!(!truncated);
        let change = changes
            .iter()
            .find(|change| change.path == "note.txt")
            .expect("untracked file must be normalized");
        let request = GitDiffRequest {
            project_id: "018f0000-0000-7000-8000-000000000001".to_owned(),
            path: "note.txt".to_owned(),
            area: GitDiffArea::Worktree,
        };
        let (kind, lines, diff_truncated) = inspect_diff(&root, &request, change)
            .await
            .expect("untracked diff must succeed");
        assert_eq!(kind, GitDiffKind::Text);
        assert!(lines
            .iter()
            .any(|line| { line.kind == GitDiffLineKind::Addition && line.text == "safe review" }));
        assert!(!diff_truncated);

        fs::remove_dir_all(&root).expect("temporary repository must be removed");
    }

    #[tokio::test]
    async fn fixed_git_commands_confine_a_subdirectory_and_disable_external_diff() {
        let root = std::env::temp_dir().join(format!("quireforge-git-{}", Uuid::now_v7()));
        let attached = root.join("attached");
        fs::create_dir_all(&attached).expect("attached subdirectory must be created");
        let initialized = StdCommand::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git must start for the repository test");
        assert!(initialized.success());
        let configured = StdCommand::new("git")
            .args(["config", "--local", "status.relativePaths", "false"])
            .current_dir(&root)
            .status()
            .expect("local status configuration must be written");
        assert!(configured.success());
        let configured = StdCommand::new("git")
            .args(["config", "--local", "diff.external", "/bin/false"])
            .current_dir(&root)
            .status()
            .expect("external diff configuration must be written");
        assert!(configured.success());
        fs::write(attached.join("note.txt"), "old\n").expect("tracked file must be written");
        fs::write(root.join("outside.txt"), "outside\n").expect("outside file must be written");
        let added = StdCommand::new("git")
            .args(["add", "--", "attached/note.txt"])
            .current_dir(&root)
            .status()
            .expect("temporary file must be staged");
        assert!(added.success());
        fs::write(attached.join("note.txt"), "new\n").expect("tracked file must be modified");

        let (_, changes, truncated) = inspect_status(&attached, &root)
            .await
            .expect("subdirectory status must succeed");
        assert!(!truncated);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "note.txt");
        assert_eq!(changes[0].worktree, Some(GitChangeKind::Modified));
        let request = GitDiffRequest {
            project_id: "018f0000-0000-7000-8000-000000000001".to_owned(),
            path: "note.txt".to_owned(),
            area: GitDiffArea::Worktree,
        };
        let (kind, lines, _) = inspect_diff(&attached, &request, &changes[0])
            .await
            .expect("built-in diff must override the configured external helper");
        assert_eq!(kind, GitDiffKind::Text);
        assert!(lines
            .iter()
            .any(|line| line.kind == GitDiffLineKind::Deletion && line.text == "old"));
        assert!(lines
            .iter()
            .any(|line| line.kind == GitDiffLineKind::Addition && line.text == "new"));

        fs::remove_dir_all(&root).expect("temporary repository must be removed");
    }
}
