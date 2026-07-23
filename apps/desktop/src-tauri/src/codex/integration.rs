use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const INTEGRATION_SCHEMA_VERSION: u16 = 2;
pub const INTEGRATION_ADAPTER_VERSION: &str = "codex-integration-v2";
pub const INTEGRATION_MUTATION_SCHEMA_VERSION: u16 = 1;
pub const INTEGRATION_CONTROL_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationCatalogSnapshot {
    pub schema_version: u16,
    pub adapter_version: String,
    pub cli_version: String,
    pub catalog_state: IntegrationAvailability,
    pub capabilities: Vec<IntegrationCapability>,
    pub entries: Vec<IntegrationEntry>,
    pub scheduled_tasks: Vec<ScheduledTaskTemplate>,
    pub policy: IntegrationPolicySnapshot,
    pub dynamic_tool: DynamicToolContract,
    pub refresh_reasons: Vec<IntegrationRefreshReason>,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IntegrationContractError {
    #[error("integration contract version is invalid")]
    InvalidVersion,
    #[error("integration contract exceeds a size bound")]
    BoundsExceeded,
    #[error("integration contract contains an invalid identity")]
    InvalidIdentity,
    #[error("integration contract contains unsafe display text")]
    UnsafeDisplayText,
    #[error("integration contract contains a duplicate identity")]
    DuplicateIdentity,
    #[error("integration entry references an unknown capability")]
    UnknownCapability,
    #[error("scheduled task references an unknown plugin")]
    UnknownSourcePlugin,
    #[error("integration availability and diagnostics are inconsistent")]
    InconsistentState,
    #[error("mutating integration capability lacks confirmation")]
    ConfirmationRequired,
    #[error("dynamic-tool lifecycle is inconsistent")]
    InvalidDynamicTool,
    #[error("scheduled task schedule is invalid")]
    InvalidSchedule,
}

impl IntegrationCatalogSnapshot {
    pub fn validate(&self) -> Result<(), IntegrationContractError> {
        if self.schema_version != INTEGRATION_SCHEMA_VERSION
            || self.adapter_version != INTEGRATION_ADAPTER_VERSION
            || !is_cli_version(&self.cli_version)
        {
            return Err(IntegrationContractError::InvalidVersion);
        }
        if self.capabilities.len() > 128
            || self.entries.len() > 512
            || self.scheduled_tasks.len() > 256
            || self.refresh_reasons.len() > 4
        {
            return Err(IntegrationContractError::BoundsExceeded);
        }

        let mut capability_ids = HashSet::with_capacity(self.capabilities.len());
        for capability in &self.capabilities {
            if !is_identifier(&capability.id, 128) {
                return Err(IntegrationContractError::InvalidIdentity);
            }
            if !capability_ids.insert(capability.id.as_str()) {
                return Err(IntegrationContractError::DuplicateIdentity);
            }
            validate_availability(capability.availability, &capability.diagnostic_code)?;
            if capability.mutating != capability.requires_confirmation {
                return Err(IntegrationContractError::ConfirmationRequired);
            }
        }

        let mut entry_ids = HashSet::with_capacity(self.entries.len());
        for entry in &self.entries {
            if !is_identifier(&entry.id, 128) {
                return Err(IntegrationContractError::InvalidIdentity);
            }
            if !entry_ids.insert(entry.id.as_str()) {
                return Err(IntegrationContractError::DuplicateIdentity);
            }
            validate_display(&entry.display_name, 128)?;
            validate_display(&entry.summary, 320)?;
            validate_optional_display(entry.publisher.as_deref(), 128)?;
            if let Some(version) = entry.version.as_deref() {
                if !is_protocol_identifier(version, 64) {
                    return Err(IntegrationContractError::InvalidIdentity);
                }
            }
            if entry.capability_ids.len() > 32
                || entry.permissions.len() > 64
                || entry.requirements.len() > 64
                || entry.health.diagnostic_codes.len() > 16
            {
                return Err(IntegrationContractError::BoundsExceeded);
            }

            let mut entry_capabilities = HashSet::with_capacity(entry.capability_ids.len());
            for capability_id in &entry.capability_ids {
                if !capability_ids.contains(capability_id.as_str()) {
                    return Err(IntegrationContractError::UnknownCapability);
                }
                if !entry_capabilities.insert(capability_id.as_str()) {
                    return Err(IntegrationContractError::DuplicateIdentity);
                }
            }
            for permission in &entry.permissions {
                validate_display(&permission.target, 160)?;
            }
            for requirement in &entry.requirements {
                validate_display(&requirement.name, 128)?;
                validate_optional_display(requirement.detail.as_deref(), 240)?;
                if matches!(
                    requirement.state,
                    IntegrationRequirementState::Missing | IntegrationRequirementState::Blocked
                ) && requirement.detail.is_none()
                {
                    return Err(IntegrationContractError::InconsistentState);
                }
            }
            validate_optional_display(entry.policy.reason.as_deref(), 240)?;
            validate_health(&entry.health)?;
        }

        let scheduled_capability = self
            .capabilities
            .iter()
            .find(|capability| capability.id == "scheduled-task.catalog")
            .ok_or(IntegrationContractError::UnknownCapability)?;
        if scheduled_capability.domain != IntegrationDomain::ScheduledTask
            || scheduled_capability.operation != IntegrationOperation::Discover
            || scheduled_capability.mutating
            || scheduled_capability.requires_confirmation
        {
            return Err(IntegrationContractError::UnknownCapability);
        }
        let plugin_ids = self
            .entries
            .iter()
            .filter(|entry| entry.kind == IntegrationEntryKind::Plugin)
            .map(|entry| entry.id.as_str())
            .collect::<HashSet<_>>();
        let mut task_ids = HashSet::with_capacity(self.scheduled_tasks.len());
        for task in &self.scheduled_tasks {
            if !is_identifier(&task.id, 128) || !is_identifier(&task.source_plugin_id, 128) {
                return Err(IntegrationContractError::InvalidIdentity);
            }
            if !task_ids.insert(task.id.as_str()) {
                return Err(IntegrationContractError::DuplicateIdentity);
            }
            if !plugin_ids.contains(task.source_plugin_id.as_str()) {
                return Err(IntegrationContractError::UnknownSourcePlugin);
            }
            validate_display(&task.name, 128)?;
            validate_display(&task.prompt_preview, 1200)?;
            task.schedule.validate()?;
        }

        if self.catalog_state == IntegrationAvailability::Ready
            && self
                .capabilities
                .iter()
                .any(|capability| capability.availability != IntegrationAvailability::Ready)
        {
            return Err(IntegrationContractError::InconsistentState);
        }
        if !self.policy.mutation_confirmation_required {
            return Err(IntegrationContractError::ConfirmationRequired);
        }

        self.validate_dynamic_tool(&capability_ids)
    }

    fn validate_dynamic_tool(
        &self,
        capability_ids: &HashSet<&str>,
    ) -> Result<(), IntegrationContractError> {
        let contract = &self.dynamic_tool;
        validate_availability(contract.state, &contract.diagnostic_code)?;
        if contract.route != IntegrationRoute::AppServer
            || contract.registration_method != "thread/start"
            || contract.invocation_method != "item/tool/call"
            || contract.response_correlation != "json-rpc-request-id"
            || contract.registration_scope != "thread"
            || contract.current_turn_model_mutable
            || contract.output_content_kinds.is_empty()
            || contract.output_content_kinds.len() > 3
            || !capability_ids.contains("dynamic-tool.lifecycle")
        {
            return Err(IntegrationContractError::InvalidDynamicTool);
        }
        let dynamic_capability = self
            .capabilities
            .iter()
            .find(|capability| capability.id == "dynamic-tool.lifecycle")
            .ok_or(IntegrationContractError::InvalidDynamicTool)?;
        if contract.state == IntegrationAvailability::Ready
            && (dynamic_capability.availability != IntegrationAvailability::Ready
                || dynamic_capability.implementation != IntegrationImplementation::ContractOnly)
        {
            return Err(IntegrationContractError::InvalidDynamicTool);
        }

        let unique_content = contract
            .output_content_kinds
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        if unique_content.len() != contract.output_content_kinds.len() {
            return Err(IntegrationContractError::DuplicateIdentity);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskTemplate {
    pub id: String,
    pub source_plugin_id: String,
    pub name: String,
    pub prompt_preview: String,
    pub prompt_truncated: bool,
    pub schedule: ScheduledTaskSchedule,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ScheduledTaskSchedule {
    #[serde(rename = "hourly")]
    Hourly {
        #[serde(rename = "intervalHours")]
        interval_hours: u32,
        days: Option<Vec<ScheduledTaskWeekday>>,
    },
    #[serde(rename = "daily")]
    Daily { time: String },
    #[serde(rename = "weekdays")]
    Weekdays { time: String },
    #[serde(rename = "weekly")]
    Weekly {
        days: Vec<ScheduledTaskWeekday>,
        time: String,
    },
}

impl ScheduledTaskSchedule {
    pub(crate) fn validate(&self) -> Result<(), IntegrationContractError> {
        match self {
            Self::Hourly {
                interval_hours,
                days,
            } => {
                if !(1..=168).contains(interval_hours) {
                    return Err(IntegrationContractError::InvalidSchedule);
                }
                if let Some(days) = days {
                    validate_schedule_days(days, false)?;
                }
            }
            Self::Daily { time } | Self::Weekdays { time } => {
                if !is_schedule_time(time) {
                    return Err(IntegrationContractError::InvalidSchedule);
                }
            }
            Self::Weekly { days, time } => {
                validate_schedule_days(days, true)?;
                if !is_schedule_time(time) {
                    return Err(IntegrationContractError::InvalidSchedule);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ScheduledTaskWeekday {
    MO,
    TU,
    WE,
    TH,
    FR,
    SA,
    SU,
}

fn validate_schedule_days(
    days: &[ScheduledTaskWeekday],
    require_nonempty: bool,
) -> Result<(), IntegrationContractError> {
    if days.len() > 7 || (require_nonempty && days.is_empty()) {
        return Err(IntegrationContractError::InvalidSchedule);
    }
    let unique = days.iter().copied().collect::<HashSet<_>>();
    if unique.len() != days.len() {
        return Err(IntegrationContractError::InvalidSchedule);
    }
    Ok(())
}

fn is_schedule_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 5
        && bytes[2] == b':'
        && bytes[..2].iter().all(u8::is_ascii_digit)
        && bytes[3..].iter().all(u8::is_ascii_digit)
        && value[..2].parse::<u8>().is_ok_and(|hours| hours < 24)
        && value[3..].parse::<u8>().is_ok_and(|minutes| minutes < 60)
}

fn validate_availability(
    availability: IntegrationAvailability,
    diagnostic_code: &Option<String>,
) -> Result<(), IntegrationContractError> {
    if let Some(code) = diagnostic_code {
        if !is_identifier(code, 64) {
            return Err(IntegrationContractError::InvalidIdentity);
        }
    }
    let diagnostics_expected = matches!(
        availability,
        IntegrationAvailability::Degraded
            | IntegrationAvailability::Blocked
            | IntegrationAvailability::Unavailable
    );
    if (availability == IntegrationAvailability::Ready && diagnostic_code.is_some())
        || (diagnostics_expected && diagnostic_code.is_none())
    {
        return Err(IntegrationContractError::InconsistentState);
    }
    Ok(())
}

fn validate_health(health: &IntegrationHealth) -> Result<(), IntegrationContractError> {
    for code in &health.diagnostic_codes {
        if !is_identifier(code, 64) {
            return Err(IntegrationContractError::InvalidIdentity);
        }
    }
    let diagnostics_expected = !matches!(
        health.state,
        IntegrationAvailability::Ready | IntegrationAvailability::Unknown
    );
    if (health.state == IntegrationAvailability::Ready && !health.diagnostic_codes.is_empty())
        || (diagnostics_expected && health.diagnostic_codes.is_empty())
    {
        return Err(IntegrationContractError::InconsistentState);
    }
    Ok(())
}

fn validate_optional_display(
    value: Option<&str>,
    maximum: usize,
) -> Result<(), IntegrationContractError> {
    if let Some(value) = value {
        validate_display(value, maximum)?;
    }
    Ok(())
}

fn validate_display(value: &str, maximum: usize) -> Result<(), IntegrationContractError> {
    if value.is_empty() || value.len() > maximum || value.chars().any(is_unsafe_display_character) {
        return Err(IntegrationContractError::UnsafeDisplayText);
    }
    Ok(())
}

fn is_unsafe_display_character(character: char) -> bool {
    let code = u32::from(character);
    character.is_control()
        || (0x7f..=0x9f).contains(&code)
        || (0x200b..=0x200f).contains(&code)
        || (0x202a..=0x202e).contains(&code)
        || (0x2060..=0x206f).contains(&code)
        || code == 0xfeff
}

fn is_identifier(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

fn is_protocol_identifier(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')
        })
}

fn is_cli_version(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }
    let (core, suffix) = value
        .split_once(['-', '+'])
        .map_or((value, None), |(core, suffix)| (core, Some(suffix)));
    if suffix.is_some_and(|suffix| {
        suffix.is_empty()
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    }) {
        return false;
    }
    let segments = core.split('.').collect::<Vec<_>>();
    segments.len() == 3
        && segments
            .iter()
            .all(|segment| !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_digit()))
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationCapability {
    pub id: String,
    pub domain: IntegrationDomain,
    pub operation: IntegrationOperation,
    pub route: IntegrationRoute,
    pub stability: IntegrationStability,
    pub availability: IntegrationAvailability,
    pub implementation: IntegrationImplementation,
    pub mutating: bool,
    pub requires_confirmation: bool,
    pub diagnostic_code: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationDomain {
    Connector,
    Plugin,
    Marketplace,
    Skill,
    Mcp,
    Policy,
    ScheduledTask,
    DynamicTool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationOperation {
    Discover,
    Inspect,
    Install,
    Remove,
    Configure,
    Authorize,
    Health,
    Invoke,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationRoute {
    AppServer,
    Cli,
    Native,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationStability {
    Stable,
    StableMethodExperimentalServer,
    Experimental,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationAvailability {
    Ready,
    Degraded,
    Blocked,
    Unavailable,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationImplementation {
    ContractOnly,
    Ready,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationEntry {
    pub id: String,
    pub kind: IntegrationEntryKind,
    pub display_name: String,
    pub summary: String,
    pub scope: IntegrationScope,
    pub source: IntegrationSource,
    pub installation: IntegrationInstallationState,
    pub enablement: IntegrationEnablementState,
    pub authentication: IntegrationAuthenticationState,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub capability_ids: Vec<String>,
    pub permissions: Vec<IntegrationPermission>,
    pub requirements: Vec<IntegrationRequirement>,
    pub policy: IntegrationEntryPolicy,
    pub health: IntegrationHealth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationEntryKind {
    Connector,
    Plugin,
    Marketplace,
    Skill,
    McpServer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationScope {
    Account,
    User,
    Project,
    Managed,
    Remote,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationSource {
    Official,
    Marketplace,
    Local,
    Repository,
    Configuration,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationInstallationState {
    Available,
    Installed,
    NotApplicable,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationEnablementState {
    Enabled,
    Disabled,
    Blocked,
    NotApplicable,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationAuthenticationState {
    Connected,
    NotConnected,
    Required,
    NotApplicable,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPermission {
    pub kind: IntegrationPermissionKind,
    pub access: IntegrationPermissionAccess,
    pub target: String,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationPermissionKind {
    Filesystem,
    Network,
    Account,
    Tool,
    Hook,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationPermissionAccess {
    Read,
    Write,
    Execute,
    Authorize,
    Connect,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationRequirement {
    pub kind: IntegrationRequirementKind,
    pub name: String,
    pub state: IntegrationRequirementState,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationRequirementKind {
    Binary,
    Configuration,
    Network,
    Platform,
    Policy,
    Authentication,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationRequirementState {
    Satisfied,
    Missing,
    Blocked,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationEntryPolicy {
    pub state: IntegrationPolicyState,
    pub managed: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationPolicyState {
    Allowed,
    ApprovalRequired,
    Blocked,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationHealth {
    pub state: IntegrationAvailability,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPolicySnapshot {
    pub state: IntegrationAvailability,
    pub source: IntegrationPolicySource,
    pub permission_profiles: IntegrationAvailability,
    pub managed_requirements_present: bool,
    pub mutation_confirmation_required: bool,
    pub installation: IntegrationPolicyState,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationPolicySource {
    ConfigRequirements,
    UserConfiguration,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct DynamicToolContract {
    pub state: IntegrationAvailability,
    pub route: IntegrationRoute,
    pub registration_method: String,
    pub invocation_method: String,
    pub response_correlation: String,
    pub registration_scope: String,
    pub supports_namespaces: bool,
    pub output_content_kinds: Vec<DynamicToolContentKind>,
    pub current_turn_model_mutable: bool,
    pub diagnostic_code: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DynamicToolContentKind {
    Text,
    Image,
    Audio,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationRefreshReason {
    AppListUpdated,
    SkillsChanged,
    McpStatusUpdated,
    ConfigWarning,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMutationOperation {
    PluginInstall,
    PluginRemove,
    MarketplaceAdd,
    MarketplaceRemove,
    MarketplaceUpgrade,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationMutationPreviewRequest {
    pub operation: IntegrationMutationOperation,
    pub target_entry_id: Option<String>,
    pub repository: Option<String>,
    pub reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationMutationConfirmRequest {
    pub confirmation_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMutationPreviewState {
    Ready,
    Blocked,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMutationResultState {
    Applied,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMutationWarning {
    LocalSource,
    RepositorySource,
    PackageRegistrySource,
    NetworkAccess,
    HookExecution,
    McpServers,
    ConnectorApps,
    SkillContent,
    AuthenticationAfterInstall,
    MutableRemoteSource,
    RemovesCachedPlugin,
    RemovesMarketplaceSnapshot,
    UpdatesMarketplaceSnapshot,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMutationDiagnosticCode {
    InvalidRequest,
    CliUnavailable,
    VersionUnsupported,
    CatalogUnavailable,
    TargetNotFound,
    OperationUnavailable,
    PolicyBlocked,
    SourceInvalid,
    SourceUnpinned,
    SourceUnreviewable,
    CapacityReached,
    ConfirmationExpired,
    StalePreview,
    MutationFailed,
    ResponseInvalid,
    PostconditionFailed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationControlOperation {
    ConnectorAuthorize,
    SkillEnable,
    SkillDisable,
    McpAuthorize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControlPreviewRequest {
    pub operation: IntegrationControlOperation,
    pub target_entry_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControlConfirmationRequest {
    pub confirmation_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControlActionRequest {
    pub action_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationControlPreviewState {
    Ready,
    Blocked,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationControlResultState {
    Applied,
    HandoffReady,
    Pending,
    Completed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationControlWarning {
    OpensExternalBrowser,
    AccountAuthorization,
    NetworkAuthorization,
    ChangesCodexConfiguration,
    ProjectScoped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationControlDiagnosticCode {
    InvalidRequest,
    CliUnavailable,
    VersionUnsupported,
    CatalogUnavailable,
    TargetNotFound,
    OperationUnavailable,
    PolicyBlocked,
    CapacityReached,
    ConfirmationExpired,
    StalePreview,
    HandoffUnavailable,
    AuthorizationFailed,
    MutationFailed,
    ResponseInvalid,
    PostconditionFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControlPreviewSnapshot {
    pub schema_version: u16,
    pub state: IntegrationControlPreviewState,
    pub operation: IntegrationControlOperation,
    pub target_entry_id: String,
    pub target_display_name: Option<String>,
    pub permissions: Vec<IntegrationPermission>,
    pub warnings: Vec<IntegrationControlWarning>,
    pub confirmation_id: Option<String>,
    pub diagnostic_code: Option<IntegrationControlDiagnosticCode>,
}

impl IntegrationControlPreviewSnapshot {
    pub fn unavailable(
        request: &IntegrationControlPreviewRequest,
        state: IntegrationControlPreviewState,
        diagnostic_code: IntegrationControlDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
            state,
            operation: request.operation,
            target_entry_id: request.target_entry_id.clone(),
            target_display_name: None,
            permissions: Vec::new(),
            warnings: Vec::new(),
            confirmation_id: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControlResultSnapshot {
    pub schema_version: u16,
    pub state: IntegrationControlResultState,
    pub operation: Option<IntegrationControlOperation>,
    pub target_entry_id: Option<String>,
    pub action_id: Option<String>,
    pub browser_handoff_available: bool,
    pub catalog_refresh_required: bool,
    pub diagnostic_code: Option<IntegrationControlDiagnosticCode>,
}

impl IntegrationControlResultSnapshot {
    pub fn unavailable(
        operation: Option<IntegrationControlOperation>,
        target_entry_id: Option<String>,
        diagnostic_code: IntegrationControlDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: INTEGRATION_CONTROL_SCHEMA_VERSION,
            state: IntegrationControlResultState::Unavailable,
            operation,
            target_entry_id,
            action_id: None,
            browser_handoff_available: false,
            catalog_refresh_required: false,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationMutationPreviewSnapshot {
    pub schema_version: u16,
    pub state: IntegrationMutationPreviewState,
    pub operation: IntegrationMutationOperation,
    pub target_entry_id: Option<String>,
    pub target_display_name: Option<String>,
    pub source: IntegrationSource,
    pub permissions: Vec<IntegrationPermission>,
    pub warnings: Vec<IntegrationMutationWarning>,
    pub destructive: bool,
    pub confirmation_id: Option<String>,
    pub diagnostic_code: Option<IntegrationMutationDiagnosticCode>,
}

impl IntegrationMutationPreviewSnapshot {
    pub fn unavailable(
        request: &IntegrationMutationPreviewRequest,
        state: IntegrationMutationPreviewState,
        diagnostic_code: IntegrationMutationDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: INTEGRATION_MUTATION_SCHEMA_VERSION,
            state,
            operation: request.operation,
            target_entry_id: request.target_entry_id.clone(),
            target_display_name: None,
            source: IntegrationSource::Unknown,
            permissions: Vec::new(),
            warnings: Vec::new(),
            destructive: matches!(
                request.operation,
                IntegrationMutationOperation::PluginRemove
                    | IntegrationMutationOperation::MarketplaceRemove
            ),
            confirmation_id: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationMutationResultSnapshot {
    pub schema_version: u16,
    pub state: IntegrationMutationResultState,
    pub operation: Option<IntegrationMutationOperation>,
    pub target_entry_id: Option<String>,
    pub catalog_refresh_required: bool,
    pub diagnostic_code: Option<IntegrationMutationDiagnosticCode>,
}

impl IntegrationMutationResultSnapshot {
    pub fn unavailable(
        operation: Option<IntegrationMutationOperation>,
        target_entry_id: Option<String>,
        diagnostic_code: IntegrationMutationDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: INTEGRATION_MUTATION_SCHEMA_VERSION,
            state: IntegrationMutationResultState::Unavailable,
            operation,
            target_entry_id,
            catalog_refresh_required: false,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct IntegrationMutationFixture {
        preview: IntegrationMutationPreviewSnapshot,
        result: IntegrationMutationResultSnapshot,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct IntegrationControlFixture {
        preview: IntegrationControlPreviewSnapshot,
        result: IntegrationControlResultSnapshot,
    }

    #[test]
    fn shared_fixture_matches_the_native_contract() {
        let raw = include_str!("../../../fixtures/integration-catalog.json");
        let snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        let original: serde_json::Value =
            serde_json::from_str(raw).expect("integration fixture must be valid JSON");
        let round_trip = serde_json::to_value(&snapshot)
            .expect("normalized integration contract must serialize");

        assert_eq!(snapshot.schema_version, INTEGRATION_SCHEMA_VERSION);
        assert_eq!(snapshot.adapter_version, INTEGRATION_ADAPTER_VERSION);
        assert_eq!(snapshot.capabilities.len(), 16);
        assert_eq!(snapshot.entries.len(), 5);
        assert_eq!(snapshot.scheduled_tasks.len(), 1);
        assert!(!snapshot.dynamic_tool.current_turn_model_mutable);
        snapshot.validate().expect("shared fixture must be strict");
        assert_eq!(round_trip, original);
    }

    #[test]
    fn rejects_unknown_fields_and_unsafe_contract_invariants() {
        let raw = include_str!("../../../fixtures/integration-catalog.json");
        let mut value: serde_json::Value =
            serde_json::from_str(raw).expect("integration fixture must be valid JSON");
        value
            .as_object_mut()
            .expect("fixture root must be an object")
            .insert("accountId".to_owned(), "raw-identity".into());
        assert!(serde_json::from_value::<IntegrationCatalogSnapshot>(value).is_err());

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot
            .capabilities
            .iter_mut()
            .find(|capability| capability.id == "plugin.install")
            .expect("fixture must include plugin installation")
            .requires_confirmation = false;
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::ConfirmationRequired)
        );

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot.entries[0].display_name = "unsafe\u{202e}name".to_owned();
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::UnsafeDisplayText)
        );

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot.dynamic_tool.current_turn_model_mutable = true;
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::InvalidDynamicTool)
        );

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot.cli_version = "0.145.0-".to_owned();
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::InvalidVersion)
        );

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot.scheduled_tasks[0].source_plugin_id = "plugin:missing".to_owned();
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::UnknownSourcePlugin)
        );

        let mut snapshot: IntegrationCatalogSnapshot =
            serde_json::from_str(raw).expect("integration fixture must match native types");
        snapshot.scheduled_tasks[0].schedule = ScheduledTaskSchedule::Daily {
            time: "24:00".to_owned(),
        };
        assert_eq!(
            snapshot.validate(),
            Err(IntegrationContractError::InvalidSchedule)
        );
    }

    #[test]
    fn shared_mutation_fixture_matches_the_native_contract() {
        let raw = include_str!("../../../fixtures/integration-mutation.json");
        let fixture: IntegrationMutationFixture =
            serde_json::from_str(raw).expect("mutation fixture must match native types");
        let original: serde_json::Value =
            serde_json::from_str(raw).expect("mutation fixture must be valid JSON");
        let round_trip =
            serde_json::to_value(&fixture).expect("normalized mutation contract must serialize");

        assert_eq!(
            fixture.preview.schema_version,
            INTEGRATION_MUTATION_SCHEMA_VERSION
        );
        assert_eq!(
            fixture.preview.state,
            IntegrationMutationPreviewState::Ready
        );
        assert_eq!(
            fixture.result.state,
            IntegrationMutationResultState::Applied
        );
        assert_eq!(round_trip, original);
    }

    #[test]
    fn shared_control_fixture_matches_the_native_contract() {
        let raw = include_str!("../../../fixtures/integration-control.json");
        let fixture: IntegrationControlFixture =
            serde_json::from_str(raw).expect("control fixture must match native types");
        let original: serde_json::Value =
            serde_json::from_str(raw).expect("control fixture must be valid JSON");
        let round_trip =
            serde_json::to_value(&fixture).expect("normalized control contract must serialize");

        assert_eq!(
            fixture.preview.schema_version,
            INTEGRATION_CONTROL_SCHEMA_VERSION
        );
        assert_eq!(fixture.preview.state, IntegrationControlPreviewState::Ready);
        assert_eq!(
            fixture.result.state,
            IntegrationControlResultState::HandoffReady
        );
        assert_eq!(round_trip, original);
    }
}
