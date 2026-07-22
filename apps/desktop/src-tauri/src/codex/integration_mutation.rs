use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use serde_json::{Map, Value};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::Mutex,
    time::timeout,
};
use url::Url;
use uuid::Uuid;

use super::{
    error::CodexAdapterError,
    integration::{
        IntegrationAvailability, IntegrationCatalogSnapshot, IntegrationEntry,
        IntegrationImplementation, IntegrationInstallationState, IntegrationMutationConfirmRequest,
        IntegrationMutationDiagnosticCode, IntegrationMutationOperation,
        IntegrationMutationPreviewRequest, IntegrationMutationPreviewSnapshot,
        IntegrationMutationPreviewState, IntegrationMutationResultSnapshot,
        IntegrationMutationResultState, IntegrationMutationWarning, IntegrationPermission,
        IntegrationPermissionAccess, IntegrationPermissionKind, IntegrationPolicySnapshot,
        IntegrationPolicyState, IntegrationSource, INTEGRATION_MUTATION_SCHEMA_VERSION,
    },
    probe::probe_cli_version,
};

const SUPPORTED_CLI_MINOR: (u64, u64) = (0, 145);
const CONFIRMATION_TTL: Duration = Duration::from_secs(5 * 60);
const CLI_MUTATION_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CLI_JSON_BYTES: usize = 1024 * 1024;
const MAX_PLUGIN_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_PENDING_MUTATIONS: usize = 32;

pub struct IntegrationMutationService {
    command: CliCommand,
    cli_version_override: Option<String>,
    state: Mutex<MutationState>,
    execution: Mutex<()>,
}

#[derive(Clone)]
struct CliCommand {
    program: String,
    prefix_arguments: Vec<String>,
    environment: Vec<(OsString, OsString)>,
}

#[derive(Default)]
struct MutationState {
    pending: HashMap<String, PendingMutation>,
}

struct PendingMutation {
    operation: IntegrationMutationOperation,
    target_entry_id: Option<String>,
    cli_version: String,
    policy: IntegrationPolicySnapshot,
    entry: Option<IntegrationEntry>,
    evidence: PendingEvidence,
    expires_at: Instant,
}

enum PendingEvidence {
    Plugin {
        raw: Value,
        plugin_id: String,
        plugin_name: String,
        marketplace_name: String,
        local_review: Option<LocalPluginEvidence>,
    },
    Marketplace {
        raw: Value,
        marketplace_name: String,
    },
    MarketplaceAdd {
        repository: String,
        reference: String,
    },
}

#[derive(Eq, PartialEq)]
struct LocalPluginEvidence {
    canonical_root: PathBuf,
    manifest: Vec<u8>,
    default_hooks_present: bool,
}

struct PluginSourceReview {
    source: IntegrationSource,
    permissions: Vec<IntegrationPermission>,
    warnings: Vec<IntegrationMutationWarning>,
    local_evidence: Option<LocalPluginEvidence>,
}

struct PreparedPreview {
    target_entry_id: Option<String>,
    target_display_name: Option<String>,
    source: IntegrationSource,
    permissions: Vec<IntegrationPermission>,
    warnings: Vec<IntegrationMutationWarning>,
    entry: Option<IntegrationEntry>,
    evidence: PendingEvidence,
}

#[derive(Debug)]
struct PreparationFailure {
    state: IntegrationMutationPreviewState,
    diagnostic: IntegrationMutationDiagnosticCode,
}

impl Default for IntegrationMutationService {
    fn default() -> Self {
        Self {
            command: CliCommand {
                program: "codex".to_owned(),
                prefix_arguments: Vec::new(),
                environment: Vec::new(),
            },
            cli_version_override: None,
            state: Mutex::new(MutationState::default()),
            execution: Mutex::new(()),
        }
    }
}

