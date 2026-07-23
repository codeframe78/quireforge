pub mod types;

use std::{
    collections::{HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use uuid::Uuid;

use crate::{
    preview::{types::FilePreviewDiagnosticCode, validate_attachment_image},
    project::{ProjectExecutionError, ProjectService},
};
use types::{
    ConversationAttachmentCancelRequest, ConversationAttachmentDiagnosticCode,
    ConversationAttachmentDropRequest, ConversationAttachmentSnapshot,
    ConversationAttachmentSource, ConversationAttachmentState, ConversationAttachmentSummary,
    CONVERSATION_ATTACHMENT_SCHEMA_VERSION,
};

pub const MAX_CONVERSATION_ATTACHMENTS: usize = 4;
const MAX_ATTACHMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = MAX_CONVERSATION_ATTACHMENTS * MAX_ATTACHMENT_BYTES;
const MAX_BASE64_BYTES: usize = MAX_ATTACHMENT_BYTES.div_ceil(3) * 4;
const ATTACHMENT_TTL: Duration = Duration::from_secs(15 * 60);
const NATIVE_DROP_TTL: Duration = Duration::from_secs(30);

pub struct ConversationAttachmentService {
    staging_root: Option<PathBuf>,
    state: Mutex<AttachmentState>,
}

#[derive(Default)]
struct AttachmentState {
    pending: Vec<StagedAttachment>,
    in_flight: HashMap<String, Vec<StagedAttachment>>,
    native_drop: Option<CapturedNativeDrop>,
}

struct CapturedNativeDrop {
    paths: Vec<PathBuf>,
    captured_at: Instant,
}

#[derive(Clone)]
struct StagedAttachment {
    summary: ConversationAttachmentSummary,
    project_id: String,
    path: PathBuf,
    device: u64,
    inode: u64,
    created_at: Instant,
}

struct PreparedAttachment {
    display_name: String,
    source: ConversationAttachmentSource,
    mime_type: &'static str,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedConversationAttachment {
    path: PathBuf,
}

impl ResolvedConversationAttachment {
    pub(crate) fn protocol_input(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "localImage",
            "path": self.path,
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(path: PathBuf) -> Self {
        Self { path }
    }
}

pub(crate) struct ClaimedConversationAttachments {
    attachments: Vec<StagedAttachment>,
}

impl ClaimedConversationAttachments {
    pub(crate) fn resolved(&self) -> Vec<ResolvedConversationAttachment> {
        self.attachments
            .iter()
            .map(|attachment| ResolvedConversationAttachment {
                path: attachment.path.clone(),
            })
            .collect()
    }
}

impl Drop for ClaimedConversationAttachments {
    fn drop(&mut self) {
        for attachment in &self.attachments {
            let _ = remove_staged_file(attachment);
        }
    }
}

impl ConversationAttachmentService {
    pub fn open(staging_root: PathBuf) -> Self {
        let staging_root = prepare_staging_root(&staging_root).ok();
        Self {
            staging_root,
            state: Mutex::new(AttachmentState::default()),
        }
    }

    pub fn unavailable() -> Self {
        Self {
            staging_root: None,
            state: Mutex::new(AttachmentState::default()),
        }
    }

    pub fn status(
        &self,
        project_id: String,
        projects: &ProjectService,
    ) -> ConversationAttachmentSnapshot {
        if let Err(code) = validate_project(&project_id, projects) {
            return ConversationAttachmentSnapshot::unavailable(
                project_for_error(&project_id, code),
                code,
            );
        }
        self.snapshot(&project_id)
    }

    pub fn stage_picker_paths(
        &self,
        project_id: String,
        selected_paths: Vec<PathBuf>,
        projects: &ProjectService,
    ) -> ConversationAttachmentSnapshot {
        if let Err(code) = validate_project(&project_id, projects) {
            return ConversationAttachmentSnapshot::unavailable(
                project_for_error(&project_id, code),
                code,
            );
        }
        if selected_paths.is_empty() {
            return self.snapshot(&project_id);
        }
        if selected_paths.len() > MAX_CONVERSATION_ATTACHMENTS {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::TooManyFiles,
            );
        }
        let prepared = selected_paths
            .iter()
            .map(|path| prepare_path_attachment(path, ConversationAttachmentSource::NativePicker))
            .collect::<Result<Vec<_>, _>>();
        match prepared {
            Ok(prepared) => self.stage_prepared(project_id, prepared),
            Err(code) => ConversationAttachmentSnapshot::unavailable(Some(project_id), code),
        }
    }

    pub fn stage_drop(
        &self,
        request: ConversationAttachmentDropRequest,
        projects: &ProjectService,
    ) -> ConversationAttachmentSnapshot {
        let project_id = request.project_id;
        if let Err(code) = validate_project(&project_id, projects) {
            return ConversationAttachmentSnapshot::unavailable(
                project_for_error(&project_id, code),
                code,
            );
        }
        if let Ok(mut state) = self.state.lock() {
            // Some WebKitGTK versions expose both browser bytes and the GTK
            // URI selection. A successful byte-route attempt consumes the
            // corresponding native fallback so it cannot be replayed.
            state.native_drop = None;
        }
        if request.files.is_empty() || request.files.len() > MAX_CONVERSATION_ATTACHMENTS {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                if request.files.is_empty() {
                    ConversationAttachmentDiagnosticCode::InvalidRequest
                } else {
                    ConversationAttachmentDiagnosticCode::TooManyFiles
                },
            );
        }
        let prepared = request
            .files
            .into_iter()
            .map(|file| {
                if file.base64_data.len() > MAX_BASE64_BYTES {
                    return Err(ConversationAttachmentDiagnosticCode::FileTooLarge);
                }
                let display_name = validate_display_name(&file.display_name)?;
                let bytes = BASE64
                    .decode(file.base64_data)
                    .map_err(|_| ConversationAttachmentDiagnosticCode::InvalidContent)?;
                let image = validate_attachment_image(&bytes).map_err(map_preview_error)?;
                if image.mime_type != file.declared_mime_type {
                    return Err(ConversationAttachmentDiagnosticCode::InvalidContent);
                }
                Ok(PreparedAttachment {
                    display_name,
                    source: ConversationAttachmentSource::DragDrop,
                    mime_type: image.mime_type,
                    bytes,
                    width: image.width,
                    height: image.height,
                })
            })
            .collect::<Result<Vec<_>, _>>();
        match prepared {
            Ok(prepared) => self.stage_prepared(project_id, prepared),
            Err(code) => ConversationAttachmentSnapshot::unavailable(Some(project_id), code),
        }
    }

    pub fn capture_native_drop(&self, paths: Vec<PathBuf>) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.native_drop = Some(CapturedNativeDrop {
            paths: paths
                .into_iter()
                .take(MAX_CONVERSATION_ATTACHMENTS + 1)
                .collect(),
            captured_at: Instant::now(),
        });
    }

    pub fn stage_native_drop(
        &self,
        project_id: String,
        projects: &ProjectService,
    ) -> ConversationAttachmentSnapshot {
        if let Err(code) = validate_project(&project_id, projects) {
            return ConversationAttachmentSnapshot::unavailable(
                project_for_error(&project_id, code),
                code,
            );
        }
        let capture = match self.state.lock() {
            Ok(mut state) => state.native_drop.take(),
            Err(_) => {
                return ConversationAttachmentSnapshot::unavailable(
                    Some(project_id),
                    ConversationAttachmentDiagnosticCode::StagingUnavailable,
                )
            }
        };
        let Some(capture) =
            capture.filter(|capture| capture.captured_at.elapsed() <= NATIVE_DROP_TTL)
        else {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::InvalidRequest,
            );
        };
        if capture.paths.is_empty() || capture.paths.len() > MAX_CONVERSATION_ATTACHMENTS {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                if capture.paths.is_empty() {
                    ConversationAttachmentDiagnosticCode::InvalidRequest
                } else {
                    ConversationAttachmentDiagnosticCode::TooManyFiles
                },
            );
        }
        let prepared = capture
            .paths
            .iter()
            .map(|path| prepare_path_attachment(path, ConversationAttachmentSource::DragDrop))
            .collect::<Result<Vec<_>, _>>();
        match prepared {
            Ok(prepared) => self.stage_prepared(project_id, prepared),
            Err(code) => ConversationAttachmentSnapshot::unavailable(Some(project_id), code),
        }
    }

    pub fn cancel(
        &self,
        request: ConversationAttachmentCancelRequest,
    ) -> ConversationAttachmentSnapshot {
        if !valid_uuid_v7(&request.project_id)
            || request.attachment_ids.is_empty()
            || request.attachment_ids.len() > MAX_CONVERSATION_ATTACHMENTS
            || request.attachment_ids.iter().any(|id| !valid_uuid_v7(id))
            || request.attachment_ids.iter().collect::<HashSet<_>>().len()
                != request.attachment_ids.len()
        {
            return ConversationAttachmentSnapshot::unavailable(
                None,
                ConversationAttachmentDiagnosticCode::InvalidRequest,
            );
        }
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return ConversationAttachmentSnapshot::unavailable(
                    Some(request.project_id),
                    ConversationAttachmentDiagnosticCode::StagingUnavailable,
                )
            }
        };
        if prune_expired(&mut state).is_err() {
            return ConversationAttachmentSnapshot::unavailable(
                Some(request.project_id),
                ConversationAttachmentDiagnosticCode::CleanupFailed,
            );
        }
        if request.attachment_ids.iter().any(|id| {
            !state.pending.iter().any(|attachment| {
                attachment.summary.attachment_id == *id
                    && attachment.project_id == request.project_id
            })
        }) {
            return ConversationAttachmentSnapshot::unavailable(
                Some(request.project_id),
                ConversationAttachmentDiagnosticCode::AttachmentNotFound,
            );
        }
        let ids = request.attachment_ids.into_iter().collect::<HashSet<_>>();
        let mut cleanup_failed = false;
        state.pending.retain(|attachment| {
            if attachment.project_id == request.project_id
                && ids.contains(&attachment.summary.attachment_id)
            {
                cleanup_failed |= remove_staged_file(attachment).is_err();
                false
            } else {
                true
            }
        });
        if cleanup_failed {
            return ConversationAttachmentSnapshot::unavailable(
                Some(request.project_id),
                ConversationAttachmentDiagnosticCode::CleanupFailed,
            );
        }
        snapshot_from_state(&state, &request.project_id)
    }

    pub(crate) fn claim(
        &self,
        project_id: &str,
        attachment_ids: &[String],
        projects: &ProjectService,
    ) -> Result<ClaimedConversationAttachments, ConversationAttachmentDiagnosticCode> {
        if attachment_ids.len() > MAX_CONVERSATION_ATTACHMENTS
            || attachment_ids.iter().any(|id| !valid_uuid_v7(id))
            || attachment_ids.iter().collect::<HashSet<_>>().len() != attachment_ids.len()
        {
            return Err(ConversationAttachmentDiagnosticCode::InvalidRequest);
        }
        if attachment_ids.is_empty() {
            return Ok(ClaimedConversationAttachments {
                attachments: Vec::new(),
            });
        }
        validate_project(project_id, projects)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| ConversationAttachmentDiagnosticCode::StagingUnavailable)?;
        let expired = prune_expired(&mut state)?;
        if expired.iter().any(|id| attachment_ids.contains(id)) {
            return Err(ConversationAttachmentDiagnosticCode::AttachmentExpired);
        }
        let mut claimed = Vec::with_capacity(attachment_ids.len());
        for id in attachment_ids {
            let attachment = state
                .pending
                .iter()
                .find(|attachment| {
                    attachment.summary.attachment_id == *id && attachment.project_id == project_id
                })
                .ok_or(ConversationAttachmentDiagnosticCode::AttachmentNotFound)?;
            verify_staged_file(attachment)?;
            claimed.push(attachment.clone());
        }
        let ids = attachment_ids.iter().collect::<HashSet<_>>();
        state
            .pending
            .retain(|attachment| !ids.contains(&attachment.summary.attachment_id));
        Ok(ClaimedConversationAttachments {
            attachments: claimed,
        })
    }

    pub(crate) fn retain_for_conversation(
        &self,
        conversation_id: &str,
        mut claimed: ClaimedConversationAttachments,
    ) -> Result<(), ConversationAttachmentDiagnosticCode> {
        if claimed.attachments.is_empty() {
            return Ok(());
        }
        if !valid_uuid_v7(conversation_id) {
            return Err(ConversationAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| ConversationAttachmentDiagnosticCode::StagingUnavailable)?;
        state
            .in_flight
            .entry(conversation_id.to_owned())
            .or_default()
            .append(&mut claimed.attachments);
        Ok(())
    }

    pub(crate) fn cleanup_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<(), ConversationAttachmentDiagnosticCode> {
        if !valid_uuid_v7(conversation_id) {
            return Err(ConversationAttachmentDiagnosticCode::InvalidRequest);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| ConversationAttachmentDiagnosticCode::StagingUnavailable)?;
        let Some(attachments) = state.in_flight.remove(conversation_id) else {
            return Ok(());
        };
        let mut cleanup_failed = false;
        for attachment in &attachments {
            cleanup_failed |= remove_staged_file(attachment).is_err();
        }
        if cleanup_failed {
            Err(ConversationAttachmentDiagnosticCode::CleanupFailed)
        } else {
            Ok(())
        }
    }

    fn stage_prepared(
        &self,
        project_id: String,
        prepared: Vec<PreparedAttachment>,
    ) -> ConversationAttachmentSnapshot {
        let Some(root) = self.staging_root.as_ref() else {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::StagingUnavailable,
            );
        };
        if prepared.iter().map(|item| item.bytes.len()).sum::<usize>() > MAX_TOTAL_BYTES {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::FileTooLarge,
            );
        }
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return ConversationAttachmentSnapshot::unavailable(
                    Some(project_id),
                    ConversationAttachmentDiagnosticCode::StagingUnavailable,
                )
            }
        };
        if prune_expired(&mut state).is_err() {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::CleanupFailed,
            );
        }
        let current_count = state
            .pending
            .iter()
            .filter(|attachment| attachment.project_id == project_id)
            .count();
        if current_count + prepared.len() > MAX_CONVERSATION_ATTACHMENTS {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id),
                ConversationAttachmentDiagnosticCode::TooManyFiles,
            );
        }

        let mut staged = Vec::with_capacity(prepared.len());
        for item in prepared {
            match write_staged_file(root, &project_id, item) {
                Ok(attachment) => staged.push(attachment),
                Err(code) => {
                    for attachment in &staged {
                        let _ = remove_staged_file(attachment);
                    }
                    return ConversationAttachmentSnapshot::unavailable(Some(project_id), code);
                }
            }
        }
        state.pending.extend(staged);
        snapshot_from_state(&state, &project_id)
    }

    fn snapshot(&self, project_id: &str) -> ConversationAttachmentSnapshot {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return ConversationAttachmentSnapshot::unavailable(
                    Some(project_id.to_owned()),
                    ConversationAttachmentDiagnosticCode::StagingUnavailable,
                )
            }
        };
        if prune_expired(&mut state).is_err() {
            return ConversationAttachmentSnapshot::unavailable(
                Some(project_id.to_owned()),
                ConversationAttachmentDiagnosticCode::CleanupFailed,
            );
        }
        snapshot_from_state(&state, project_id)
    }
}

