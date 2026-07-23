pub mod types;

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::fd::AsRawFd,
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Component, Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use uuid::Uuid;

use crate::project::{ProjectExecutionError, ProjectService};
use types::{
    FilePreviewDiagnosticCode, FilePreviewHandoffRequest, FilePreviewKind, FilePreviewRendering,
    FilePreviewSnapshot, FilePreviewState, FILE_PREVIEW_SCHEMA_VERSION,
};

const MAX_SOURCE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 128 * 1024;
const MAX_TEXT_LINES: usize = 2_000;
const MAX_IMAGE_DIMENSION: u32 = 8_192;
const MAX_IMAGE_PIXELS: u64 = 16_000_000;
const SNIFF_BYTES: usize = 32 * 1024;
const HANDOFF_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_PENDING_HANDOFFS: usize = 16;

#[derive(Default)]
pub struct FilePreviewService {
    state: Mutex<FilePreviewServiceState>,
}

#[derive(Default)]
struct FilePreviewServiceState {
    pending: HashMap<String, PendingFileHandoff>,
}

struct PendingFileHandoff {
    open_action_id: String,
    project_id: String,
    relative_path: PathBuf,
    device: u64,
    inode: u64,
    created_at: Instant,
}

struct PreparedPreview {
    snapshot: FilePreviewSnapshot,
    pending: PendingFileHandoff,
}

pub struct ClaimedFileHandoff(PendingFileHandoff);

impl ClaimedFileHandoff {
    pub fn path(&self, projects: &ProjectService) -> Result<PathBuf, FilePreviewDiagnosticCode> {
        revalidate_handoff(&self.0, projects)
    }
}

enum DetectedPreview {
    Text,
    Png(u32, u32),
    Jpeg(u32, u32),
    Pdf,
}

pub(crate) struct ValidatedAttachmentImage {
    pub(crate) mime_type: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) fn validate_attachment_image(
    bytes: &[u8],
) -> Result<ValidatedAttachmentImage, FilePreviewDiagnosticCode> {
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err(FilePreviewDiagnosticCode::FileTooLarge);
    }
    let sniff = &bytes[..bytes.len().min(SNIFF_BYTES)];
    let (mime_type, sniff_dimensions) = match detect_preview(sniff)? {
        DetectedPreview::Png(width, height) => ("image/png", (width, height)),
        DetectedPreview::Jpeg(width, height) => ("image/jpeg", (width, height)),
        _ => return Err(FilePreviewDiagnosticCode::UnsupportedType),
    };
    let full_dimensions = match mime_type {
        "image/png" => validated_png_dimensions(bytes)?,
        "image/jpeg" if bytes.ends_with(&[0xff, 0xd9]) => {
            jpeg_dimensions(bytes).ok_or(FilePreviewDiagnosticCode::InvalidContent)?
        }
        _ => return Err(FilePreviewDiagnosticCode::InvalidContent),
    };
    if full_dimensions != sniff_dimensions {
        return Err(FilePreviewDiagnosticCode::InvalidContent);
    }
    validate_dimensions(full_dimensions.0, full_dimensions.1)?;
    Ok(ValidatedAttachmentImage {
        mime_type,
        width: full_dimensions.0,
        height: full_dimensions.1,
    })
}

impl FilePreviewService {
    pub fn preview_selected(
        &self,
        project_id: String,
        selected_path: PathBuf,
        projects: &ProjectService,
    ) -> FilePreviewSnapshot {
        self.clear_project(&project_id);
        let root = match projects.content_root(&project_id) {
            Ok(root) => root,
            Err(error) => return FilePreviewSnapshot::unavailable(None, map_project_error(error)),
        };
        let open_action_id = Uuid::now_v7().to_string();
        match preview_path(&project_id, &root, &selected_path, open_action_id) {
            Ok(prepared) => {
                let snapshot = prepared.snapshot;
                if self.insert(prepared.pending).is_err() {
                    return FilePreviewSnapshot::unavailable(
                        Some(project_id),
                        FilePreviewDiagnosticCode::ReadFailed,
                    );
                }
                snapshot
            }
            Err(code) => FilePreviewSnapshot::unavailable(Some(project_id), code),
        }
    }