impl IntegrationMutationService {
    pub async fn preview(
        &self,
        request: IntegrationMutationPreviewRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> IntegrationMutationPreviewSnapshot {
        if !valid_request_shape(&request) {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Unavailable,
                IntegrationMutationDiagnosticCode::InvalidRequest,
            );
        }
        if !supports_mutation_routes(&catalog.cli_version) {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Unavailable,
                IntegrationMutationDiagnosticCode::VersionUnsupported,
            );
        }
        if catalog.catalog_state == IntegrationAvailability::Unavailable {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Unavailable,
                IntegrationMutationDiagnosticCode::CatalogUnavailable,
            );
        }
        if !mutation_policy_allows(&catalog.policy) {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Blocked,
                IntegrationMutationDiagnosticCode::PolicyBlocked,
            );
        }
        if !capability_ready(catalog, request.operation) {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Unavailable,
                IntegrationMutationDiagnosticCode::OperationUnavailable,
            );
        }

        let prepared = match self.prepare(&request, catalog).await {
            Ok(prepared) => prepared,
            Err(failure) => {
                return IntegrationMutationPreviewSnapshot::unavailable(
                    &request,
                    failure.state,
                    failure.diagnostic,
                );
            }
        };
        let confirmation_id = Uuid::now_v7().to_string();
        let mut state = self.state.lock().await;
        clear_expired(&mut state);
        if state.pending.len() >= MAX_PENDING_MUTATIONS {
            return IntegrationMutationPreviewSnapshot::unavailable(
                &request,
                IntegrationMutationPreviewState::Unavailable,
                IntegrationMutationDiagnosticCode::CapacityReached,
            );
        }
        state.pending.insert(
            confirmation_id.clone(),
            PendingMutation {
                operation: request.operation,
                target_entry_id: prepared.target_entry_id.clone(),
                cli_version: catalog.cli_version.clone(),
                policy: catalog.policy.clone(),
                entry: prepared.entry,
                evidence: prepared.evidence,
                expires_at: Instant::now() + CONFIRMATION_TTL,
            },
        );
        IntegrationMutationPreviewSnapshot {
            schema_version: INTEGRATION_MUTATION_SCHEMA_VERSION,
            state: IntegrationMutationPreviewState::Ready,
            operation: request.operation,
            target_entry_id: prepared.target_entry_id,
            target_display_name: prepared.target_display_name,
            source: prepared.source,
            permissions: prepared.permissions,
            warnings: unique_warnings(prepared.warnings),
            destructive: matches!(
                request.operation,
                IntegrationMutationOperation::PluginRemove
                    | IntegrationMutationOperation::MarketplaceRemove
            ),
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn confirm(
        &self,
        request: IntegrationMutationConfirmRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> IntegrationMutationResultSnapshot {
        if !valid_confirmation_id(&request.confirmation_id) {
            return IntegrationMutationResultSnapshot::unavailable(
                None,
                None,
                IntegrationMutationDiagnosticCode::ConfirmationExpired,
            );
        }
        let pending = {
            let mut state = self.state.lock().await;
            clear_expired(&mut state);
            state.pending.remove(&request.confirmation_id)
        };
        let Some(pending) = pending else {
            return IntegrationMutationResultSnapshot::unavailable(
                None,
                None,
                IntegrationMutationDiagnosticCode::ConfirmationExpired,
            );
        };
        if pending.expires_at <= Instant::now() {
            return IntegrationMutationResultSnapshot::unavailable(
                Some(pending.operation),
                pending.target_entry_id,
                IntegrationMutationDiagnosticCode::ConfirmationExpired,
            );
        }

        let _execution = self.execution.lock().await;
        let current_version = match self.cli_version().await {
            Ok(version) => version,
            Err(diagnostic) => {
                return IntegrationMutationResultSnapshot::unavailable(
                    Some(pending.operation),
                    pending.target_entry_id,
                    diagnostic,
                );
            }
        };
        if current_version != pending.cli_version
            || catalog.cli_version != pending.cli_version
            || catalog.policy != pending.policy
            || !mutation_policy_allows(&catalog.policy)
            || !capability_ready(catalog, pending.operation)
            || pending.entry.as_ref().is_some_and(|expected| {
                catalog.entries.iter().find(|entry| entry.id == expected.id) != Some(expected)
            })
        {
            return IntegrationMutationResultSnapshot::unavailable(
                Some(pending.operation),
                pending.target_entry_id,
                IntegrationMutationDiagnosticCode::StalePreview,
            );
        }

        match self.execute(pending).await {
            Ok((operation, target_entry_id)) => IntegrationMutationResultSnapshot {
                schema_version: INTEGRATION_MUTATION_SCHEMA_VERSION,
                state: IntegrationMutationResultState::Applied,
                operation: Some(operation),
                target_entry_id,
                catalog_refresh_required: true,
                diagnostic_code: None,
            },
            Err((operation, target_entry_id, diagnostic)) => {
                IntegrationMutationResultSnapshot::unavailable(
                    Some(operation),
                    target_entry_id,
                    diagnostic,
                )
            }
        }
    }

    async fn prepare(
        &self,
        request: &IntegrationMutationPreviewRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> Result<PreparedPreview, PreparationFailure> {
        match request.operation {
            IntegrationMutationOperation::PluginInstall
            | IntegrationMutationOperation::PluginRemove => {
                self.prepare_plugin(request, catalog).await
            }
            IntegrationMutationOperation::MarketplaceRemove
            | IntegrationMutationOperation::MarketplaceUpgrade => {
                self.prepare_marketplace(request, catalog).await
            }
            IntegrationMutationOperation::MarketplaceAdd => {
                let repository = request.repository.as_deref().expect("validated repository");
                let reference = request.reference.as_deref().expect("validated reference");
                if !valid_repository(repository) {
                    return Err(blocked(IntegrationMutationDiagnosticCode::SourceInvalid));
                }
                if !valid_commit_reference(reference) {
                    return Err(blocked(IntegrationMutationDiagnosticCode::SourceUnpinned));
                }
                Ok(PreparedPreview {
                    target_entry_id: None,
                    target_display_name: Some("Pinned repository marketplace".to_owned()),
                    source: IntegrationSource::Repository,
                    permissions: vec![IntegrationPermission {
                        kind: IntegrationPermissionKind::Network,
                        access: IntegrationPermissionAccess::Read,
                        target: "Pinned marketplace repository".to_owned(),
                        required: true,
                    }],
                    warnings: vec![
                        IntegrationMutationWarning::RepositorySource,
                        IntegrationMutationWarning::NetworkAccess,
                    ],
                    entry: None,
                    evidence: PendingEvidence::MarketplaceAdd {
                        repository: repository.to_owned(),
                        reference: reference.to_owned(),
                    },
                })
            }
        }
    }

    async fn prepare_plugin(
        &self,
        request: &IntegrationMutationPreviewRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> Result<PreparedPreview, PreparationFailure> {
        let target_id = request
            .target_entry_id
            .as_deref()
            .expect("validated target entry");
        let entry = catalog
            .entries
            .iter()
            .find(|entry| entry.id == target_id)
            .cloned()
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::TargetNotFound))?;
        if entry.kind != super::integration::IntegrationEntryKind::Plugin
            || entry.policy.state == IntegrationPolicyState::Blocked
        {
            return Err(blocked(IntegrationMutationDiagnosticCode::PolicyBlocked));
        }

        let plugins = self
            .run_json(&strings(&["plugin", "list", "--available", "--json"]))
            .await
            .map_err(|_| unavailable(IntegrationMutationDiagnosticCode::CliUnavailable))?;
        let raw = find_raw_plugin(&plugins, target_id)
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::TargetNotFound))?;
        let object = raw
            .as_object()
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::ResponseInvalid))?;
        let (plugin_id, plugin_name, marketplace_name, installed) = plugin_identity(object)
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::ResponseInvalid))?;
        match request.operation {
            IntegrationMutationOperation::PluginInstall
                if installed || entry.installation != IntegrationInstallationState::Available =>
            {
                return Err(blocked(
                    IntegrationMutationDiagnosticCode::OperationUnavailable,
                ));
            }
            IntegrationMutationOperation::PluginRemove
                if !installed || entry.installation != IntegrationInstallationState::Installed =>
            {
                return Err(blocked(
                    IntegrationMutationDiagnosticCode::OperationUnavailable,
                ));
            }
            _ => {}
        }

        let mut source_review = if request.operation == IntegrationMutationOperation::PluginInstall
        {
            review_plugin_source(object)?
        } else {
            PluginSourceReview {
                source: entry.source,
                permissions: entry.permissions.clone(),
                warnings: vec![IntegrationMutationWarning::RemovesCachedPlugin],
                local_evidence: None,
            }
        };
        if request.operation == IntegrationMutationOperation::PluginInstall
            && object.get("authPolicy").and_then(Value::as_str) == Some("ON_INSTALL")
        {
            source_review
                .warnings
                .push(IntegrationMutationWarning::AuthenticationAfterInstall);
            source_review.permissions.push(IntegrationPermission {
                kind: IntegrationPermissionKind::Account,
                access: IntegrationPermissionAccess::Authorize,
                target: "Plugin account authorization".to_owned(),
                required: false,
            });
        }
        Ok(PreparedPreview {
            target_entry_id: Some(entry.id.clone()),
            target_display_name: Some(entry.display_name.clone()),
            source: source_review.source,
            permissions: source_review.permissions,
            warnings: source_review.warnings,
            entry: Some(entry),
            evidence: PendingEvidence::Plugin {
                raw,
                plugin_id,
                plugin_name,
                marketplace_name,
                local_review: source_review.local_evidence,
            },
        })
    }

    async fn prepare_marketplace(
        &self,
        request: &IntegrationMutationPreviewRequest,
        catalog: &IntegrationCatalogSnapshot,
    ) -> Result<PreparedPreview, PreparationFailure> {
        let target_id = request
            .target_entry_id
            .as_deref()
            .expect("validated target entry");
        let entry = catalog
            .entries
            .iter()
            .find(|entry| entry.id == target_id)
            .cloned()
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::TargetNotFound))?;
        if entry.kind != super::integration::IntegrationEntryKind::Marketplace
            || entry.policy.state == IntegrationPolicyState::Blocked
        {
            return Err(blocked(IntegrationMutationDiagnosticCode::PolicyBlocked));
        }
        let marketplaces = self
            .run_json(&strings(&["plugin", "marketplace", "list", "--json"]))
            .await
            .map_err(|_| unavailable(IntegrationMutationDiagnosticCode::CliUnavailable))?;
        let raw = find_raw_marketplace(&marketplaces, target_id)
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::TargetNotFound))?;
        let object = raw
            .as_object()
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::ResponseInvalid))?;
        let marketplace_name = object
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| valid_cli_name(value))
            .map(str::to_owned)
            .ok_or_else(|| unavailable(IntegrationMutationDiagnosticCode::ResponseInvalid))?;
        let source = marketplace_source(object)?;
        if request.operation == IntegrationMutationOperation::MarketplaceRemove
            && !is_configured_marketplace(object)
        {
            return Err(blocked(
                IntegrationMutationDiagnosticCode::OperationUnavailable,
            ));
        }
        if request.operation == IntegrationMutationOperation::MarketplaceUpgrade
            && source != IntegrationSource::Repository
        {
            return Err(blocked(
                IntegrationMutationDiagnosticCode::OperationUnavailable,
            ));
        }
        let warning = if request.operation == IntegrationMutationOperation::MarketplaceRemove {
            IntegrationMutationWarning::RemovesMarketplaceSnapshot
        } else {
            IntegrationMutationWarning::UpdatesMarketplaceSnapshot
        };
        let mut warnings = vec![warning];
        if source == IntegrationSource::Repository {
            warnings.push(IntegrationMutationWarning::NetworkAccess);
            if request.operation == IntegrationMutationOperation::MarketplaceUpgrade {
                warnings.push(IntegrationMutationWarning::MutableRemoteSource);
            }
        }
        Ok(PreparedPreview {
            target_entry_id: Some(entry.id.clone()),
            target_display_name: Some(entry.display_name.clone()),
            source,
            permissions: (source == IntegrationSource::Repository)
                .then(|| IntegrationPermission {
                    kind: IntegrationPermissionKind::Network,
                    access: IntegrationPermissionAccess::Read,
                    target: "Configured marketplace repository".to_owned(),
                    required: true,
                })
                .into_iter()
                .collect(),
            warnings,
            entry: Some(entry),
            evidence: PendingEvidence::Marketplace {
                raw,
                marketplace_name,
            },
        })
    }

    async fn execute(
        &self,
        pending: PendingMutation,
    ) -> Result<
        (IntegrationMutationOperation, Option<String>),
        (
            IntegrationMutationOperation,
            Option<String>,
            IntegrationMutationDiagnosticCode,
        ),
    > {
        let operation = pending.operation;
        let target_entry_id = pending.target_entry_id.clone();
        let failure = |diagnostic| (operation, target_entry_id.clone(), diagnostic);
        match pending.evidence {
            PendingEvidence::Plugin {
                raw,
                plugin_id,
                plugin_name,
                marketplace_name,
                local_review,
            } => {
                let plugins = self
                    .run_json(&strings(&["plugin", "list", "--available", "--json"]))
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::CliUnavailable))?;
                if find_raw_plugin(&plugins, target_entry_id.as_deref().unwrap_or_default())
                    != Some(raw.clone())
                {
                    return Err(failure(IntegrationMutationDiagnosticCode::StalePreview));
                }
                if let Some(expected) = local_review {
                    let actual = raw
                        .as_object()
                        .and_then(|plugin| {
                            plugin
                                .get("source")
                                .and_then(Value::as_object)
                                .and_then(|source| source.get("path"))
                                .and_then(Value::as_str)
                                .map(Path::new)
                                .and_then(|path| {
                                    inspect_local_plugin(path, plugin)
                                        .ok()
                                        .map(|(_, _, evidence)| evidence)
                                })
                        })
                        .filter(|actual| actual == &expected);
                    if actual.is_none() {
                        return Err(failure(IntegrationMutationDiagnosticCode::StalePreview));
                    }
                }
                let arguments = match operation {
                    IntegrationMutationOperation::PluginInstall => vec![
                        "plugin".to_owned(),
                        "add".to_owned(),
                        plugin_id.clone(),
                        "--json".to_owned(),
                    ],
                    IntegrationMutationOperation::PluginRemove => vec![
                        "plugin".to_owned(),
                        "remove".to_owned(),
                        plugin_name.clone(),
                        "--marketplace".to_owned(),
                        marketplace_name.clone(),
                        "--json".to_owned(),
                    ],
                    _ => unreachable!("plugin evidence requires a plugin operation"),
                };
                let output = self
                    .run_json(&arguments)
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::MutationFailed))?;
                if !valid_plugin_mutation_output(
                    &output,
                    operation,
                    &plugin_id,
                    &plugin_name,
                    &marketplace_name,
                ) {
                    return Err(failure(IntegrationMutationDiagnosticCode::ResponseInvalid));
                }
                let after = self
                    .run_json(&strings(&["plugin", "list", "--available", "--json"]))
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::PostconditionFailed))?;
                let installed =
                    find_raw_plugin(&after, target_entry_id.as_deref().unwrap_or_default())
                        .and_then(|value| value.get("installed").and_then(Value::as_bool));
                let expected = operation == IntegrationMutationOperation::PluginInstall;
                if installed != Some(expected) {
                    return Err(failure(
                        IntegrationMutationDiagnosticCode::PostconditionFailed,
                    ));
                }
                Ok((operation, target_entry_id))
            }
            PendingEvidence::Marketplace {
                raw,
                marketplace_name,
            } => {
                let before = self
                    .run_json(&strings(&["plugin", "marketplace", "list", "--json"]))
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::CliUnavailable))?;
                if find_raw_marketplace(&before, target_entry_id.as_deref().unwrap_or_default())
                    != Some(raw)
                {
                    return Err(failure(IntegrationMutationDiagnosticCode::StalePreview));
                }
                let arguments = match operation {
                    IntegrationMutationOperation::MarketplaceRemove => vec![
                        "plugin".to_owned(),
                        "marketplace".to_owned(),
                        "remove".to_owned(),
                        marketplace_name.clone(),
                        "--json".to_owned(),
                    ],
                    IntegrationMutationOperation::MarketplaceUpgrade => vec![
                        "plugin".to_owned(),
                        "marketplace".to_owned(),
                        "upgrade".to_owned(),
                        marketplace_name.clone(),
                        "--json".to_owned(),
                    ],
                    _ => unreachable!("marketplace evidence requires configure operation"),
                };
                let output = self
                    .run_json(&arguments)
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::MutationFailed))?;
                if !valid_marketplace_mutation_output(&output, operation, &marketplace_name) {
                    return Err(failure(IntegrationMutationDiagnosticCode::ResponseInvalid));
                }
                let after = self
                    .run_json(&strings(&["plugin", "marketplace", "list", "--json"]))
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::PostconditionFailed))?;
                let remains =
                    find_raw_marketplace(&after, target_entry_id.as_deref().unwrap_or_default())
                        .is_some();
                let expected = operation == IntegrationMutationOperation::MarketplaceUpgrade;
                if remains != expected {
                    return Err(failure(
                        IntegrationMutationDiagnosticCode::PostconditionFailed,
                    ));
                }
                Ok((operation, target_entry_id))
            }
            PendingEvidence::MarketplaceAdd {
                repository,
                reference,
            } => {
                let output = self
                    .run_json(&[
                        "plugin".to_owned(),
                        "marketplace".to_owned(),
                        "add".to_owned(),
                        repository,
                        "--ref".to_owned(),
                        reference,
                        "--json".to_owned(),
                    ])
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::MutationFailed))?;
                let Some(name) = valid_marketplace_add_output(&output) else {
                    return Err(failure(IntegrationMutationDiagnosticCode::ResponseInvalid));
                };
                let new_id = normalized_entry_id("marketplace", &name)
                    .ok_or_else(|| failure(IntegrationMutationDiagnosticCode::ResponseInvalid))?;
                let after = self
                    .run_json(&strings(&["plugin", "marketplace", "list", "--json"]))
                    .await
                    .map_err(|_| failure(IntegrationMutationDiagnosticCode::PostconditionFailed))?;
                if find_raw_marketplace(&after, &new_id).is_none() {
                    return Err(failure(
                        IntegrationMutationDiagnosticCode::PostconditionFailed,
                    ));
                }
                Ok((operation, Some(new_id)))
            }
        }
    }

    async fn cli_version(&self) -> Result<String, IntegrationMutationDiagnosticCode> {
        if let Some(version) = self.cli_version_override.as_ref() {
            return supports_mutation_routes(version)
                .then(|| version.clone())
                .ok_or(IntegrationMutationDiagnosticCode::VersionUnsupported);
        }
        let version = probe_cli_version(&self.command.program)
            .await
            .map_err(|_| IntegrationMutationDiagnosticCode::CliUnavailable)?;
        supports_mutation_routes(&version)
            .then_some(version)
            .ok_or(IntegrationMutationDiagnosticCode::VersionUnsupported)
    }

    async fn run_json(&self, arguments: &[String]) -> Result<Value, CodexAdapterError> {
        run_cli_json(&self.command, arguments).await
    }

    #[cfg(test)]
    fn with_command(
        program: &str,
        prefix_arguments: &[&str],
        environment: Vec<(OsString, OsString)>,
        cli_version: &str,
    ) -> Self {
        Self {
            command: CliCommand {
                program: program.to_owned(),
                prefix_arguments: prefix_arguments
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
                environment,
            },
            cli_version_override: Some(cli_version.to_owned()),
            state: Mutex::new(MutationState::default()),
            execution: Mutex::new(()),
        }
    }
}