fn prepare_staging_root(path: &Path) -> Result<PathBuf, ()> {
    if path.exists() {
        let metadata = path.symlink_metadata().map_err(|_| ())?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(());
        }
    } else {
        fs::create_dir(path).map_err(|_| ())?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| ())?;
    let root = path.canonicalize().map_err(|_| ())?;
    for entry in fs::read_dir(&root).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        let file_name = entry.file_name();
        let file_name = file_name.to_str().ok_or(())?;
        let metadata = entry.path().symlink_metadata().map_err(|_| ())?;
        if !valid_staged_filename(file_name)
            || (!metadata.is_file() && !metadata.file_type().is_symlink())
            || fs::remove_file(entry.path()).is_err()
        {
            return Err(());
        }
    }
    Ok(root)
}

fn valid_staged_filename(value: &str) -> bool {
    let Some((id, extension)) = value.rsplit_once('.') else {
        return false;
    };
    valid_uuid_v7(id) && matches!(extension, "png" | "jpg")
}

fn prepare_path_attachment(
    selected_path: &Path,
    source: ConversationAttachmentSource,
) -> Result<PreparedAttachment, ConversationAttachmentDiagnosticCode> {
    if !selected_path.is_absolute() {
        return Err(ConversationAttachmentDiagnosticCode::InvalidRequest);
    }
    let selected_metadata = selected_path
        .symlink_metadata()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    if selected_metadata.file_type().is_symlink() || !selected_metadata.is_file() {
        return Err(ConversationAttachmentDiagnosticCode::ReadFailed);
    }
    if selected_metadata.len() > MAX_ATTACHMENT_BYTES as u64 {
        return Err(ConversationAttachmentDiagnosticCode::FileTooLarge);
    }
    let display_name = selected_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ConversationAttachmentDiagnosticCode::UnsafeName)
        .and_then(validate_display_name)?;
    let resolved = selected_path
        .canonicalize()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&resolved)
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    let current = resolved
        .metadata()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    if !opened.is_file()
        || opened.len() > MAX_ATTACHMENT_BYTES as u64
        || opened.dev() != selected_metadata.dev()
        || opened.ino() != selected_metadata.ino()
        || opened.len() != selected_metadata.len()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
        || descriptor_path(&file)? != resolved
    {
        return Err(ConversationAttachmentDiagnosticCode::ReadFailed);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_ATTACHMENT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != opened.len() {
        return Err(ConversationAttachmentDiagnosticCode::ReadFailed);
    }
    let image = validate_attachment_image(&bytes).map_err(map_preview_error)?;
    Ok(PreparedAttachment {
        display_name,
        source,
        mime_type: image.mime_type,
        bytes,
        width: image.width,
        height: image.height,
    })
}