    pub fn claim_handoff(
        &self,
        request: &FilePreviewHandoffRequest,
    ) -> Result<ClaimedFileHandoff, FilePreviewDiagnosticCode> {
        if !valid_project_id(&request.open_action_id) {
            return Err(FilePreviewDiagnosticCode::HandoffExpired);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
        state.remove_expired();
        state
            .pending
            .remove(&request.open_action_id)
            .map(ClaimedFileHandoff)
            .ok_or(FilePreviewDiagnosticCode::HandoffExpired)
    }

    pub fn restore_handoff(&self, claimed: ClaimedFileHandoff) {
        if claimed.0.created_at.elapsed() >= HANDOFF_TTL {
            return;
        }
        if let Ok(mut state) = self.state.lock() {
            state.remove_expired();
            if state
                .pending
                .values()
                .any(|pending| pending.project_id == claimed.0.project_id)
            {
                return;
            }
            state.make_room();
            state
                .pending
                .insert(claimed.0.open_action_id.clone(), claimed.0);
        }
    }

    pub fn cancel_handoff(&self, request: &FilePreviewHandoffRequest) -> bool {
        if !valid_project_id(&request.open_action_id) {
            return false;
        }
        self.state.lock().is_ok_and(|mut state| {
            state.remove_expired();
            state.pending.remove(&request.open_action_id).is_some()
        })
    }

    pub fn clear_project(&self, project_id: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.remove_expired();
            state
                .pending
                .retain(|_, pending| pending.project_id != project_id);
        }
    }

    fn insert(&self, pending: PendingFileHandoff) -> Result<(), FilePreviewDiagnosticCode> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
        state.remove_expired();
        state
            .pending
            .retain(|_, existing| existing.project_id != pending.project_id);
        state.make_room();
        state
            .pending
            .insert(pending.open_action_id.clone(), pending);
        Ok(())
    }
}

impl FilePreviewServiceState {
    fn remove_expired(&mut self) {
        self.pending
            .retain(|_, pending| pending.created_at.elapsed() < HANDOFF_TTL);
    }

    fn make_room(&mut self) {
        if self.pending.len() >= MAX_PENDING_HANDOFFS {
            if let Some(oldest) = self
                .pending
                .iter()
                .min_by_key(|(_, existing)| existing.created_at)
                .map(|(action_id, _)| action_id.clone())
            {
                self.pending.remove(&oldest);
            }
        }
    }
}

pub(crate) fn valid_project_id(project_id: &str) -> bool {
    Uuid::parse_str(project_id)
        .is_ok_and(|id| id.get_version_num() == 7 && id.hyphenated().to_string() == project_id)
}