async fn run_cli_json(
    command: &CliCommand,
    arguments: &[String],
) -> Result<Value, CodexAdapterError> {
    let mut child = Command::new(&command.program)
        .current_dir("/")
        .args(&command.prefix_arguments)
        .args(arguments)
        .envs(command.environment.iter().cloned())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| CodexAdapterError::CliNotFound)?;
    let stdout = child
        .stdout
        .take()
        .ok_or(CodexAdapterError::ProcessSpawnFailed)?;
    let result = timeout(CLI_MUTATION_TIMEOUT, async {
        let output = read_bounded_output(stdout).await?;
        let status = child
            .wait()
            .await
            .map_err(|_| CodexAdapterError::ProcessExited)?;
        Ok::<_, CodexAdapterError>((output, status.success()))
    })
    .await;
    let (output, success) = match result {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(error);
        }
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(CodexAdapterError::TransportTimeout);
        }
    };
    if !success {
        return Err(CodexAdapterError::RpcRejected);
    }
    serde_json::from_slice(&output).map_err(|_| CodexAdapterError::InvalidProtocolMessage)
}

async fn read_bounded_output(reader: impl AsyncRead + Unpin) -> Result<Vec<u8>, CodexAdapterError> {
    let mut output = Vec::with_capacity(16 * 1024);
    reader
        .take((MAX_CLI_JSON_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .await
        .map_err(|_| CodexAdapterError::TransportClosed)?;
    if output.len() > MAX_CLI_JSON_BYTES {
        return Err(CodexAdapterError::MessageTooLarge);
    }
    Ok(output)
}

fn valid_request_shape(request: &IntegrationMutationPreviewRequest) -> bool {
    match request.operation {
        IntegrationMutationOperation::MarketplaceAdd => {
            request.target_entry_id.is_none()
                && request.repository.as_deref().is_some_and(valid_repository)
                && request
                    .reference
                    .as_deref()
                    .is_some_and(|value| !value.is_empty() && value.len() <= 64)
        }
        IntegrationMutationOperation::PluginInstall
        | IntegrationMutationOperation::PluginRemove
        | IntegrationMutationOperation::MarketplaceRemove
        | IntegrationMutationOperation::MarketplaceUpgrade => {
            request
                .target_entry_id
                .as_deref()
                .is_some_and(valid_entry_id)
                && request.repository.is_none()
                && request.reference.is_none()
        }
    }
}

fn mutation_policy_allows(policy: &IntegrationPolicySnapshot) -> bool {
    policy.mutation_confirmation_required
        && matches!(
            policy.installation,
            IntegrationPolicyState::Allowed | IntegrationPolicyState::ApprovalRequired
        )
}

fn capability_ready(
    catalog: &IntegrationCatalogSnapshot,
    operation: IntegrationMutationOperation,
) -> bool {
    let capability_id = match operation {
        IntegrationMutationOperation::PluginInstall => "plugin.install",
        IntegrationMutationOperation::PluginRemove => "plugin.remove",
        IntegrationMutationOperation::MarketplaceAdd
        | IntegrationMutationOperation::MarketplaceRemove
        | IntegrationMutationOperation::MarketplaceUpgrade => "marketplace.configure",
    };
    catalog.capabilities.iter().any(|capability| {
        capability.id == capability_id
            && capability.availability == IntegrationAvailability::Ready
            && capability.implementation == IntegrationImplementation::Ready
            && capability.mutating
            && capability.requires_confirmation
    })
}

fn review_plugin_source(
    plugin: &Map<String, Value>,
) -> Result<PluginSourceReview, PreparationFailure> {
    let source = plugin
        .get("source")
        .and_then(Value::as_object)
        .ok_or_else(|| blocked(IntegrationMutationDiagnosticCode::SourceInvalid))?;
    match source.get("source").and_then(Value::as_str) {
        Some("local") => {
            let path = source
                .get("path")
                .and_then(Value::as_str)
                .map(Path::new)
                .ok_or_else(|| blocked(IntegrationMutationDiagnosticCode::SourceInvalid))?;
            let (permissions, mut warnings, evidence) = inspect_local_plugin(path, plugin)?;
            warnings.push(IntegrationMutationWarning::LocalSource);
            Ok(PluginSourceReview {
                source: IntegrationSource::Local,
                permissions,
                warnings,
                local_evidence: Some(evidence),
            })
        }
        Some("git") | Some("git-subdir") => {
            let url = source
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| blocked(IntegrationMutationDiagnosticCode::SourceInvalid))?;
            if !safe_remote_url(url) {
                return Err(blocked(IntegrationMutationDiagnosticCode::SourceInvalid));
            }
            if !source
                .get("sha")
                .and_then(Value::as_str)
                .is_some_and(valid_commit_reference)
            {
                return Err(blocked(IntegrationMutationDiagnosticCode::SourceUnpinned));
            }
            Ok(PluginSourceReview {
                source: IntegrationSource::Repository,
                permissions: vec![IntegrationPermission {
                    kind: IntegrationPermissionKind::Network,
                    access: IntegrationPermissionAccess::Read,
                    target: "Pinned plugin repository".to_owned(),
                    required: true,
                }],
                warnings: vec![
                    IntegrationMutationWarning::RepositorySource,
                    IntegrationMutationWarning::NetworkAccess,
                ],
                local_evidence: None,
            })
        }
        Some("npm") => {
            let package = source.get("package").and_then(Value::as_str);
            let version = source.get("version").and_then(Value::as_str);
            let registry_safe = source
                .get("registry")
                .and_then(Value::as_str)
                .is_none_or(safe_remote_url);
            if !package.is_some_and(valid_package_name)
                || !version.is_some_and(valid_pinned_version)
                || !registry_safe
            {
                return Err(blocked(IntegrationMutationDiagnosticCode::SourceUnpinned));
            }
            Ok(PluginSourceReview {
                source: IntegrationSource::Repository,
                permissions: vec![IntegrationPermission {
                    kind: IntegrationPermissionKind::Network,
                    access: IntegrationPermissionAccess::Read,
                    target: "Pinned package registry artifact".to_owned(),
                    required: true,
                }],
                warnings: vec![
                    IntegrationMutationWarning::PackageRegistrySource,
                    IntegrationMutationWarning::NetworkAccess,
                ],
                local_evidence: None,
            })
        }
        _ => Err(blocked(IntegrationMutationDiagnosticCode::SourceInvalid)),
    }
}

fn inspect_local_plugin(
    root: &Path,
    plugin: &Map<String, Value>,
) -> Result<
    (
        Vec<IntegrationPermission>,
        Vec<IntegrationMutationWarning>,
        LocalPluginEvidence,
    ),
    PreparationFailure,
> {
    let root = root
        .canonicalize()
        .map_err(|_| blocked(IntegrationMutationDiagnosticCode::SourceUnreviewable))?;
    if !root.is_dir() {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }
    let manifest_directory = root.join(".codex-plugin");
    let directory_metadata = fs::symlink_metadata(&manifest_directory)
        .map_err(|_| blocked(IntegrationMutationDiagnosticCode::SourceUnreviewable))?;
    if !directory_metadata.is_dir() || directory_metadata.file_type().is_symlink() {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }
    let manifest_path = manifest_directory.join("plugin.json");
    let metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|_| blocked(IntegrationMutationDiagnosticCode::SourceUnreviewable))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_PLUGIN_MANIFEST_BYTES
    {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }
    let manifest_bytes = fs::read(&manifest_path)
        .ok()
        .filter(|bytes| bytes.len() as u64 <= MAX_PLUGIN_MANIFEST_BYTES)
        .ok_or_else(|| blocked(IntegrationMutationDiagnosticCode::SourceUnreviewable))?;
    let manifest = serde_json::from_slice::<Value>(&manifest_bytes)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| blocked(IntegrationMutationDiagnosticCode::SourceUnreviewable))?;
    let expected_name = plugin.get("name").and_then(Value::as_str);
    let expected_version = plugin.get("version").and_then(Value::as_str);
    if manifest.get("name").and_then(Value::as_str) != expected_name
        || (expected_version.is_some()
            && manifest.get("version").and_then(Value::as_str) != expected_version)
    {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }

    let mut permissions = Vec::new();
    let mut warnings = Vec::new();
    let has_declared_hooks = manifest.get("hooks").is_some_and(|value| !value.is_null());
    let has_default_hooks = !has_declared_hooks && default_hook_manifest_exists(&root)?;
    if has_declared_hooks || has_default_hooks {
        permissions.push(IntegrationPermission {
            kind: IntegrationPermissionKind::Hook,
            access: IntegrationPermissionAccess::Execute,
            target: "Bundled plugin hooks (separate trust required)".to_owned(),
            required: false,
        });
        warnings.push(IntegrationMutationWarning::HookExecution);
    }
    if manifest
        .get("mcpServers")
        .is_some_and(|value| !value.is_null())
    {
        permissions.push(IntegrationPermission {
            kind: IntegrationPermissionKind::Tool,
            access: IntegrationPermissionAccess::Connect,
            target: "Plugin MCP servers".to_owned(),
            required: true,
        });
        warnings.push(IntegrationMutationWarning::McpServers);
    }
    if manifest.get("apps").is_some_and(|value| !value.is_null()) {
        permissions.push(IntegrationPermission {
            kind: IntegrationPermissionKind::Account,
            access: IntegrationPermissionAccess::Connect,
            target: "Plugin connector apps".to_owned(),
            required: true,
        });
        warnings.push(IntegrationMutationWarning::ConnectorApps);
    }
    if manifest.get("skills").is_some_and(|value| !value.is_null()) {
        permissions.push(IntegrationPermission {
            kind: IntegrationPermissionKind::Filesystem,
            access: IntegrationPermissionAccess::Read,
            target: "Plugin skill content".to_owned(),
            required: true,
        });
        warnings.push(IntegrationMutationWarning::SkillContent);
    }
    Ok((
        permissions,
        warnings,
        LocalPluginEvidence {
            canonical_root: root,
            manifest: manifest_bytes,
            default_hooks_present: has_default_hooks,
        },
    ))
}

