use serde::{Deserialize, Serialize};

pub const CONVERSATION_ATTACHMENT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationAttachmentState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationAttachmentSource {
    NativePicker,
    DragDrop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationAttachmentDiagnosticCode {
    InvalidRequest,
    ProjectNotFound,
    ProjectUnavailable,
    ProjectIdentityChanged,
    ProjectNotWritable,
    StagingUnavailable,
    TooManyFiles,
    FileTooLarge,
    UnsupportedType,
    InvalidContent,
    UnsafeName,
    ReadFailed,
    AttachmentNotFound,
    AttachmentExpired,
    CleanupFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationAttachmentSummary {
    pub attachment_id: String,
    pub display_name: String,
    pub source: ConversationAttachmentSource,
    pub mime_type: String,
    pub byte_size: u64,
    pub image_width: u32,
    pub image_height: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationAttachmentSnapshot {
    pub schema_version: u16,
    pub state: ConversationAttachmentState,
    pub project_id: Option<String>,
    pub attachments: Vec<ConversationAttachmentSummary>,
    pub diagnostic_code: Option<ConversationAttachmentDiagnosticCode>,
}

impl ConversationAttachmentSnapshot {
    pub(crate) fn unavailable(
        project_id: Option<String>,
        diagnostic_code: ConversationAttachmentDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: CONVERSATION_ATTACHMENT_SCHEMA_VERSION,
            state: ConversationAttachmentState::Unavailable,
            project_id,
            attachments: Vec::new(),
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationAttachmentDropRequest {
    pub project_id: String,
    pub files: Vec<ConversationAttachmentDropFile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationAttachmentDropFile {
    pub display_name: String,
    pub declared_mime_type: String,
    pub base64_data: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationAttachmentCancelRequest {
    pub project_id: String,
    pub attachment_ids: Vec<String>,
}