#[cfg(target_os = "linux")]
pub fn install_native_drop_capture(app: &tauri::AppHandle) -> tauri::Result<()> {
    use gtk::prelude::*;
    use tauri::Manager;

    let Some(webview) = app.get_webview_window("main") else {
        return Err(tauri::Error::WindowNotFound);
    };
    let handle = app.clone();
    webview.with_webview(move |platform_webview| {
        // WebKitGTK can deliver an empty HTML FileList for a real file-manager
        // drop. Capture only file URIs in native process memory; the drop-zone
        // command can claim them without serializing a source path to React.
        platform_webview
            .inner()
            .connect_drag_data_received(move |_, _, _, _, data, _, _| {
                let paths = data
                    .uris()
                    .into_iter()
                    .filter_map(|uri| url::Url::parse(uri.as_str()).ok())
                    .filter_map(|uri| uri.to_file_path().ok())
                    .collect::<Vec<_>>();
                if !paths.is_empty() {
                    handle
                        .state::<ConversationAttachmentService>()
                        .capture_native_drop(paths);
                }
            });
    })
}

fn write_staged_file(
    root: &Path,
    project_id: &str,
    item: PreparedAttachment,
) -> Result<StagedAttachment, ConversationAttachmentDiagnosticCode> {
    let attachment_id = Uuid::now_v7().to_string();
    let extension = if item.mime_type == "image/png" {
        "png"
    } else {
        "jpg"
    };
    let path = root.join(format!("{attachment_id}.{extension}"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&path)
        .map_err(|_| ConversationAttachmentDiagnosticCode::StagingUnavailable)?;
    if file.write_all(&item.bytes).is_err() || file.sync_all().is_err() {
        let _ = fs::remove_file(&path);
        return Err(ConversationAttachmentDiagnosticCode::StagingUnavailable);
    }
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            let _ = fs::remove_file(&path);
            return Err(ConversationAttachmentDiagnosticCode::StagingUnavailable);
        }
    };
    let descriptor_matches = descriptor_path(&file).is_ok_and(|opened_path| opened_path == path);
    if !metadata.is_file() || metadata.len() != item.bytes.len() as u64 || !descriptor_matches {
        let _ = fs::remove_file(&path);
        return Err(ConversationAttachmentDiagnosticCode::StagingUnavailable);
    }
    Ok(StagedAttachment {
        summary: ConversationAttachmentSummary {
            attachment_id,
            display_name: item.display_name,
            source: item.source,
            mime_type: item.mime_type.to_owned(),
            byte_size: item.bytes.len() as u64,
            image_width: item.width,
            image_height: item.height,
        },
        project_id: project_id.to_owned(),
        path,
        device: metadata.dev(),
        inode: metadata.ino(),
        created_at: Instant::now(),
    })
}