fn default_hook_manifest_exists(root: &Path) -> Result<bool, PreparationFailure> {
    let hooks_directory = root.join("hooks");
    let directory_metadata = match fs::symlink_metadata(&hooks_directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => {
            return Err(blocked(
                IntegrationMutationDiagnosticCode::SourceUnreviewable,
            ));
        }
    };
    if !directory_metadata.is_dir() || directory_metadata.file_type().is_symlink() {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }
    let default_hooks = hooks_directory.join("hooks.json");
    let metadata = match fs::symlink_metadata(default_hooks) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => {
            return Err(blocked(
                IntegrationMutationDiagnosticCode::SourceUnreviewable,
            ));
        }
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(blocked(
            IntegrationMutationDiagnosticCode::SourceUnreviewable,
        ));
    }
    Ok(true)
}

fn marketplace_source(
    marketplace: &Map<String, Value>,
) -> Result<IntegrationSource, PreparationFailure> {
    match marketplace.get("marketplaceSource") {
        None | Some(Value::Null) => Ok(IntegrationSource::Local),
        Some(Value::Object(source)) => match (
            source.get("sourceType").and_then(Value::as_str),
            source.get("source").and_then(Value::as_str),
        ) {
            (Some("local"), Some(_)) => Ok(IntegrationSource::Local),
            (Some("git"), Some(value)) if safe_remote_url(value) => {
                Ok(IntegrationSource::Repository)
            }
            _ => Err(blocked(IntegrationMutationDiagnosticCode::SourceInvalid)),
        },
        _ => Err(blocked(IntegrationMutationDiagnosticCode::SourceInvalid)),
    }
}