fn preview_path(
    project_id: &str,
    root: &Path,
    selected_path: &Path,
    open_action_id: String,
) -> Result<PreparedPreview, FilePreviewDiagnosticCode> {
    let root_file = open_revalidated_root(root)?;
    let selected_metadata = selected_path
        .symlink_metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if selected_metadata.file_type().is_symlink() || !selected_metadata.is_file() {
        return Err(FilePreviewDiagnosticCode::UnsafePath);
    }
    let resolved = selected_path
        .canonicalize()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if !resolved.starts_with(root) {
        return Err(FilePreviewDiagnosticCode::OutsideProject);
    }
    let relative = resolved
        .strip_prefix(root)
        .map_err(|_| FilePreviewDiagnosticCode::OutsideProject)?;
    let display_path =
        safe_relative_display(relative).ok_or(FilePreviewDiagnosticCode::UnsafePath)?;

    let mut file = open_revalidated_file(root, &root_file, relative, &resolved)?;
    let metadata = file
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(FilePreviewDiagnosticCode::FileTooLarge);
    }

    let mut sniff = Vec::with_capacity(SNIFF_BYTES);
    (&mut file)
        .take(SNIFF_BYTES as u64)
        .read_to_end(&mut sniff)
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let detected = detect_preview(&sniff)?;

    let snapshot = match detected {
        DetectedPreview::Text => text_snapshot(
            project_id,
            display_path,
            metadata.len(),
            &mut file,
            &open_action_id,
        ),
        DetectedPreview::Png(width, height) => image_snapshot(
            project_id,
            display_path,
            metadata.len(),
            &mut file,
            "image/png",
            width,
            height,
            &open_action_id,
        ),
        DetectedPreview::Jpeg(width, height) => image_snapshot(
            project_id,
            display_path,
            metadata.len(),
            &mut file,
            "image/jpeg",
            width,
            height,
            &open_action_id,
        ),
        DetectedPreview::Pdf => Ok(FilePreviewSnapshot {
            schema_version: FILE_PREVIEW_SCHEMA_VERSION,
            state: FilePreviewState::Ready,
            project_id: Some(project_id.to_owned()),
            display_path: Some(display_path),
            kind: Some(FilePreviewKind::Pdf),
            rendering: Some(FilePreviewRendering::MetadataOnly),
            mime_type: Some("application/pdf".to_owned()),
            byte_size: Some(metadata.len()),
            truncated: false,
            text_content: None,
            image_data_url: None,
            image_width: None,
            image_height: None,
            open_action_id: Some(open_action_id.clone()),
            diagnostic_code: None,
        }),
    }?;
    Ok(PreparedPreview {
        snapshot,
        pending: PendingFileHandoff {
            open_action_id,
            project_id: project_id.to_owned(),
            relative_path: relative.to_path_buf(),
            device: metadata.dev(),
            inode: metadata.ino(),
            created_at: Instant::now(),
        },
    })
}

fn revalidate_handoff(
    pending: &PendingFileHandoff,
    projects: &ProjectService,
) -> Result<PathBuf, FilePreviewDiagnosticCode> {
    let root = projects
        .content_root(&pending.project_id)
        .map_err(map_project_error)?;
    let expected_path = root.join(&pending.relative_path);
    let selected = expected_path
        .symlink_metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if selected.file_type().is_symlink() || !selected.is_file() {
        return Err(FilePreviewDiagnosticCode::UnsafePath);
    }
    let resolved = expected_path
        .canonicalize()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if resolved != expected_path || !resolved.starts_with(&root) {
        return Err(FilePreviewDiagnosticCode::UnsafePath);
    }
    let root_file = open_revalidated_root(&root)?;
    let file = open_revalidated_file(&root, &root_file, &pending.relative_path, &resolved)?;
    let metadata = file
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if metadata.dev() != pending.device || metadata.ino() != pending.inode {
        return Err(FilePreviewDiagnosticCode::IdentityChanged);
    }
    Ok(resolved)
}

fn open_revalidated_root(root: &Path) -> Result<File, FilePreviewDiagnosticCode> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(root)
        .map_err(|_| FilePreviewDiagnosticCode::UnsafePath)?;
    let opened = file
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let current = root
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if !opened.is_dir()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
        || descriptor_path(&file)? != root
    {
        return Err(FilePreviewDiagnosticCode::UnsafePath);
    }
    Ok(file)
}

fn open_revalidated_file(
    root: &Path,
    root_file: &File,
    relative: &Path,
    expected_path: &Path,
) -> Result<File, FilePreviewDiagnosticCode> {
    let anchored_path = PathBuf::from("/proc/self/fd")
        .join(root_file.as_raw_fd().to_string())
        .join(relative);
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(anchored_path)
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let current = expected_path
        .metadata()
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let opened_path = descriptor_path(&file)?;
    if !opened.is_file()
        || opened.dev() != current.dev()
        || opened.ino() != current.ino()
        || opened_path != expected_path
        || !opened_path.starts_with(root)
    {
        return Err(FilePreviewDiagnosticCode::UnsafePath);
    }
    Ok(file)
}

fn descriptor_path(file: &File) -> Result<PathBuf, FilePreviewDiagnosticCode> {
    PathBuf::from("/proc/self/fd")
        .join(file.as_raw_fd().to_string())
        .canonicalize()
        .map_err(|_| FilePreviewDiagnosticCode::UnsafePath)
}