fn verify_staged_file(
    attachment: &StagedAttachment,
) -> Result<(), ConversationAttachmentDiagnosticCode> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&attachment.path)
        .map_err(|_| ConversationAttachmentDiagnosticCode::AttachmentNotFound)?;
    let metadata = file
        .metadata()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    if !metadata.is_file()
        || metadata.dev() != attachment.device
        || metadata.ino() != attachment.inode
        || metadata.len() != attachment.summary.byte_size
        || descriptor_path(&file)? != attachment.path
    {
        return Err(ConversationAttachmentDiagnosticCode::InvalidContent);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    (&mut file)
        .take(MAX_ATTACHMENT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(ConversationAttachmentDiagnosticCode::ReadFailed);
    }
    let image = validate_attachment_image(&bytes).map_err(map_preview_error)?;
    if image.mime_type != attachment.summary.mime_type
        || image.width != attachment.summary.image_width
        || image.height != attachment.summary.image_height
    {
        return Err(ConversationAttachmentDiagnosticCode::InvalidContent);
    }
    Ok(())
}

fn remove_staged_file(
    attachment: &StagedAttachment,
) -> Result<(), ConversationAttachmentDiagnosticCode> {
    let metadata = attachment
        .path
        .symlink_metadata()
        .map_err(|_| ConversationAttachmentDiagnosticCode::CleanupFailed)?;
    if (!metadata.is_file() && !metadata.file_type().is_symlink())
        || (metadata.is_file()
            && (metadata.dev() != attachment.device || metadata.ino() != attachment.inode))
    {
        return Err(ConversationAttachmentDiagnosticCode::CleanupFailed);
    }
    fs::remove_file(&attachment.path)
        .map_err(|_| ConversationAttachmentDiagnosticCode::CleanupFailed)
}