fn is_configured_marketplace(marketplace: &Map<String, Value>) -> bool {
    marketplace
        .get("marketplaceSource")
        .is_some_and(Value::is_object)
}

fn find_raw_plugin(response: &Value, target_id: &str) -> Option<Value> {
    let object = known_object(response, &["available", "installed"])?;
    let installed = object.get("installed")?.as_array()?;
    let available = object.get("available")?.as_array()?;
    installed
        .iter()
        .chain(available)
        .find(|plugin| {
            plugin
                .get("pluginId")
                .and_then(Value::as_str)
                .and_then(|raw| normalized_entry_id("plugin", raw))
                .as_deref()
                == Some(target_id)
        })
        .cloned()
}

fn find_raw_marketplace(response: &Value, target_id: &str) -> Option<Value> {
    let object = known_object(response, &["marketplaces"])?;
    object
        .get("marketplaces")?
        .as_array()?
        .iter()
        .find(|marketplace| {
            marketplace
                .get("name")
                .and_then(Value::as_str)
                .and_then(|raw| normalized_entry_id("marketplace", raw))
                .as_deref()
                == Some(target_id)
        })
        .cloned()
}

fn plugin_identity(object: &Map<String, Value>) -> Option<(String, String, String, bool)> {
    let plugin_id = object.get("pluginId")?.as_str()?;
    let name = object.get("name")?.as_str()?;
    let marketplace = object.get("marketplaceName")?.as_str()?;
    let installed = object.get("installed")?.as_bool()?;
    (plugin_id == format!("{name}@{marketplace}")
        && valid_cli_name(name)
        && valid_cli_name(marketplace))
    .then(|| {
        (
            plugin_id.to_owned(),
            name.to_owned(),
            marketplace.to_owned(),
            installed,
        )
    })
}

fn valid_plugin_mutation_output(
    output: &Value,
    operation: IntegrationMutationOperation,
    plugin_id: &str,
    plugin_name: &str,
    marketplace_name: &str,
) -> bool {
    let allowed = match operation {
        IntegrationMutationOperation::PluginInstall => &[
            "authPolicy",
            "installedPath",
            "marketplaceName",
            "name",
            "pluginId",
            "version",
        ][..],
        IntegrationMutationOperation::PluginRemove => &["marketplaceName", "name", "pluginId"][..],
        _ => return false,
    };
    let Some(object) = known_object(output, allowed) else {
        return false;
    };
    if object.get("pluginId").and_then(Value::as_str) != Some(plugin_id)
        || object.get("name").and_then(Value::as_str) != Some(plugin_name)
        || object.get("marketplaceName").and_then(Value::as_str) != Some(marketplace_name)
    {
        return false;
    }
    operation == IntegrationMutationOperation::PluginRemove
        || (object.get("installedPath").is_some_and(Value::is_string)
            && object.get("version").is_some_and(Value::is_string)
            && object
                .get("authPolicy")
                .and_then(Value::as_str)
                .is_some_and(|value| matches!(value, "ON_INSTALL" | "ON_USE")))
}

fn valid_marketplace_mutation_output(
    output: &Value,
    operation: IntegrationMutationOperation,
    marketplace_name: &str,
) -> bool {
    match operation {
        IntegrationMutationOperation::MarketplaceRemove => {
            known_object(output, &["installedRoot", "marketplaceName"]).is_some_and(|object| {
                object.get("marketplaceName").and_then(Value::as_str) == Some(marketplace_name)
                    && object
                        .get("installedRoot")
                        .is_some_and(|value| value.is_null() || value.is_string())
            })
        }
        IntegrationMutationOperation::MarketplaceUpgrade => known_object(
            output,
            &["errors", "selectedMarketplaces", "upgradedRoots"],
        )
        .is_some_and(|object| {
            object
                .get("errors")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
                && object
                    .get("selectedMarketplaces")
                    .and_then(Value::as_array)
                    .is_some_and(|selected| {
                        selected.len() == 1 && selected[0].as_str() == Some(marketplace_name)
                    })
                && object
                    .get("upgradedRoots")
                    .and_then(Value::as_array)
                    .is_some_and(|roots| roots.iter().all(Value::is_string))
        }),
        _ => false,
    }
}

fn valid_marketplace_add_output(output: &Value) -> Option<String> {
    let object = known_object(
        output,
        &["alreadyAdded", "installedRoot", "marketplaceName"],
    )?;
    let name = object
        .get("marketplaceName")?
        .as_str()
        .filter(|value| valid_cli_name(value))?;
    if !object.get("installedRoot").is_some_and(Value::is_string)
        || object.get("alreadyAdded").and_then(Value::as_bool) != Some(false)
    {
        return None;
    }
    Some(name.to_owned())
}

fn known_object<'a>(value: &'a Value, allowed: &[&str]) -> Option<&'a Map<String, Value>> {
    let object = value.as_object()?;
    object
        .keys()
        .all(|key| allowed.contains(&key.as_str()))
        .then_some(object)
}

