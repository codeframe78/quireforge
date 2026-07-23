use serde::{Deserialize, Serialize};

pub const FILE_PREVIEW_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilePreviewHandoffRequest {
    pub open_action_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePreviewState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePreviewKind {
    Text,
    Image,
    Pdf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePreviewRendering {
    NormalizedText,
    BoundedImage,
    MetadataOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePreviewDiagnosticCode {
    InvalidRequest,
    ProjectNotFound,
    DirectoryUnavailable,
    IdentityChanged,
    PickerUnavailable,
    OutsideProject,
    UnsafePath,
    UnsupportedType,
    FileTooLarge,
    ReadFailed,
    InvalidContent,
    ImageDimensionsTooLarge,
    HandoffExpired,
    OpenFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePreviewSnapshot {
    pub schema_version: u16,
    pub state: FilePreviewState,
    pub project_id: Option<String>,
    pub display_path: Option<String>,
    pub kind: Option<FilePreviewKind>,
    pub rendering: Option<FilePreviewRendering>,
    pub mime_type: Option<String>,
    pub byte_size: Option<u64>,
    pub truncated: bool,
    pub text_content: Option<String>,
    pub image_data_url: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub open_action_id: Option<String>,
    pub diagnostic_code: Option<FilePreviewDiagnosticCode>,
}

impl FilePreviewSnapshot {
    pub(crate) fn empty(project_id: Option<String>) -> Self {
        Self {
            schema_version: FILE_PREVIEW_SCHEMA_VERSION,
            state: FilePreviewState::Empty,
            project_id,
            display_path: None,
            kind: None,
            rendering: None,
            mime_type: None,
            byte_size: None,
            truncated: false,
            text_content: None,
            image_data_url: None,
            image_width: None,
            image_height: None,
            open_action_id: None,
            diagnostic_code: None,
        }
    }

    pub(crate) fn unavailable(
        project_id: Option<String>,
        diagnostic_code: FilePreviewDiagnosticCode,
    ) -> Self {
        Self {
            diagnostic_code: Some(diagnostic_code),
            state: FilePreviewState::Unavailable,
            ..Self::empty(project_id)
        }
    }
}