fn prune_expired(
    state: &mut AttachmentState,
) -> Result<Vec<String>, ConversationAttachmentDiagnosticCode> {
    let mut expired = Vec::new();
    let mut cleanup_failed = false;
    state.pending.retain(|attachment| {
        if attachment.created_at.elapsed() >= ATTACHMENT_TTL {
            expired.push(attachment.summary.attachment_id.clone());
            cleanup_failed |= remove_staged_file(attachment).is_err();
            false
        } else {
            true
        }
    });
    if cleanup_failed {
        Err(ConversationAttachmentDiagnosticCode::CleanupFailed)
    } else {
        Ok(expired)
    }
}

fn snapshot_from_state(
    state: &AttachmentState,
    project_id: &str,
) -> ConversationAttachmentSnapshot {
    let attachments = state
        .pending
        .iter()
        .filter(|attachment| attachment.project_id == project_id)
        .map(|attachment| attachment.summary.clone())
        .collect::<Vec<_>>();
    ConversationAttachmentSnapshot {
        schema_version: CONVERSATION_ATTACHMENT_SCHEMA_VERSION,
        state: if attachments.is_empty() {
            ConversationAttachmentState::Empty
        } else {
            ConversationAttachmentState::Ready
        },
        project_id: Some(project_id.to_owned()),
        attachments,
        diagnostic_code: None,
    }
}

fn descriptor_path(file: &File) -> Result<PathBuf, ConversationAttachmentDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| ConversationAttachmentDiagnosticCode::ReadFailed)
}