fn supports_mutation_routes(version: &str) -> bool {
    let core = version
        .split_once(['-', '+'])
        .map_or(version, |(core, _)| core);
    let segments = core
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>();
    matches!(segments.as_deref(), Ok([major, minor, _]) if (*major, *minor) == SUPPORTED_CLI_MINOR)
}

fn valid_entry_id(value: &str) -> bool {
    value.len() <= 128
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

fn valid_cli_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn valid_repository(value: &str) -> bool {
    if value.is_empty() || value.len() > 160 || value.matches('/').count() != 1 {
        return false;
    }
    value.split('/').all(|segment| {
        !segment.is_empty()
            && segment.len() <= 80
            && !segment.starts_with('.')
            && !segment.ends_with('.')
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    })
}

fn valid_commit_reference(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_package_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 214
        && !value.chars().any(char::is_whitespace)
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'@' | b'/' | b'.' | b'_' | b'-')
        })
}

fn valid_pinned_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-'))
}

fn safe_remote_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn normalized_entry_id(prefix: &str, raw: &str) -> Option<String> {
    let mut suffix = String::new();
    let mut separator = false;
    for character in raw.chars() {
        let normalized = character.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() || matches!(normalized, '.' | '_' | '-') {
            if separator && !suffix.is_empty() {
                suffix.push('-');
            }
            separator = false;
            suffix.push(normalized);
        } else {
            separator = true;
        }
        if prefix.len() + 1 + suffix.len() >= 128 {
            break;
        }
    }
    while suffix.ends_with(['.', '_', '-']) {
        suffix.pop();
    }
    if suffix.is_empty() || !suffix.as_bytes()[0].is_ascii_alphanumeric() {
        return None;
    }
    Some(format!("{prefix}:{suffix}"))
}

fn unique_warnings(warnings: Vec<IntegrationMutationWarning>) -> Vec<IntegrationMutationWarning> {
    let mut seen = HashSet::new();
    warnings
        .into_iter()
        .filter(|warning| seen.insert(*warning))
        .collect()
}