fn detect_preview(bytes: &[u8]) -> Result<DetectedPreview, FilePreviewDiagnosticCode> {
    if let Some((width, height)) = png_dimensions(bytes) {
        validate_dimensions(width, height)?;
        return Ok(DetectedPreview::Png(width, height));
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        let (width, height) =
            jpeg_dimensions(bytes).ok_or(FilePreviewDiagnosticCode::InvalidContent)?;
        validate_dimensions(width, height)?;
        return Ok(DetectedPreview::Jpeg(width, height));
    }
    if bytes.starts_with(b"%PDF-") {
        return Ok(DetectedPreview::Pdf);
    }
    if !bytes.contains(&0) && valid_utf8_prefix(bytes) {
        return Ok(DetectedPreview::Text);
    }
    Err(FilePreviewDiagnosticCode::UnsupportedType)
}

fn valid_utf8_prefix(bytes: &[u8]) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(_) => true,
        Err(error) => error.error_len().is_none(),
    }
}

fn text_snapshot(
    project_id: &str,
    display_path: String,
    byte_size: u64,
    file: &mut File,
    open_action_id: &str,
) -> Result<FilePreviewSnapshot, FilePreviewDiagnosticCode> {
    file.seek(SeekFrom::Start(0))
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let mut bytes = Vec::with_capacity(MAX_TEXT_BYTES + 1);
    file.take((MAX_TEXT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let source_truncated = bytes.len() > MAX_TEXT_BYTES || byte_size > MAX_TEXT_BYTES as u64;
    bytes.truncate(MAX_TEXT_BYTES);
    while std::str::from_utf8(&bytes).is_err_and(|error| error.error_len().is_none()) {
        bytes.pop();
    }
    let text =
        std::str::from_utf8(&bytes).map_err(|_| FilePreviewDiagnosticCode::InvalidContent)?;
    let (text_content, normalized_truncated) = normalize_text(text);
    Ok(FilePreviewSnapshot {
        schema_version: FILE_PREVIEW_SCHEMA_VERSION,
        state: FilePreviewState::Ready,
        project_id: Some(project_id.to_owned()),
        display_path: Some(display_path),
        kind: Some(FilePreviewKind::Text),
        rendering: Some(FilePreviewRendering::NormalizedText),
        mime_type: Some("text/plain; charset=utf-8".to_owned()),
        byte_size: Some(byte_size),
        truncated: source_truncated || normalized_truncated,
        text_content: Some(text_content),
        image_data_url: None,
        image_width: None,
        image_height: None,
        open_action_id: Some(open_action_id.to_owned()),
        diagnostic_code: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn image_snapshot(
    project_id: &str,
    display_path: String,
    byte_size: u64,
    file: &mut File,
    mime_type: &str,
    width: u32,
    height: u32,
    open_action_id: &str,
) -> Result<FilePreviewSnapshot, FilePreviewDiagnosticCode> {
    if byte_size > MAX_IMAGE_BYTES {
        return Err(FilePreviewDiagnosticCode::FileTooLarge);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    let mut bytes = Vec::with_capacity(byte_size as usize);
    file.take(MAX_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| FilePreviewDiagnosticCode::ReadFailed)?;
    if bytes.len() as u64 != byte_size {
        return Err(FilePreviewDiagnosticCode::ReadFailed);
    }
    let full_dimensions = match mime_type {
        "image/png" => validated_png_dimensions(&bytes)?,
        "image/jpeg" if bytes.ends_with(&[0xff, 0xd9]) => {
            jpeg_dimensions(&bytes).ok_or(FilePreviewDiagnosticCode::InvalidContent)?
        }
        _ => return Err(FilePreviewDiagnosticCode::InvalidContent),
    };
    if full_dimensions != (width, height) {
        return Err(FilePreviewDiagnosticCode::InvalidContent);
    }
    validate_dimensions(full_dimensions.0, full_dimensions.1)?;
    let image_data_url = format!("data:{mime_type};base64,{}", BASE64.encode(bytes));
    Ok(FilePreviewSnapshot {
        schema_version: FILE_PREVIEW_SCHEMA_VERSION,
        state: FilePreviewState::Ready,
        project_id: Some(project_id.to_owned()),
        display_path: Some(display_path),
        kind: Some(FilePreviewKind::Image),
        rendering: Some(FilePreviewRendering::BoundedImage),
        mime_type: Some(mime_type.to_owned()),
        byte_size: Some(byte_size),
        truncated: false,
        text_content: None,
        image_data_url: Some(image_data_url),
        image_width: Some(width),
        image_height: Some(height),
        open_action_id: Some(open_action_id.to_owned()),
        diagnostic_code: None,
    })
}

fn normalize_text(text: &str) -> (String, bool) {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut output = String::with_capacity(normalized.len().min(MAX_TEXT_BYTES));
    let mut lines = 1usize;
    let mut truncated = false;
    for character in normalized.chars() {
        if character == '\n' && lines >= MAX_TEXT_LINES {
            truncated = true;
            break;
        }
        let normalized_character = if character == '\n'
            || character == '\t'
            || (!character.is_control() && !is_bidi(character))
        {
            character
        } else {
            '\u{fffd}'
        };
        if output.len() + normalized_character.len_utf8() > MAX_TEXT_BYTES {
            truncated = true;
            break;
        }
        output.push(normalized_character);
        if character == '\n' {
            lines += 1;
        }
    }
    (output, truncated)
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24
        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.get(8..12) != Some(b"\0\0\0\r")
        || bytes.get(12..16) != Some(b"IHDR")
    {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    Some((width, height))
}

fn validated_png_dimensions(bytes: &[u8]) -> Result<(u32, u32), FilePreviewDiagnosticCode> {
    let dimensions = png_dimensions(bytes).ok_or(FilePreviewDiagnosticCode::InvalidContent)?;
    let mut index = 8usize;
    let mut first = true;
    let mut saw_image_data = false;
    loop {
        let length = u32::from_be_bytes(
            bytes
                .get(index..index + 4)
                .and_then(|value| value.try_into().ok())
                .ok_or(FilePreviewDiagnosticCode::InvalidContent)?,
        ) as usize;
        let chunk_type = bytes
            .get(index + 4..index + 8)
            .ok_or(FilePreviewDiagnosticCode::InvalidContent)?;
        let end = index
            .checked_add(12)
            .and_then(|value| value.checked_add(length))
            .filter(|end| *end <= bytes.len())
            .ok_or(FilePreviewDiagnosticCode::InvalidContent)?;
        if first && (chunk_type != b"IHDR" || length != 13) {
            return Err(FilePreviewDiagnosticCode::InvalidContent);
        }
        if chunk_type == b"acTL" {
            return Err(FilePreviewDiagnosticCode::UnsupportedType);
        }
        if chunk_type == b"IDAT" {
            saw_image_data = true;
        }
        if chunk_type == b"IEND" {
            return if length == 0 && saw_image_data && end == bytes.len() {
                Ok(dimensions)
            } else {
                Err(FilePreviewDiagnosticCode::InvalidContent)
            };
        }
        index = end;
        first = false;
    }
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut index = 2usize;
    while index + 3 < bytes.len() {
        while index < bytes.len() && bytes[index] != 0xff {
            index += 1;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        let marker = *bytes.get(index)?;
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd8).contains(&marker) {
            continue;
        }
        let length = u16::from_be_bytes(bytes.get(index..index + 2)?.try_into().ok()?) as usize;
        if length < 2 || index.checked_add(length)? > bytes.len() {
            return None;
        }
        if matches!(
            marker,
            0xc0 | 0xc1
                | 0xc2
                | 0xc3
                | 0xc5
                | 0xc6
                | 0xc7
                | 0xc9
                | 0xca
                | 0xcb
                | 0xcd
                | 0xce
                | 0xcf
        ) {
            if length < 7 {
                return None;
            }
            let height = u16::from_be_bytes(bytes.get(index + 3..index + 5)?.try_into().ok()?);
            let width = u16::from_be_bytes(bytes.get(index + 5..index + 7)?.try_into().ok()?);
            return Some((u32::from(width), u32::from(height)));
        }
        index += length;
    }
    None
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), FilePreviewDiagnosticCode> {
    if width == 0
        || height == 0
        || width > MAX_IMAGE_DIMENSION
        || height > MAX_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err(FilePreviewDiagnosticCode::ImageDimensionsTooLarge);
    }
    Ok(())
}

fn safe_relative_display(path: &Path) -> Option<String> {
    if path.as_os_str().is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(part) = component else {
            return None;
        };
        let part = part.to_str()?;
        if part.is_empty()
            || part.contains('\\')
            || part
                .chars()
                .any(|character| character.is_control() || is_bidi(character))
        {
            return None;
        }
        parts.push(part);
    }
    let display = parts.join("/");
    (display.len() <= 4_096).then_some(display)
}

fn is_bidi(character: char) -> bool {
    matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn map_project_error(error: ProjectExecutionError) -> FilePreviewDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId => FilePreviewDiagnosticCode::InvalidRequest,
        ProjectExecutionError::ProjectNotFound => FilePreviewDiagnosticCode::ProjectNotFound,
        ProjectExecutionError::IdentityChanged => FilePreviewDiagnosticCode::IdentityChanged,
        ProjectExecutionError::MetadataUnavailable
        | ProjectExecutionError::DirectoryUnavailable
        | ProjectExecutionError::NotRepository
        | ProjectExecutionError::NotWritable
        | ProjectExecutionError::ProjectBusy => FilePreviewDiagnosticCode::DirectoryUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink};

    use super::*;
    use uuid::Uuid;

    fn temporary_directory(label: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("quireforge-preview-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("preview test directory must be created");
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

    fn png_fixture(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(&width.to_be_bytes());
        header.extend_from_slice(&height.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        append_png_chunk(&mut bytes, b"IHDR", &header);
        append_png_chunk(&mut bytes, b"IDAT", &[]);
        append_png_chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    #[test]
    fn previews_normalized_text_without_exposing_an_absolute_path() {
        let directory = temporary_directory("text");
        let file = directory.join("notes.txt");
        fs::write(&file, "hello\r\nworld\u{202e}").expect("text fixture must be written");
        let (projects, project_id) = attached_project(&directory);
        let service = FilePreviewService::default();

        let snapshot = service.preview_selected(project_id, file, &projects);

        assert_eq!(snapshot.state, FilePreviewState::Ready);
        assert_eq!(snapshot.kind, Some(FilePreviewKind::Text));
        assert_eq!(snapshot.display_path.as_deref(), Some("notes.txt"));
        assert_eq!(snapshot.text_content.as_deref(), Some("hello\nworld�"));
        assert!(!serde_json::to_string(&snapshot)
            .expect("preview must serialize")
            .contains(directory.to_string_lossy().as_ref()));
        fs::remove_dir_all(directory).expect("preview test directory must be removed");
    }

    #[test]
    fn recognizes_bounded_png_and_metadata_only_pdf_content() {
        let directory = temporary_directory("media");
        let png = directory.join("pixel.png");
        fs::write(&png, png_fixture(1, 1)).expect("PNG fixture must be written");
        let pdf = directory.join("review.pdf");
        fs::write(&pdf, b"%PDF-1.7\nfixture").expect("PDF fixture must be written");
        let (projects, project_id) = attached_project(&directory);
        let service = FilePreviewService::default();

        let image = service.preview_selected(project_id.clone(), png, &projects);
        assert_eq!(image.kind, Some(FilePreviewKind::Image));
        assert_eq!(image.rendering, Some(FilePreviewRendering::BoundedImage));
        assert_eq!((image.image_width, image.image_height), (Some(1), Some(1)));
        assert!(image
            .image_data_url
            .as_deref()
            .is_some_and(|value| value.starts_with("data:image/png;base64,")));

        let document = service.preview_selected(project_id, pdf, &projects);
        assert_eq!(document.kind, Some(FilePreviewKind::Pdf));
        assert_eq!(document.rendering, Some(FilePreviewRendering::MetadataOnly));
        assert_eq!(document.text_content, None);
        assert_eq!(document.image_data_url, None);
        fs::remove_dir_all(directory).expect("preview test directory must be removed");
    }

    #[test]
    fn rejects_outside_symlink_binary_and_oversized_image_inputs() {
        let directory = temporary_directory("reject");
        let outside = temporary_directory("outside").join("outside.txt");
        fs::write(&outside, "outside").expect("outside fixture must be written");
        let link = directory.join("link.txt");
        symlink(&outside, &link).expect("symlink fixture must be created");
        let parent_link = directory.join("outside-directory");
        symlink(outside.parent().expect("outside directory"), &parent_link)
            .expect("parent symlink fixture must be created");
        let binary = directory.join("binary.dat");
        fs::write(&binary, [0, 1, 2, 3]).expect("binary fixture must be written");
        let broken_jpeg = directory.join("broken.jpg");
        fs::write(&broken_jpeg, [0xff, 0xd8, 0xff, 0xc0, 0, 7, 8, 0, 1, 0, 1])
            .expect("broken JPEG fixture must be written");
        let image = directory.join("large.png");
        let mut image_bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        image_bytes.extend_from_slice(&9_000u32.to_be_bytes());
        image_bytes.extend_from_slice(&9_000u32.to_be_bytes());
        fs::write(&image, image_bytes).expect("large image fixture must be written");
        let (projects, project_id) = attached_project(&directory);
        let service = FilePreviewService::default();

        let root_file =
            open_revalidated_root(&directory).expect("attached preview root must remain open");
        assert!(matches!(
            open_revalidated_file(
                &directory,
                &root_file,
                Path::new("outside-directory/outside.txt"),
                &parent_link.join("outside.txt"),
            ),
            Err(FilePreviewDiagnosticCode::UnsafePath)
        ));

        for (path, expected) in [
            (outside.clone(), FilePreviewDiagnosticCode::OutsideProject),
            (link, FilePreviewDiagnosticCode::UnsafePath),
            (binary, FilePreviewDiagnosticCode::UnsupportedType),
            (broken_jpeg, FilePreviewDiagnosticCode::InvalidContent),
            (image, FilePreviewDiagnosticCode::ImageDimensionsTooLarge),
        ] {
            let snapshot = service.preview_selected(project_id.clone(), path, &projects);
            assert_eq!(snapshot.state, FilePreviewState::Unavailable);
            assert_eq!(snapshot.diagnostic_code, Some(expected));
        }
        fs::remove_dir_all(directory).expect("preview test directory must be removed");
        fs::remove_dir_all(outside.parent().expect("outside directory"))
            .expect("outside test directory must be removed");
    }

    #[test]
    fn rejects_malformed_project_identity_and_full_file_apng_marker() {
        let directory = temporary_directory("deep-apng");
        let image = directory.join("animated.png");
        let mut image_bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&1u32.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        append_png_chunk(&mut image_bytes, b"IHDR", &header);
        append_png_chunk(&mut image_bytes, b"tEXt", &vec![b'a'; SNIFF_BYTES]);
        append_png_chunk(&mut image_bytes, b"acTL", &[0; 8]);
        append_png_chunk(&mut image_bytes, b"IDAT", &[]);
        append_png_chunk(&mut image_bytes, b"IEND", &[]);
        fs::write(&image, image_bytes).expect("APNG fixture must be written");
        let (projects, project_id) = attached_project(&directory);
        let service = FilePreviewService::default();

        assert!(!valid_project_id("/private/project"));
        let invalid =
            service.preview_selected("/private/project".to_owned(), image.clone(), &projects);
        assert_eq!(invalid.project_id, None);
        assert_eq!(
            invalid.diagnostic_code,
            Some(FilePreviewDiagnosticCode::InvalidRequest)
        );

        let animated = service.preview_selected(project_id, image, &projects);
        assert_eq!(animated.state, FilePreviewState::Unavailable);
        assert_eq!(
            animated.diagnostic_code,
            Some(FilePreviewDiagnosticCode::UnsupportedType)
        );
        fs::remove_dir_all(directory).expect("preview test directory must be removed");
    }

    #[test]
    fn accepts_an_incomplete_utf8_sniff_suffix_and_bounds_normalized_output() {
        assert!(valid_utf8_prefix(b"text\xe2\x82"));
        assert!(!valid_utf8_prefix(b"text\xe2("));

        let controls = "\u{1}".repeat(MAX_TEXT_BYTES);
        let (normalized, truncated) = normalize_text(&controls);
        assert!(truncated);
        assert!(normalized.len() <= MAX_TEXT_BYTES);
        assert!(normalized.chars().all(|character| character == '\u{fffd}'));

        let lines = "line\n".repeat(MAX_TEXT_LINES + 1);
        let (normalized, truncated) = normalize_text(&lines);
        assert!(truncated);
        assert_eq!(normalized.lines().count(), MAX_TEXT_LINES);
    }

    #[test]
    fn handoff_is_opaque_one_use_and_revalidates_file_identity() {
        let directory = temporary_directory("handoff");
        let file = directory.join("notes.txt");
        fs::write(&file, "review me").expect("text fixture must be written");
        let (projects, project_id) = attached_project(&directory);
        let service = FilePreviewService::default();

        let snapshot = service.preview_selected(project_id.clone(), file.clone(), &projects);
        let open_action_id = snapshot
            .open_action_id
            .clone()
            .expect("ready preview must carry an opaque open action");
        assert!(valid_project_id(&open_action_id));
        assert!(!serde_json::to_string(&snapshot)
            .expect("preview must serialize")
            .contains(directory.to_string_lossy().as_ref()));

        let request = FilePreviewHandoffRequest {
            open_action_id: open_action_id.clone(),
        };
        let claimed = service
            .claim_handoff(&request)
            .expect("first claim must succeed");
        assert_eq!(claimed.path(&projects).expect("path must revalidate"), file);
        assert!(matches!(
            service.claim_handoff(&request),
            Err(FilePreviewDiagnosticCode::HandoffExpired)
        ));

        let replacement = service.preview_selected(project_id, file.clone(), &projects);
        let replacement_request = FilePreviewHandoffRequest {
            open_action_id: replacement
                .open_action_id
                .expect("replacement must carry an action"),
        };
        service.restore_handoff(claimed);
        assert!(matches!(
            service.claim_handoff(&request),
            Err(FilePreviewDiagnosticCode::HandoffExpired)
        ));
        let claimed = service
            .claim_handoff(&replacement_request)
            .expect("replacement claim must succeed");
        let replaced_file = directory.join("replacement.txt");
        fs::write(&replaced_file, "replacement").expect("replacement fixture must be written");
        fs::rename(&replaced_file, &file).expect("replacement must be installed atomically");
        assert!(matches!(
            claimed.path(&projects),
            Err(FilePreviewDiagnosticCode::IdentityChanged)
        ));
        fs::remove_dir_all(directory).expect("preview test directory must be removed");
    }

    #[test]
    fn shared_preview_fixture_matches_the_native_contract() {
        let snapshot: FilePreviewSnapshot =
            serde_json::from_str(include_str!("../../../fixtures/file-preview.json"))
                .expect("shared file-preview fixture must parse");
        assert_eq!(snapshot.schema_version, FILE_PREVIEW_SCHEMA_VERSION);
        assert_eq!(snapshot.state, FilePreviewState::Ready);
        assert_eq!(snapshot.kind, Some(FilePreviewKind::Text));
        assert!(snapshot
            .open_action_id
            .as_deref()
            .is_some_and(valid_project_id));
        assert_eq!(
            snapshot.rendering,
            Some(FilePreviewRendering::NormalizedText)
        );
        assert!(
            serde_json::from_value::<FilePreviewHandoffRequest>(serde_json::json!({
                "openActionId": snapshot.open_action_id,
                "path": "/private/file"
            }))
            .is_err()
        );
    }
}