fn validate_display_name(value: &str) -> Result<String, ConversationAttachmentDiagnosticCode> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 255
        || value.contains(['/', '\\'])
        || value
            .chars()
            .any(|character| character.is_control() || is_bidi(character))
    {
        return Err(ConversationAttachmentDiagnosticCode::UnsafeName);
    }
    Ok(value.to_owned())
}

fn validate_project(
    project_id: &str,
    projects: &ProjectService,
) -> Result<(), ConversationAttachmentDiagnosticCode> {
    projects
        .execution_cwd(project_id)
        .map(|_| ())
        .map_err(map_project_error)
}

fn project_for_error(
    project_id: &str,
    code: ConversationAttachmentDiagnosticCode,
) -> Option<String> {
    (code != ConversationAttachmentDiagnosticCode::InvalidRequest).then(|| project_id.to_owned())
}

fn map_project_error(error: ProjectExecutionError) -> ConversationAttachmentDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId => {
            ConversationAttachmentDiagnosticCode::InvalidRequest
        }
        ProjectExecutionError::ProjectNotFound => {
            ConversationAttachmentDiagnosticCode::ProjectNotFound
        }
        ProjectExecutionError::IdentityChanged => {
            ConversationAttachmentDiagnosticCode::ProjectIdentityChanged
        }
        ProjectExecutionError::NotWritable => {
            ConversationAttachmentDiagnosticCode::ProjectNotWritable
        }
        ProjectExecutionError::MetadataUnavailable
        | ProjectExecutionError::DirectoryUnavailable
        | ProjectExecutionError::NotRepository
        | ProjectExecutionError::ProjectBusy => {
            ConversationAttachmentDiagnosticCode::ProjectUnavailable
        }
    }
}

fn map_preview_error(error: FilePreviewDiagnosticCode) -> ConversationAttachmentDiagnosticCode {
    match error {
        FilePreviewDiagnosticCode::FileTooLarge
        | FilePreviewDiagnosticCode::ImageDimensionsTooLarge => {
            ConversationAttachmentDiagnosticCode::FileTooLarge
        }
        FilePreviewDiagnosticCode::UnsupportedType => {
            ConversationAttachmentDiagnosticCode::UnsupportedType
        }
        FilePreviewDiagnosticCode::InvalidContent => {
            ConversationAttachmentDiagnosticCode::InvalidContent
        }
        FilePreviewDiagnosticCode::ReadFailed => ConversationAttachmentDiagnosticCode::ReadFailed,
        _ => ConversationAttachmentDiagnosticCode::InvalidContent,
    }
}

fn valid_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value)
        .is_ok_and(|id| id.get_version_num() == 7 && id.hyphenated().to_string() == value)
}