fn valid_confirmation_id(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

fn clear_expired(state: &mut MutationState) {
    let now = Instant::now();
    state.pending.retain(|_, pending| pending.expires_at > now);
}

fn blocked(diagnostic: IntegrationMutationDiagnosticCode) -> PreparationFailure {
    PreparationFailure {
        state: IntegrationMutationPreviewState::Blocked,
        diagnostic,
    }
}

fn unavailable(diagnostic: IntegrationMutationDiagnosticCode) -> PreparationFailure {
    PreparationFailure {
        state: IntegrationMutationPreviewState::Unavailable,
        diagnostic,
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

    use super::*;
    use crate::codex::integration::{
        IntegrationEnablementState, IntegrationEntryKind, IntegrationImplementation,
    };

    #[tokio::test]
    async fn installs_and_removes_one_exact_plugin_with_one_use_confirmations() {
        let root = temporary_directory("plugin-mutation");
        let plugin_state = root.join("plugin-installed");
        let marketplace_state = root.join("marketplace-installed");
        fs::write(&marketplace_state, b"present").expect("marketplace state must exist");
        let script = fixture_script(&plugin_state, &marketplace_state, true);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let mut catalog = fixture_catalog(false);

        let preview = service
            .preview(
                plugin_request(IntegrationMutationOperation::PluginInstall),
                &catalog,
            )
            .await;
        assert_eq!(preview.state, IntegrationMutationPreviewState::Ready);
        assert_eq!(preview.source, IntegrationSource::Repository);
        assert!(preview
            .warnings
            .contains(&IntegrationMutationWarning::RepositorySource));
        assert!(preview
            .warnings
            .contains(&IntegrationMutationWarning::AuthenticationAfterInstall));
        assert!(!serde_json::to_string(&preview)
            .expect("preview must serialize")
            .contains("example.invalid"));
        let confirmation_id = preview
            .confirmation_id
            .expect("ready preview needs confirmation");
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: confirmation_id.clone(),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Applied);
        assert!(result.catalog_refresh_required);
        assert!(plugin_state.is_file());

        let replay = service
            .confirm(
                IntegrationMutationConfirmRequest { confirmation_id },
                &catalog,
            )
            .await;
        assert_eq!(replay.state, IntegrationMutationResultState::Unavailable);
        assert_eq!(
            replay.diagnostic_code,
            Some(IntegrationMutationDiagnosticCode::ConfirmationExpired)
        );

        catalog = fixture_catalog(true);
        let preview = service
            .preview(
                plugin_request(IntegrationMutationOperation::PluginRemove),
                &catalog,
            )
            .await;
        assert_eq!(preview.state, IntegrationMutationPreviewState::Ready);
        assert!(preview.destructive);
        assert!(!preview
            .warnings
            .contains(&IntegrationMutationWarning::AuthenticationAfterInstall));
        assert!(preview
            .warnings
            .contains(&IntegrationMutationWarning::RemovesCachedPlugin));
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("remove confirmation"),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Applied);
        assert!(!plugin_state.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn rejects_unpinned_sources_and_stale_catalog_state_before_mutation() {
        let root = temporary_directory("plugin-stale");
        let plugin_state = root.join("plugin-installed");
        let marketplace_state = root.join("marketplace-installed");
        fs::write(&marketplace_state, b"present").expect("marketplace state must exist");
        let script = fixture_script(&plugin_state, &marketplace_state, false);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let catalog = fixture_catalog(false);
        let blocked = service
            .preview(
                plugin_request(IntegrationMutationOperation::PluginInstall),
                &catalog,
            )
            .await;
        assert_eq!(blocked.state, IntegrationMutationPreviewState::Blocked);
        assert_eq!(
            blocked.diagnostic_code,
            Some(IntegrationMutationDiagnosticCode::SourceUnpinned)
        );

        let pinned_script = fixture_script(&plugin_state, &marketplace_state, true);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &pinned_script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let preview = service
            .preview(
                plugin_request(IntegrationMutationOperation::PluginInstall),
                &catalog,
            )
            .await;
        let mut changed_catalog = catalog.clone();
        changed_catalog
            .entries
            .iter_mut()
            .find(|entry| entry.id == "plugin:review-fixture")
            .expect("plugin fixture must exist")
            .version = Some("9.9.9".to_owned());
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation"),
                },
                &changed_catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(IntegrationMutationDiagnosticCode::StalePreview)
        );
        assert!(!plugin_state.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn rejects_a_local_manifest_change_after_preview() {
        let root = temporary_directory("local-plugin-stale");
        let plugin_root = root.join("plugin");
        let manifest_directory = plugin_root.join(".codex-plugin");
        let plugin_state = root.join("plugin-installed");
        fs::create_dir_all(&manifest_directory).expect("manifest directory");
        let manifest_path = manifest_directory.join("plugin.json");
        fs::write(
            &manifest_path,
            r#"{"name":"review","version":"1.2.3","hooks":{}}"#,
        )
        .expect("reviewed manifest");
        let script = local_plugin_fixture_script(&plugin_root, &plugin_state);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let mut catalog = fixture_catalog(false);
        catalog
            .entries
            .iter_mut()
            .find(|entry| entry.id == "plugin:review-fixture")
            .expect("plugin fixture")
            .source = IntegrationSource::Local;
        let preview = service
            .preview(
                plugin_request(IntegrationMutationOperation::PluginInstall),
                &catalog,
            )
            .await;
        assert_eq!(preview.state, IntegrationMutationPreviewState::Ready);
        assert!(preview
            .warnings
            .contains(&IntegrationMutationWarning::LocalSource));

        fs::write(
            manifest_path,
            r#"{"name":"review","version":"1.2.3","hooks":{"afterInstall":"changed"}}"#,
        )
        .expect("changed manifest");
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation"),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(IntegrationMutationDiagnosticCode::StalePreview)
        );
        assert!(!plugin_state.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn upgrades_and_removes_only_the_reviewed_marketplace() {
        let root = temporary_directory("marketplace-mutation");
        let plugin_state = root.join("plugin-installed");
        let marketplace_state = root.join("marketplace-installed");
        fs::write(&marketplace_state, b"present").expect("marketplace state must exist");
        let script = fixture_script(&plugin_state, &marketplace_state, true);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let catalog = fixture_catalog(false);

        let preview = service
            .preview(
                marketplace_request(IntegrationMutationOperation::MarketplaceUpgrade),
                &catalog,
            )
            .await;
        assert_eq!(preview.state, IntegrationMutationPreviewState::Ready);
        assert!(!preview.destructive);
        assert!(preview
            .warnings
            .contains(&IntegrationMutationWarning::MutableRemoteSource));
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("upgrade confirmation"),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Applied);
        assert!(marketplace_state.is_file());

        let preview = service
            .preview(
                marketplace_request(IntegrationMutationOperation::MarketplaceRemove),
                &catalog,
            )
            .await;
        assert!(preview.destructive);
        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("remove confirmation"),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Applied);
        assert!(!marketplace_state.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn adds_only_the_reviewed_pinned_marketplace_and_verifies_it() {
        let root = temporary_directory("marketplace-add");
        let marketplace_state = root.join("added-marketplace");
        let reference = "b".repeat(40);
        let script = marketplace_add_fixture_script(&marketplace_state, &reference);
        let service = IntegrationMutationService::with_command(
            "sh",
            &["-c", &script, "fixture"],
            Vec::new(),
            "0.145.0",
        );
        let catalog = fixture_catalog(false);
        let preview = service
            .preview(
                IntegrationMutationPreviewRequest {
                    operation: IntegrationMutationOperation::MarketplaceAdd,
                    target_entry_id: None,
                    repository: Some("openai/example".to_owned()),
                    reference: Some(reference),
                },
                &catalog,
            )
            .await;
        assert_eq!(preview.state, IntegrationMutationPreviewState::Ready);
        assert_eq!(preview.source, IntegrationSource::Repository);
        assert_eq!(preview.target_entry_id, None);
        assert!(!preview.destructive);

        let result = service
            .confirm(
                IntegrationMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("add confirmation"),
                },
                &catalog,
            )
            .await;
        assert_eq!(result.state, IntegrationMutationResultState::Applied);
        assert_eq!(result.target_entry_id.as_deref(), Some("marketplace:added"));
        assert!(marketplace_state.is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn accepts_only_bounded_pinned_repository_add_requests() {
        let request = IntegrationMutationPreviewRequest {
            operation: IntegrationMutationOperation::MarketplaceAdd,
            target_entry_id: None,
            repository: Some("openai/example".to_owned()),
            reference: Some("a".repeat(40)),
        };
        assert!(valid_request_shape(&request));
        let mut unsafe_request = request.clone();
        unsafe_request.repository = Some("https://user:secret@example.invalid/repo".to_owned());
        assert!(!valid_request_shape(&unsafe_request));
        let mut mutable_request = request;
        mutable_request.reference = Some("main".to_owned());
        assert!(valid_request_shape(&mutable_request));
        assert!(!valid_commit_reference(
            mutable_request.reference.as_deref().expect("reference")
        ));
    }

    #[test]
    fn rejects_ambiguous_or_extended_cli_success_payloads() {
        assert!(valid_marketplace_add_output(&serde_json::json!({
            "marketplaceName": "added",
            "installedRoot": "/private/discarded",
            "alreadyAdded": true
        }))
        .is_none());
        assert!(valid_marketplace_add_output(&serde_json::json!({
            "marketplaceName": "added",
            "installedRoot": "/private/discarded",
            "alreadyAdded": false,
            "unexpected": "discard me"
        }))
        .is_none());
        assert!(!valid_plugin_mutation_output(
            &serde_json::json!({
                "pluginId": "review@fixture",
                "name": "review",
                "marketplaceName": "fixture",
                "unexpected": true
            }),
            IntegrationMutationOperation::PluginRemove,
            "review@fixture",
            "review",
            "fixture",
        ));
        assert!(!is_configured_marketplace(
            serde_json::json!({"name": "default", "marketplaceSource": null})
                .as_object()
                .expect("marketplace object")
        ));
    }

    #[test]
    fn detects_default_local_hooks_as_separately_trusted_execution() {
        let root = temporary_directory("default-plugin-hooks");
        fs::create_dir_all(root.join(".codex-plugin")).expect("manifest directory");
        fs::create_dir_all(root.join("hooks")).expect("hooks directory");
        fs::write(
            root.join(".codex-plugin").join("plugin.json"),
            r#"{"name":"review","version":"1.2.3"}"#,
        )
        .expect("plugin manifest");
        fs::write(root.join("hooks").join("hooks.json"), b"{}").expect("default hooks manifest");
        let plugin = serde_json::json!({"name": "review", "version": "1.2.3"});
        let (permissions, warnings, _) =
            inspect_local_plugin(&root, plugin.as_object().expect("plugin object"))
                .expect("local plugin review");

        assert!(warnings.contains(&IntegrationMutationWarning::HookExecution));
        assert!(permissions.iter().any(|permission| {
            permission.kind == IntegrationPermissionKind::Hook
                && permission.access == IntegrationPermissionAccess::Execute
                && !permission.required
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    #[ignore = "requires the installed reviewed Codex CLI"]
    async fn proves_an_isolated_real_cli_test_plugin_lifecycle() {
        let root = temporary_directory("real-plugin-lifecycle");
        let codex_home = root.join("codex-home");
        let isolated_home = root.join("home");
        let marketplace = root.join("marketplace");
        let plugin = marketplace.join("plugins").join("sample");
        fs::create_dir_all(plugin.join(".codex-plugin")).expect("plugin directory");
        fs::create_dir_all(marketplace.join(".agents").join("plugins"))
            .expect("marketplace directory");
        fs::create_dir_all(&codex_home).expect("Codex home");
        fs::create_dir_all(&isolated_home).expect("isolated home");
        fs::write(
            codex_home.join("config.toml"),
            "[features]\nplugins = true\n",
        )
        .expect("isolated config");
        fs::write(
            plugin.join(".codex-plugin").join("plugin.json"),
            r#"{"name":"sample","version":"1.2.3","description":"Isolated lifecycle fixture"}"#,
        )
        .expect("plugin manifest");
        fs::write(
            marketplace
                .join(".agents")
                .join("plugins")
                .join("marketplace.json"),
            r#"{"name":"debug","plugins":[{"name":"sample","source":{"source":"local","path":"./plugins/sample"}}]}"#,
        )
        .expect("marketplace manifest");

        let service = IntegrationMutationService::with_command(
            "codex",
            &[],
            vec![
                (
                    OsString::from("CODEX_HOME"),
                    codex_home.clone().into_os_string(),
                ),
                (OsString::from("HOME"), isolated_home.into_os_string()),
            ],
            "0.145.0",
        );
        let added = service
            .run_json(&[
                "plugin".to_owned(),
                "marketplace".to_owned(),
                "add".to_owned(),
                marketplace.display().to_string(),
                "--json".to_owned(),
            ])
            .await
            .expect("isolated marketplace must add");
        assert_eq!(
            added.get("marketplaceName").and_then(Value::as_str),
            Some("debug")
        );
        let available = service
            .run_json(&strings(&["plugin", "list", "--available", "--json"]))
            .await
            .expect("isolated plugin must list");
        assert!(find_raw_plugin(&available, "plugin:sample-debug").is_some());
        service
            .run_json(&strings(&["plugin", "add", "sample@debug", "--json"]))
            .await
            .expect("isolated plugin must install");
        let installed = service
            .run_json(&strings(&["plugin", "list", "--available", "--json"]))
            .await
            .expect("installed plugin must list");
        assert_eq!(
            find_raw_plugin(&installed, "plugin:sample-debug")
                .and_then(|value| value.get("installed").cloned())
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        service
            .run_json(&strings(&["plugin", "remove", "sample@debug", "--json"]))
            .await
            .expect("isolated plugin must remove");
        service
            .run_json(&strings(&[
                "plugin",
                "marketplace",
                "remove",
                "debug",
                "--json",
            ]))
            .await
            .expect("isolated marketplace must remove");
        assert!(!codex_home.join("plugins/cache/debug/sample/1.2.3").exists());
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_catalog(installed: bool) -> IntegrationCatalogSnapshot {
        let mut catalog: IntegrationCatalogSnapshot =
            serde_json::from_str(include_str!("../../../fixtures/integration-catalog.json"))
                .expect("catalog fixture");
        for capability in &mut catalog.capabilities {
            if matches!(
                capability.id.as_str(),
                "plugin.catalog"
                    | "plugin.install"
                    | "plugin.remove"
                    | "marketplace.catalog"
                    | "marketplace.configure"
            ) {
                capability.availability = IntegrationAvailability::Ready;
                capability.implementation = IntegrationImplementation::Ready;
                capability.diagnostic_code = None;
            }
        }
        let plugin = catalog
            .entries
            .iter_mut()
            .find(|entry| entry.kind == IntegrationEntryKind::Plugin)
            .expect("plugin fixture");
        plugin.id = "plugin:review-fixture".to_owned();
        plugin.display_name = "Review plugin".to_owned();
        plugin.version = Some("1.2.3".to_owned());
        plugin.source = IntegrationSource::Repository;
        plugin.installation = if installed {
            IntegrationInstallationState::Installed
        } else {
            IntegrationInstallationState::Available
        };
        plugin.enablement = if installed {
            IntegrationEnablementState::Enabled
        } else {
            IntegrationEnablementState::Disabled
        };
        plugin.policy.state = IntegrationPolicyState::ApprovalRequired;
        plugin.policy.managed = false;
        let marketplace = catalog
            .entries
            .iter_mut()
            .find(|entry| entry.kind == IntegrationEntryKind::Marketplace)
            .expect("marketplace fixture");
        marketplace.id = "marketplace:fixture".to_owned();
        marketplace.display_name = "Fixture marketplace".to_owned();
        marketplace.source = IntegrationSource::Repository;
        marketplace.policy.state = IntegrationPolicyState::ApprovalRequired;
        marketplace.policy.managed = false;
        catalog
    }

    fn plugin_request(
        operation: IntegrationMutationOperation,
    ) -> IntegrationMutationPreviewRequest {
        IntegrationMutationPreviewRequest {
            operation,
            target_entry_id: Some("plugin:review-fixture".to_owned()),
            repository: None,
            reference: None,
        }
    }

    fn marketplace_request(
        operation: IntegrationMutationOperation,
    ) -> IntegrationMutationPreviewRequest {
        IntegrationMutationPreviewRequest {
            operation,
            target_entry_id: Some("marketplace:fixture".to_owned()),
            repository: None,
            reference: None,
        }
    }

    fn fixture_script(plugin_state: &Path, marketplace_state: &Path, pinned: bool) -> String {
        let sha = if pinned {
            "a".repeat(40)
        } else {
            String::new()
        };
        let sha_field = if sha.is_empty() {
            String::new()
        } else {
            format!(r#","sha":"{sha}""#)
        };
        format!(
            r#"
PLUGIN_STATE='{plugin_state}'
MARKETPLACE_STATE='{marketplace_state}'
case "$*" in
  "plugin list --available --json")
    if [ -f "$PLUGIN_STATE" ]; then
      printf '%s\n' '{{"installed":[{{"authPolicy":"ON_INSTALL","enabled":true,"installPolicy":"AVAILABLE","installed":true,"marketplaceName":"fixture","marketplaceSource":{{"sourceType":"git","source":"https://example.invalid/fixture"}},"name":"review","pluginId":"review@fixture","source":{{"source":"git","url":"https://example.invalid/review"{sha_field}}},"version":"1.2.3"}}],"available":[]}}'
    else
      printf '%s\n' '{{"installed":[],"available":[{{"authPolicy":"ON_INSTALL","enabled":false,"installPolicy":"AVAILABLE","installed":false,"marketplaceName":"fixture","marketplaceSource":{{"sourceType":"git","source":"https://example.invalid/fixture"}},"name":"review","pluginId":"review@fixture","source":{{"source":"git","url":"https://example.invalid/review"{sha_field}}},"version":"1.2.3"}}]}}'
    fi
    ;;
  "plugin add review@fixture --json")
    : > "$PLUGIN_STATE"
    printf '%s\n' '{{"pluginId":"review@fixture","name":"review","marketplaceName":"fixture","version":"1.2.3","installedPath":"/private/discarded","authPolicy":"ON_INSTALL"}}'
    ;;
  "plugin remove review --marketplace fixture --json")
    rm -f "$PLUGIN_STATE"
    printf '%s\n' '{{"pluginId":"review@fixture","name":"review","marketplaceName":"fixture"}}'
    ;;
  "plugin marketplace list --json")
    if [ -f "$MARKETPLACE_STATE" ]; then
      printf '%s\n' '{{"marketplaces":[{{"name":"fixture","root":"/private/discarded","marketplaceSource":{{"sourceType":"git","source":"https://example.invalid/fixture"}}}}]}}'
    else
      printf '%s\n' '{{"marketplaces":[]}}'
    fi
    ;;
  "plugin marketplace upgrade fixture --json")
    printf '%s\n' '{{"selectedMarketplaces":["fixture"],"upgradedRoots":["/private/discarded"],"errors":[]}}'
    ;;
  "plugin marketplace remove fixture --json")
    rm -f "$MARKETPLACE_STATE"
    printf '%s\n' '{{"marketplaceName":"fixture","installedRoot":"/private/discarded"}}'
    ;;
  *) exit 91 ;;
esac
"#,
            plugin_state = plugin_state.display(),
            marketplace_state = marketplace_state.display(),
        )
    }

    fn marketplace_add_fixture_script(marketplace_state: &Path, reference: &str) -> String {
        format!(
            r#"
MARKETPLACE_STATE='{marketplace_state}'
case "$*" in
  "plugin marketplace add openai/example --ref {reference} --json")
    : > "$MARKETPLACE_STATE"
    printf '%s\n' '{{"marketplaceName":"added","installedRoot":"/private/discarded","alreadyAdded":false}}'
    ;;
  "plugin marketplace list --json")
    if [ -f "$MARKETPLACE_STATE" ]; then
      printf '%s\n' '{{"marketplaces":[{{"name":"added","root":"/private/discarded","marketplaceSource":{{"sourceType":"git","source":"https://example.invalid/added"}}}}]}}'
    else
      printf '%s\n' '{{"marketplaces":[]}}'
    fi
    ;;
  *) exit 91 ;;
esac
"#,
            marketplace_state = marketplace_state.display(),
        )
    }

    fn local_plugin_fixture_script(plugin_root: &Path, plugin_state: &Path) -> String {
        format!(
            r#"
PLUGIN_ROOT='{plugin_root}'
PLUGIN_STATE='{plugin_state}'
case "$*" in
  "plugin list --available --json")
    if [ -f "$PLUGIN_STATE" ]; then
      printf '%s\n' '{{"installed":[{{"authPolicy":"ON_USE","enabled":true,"installPolicy":"AVAILABLE","installed":true,"marketplaceName":"fixture","name":"review","pluginId":"review@fixture","source":{{"source":"local","path":"{plugin_root}"}},"version":"1.2.3"}}],"available":[]}}'
    else
      printf '%s\n' '{{"installed":[],"available":[{{"authPolicy":"ON_USE","enabled":false,"installPolicy":"AVAILABLE","installed":false,"marketplaceName":"fixture","name":"review","pluginId":"review@fixture","source":{{"source":"local","path":"{plugin_root}"}},"version":"1.2.3"}}]}}'
    fi
    ;;
  "plugin add review@fixture --json")
    : > "$PLUGIN_STATE"
    printf '%s\n' '{{"pluginId":"review@fixture","name":"review","marketplaceName":"fixture","version":"1.2.3","installedPath":"/private/discarded","authPolicy":"ON_USE"}}'
    ;;
  *) exit 91 ;;
esac
"#,
            plugin_root = plugin_root.display(),
            plugin_state = plugin_state.display(),
        )
    }

    fn temporary_directory(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).expect("temporary directory must exist");
        let mut permissions = fs::metadata(&root)
            .expect("temporary directory metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&root, permissions).expect("temporary directory permissions");
        root
    }
}