fn is_bidi(character: char) -> bool {
    matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink};

    use super::*;
    use crate::attachment::types::ConversationAttachmentDropFile;

    fn temporary_directory(label: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("quireforge-attachment-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("attachment test directory must be created");
        path
    }

    fn attached_project(directory: &Path) -> (ProjectService, String) {
        let projects = ProjectService::in_memory();
        projects.prepare_attachment(directory.to_path_buf());
        let snapshot = projects.confirm_pending();
        (projects, snapshot.projects[0].id.clone())
    }

    fn append_png_chunk(bytes: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(chunk_type);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&[0; 4]);
    }

    fn png_fixture() -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        append_png_chunk(&mut bytes, b"IHDR", &header);
        append_png_chunk(&mut bytes, b"IDAT", &[]);
        append_png_chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    #[test]
    fn stages_claims_retains_and_cleans_an_opaque_dragged_image() {
        let project_directory = temporary_directory("drop-project");
        let staging_directory = temporary_directory("drop-staging").join("staged");
        let (projects, project_id) = attached_project(&project_directory);
        let service = ConversationAttachmentService::open(staging_directory.clone());
        let snapshot = service.stage_drop(
            ConversationAttachmentDropRequest {
                project_id: project_id.clone(),
                files: vec![ConversationAttachmentDropFile {
                    display_name: "review.png".to_owned(),
                    declared_mime_type: "image/png".to_owned(),
                    base64_data: BASE64.encode(png_fixture()),
                }],
            },
            &projects,
        );

        assert_eq!(snapshot.state, ConversationAttachmentState::Ready);
        assert_eq!(snapshot.attachments.len(), 1);
        let serialized = serde_json::to_string(&snapshot).expect("snapshot must serialize");
        assert!(!serialized.contains(staging_directory.to_string_lossy().as_ref()));
        assert!(!serialized.contains(project_directory.to_string_lossy().as_ref()));
        let attachment_id = snapshot.attachments[0].attachment_id.clone();
        let claimed = service
            .claim(&project_id, std::slice::from_ref(&attachment_id), &projects)
            .expect("reviewed attachment must be claimed");
        let input = claimed.resolved()[0].protocol_input();
        let staged_path = input["path"]
            .as_str()
            .map(PathBuf::from)
            .expect("native protocol input needs a staged path");
        assert!(staged_path.starts_with(&staging_directory));
        assert!(staged_path.is_file());
        let conversation_id = Uuid::now_v7().to_string();
        service
            .retain_for_conversation(&conversation_id, claimed)
            .expect("claimed attachment must be retained for the active turn");
        assert!(staged_path.is_file());
        assert_eq!(
            service.status(project_id, &projects).state,
            ConversationAttachmentState::Empty
        );
        service
            .cleanup_conversation(&conversation_id)
            .expect("terminal turn must clean its retained attachment");
        assert!(!staged_path.exists());

        fs::remove_dir_all(project_directory).expect("project fixture must be removed");
        fs::remove_dir_all(
            staging_directory
                .parent()
                .expect("staging fixture needs a parent"),
        )
        .expect("staging fixture must be removed");
    }

    #[test]
    fn picker_accepts_an_explicit_external_image_and_rejects_unsafe_inputs() {
        let project_directory = temporary_directory("picker-project");
        let source_directory = temporary_directory("picker-source");
        let staging_parent = temporary_directory("picker-staging");
        let staging_directory = staging_parent.join("staged");
        let image = source_directory.join("outside.png");
        fs::write(&image, png_fixture()).expect("image fixture must be written");
        let symlink_path = source_directory.join("link.png");
        symlink(&image, &symlink_path).expect("symlink fixture must be created");
        let text = source_directory.join("notes.txt");
        fs::write(&text, "not an image").expect("text fixture must be written");
        let (projects, project_id) = attached_project(&project_directory);
        let service = ConversationAttachmentService::open(staging_directory);

        let accepted = service.stage_picker_paths(project_id.clone(), vec![image], &projects);
        assert_eq!(accepted.state, ConversationAttachmentState::Ready);
        assert_eq!(
            accepted.attachments[0].source,
            ConversationAttachmentSource::NativePicker
        );

        let symlink_result =
            service.stage_picker_paths(project_id.clone(), vec![symlink_path], &projects);
        assert_eq!(
            symlink_result.diagnostic_code,
            Some(ConversationAttachmentDiagnosticCode::ReadFailed)
        );
        let text_result = service.stage_picker_paths(project_id, vec![text], &projects);
        assert_eq!(
            text_result.diagnostic_code,
            Some(ConversationAttachmentDiagnosticCode::UnsupportedType)
        );

        fs::remove_dir_all(project_directory).expect("project fixture must be removed");
        fs::remove_dir_all(source_directory).expect("source fixture must be removed");
        fs::remove_dir_all(staging_parent).expect("staging fixture must be removed");
    }

    #[test]
    fn native_drop_capture_is_one_use_and_never_serializes_its_source_path() {
        let project_directory = temporary_directory("native-drop-project");
        let source_directory = temporary_directory("native-drop-source");
        let staging_parent = temporary_directory("native-drop-staging");
        let image = source_directory.join("dropped.png");
        fs::write(&image, png_fixture()).expect("image fixture must be written");
        let (projects, project_id) = attached_project(&project_directory);
        let service = ConversationAttachmentService::open(staging_parent.join("staged"));

        service.capture_native_drop(vec![image.clone()]);
        let accepted = service.stage_native_drop(project_id.clone(), &projects);
        assert_eq!(accepted.state, ConversationAttachmentState::Ready);
        assert_eq!(accepted.attachments.len(), 1);
        assert_eq!(
            accepted.attachments[0].source,
            ConversationAttachmentSource::DragDrop
        );
        let serialized = serde_json::to_string(&accepted).expect("snapshot must serialize");
        assert!(!serialized.contains(source_directory.to_string_lossy().as_ref()));

        let replay = service.stage_native_drop(project_id.clone(), &projects);
        assert_eq!(
            replay.diagnostic_code,
            Some(ConversationAttachmentDiagnosticCode::InvalidRequest)
        );

        service.capture_native_drop(vec![image; MAX_CONVERSATION_ATTACHMENTS + 1]);
        let overflow = service.stage_native_drop(project_id, &projects);
        assert_eq!(
            overflow.diagnostic_code,
            Some(ConversationAttachmentDiagnosticCode::TooManyFiles)
        );

        fs::remove_dir_all(project_directory).expect("project fixture must be removed");
        fs::remove_dir_all(source_directory).expect("source fixture must be removed");
        fs::remove_dir_all(staging_parent).expect("staging fixture must be removed");
    }

    #[test]
    fn cancellation_is_project_bound_and_tampering_fails_closed() {
        let first_directory = temporary_directory("first-project");
        let second_directory = temporary_directory("second-project");
        let staging_parent = temporary_directory("cancel-staging");
        let staging_directory = staging_parent.join("staged");
        let (projects, first_project_id) = attached_project(&first_directory);
        projects.prepare_attachment(second_directory.clone());
        let second_project_id = projects
            .confirm_pending()
            .projects
            .into_iter()
            .find(|project| {
                project.directory.as_ref().is_some_and(|directory| {
                    directory.display_path == second_directory.to_string_lossy().as_ref()
                })
            })
            .expect("second project must be attached")
            .id;
        let service = ConversationAttachmentService::open(staging_directory);
        let ready = service.stage_drop(
            ConversationAttachmentDropRequest {
                project_id: first_project_id.clone(),
                files: vec![ConversationAttachmentDropFile {
                    display_name: "review.png".to_owned(),
                    declared_mime_type: "image/png".to_owned(),
                    base64_data: BASE64.encode(png_fixture()),
                }],
            },
            &projects,
        );
        let attachment_id = ready.attachments[0].attachment_id.clone();

        let wrong_project = service.cancel(ConversationAttachmentCancelRequest {
            project_id: second_project_id,
            attachment_ids: vec![attachment_id.clone()],
        });
        assert_eq!(
            wrong_project.diagnostic_code,
            Some(ConversationAttachmentDiagnosticCode::AttachmentNotFound)
        );

        let staged_path = {
            let state = service.state.lock().expect("attachment state");
            state.pending[0].path.clone()
        };
        fs::write(&staged_path, b"changed").expect("staged fixture must be changed");
        assert!(matches!(
            service.claim(
                &first_project_id,
                std::slice::from_ref(&attachment_id),
                &projects
            ),
            Err(ConversationAttachmentDiagnosticCode::InvalidContent)
        ));

        fs::remove_file(staged_path).expect("changed staged fixture must be removed");
        fs::remove_dir_all(first_directory).expect("first project fixture must be removed");
        fs::remove_dir_all(second_directory).expect("second project fixture must be removed");
        fs::remove_dir_all(staging_parent).expect("staging fixture must be removed");
    }

    #[test]
    fn expired_drafts_are_deleted_and_cannot_be_claimed() {
        let project_directory = temporary_directory("expiry-project");
        let staging_parent = temporary_directory("expiry-staging");
        let (projects, project_id) = attached_project(&project_directory);
        let service = ConversationAttachmentService::open(staging_parent.join("staged"));
        let ready = service.stage_drop(
            ConversationAttachmentDropRequest {
                project_id: project_id.clone(),
                files: vec![ConversationAttachmentDropFile {
                    display_name: "expiring.png".to_owned(),
                    declared_mime_type: "image/png".to_owned(),
                    base64_data: BASE64.encode(png_fixture()),
                }],
            },
            &projects,
        );
        let attachment_id = ready.attachments[0].attachment_id.clone();
        let staged_path = {
            let mut state = service.state.lock().expect("attachment state");
            state.pending[0].created_at = Instant::now() - ATTACHMENT_TTL;
            state.pending[0].path.clone()
        };

        assert!(matches!(
            service.claim(&project_id, std::slice::from_ref(&attachment_id), &projects),
            Err(ConversationAttachmentDiagnosticCode::AttachmentExpired)
        ));
        assert!(!staged_path.exists());

        fs::remove_dir_all(project_directory).expect("project fixture must be removed");
        fs::remove_dir_all(staging_parent).expect("staging fixture must be removed");
    }

    #[test]
    fn startup_removes_only_recognized_stale_files_and_refuses_unknown_entries() {
        let parent = temporary_directory("startup");
        let staging = parent.join("staged");
        fs::create_dir(&staging).expect("staging fixture must be created");
        let stale = staging.join(format!("{}.png", Uuid::now_v7()));
        fs::write(&stale, png_fixture()).expect("stale fixture must be written");
        let available = ConversationAttachmentService::open(staging.clone());
        assert!(available.staging_root.is_some());
        assert!(!stale.exists());

        let unknown = staging.join("do-not-delete.txt");
        fs::write(&unknown, "preserve").expect("unknown fixture must be written");
        let unavailable = ConversationAttachmentService::open(staging);
        assert!(unavailable.staging_root.is_none());
        assert!(unknown.exists());

        fs::remove_dir_all(parent).expect("startup fixture must be removed");
    }

    #[test]
    fn shared_attachment_fixture_matches_the_native_contract() {
        let snapshot: ConversationAttachmentSnapshot = serde_json::from_str(include_str!(
            "../../../fixtures/conversation-attachments.json"
        ))
        .expect("shared attachment fixture must parse");
        assert_eq!(
            snapshot.schema_version,
            CONVERSATION_ATTACHMENT_SCHEMA_VERSION
        );
        assert_eq!(snapshot.state, ConversationAttachmentState::Ready);
        assert_eq!(snapshot.attachments.len(), 1);
    }

    #[test]
    fn empty_claim_defers_project_validation_to_the_conversation_boundary() {
        let service = ConversationAttachmentService::unavailable();
        let projects = ProjectService::in_memory();
        let claimed = service
            .claim("not-a-project-id", &[], &projects)
            .expect("an attachment-free request must not change conversation diagnostics");
        assert!(claimed.resolved().is_empty());
    }
}
