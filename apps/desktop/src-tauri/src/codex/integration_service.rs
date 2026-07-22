use std::{
    collections::{HashMap, HashSet},
    process::Stdio,
    time::Duration,
};

use serde_json::{json, Map, Value};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::Mutex,
    time::timeout,
};

use super::{
    app_server::{AppServerCommand, AppServerNotification, AppServerProcess},
    error::CodexAdapterError,
    integration::{
        DynamicToolContract, IntegrationAuthenticationState, IntegrationAvailability,
        IntegrationCatalogSnapshot, IntegrationEnablementState, IntegrationEntry,
        IntegrationEntryKind, IntegrationEntryPolicy, IntegrationHealth, IntegrationImplementation,
        IntegrationInstallationState, IntegrationPermission, IntegrationPermissionAccess,
        IntegrationPermissionKind, IntegrationPolicySnapshot, IntegrationPolicySource,
        IntegrationPolicyState, IntegrationRefreshReason, IntegrationRequirement,
        IntegrationRequirementKind, IntegrationRequirementState, IntegrationScope,
        IntegrationSource,
    },
    probe::probe_cli_version,
};

const SUPPORTED_CLI_MINOR: (u64, u64) = (0, 145);
const MAX_PAGES: usize = 8;
const MAX_SOURCE_ENTRIES: usize = 512;
const MAX_CURSOR_BYTES: usize = 512;
const MAX_INVALIDATIONS_PER_READ: usize = 8;
const INVALIDATION_POLL: Duration = Duration::from_millis(2);
const CLI_JSON_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_CLI_JSON_BYTES: usize = 1024 * 1024;

const CONNECTOR_CAPABILITIES: &[&str] = &["connector.catalog", "connector.installed"];
const PLUGIN_CAPABILITIES: &[&str] = &["plugin.catalog"];
const MARKETPLACE_CAPABILITIES: &[&str] = &["marketplace.catalog"];
const SKILL_CAPABILITIES: &[&str] = &["skill.catalog"];
const MCP_CAPABILITIES: &[&str] = &["mcp.health"];
const REQUIREMENT_CAPABILITIES: &[&str] = &["policy.requirements"];
const PROFILE_CAPABILITIES: &[&str] = &["permission.profiles"];

pub struct IntegrationCatalogService {
    program: String,
    command: AppServerCommand,
    cli_version_override: Option<String>,
    #[cfg(test)]
    plugin_cli_override: Option<(Value, Value)>,
    state: Mutex<IntegrationServiceState>,
}

#[derive(Default)]
struct IntegrationServiceState {
    process: Option<AppServerProcess>,
    snapshot: Option<IntegrationCatalogSnapshot>,
}

impl Default for IntegrationCatalogService {
    fn default() -> Self {
        Self {
            program: "codex".to_owned(),
            command: AppServerCommand::codex("codex"),
            cli_version_override: None,
            #[cfg(test)]
            plugin_cli_override: None,
            state: Mutex::new(IntegrationServiceState::default()),
        }
    }
}

impl IntegrationCatalogService {
    pub async fn snapshot(&self) -> IntegrationCatalogSnapshot {
        let mut state = self.state.lock().await;
        if state.snapshot.is_none() {
            return self.rebuild(&mut state).await;
        }

        let reasons = match state.process.as_mut() {
            Some(process) => collect_refresh_reasons(process).await,
            None => Err(CodexAdapterError::TransportClosed),
        };
        let reasons = match reasons {
            Ok(reasons) => reasons,
            Err(_) => return self.rebuild(&mut state).await,
        };
        if reasons.is_empty() {
            return state
                .snapshot
                .as_ref()
                .expect("cached integration snapshot must exist")
                .clone();
        }

        let mut snapshot = state
            .snapshot
            .take()
            .expect("cached integration snapshot must exist");
        if let Some(process) = state.process.as_mut() {
            refresh_snapshot(process, &mut snapshot, &reasons).await;
        }
        snapshot.refresh_reasons = reasons;
        let snapshot = validated_or_unavailable(snapshot);
        state.snapshot = Some(snapshot.clone());
        snapshot
    }

    async fn rebuild(&self, state: &mut IntegrationServiceState) -> IntegrationCatalogSnapshot {
        if let Some(mut process) = state.process.take() {
            let _ = process.shutdown().await;
        }

        let cli_version = match self.cli_version_override.as_ref() {
            Some(version) => version.clone(),
            None => match probe_cli_version(&self.program).await {
                Ok(version) => version,
                Err(_) => {
                    let snapshot = unavailable_snapshot("0.0.0", "integration-cli-unavailable");
                    state.snapshot = Some(snapshot.clone());
                    return snapshot;
                }
            },
        };
        if !supports_integration_routes(&cli_version) {
            let snapshot = unavailable_snapshot(&cli_version, "integration-version-unsupported");
            state.snapshot = Some(snapshot.clone());
            return snapshot;
        }

        let mut process = match AppServerProcess::spawn(self.command.clone()) {
            Ok(process) => process,
            Err(_) => {
                let snapshot =
                    unavailable_snapshot(&cli_version, "integration-process-unavailable");
                state.snapshot = Some(snapshot.clone());
                return snapshot;
            }
        };
        if process.initialize().await.is_err() {
            let _ = process.shutdown().await;
            let snapshot = unavailable_snapshot(&cli_version, "integration-protocol-unavailable");
            state.snapshot = Some(snapshot.clone());
            return snapshot;
        }

        let mut snapshot = template_snapshot(&cli_version);
        refresh_snapshot(
            &mut process,
            &mut snapshot,
            &[IntegrationRefreshReason::AppListUpdated],
        )
        .await;
        let plugins = self.discover_plugins().await;
        replace_entries(
            &mut snapshot,
            IntegrationEntryKind::Plugin,
            plugins.plugin_entries,
        );
        replace_entries(
            &mut snapshot,
            IntegrationEntryKind::Marketplace,
            plugins.marketplace_entries,
        );
        set_capabilities(
            &mut snapshot,
            PLUGIN_CAPABILITIES,
            plugins.plugin_state,
            plugins.plugin_diagnostic.as_deref(),
        );
        set_capabilities(
            &mut snapshot,
            MARKETPLACE_CAPABILITIES,
            plugins.marketplace_state,
            plugins.marketplace_diagnostic.as_deref(),
        );
        refresh_snapshot(
            &mut process,
            &mut snapshot,
            &[
                IntegrationRefreshReason::SkillsChanged,
                IntegrationRefreshReason::McpStatusUpdated,
                IntegrationRefreshReason::ConfigWarning,
            ],
        )
        .await;

        let queued = collect_refresh_reasons(&mut process)
            .await
            .unwrap_or_default();
        if !queued.is_empty() {
            refresh_snapshot(&mut process, &mut snapshot, &queued).await;
            snapshot.refresh_reasons = queued;
        } else {
            snapshot.refresh_reasons.clear();
        }
        let snapshot = validated_or_unavailable(snapshot);
        state.process = Some(process);
        state.snapshot = Some(snapshot.clone());
        snapshot
    }

    #[cfg(test)]
    fn with_command(
        command: AppServerCommand,
        cli_version: &str,
        plugin_cli_override: Option<(Value, Value)>,
    ) -> Self {
        Self {
            program: "fixture-codex".to_owned(),
            command,
            cli_version_override: Some(cli_version.to_owned()),
            plugin_cli_override,
            state: Mutex::new(IntegrationServiceState::default()),
        }
    }

    async fn discover_plugins(&self) -> PluginDiscovery {
        #[cfg(test)]
        if let Some((plugins, marketplaces)) = self.plugin_cli_override.as_ref() {
            return plugin_discovery_from_cli(
                normalize_cli_plugins(plugins),
                normalize_cli_marketplaces(marketplaces),
            );
        }

        let plugins = run_cli_json(&self.program, &["plugin", "list", "--available", "--json"])
            .await
            .map_or_else(
                |_| SourceResult::failed("plugin-discovery-failed"),
                |value| normalize_cli_plugins(&value),
            );
        let marketplaces =
            run_cli_json(&self.program, &["plugin", "marketplace", "list", "--json"])
                .await
                .map_or_else(
                    |_| SourceResult::failed("marketplace-discovery-failed"),
                    |value| normalize_cli_marketplaces(&value),
                );
        plugin_discovery_from_cli(plugins, marketplaces)
    }
}

async fn refresh_snapshot(
    process: &mut AppServerProcess,
    snapshot: &mut IntegrationCatalogSnapshot,
    reasons: &[IntegrationRefreshReason],
) {
    if reasons.contains(&IntegrationRefreshReason::AppListUpdated) {
        let source = discover_connectors(process).await;
        replace_entries(snapshot, IntegrationEntryKind::Connector, source.entries);
        set_capabilities(
            snapshot,
            CONNECTOR_CAPABILITIES,
            source.state,
            source.diagnostic.as_deref(),
        );
    }
    if reasons.contains(&IntegrationRefreshReason::SkillsChanged) {
        let source = discover_skills(process).await;
        replace_entries(snapshot, IntegrationEntryKind::Skill, source.entries);
        set_capabilities(
            snapshot,
            SKILL_CAPABILITIES,
            source.state,
            source.diagnostic.as_deref(),
        );
    }
    if reasons.contains(&IntegrationRefreshReason::McpStatusUpdated) {
        let source = discover_mcp_servers(process).await;
        replace_entries(snapshot, IntegrationEntryKind::McpServer, source.entries);
        set_capabilities(
            snapshot,
            MCP_CAPABILITIES,
            source.state,
            source.diagnostic.as_deref(),
        );
    }
    if reasons.contains(&IntegrationRefreshReason::ConfigWarning) {
        let policy = discover_policy(process).await;
        snapshot.policy = policy.snapshot;
        set_capabilities(
            snapshot,
            REQUIREMENT_CAPABILITIES,
            policy.requirements_state,
            policy.requirements_diagnostic.as_deref(),
        );
        set_capabilities(
            snapshot,
            PROFILE_CAPABILITIES,
            policy.profiles_state,
            policy.profiles_diagnostic.as_deref(),
        );
    }
    finalize_catalog_state(snapshot);
}

async fn collect_refresh_reasons(
    process: &mut AppServerProcess,
) -> Result<Vec<IntegrationRefreshReason>, CodexAdapterError> {
    let mut reasons = Vec::new();
    for _ in 0..MAX_INVALIDATIONS_PER_READ {
        let Some(notification) = process
            .next_notification_with_timeout(INVALIDATION_POLL)
            .await?
        else {
            break;
        };
        let reason = match notification {
            AppServerNotification::IntegrationRefresh(reason) => Some(reason),
            AppServerNotification::AccountUpdated
            | AppServerNotification::AccountLoginCompleted { .. } => {
                Some(IntegrationRefreshReason::AppListUpdated)
            }
            AppServerNotification::Conversation(_)
            | AppServerNotification::ConversationRequest(_) => {
                return Err(CodexAdapterError::UnexpectedServerRequest);
            }
        };
        if let Some(reason) = reason {
            if !reasons.contains(&reason) {
                reasons.push(reason);
            }
        }
    }
    Ok(reasons)
}

struct SourceResult {
    entries: Vec<IntegrationEntry>,
    state: IntegrationAvailability,
    diagnostic: Option<String>,
}

impl SourceResult {
    fn ready(entries: Vec<IntegrationEntry>) -> Self {
        Self {
            entries,
            state: IntegrationAvailability::Ready,
            diagnostic: None,
        }
    }

    fn failed(code: &str) -> Self {
        Self {
            entries: Vec::new(),
            state: IntegrationAvailability::Degraded,
            diagnostic: Some(code.to_owned()),
        }
    }

    fn mark_degraded(&mut self, code: &str) {
        self.state = IntegrationAvailability::Degraded;
        self.diagnostic = Some(code.to_owned());
    }
}

struct PluginDiscovery {
    plugin_entries: Vec<IntegrationEntry>,
    marketplace_entries: Vec<IntegrationEntry>,
    plugin_state: IntegrationAvailability,
    plugin_diagnostic: Option<String>,
    marketplace_state: IntegrationAvailability,
    marketplace_diagnostic: Option<String>,
}

struct PolicyDiscovery {
    snapshot: IntegrationPolicySnapshot,
    requirements_state: IntegrationAvailability,
    requirements_diagnostic: Option<String>,
    profiles_state: IntegrationAvailability,
    profiles_diagnostic: Option<String>,
}

async fn discover_connectors(process: &mut AppServerProcess) -> SourceResult {
    let list = match paginated_request(
        process,
        "app/list",
        json!({"limit": 128, "forceRefetch": true, "threadId": null}),
    )
    .await
    {
        Ok(data) => data,
        Err(_) => return SourceResult::failed("connector-discovery-failed"),
    };

    let installed = match process
        .request(
            "app/installed",
            json!({"forceRefresh": false, "threadId": null}),
        )
        .await
    {
        Ok(response) => response,
        Err(_) => return SourceResult::failed("connector-status-failed"),
    };
    let Some(installed_object) = known_object(&installed, &["apps"]) else {
        return SourceResult::failed("connector-response-invalid");
    };
    let Some(installed_apps) = installed_object.get("apps").and_then(Value::as_array) else {
        return SourceResult::failed("connector-response-invalid");
    };
    if installed_apps.len() > MAX_SOURCE_ENTRIES {
        return SourceResult::failed("connector-response-invalid");
    }
    let mut installed = HashMap::new();
    for app in installed_apps {
        let Some(object) = known_object(app, &["callable", "enabled", "id", "runtimeName"]) else {
            return SourceResult::failed("connector-response-invalid");
        };
        let (Some(id), Some(callable), Some(enabled)) = (
            object.get("id").and_then(Value::as_str),
            object.get("callable").and_then(Value::as_bool),
            object.get("enabled").and_then(Value::as_bool),
        ) else {
            return SourceResult::failed("connector-response-invalid");
        };
        if installed
            .insert(id.to_owned(), (callable, enabled))
            .is_some()
        {
            return SourceResult::failed("connector-response-invalid");
        }
    }

    normalize_connectors(&list, &installed)
}

fn normalize_connectors(apps: &[Value], installed: &HashMap<String, (bool, bool)>) -> SourceResult {
    const APP_KEYS: &[&str] = &[
        "appMetadata",
        "branding",
        "description",
        "distributionChannel",
        "iconAssets",
        "iconDarkAssets",
        "id",
        "installUrl",
        "isAccessible",
        "isEnabled",
        "labels",
        "logoUrl",
        "logoUrlDark",
        "name",
        "pluginDisplayNames",
    ];
    let mut result = SourceResult::ready(Vec::new());
    let mut ids = HashSet::new();
    for app in apps.iter().take(MAX_SOURCE_ENTRIES) {
        let Some(object) = known_object(app, APP_KEYS) else {
            result.mark_degraded("connector-entry-invalid");
            continue;
        };
        let (Some(raw_id), Some(raw_name)) = (
            object.get("id").and_then(Value::as_str),
            object.get("name").and_then(Value::as_str),
        ) else {
            result.mark_degraded("connector-entry-invalid");
            continue;
        };
        let Some(id) = normalized_entry_id("connector", raw_id) else {
            result.mark_degraded("connector-entry-invalid");
            continue;
        };
        if !ids.insert(id.clone()) {
            result.mark_degraded("connector-entry-duplicate");
            continue;
        }
        let summary = object
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("Connector discovered through Codex.");
        let is_accessible = object
            .get("isAccessible")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let is_enabled = object
            .get("isEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let installed_state = installed.get(raw_id).copied();
        let callable = installed_state.is_none_or(|(callable, _)| callable);
        let enabled = installed_state.map_or(is_enabled, |(_, enabled)| enabled && is_enabled);
        let authentication = if is_accessible {
            IntegrationAuthenticationState::Connected
        } else {
            IntegrationAuthenticationState::Required
        };
        let mut requirements = Vec::new();
        if !is_accessible {
            requirements.push(IntegrationRequirement {
                kind: IntegrationRequirementKind::Authentication,
                name: "Connector authorization".to_owned(),
                state: IntegrationRequirementState::Missing,
                detail: Some("Authorization must use the official Codex handoff.".to_owned()),
            });
        }
        result.entries.push(IntegrationEntry {
            id,
            kind: IntegrationEntryKind::Connector,
            display_name: normalized_display(raw_name, 128, "Connector"),
            summary: normalized_display(summary, 320, "Connector discovered through Codex."),
            scope: IntegrationScope::Account,
            source: IntegrationSource::Unknown,
            installation: if installed_state.is_some() {
                IntegrationInstallationState::Installed
            } else {
                IntegrationInstallationState::Available
            },
            enablement: if enabled {
                IntegrationEnablementState::Enabled
            } else {
                IntegrationEnablementState::Disabled
            },
            authentication,
            version: None,
            publisher: None,
            capability_ids: vec![
                "connector.catalog".to_owned(),
                "connector.installed".to_owned(),
            ],
            permissions: Vec::new(),
            requirements,
            policy: IntegrationEntryPolicy {
                state: IntegrationPolicyState::Allowed,
                managed: false,
                reason: None,
            },
            health: if is_accessible && callable {
                IntegrationHealth {
                    state: IntegrationAvailability::Ready,
                    diagnostic_codes: Vec::new(),
                }
            } else {
                IntegrationHealth {
                    state: IntegrationAvailability::Degraded,
                    diagnostic_codes: if !is_accessible {
                        vec!["authorization-required".to_owned()]
                    } else {
                        vec!["connector-not-callable".to_owned()]
                    },
                }
            },
        });
    }
    result
}

fn plugin_discovery_from_cli(plugins: SourceResult, marketplaces: SourceResult) -> PluginDiscovery {
    PluginDiscovery {
        plugin_entries: plugins.entries,
        marketplace_entries: marketplaces.entries,
        plugin_state: plugins.state,
        plugin_diagnostic: plugins.diagnostic,
        marketplace_state: marketplaces.state,
        marketplace_diagnostic: marketplaces.diagnostic,
    }
}

fn normalize_cli_plugins(response: &Value) -> SourceResult {
    const PLUGIN_KEYS: &[&str] = &[
        "authPolicy",
        "enabled",
        "installPolicy",
        "installed",
        "marketplaceName",
        "marketplaceSource",
        "name",
        "pluginId",
        "source",
        "version",
    ];
    let Some(object) = known_object(response, &["available", "installed"]) else {
        return SourceResult::failed("plugin-response-invalid");
    };
    let (Some(installed), Some(available)) = (
        object.get("installed").and_then(Value::as_array),
        object.get("available").and_then(Value::as_array),
    ) else {
        return SourceResult::failed("plugin-response-invalid");
    };
    if installed.len().saturating_add(available.len()) > MAX_SOURCE_ENTRIES {
        return SourceResult::failed("plugin-response-invalid");
    }

    let mut result = SourceResult::ready(Vec::new());
    let mut ids = HashSet::new();
    for (entries, expected_installed) in [(installed, true), (available, false)] {
        for plugin in entries {
            let Some(plugin) = known_object(plugin, PLUGIN_KEYS) else {
                result.mark_degraded("plugin-entry-invalid");
                continue;
            };
            let (
                Some(raw_id),
                Some(raw_name),
                Some(marketplace_name),
                Some(installed),
                Some(enabled),
                Some(auth_policy),
                Some(install_policy),
                Some(source_type),
            ) = (
                plugin.get("pluginId").and_then(Value::as_str),
                plugin.get("name").and_then(Value::as_str),
                plugin.get("marketplaceName").and_then(Value::as_str),
                plugin.get("installed").and_then(Value::as_bool),
                plugin.get("enabled").and_then(Value::as_bool),
                plugin.get("authPolicy").and_then(Value::as_str),
                plugin.get("installPolicy").and_then(Value::as_str),
                plugin.get("source").and_then(cli_plugin_source_type),
            )
            else {
                result.mark_degraded("plugin-entry-invalid");
                continue;
            };
            let marketplace_source_valid = plugin.get("marketplaceSource").is_none_or(|value| {
                value.is_null() || cli_marketplace_source_type(value).is_some()
            });
            let expected_id = format!("{raw_name}@{marketplace_name}");
            if installed != expected_installed
                || raw_id != expected_id
                || !matches!(auth_policy, "ON_INSTALL" | "ON_USE")
                || !matches!(
                    install_policy,
                    "NOT_AVAILABLE" | "AVAILABLE" | "INSTALLED_BY_DEFAULT"
                )
                || !marketplace_source_valid
            {
                result.mark_degraded("plugin-entry-invalid");
                continue;
            }
            let Some(id) = normalized_entry_id("plugin", raw_id) else {
                result.mark_degraded("plugin-entry-invalid");
                continue;
            };
            if !ids.insert(id.clone()) {
                result.mark_degraded("plugin-entry-duplicate");
                continue;
            }
            let install_available = install_policy != "NOT_AVAILABLE";
            let local = source_type == "local";
            let version = match plugin.get("version") {
                None | Some(Value::Null) => None,
                Some(Value::String(version)) => normalized_version(version),
                Some(_) => {
                    result.mark_degraded("plugin-entry-invalid");
                    continue;
                }
            };
            if plugin.get("version").is_some_and(Value::is_string) && version.is_none() {
                result.mark_degraded("plugin-entry-invalid");
                continue;
            }
            result.entries.push(IntegrationEntry {
                id,
                kind: IntegrationEntryKind::Plugin,
                display_name: normalized_display(raw_name, 128, "Plugin"),
                summary: "Plugin discovered through the supported Codex CLI catalog.".to_owned(),
                scope: if local {
                    IntegrationScope::User
                } else {
                    IntegrationScope::Remote
                },
                source: if local {
                    IntegrationSource::Local
                } else {
                    IntegrationSource::Marketplace
                },
                installation: if installed {
                    IntegrationInstallationState::Installed
                } else if install_available {
                    IntegrationInstallationState::Available
                } else {
                    IntegrationInstallationState::Unknown
                },
                enablement: if !install_available {
                    IntegrationEnablementState::Blocked
                } else if installed && enabled {
                    IntegrationEnablementState::Enabled
                } else {
                    IntegrationEnablementState::Disabled
                },
                authentication: IntegrationAuthenticationState::NotApplicable,
                version,
                publisher: None,
                capability_ids: vec![
                    "plugin.catalog".to_owned(),
                    "plugin.install".to_owned(),
                    "plugin.remove".to_owned(),
                ],
                permissions: Vec::new(),
                requirements: vec![IntegrationRequirement {
                    kind: IntegrationRequirementKind::Policy,
                    name: "Plugin trust review".to_owned(),
                    state: if install_available {
                        IntegrationRequirementState::Satisfied
                    } else {
                        IntegrationRequirementState::Blocked
                    },
                    detail: (!install_available)
                        .then(|| "Effective policy prevents installation.".to_owned()),
                }],
                policy: IntegrationEntryPolicy {
                    state: if install_available {
                        IntegrationPolicyState::ApprovalRequired
                    } else {
                        IntegrationPolicyState::Blocked
                    },
                    managed: false,
                    reason: Some(if install_available {
                        "Installation requires explicit trust review.".to_owned()
                    } else {
                        "Effective policy prevents installation.".to_owned()
                    }),
                },
                health: if install_available {
                    IntegrationHealth {
                        state: IntegrationAvailability::Ready,
                        diagnostic_codes: Vec::new(),
                    }
                } else {
                    IntegrationHealth {
                        state: IntegrationAvailability::Blocked,
                        diagnostic_codes: vec!["plugin-policy-blocked".to_owned()],
                    }
                },
            });
        }
    }
    result
}

fn normalize_cli_marketplaces(response: &Value) -> SourceResult {
    let Some(object) = known_object(response, &["marketplaces"]) else {
        return SourceResult::failed("marketplace-response-invalid");
    };
    let Some(marketplaces) = object.get("marketplaces").and_then(Value::as_array) else {
        return SourceResult::failed("marketplace-response-invalid");
    };
    if marketplaces.len() > MAX_SOURCE_ENTRIES {
        return SourceResult::failed("marketplace-response-invalid");
    }

    let mut result = SourceResult::ready(Vec::new());
    let mut ids = HashSet::new();
    for marketplace in marketplaces {
        let Some(marketplace) = known_object(marketplace, &["marketplaceSource", "name", "root"])
        else {
            result.mark_degraded("marketplace-entry-invalid");
            continue;
        };
        let (Some(raw_name), Some(_root)) = (
            marketplace.get("name").and_then(Value::as_str),
            marketplace.get("root").and_then(Value::as_str),
        ) else {
            result.mark_degraded("marketplace-entry-invalid");
            continue;
        };
        let source_type = match marketplace.get("marketplaceSource") {
            None | Some(Value::Null) => "local",
            Some(source) => match cli_marketplace_source_type(source) {
                Some(source_type) => source_type,
                None => {
                    result.mark_degraded("marketplace-entry-invalid");
                    continue;
                }
            },
        };
        let Some(id) = normalized_entry_id("marketplace", raw_name) else {
            result.mark_degraded("marketplace-entry-invalid");
            continue;
        };
        if !ids.insert(id.clone()) {
            result.mark_degraded("marketplace-entry-duplicate");
            continue;
        }
        let local = source_type == "local";
        result.entries.push(IntegrationEntry {
            id,
            kind: IntegrationEntryKind::Marketplace,
            display_name: normalized_display(raw_name, 128, "Marketplace"),
            summary: "Marketplace discovered through the supported Codex CLI catalog.".to_owned(),
            scope: if local {
                IntegrationScope::User
            } else {
                IntegrationScope::Remote
            },
            source: if local {
                IntegrationSource::Local
            } else {
                IntegrationSource::Marketplace
            },
            installation: IntegrationInstallationState::Installed,
            enablement: IntegrationEnablementState::Enabled,
            authentication: IntegrationAuthenticationState::NotApplicable,
            version: None,
            publisher: None,
            capability_ids: vec![
                "marketplace.catalog".to_owned(),
                "marketplace.configure".to_owned(),
            ],
            permissions: Vec::new(),
            requirements: Vec::new(),
            policy: IntegrationEntryPolicy {
                state: IntegrationPolicyState::ApprovalRequired,
                managed: false,
                reason: Some("Marketplace changes require explicit review.".to_owned()),
            },
            health: IntegrationHealth {
                state: IntegrationAvailability::Ready,
                diagnostic_codes: Vec::new(),
            },
        });
    }
    result
}

fn cli_plugin_source_type(source: &Value) -> Option<&str> {
    let object = source.as_object()?;
    let source_type = object.get("source")?.as_str()?;
    let allowed_keys: &[&str] = match source_type {
        "local" => &["path", "source"],
        "git" => &["ref", "sha", "source", "url"],
        "git-subdir" => &["path", "ref", "sha", "source", "url"],
        "npm" => &["package", "registry", "source", "version"],
        _ => return None,
    };
    if object
        .keys()
        .any(|key| !allowed_keys.contains(&key.as_str()))
    {
        return None;
    }
    let optional_string = |key: &str| object.get(key).is_none_or(Value::is_string);
    let required_valid = match source_type {
        "local" => object.get("path").is_some_and(Value::is_string),
        "git" => {
            object.get("url").is_some_and(Value::is_string)
                && optional_string("ref")
                && optional_string("sha")
        }
        "git-subdir" => {
            object.get("url").is_some_and(Value::is_string)
                && object.get("path").is_some_and(Value::is_string)
                && optional_string("ref")
                && optional_string("sha")
        }
        "npm" => {
            object.get("package").is_some_and(Value::is_string)
                && optional_string("registry")
                && optional_string("version")
        }
        _ => false,
    };
    required_valid.then_some(source_type)
}

fn cli_marketplace_source_type(source: &Value) -> Option<&str> {
    let object = known_object(source, &["source", "sourceType"])?;
    let source_type = object.get("sourceType")?.as_str()?;
    if !matches!(source_type, "local" | "git")
        || !object.get("source").is_some_and(Value::is_string)
    {
        return None;
    }
    Some(source_type)
}

async fn run_cli_json(program: &str, arguments: &[&str]) -> Result<Value, CodexAdapterError> {
    let mut child = Command::new(program)
        .current_dir("/")
        .args(arguments)
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
    let result = timeout(CLI_JSON_TIMEOUT, async {
        let output = read_bounded_cli_output(stdout).await?;
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

async fn read_bounded_cli_output(
    reader: impl AsyncRead + Unpin,
) -> Result<Vec<u8>, CodexAdapterError> {
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

async fn discover_skills(process: &mut AppServerProcess) -> SourceResult {
    let cwd = std::env::temp_dir().to_string_lossy().into_owned();
    let response = match process
        .request("skills/list", json!({"cwds": [cwd], "forceReload": true}))
        .await
    {
        Ok(response) => response,
        Err(_) => return SourceResult::failed("skill-discovery-failed"),
    };
    normalize_skills(&response)
}

fn normalize_skills(response: &Value) -> SourceResult {
    const ENTRY_KEYS: &[&str] = &["cwd", "errors", "skills"];
    const SKILL_KEYS: &[&str] = &[
        "dependencies",
        "description",
        "enabled",
        "interface",
        "name",
        "path",
        "scope",
        "shortDescription",
    ];
    const INTERFACE_KEYS: &[&str] = &[
        "brandColor",
        "defaultPrompt",
        "displayName",
        "iconLarge",
        "iconSmall",
        "shortDescription",
    ];
    let Some(object) = known_object(response, &["data"]) else {
        return SourceResult::failed("skill-response-invalid");
    };
    let Some(groups) = object.get("data").and_then(Value::as_array) else {
        return SourceResult::failed("skill-response-invalid");
    };
    if groups.len() > MAX_SOURCE_ENTRIES {
        return SourceResult::failed("skill-response-invalid");
    }
    let mut result = SourceResult::ready(Vec::new());
    let mut ids = HashSet::new();
    let mut skill_count = 0_usize;
    for group in groups {
        let Some(group) = known_object(group, ENTRY_KEYS) else {
            result.mark_degraded("skill-entry-invalid");
            continue;
        };
        let group_has_errors = group
            .get("errors")
            .and_then(Value::as_array)
            .is_none_or(|errors| !errors.is_empty() || errors.len() > 128);
        if group_has_errors {
            result.mark_degraded("skill-source-warning");
        }
        let Some(skills) = group.get("skills").and_then(Value::as_array) else {
            result.mark_degraded("skill-entry-invalid");
            continue;
        };
        skill_count = skill_count.saturating_add(skills.len());
        if skill_count > MAX_SOURCE_ENTRIES {
            return SourceResult::failed("skill-response-invalid");
        }
        for skill in skills {
            let Some(skill) = known_object(skill, SKILL_KEYS) else {
                result.mark_degraded("skill-entry-invalid");
                continue;
            };
            let (Some(raw_name), Some(enabled), Some(scope)) = (
                skill.get("name").and_then(Value::as_str),
                skill.get("enabled").and_then(Value::as_bool),
                skill.get("scope").and_then(Value::as_str),
            ) else {
                result.mark_degraded("skill-entry-invalid");
                continue;
            };
            let Some(id) = normalized_entry_id("skill", raw_name) else {
                result.mark_degraded("skill-entry-invalid");
                continue;
            };
            if !ids.insert(id.clone()) {
                result.mark_degraded("skill-entry-duplicate");
                continue;
            }
            let interface = skill
                .get("interface")
                .and_then(|value| known_object(value, INTERFACE_KEYS));
            if skill.get("interface").is_some_and(|value| !value.is_null()) && interface.is_none() {
                result.mark_degraded("skill-entry-invalid");
                continue;
            }
            let display_name = interface
                .and_then(|interface| interface.get("displayName"))
                .and_then(Value::as_str)
                .unwrap_or(raw_name);
            let summary = interface
                .and_then(|interface| interface.get("shortDescription"))
                .or_else(|| skill.get("shortDescription"))
                .or_else(|| skill.get("description"))
                .and_then(Value::as_str)
                .unwrap_or("Skill discovered through Codex.");
            let has_dependencies = skill
                .get("dependencies")
                .and_then(Value::as_object)
                .and_then(|dependencies| dependencies.get("tools"))
                .and_then(Value::as_array)
                .is_some_and(|tools| !tools.is_empty());
            let (scope, source, managed) = match scope {
                "repo" => (
                    IntegrationScope::Project,
                    IntegrationSource::Repository,
                    false,
                ),
                "admin" => (
                    IntegrationScope::Managed,
                    IntegrationSource::Configuration,
                    true,
                ),
                "system" => (IntegrationScope::Managed, IntegrationSource::Local, true),
                "user" => (IntegrationScope::User, IntegrationSource::Local, false),
                _ => {
                    result.mark_degraded("skill-entry-invalid");
                    continue;
                }
            };
            result.entries.push(IntegrationEntry {
                id,
                kind: IntegrationEntryKind::Skill,
                display_name: normalized_display(display_name, 128, "Skill"),
                summary: normalized_display(summary, 320, "Skill discovered through Codex."),
                scope,
                source,
                installation: IntegrationInstallationState::Installed,
                enablement: if group_has_errors {
                    IntegrationEnablementState::Blocked
                } else if enabled {
                    IntegrationEnablementState::Enabled
                } else {
                    IntegrationEnablementState::Disabled
                },
                authentication: IntegrationAuthenticationState::NotApplicable,
                version: None,
                publisher: None,
                capability_ids: vec!["skill.catalog".to_owned(), "skill.configure".to_owned()],
                permissions: if scope == IntegrationScope::Project {
                    vec![IntegrationPermission {
                        kind: IntegrationPermissionKind::Filesystem,
                        access: IntegrationPermissionAccess::Read,
                        target: "Attached project".to_owned(),
                        required: true,
                    }]
                } else {
                    Vec::new()
                },
                requirements: if has_dependencies {
                    vec![IntegrationRequirement {
                        kind: IntegrationRequirementKind::Binary,
                        name: "Declared skill dependency".to_owned(),
                        state: IntegrationRequirementState::Unknown,
                        detail: Some(
                            "Dependency details remain native-only until validated.".to_owned(),
                        ),
                    }]
                } else {
                    Vec::new()
                },
                policy: IntegrationEntryPolicy {
                    state: IntegrationPolicyState::Allowed,
                    managed,
                    reason: None,
                },
                health: if group_has_errors {
                    IntegrationHealth {
                        state: IntegrationAvailability::Degraded,
                        diagnostic_codes: vec!["skill-source-warning".to_owned()],
                    }
                } else {
                    IntegrationHealth {
                        state: IntegrationAvailability::Ready,
                        diagnostic_codes: Vec::new(),
                    }
                },
            });
        }
    }
    result
}

async fn discover_mcp_servers(process: &mut AppServerProcess) -> SourceResult {
    let data = match paginated_request(
        process,
        "mcpServerStatus/list",
        json!({"limit": 128, "detail": "toolsAndAuthOnly", "threadId": null}),
    )
    .await
    {
        Ok(data) => data,
        Err(_) => return SourceResult::failed("mcp-discovery-failed"),
    };
    normalize_mcp_servers(&data)
}

fn normalize_mcp_servers(servers: &[Value]) -> SourceResult {
    const STATUS_KEYS: &[&str] = &[
        "authStatus",
        "name",
        "resourceTemplates",
        "resources",
        "serverInfo",
        "tools",
    ];
    const INFO_KEYS: &[&str] = &[
        "description",
        "icons",
        "name",
        "title",
        "version",
        "websiteUrl",
    ];
    let mut result = SourceResult::ready(Vec::new());
    let mut ids = HashSet::new();
    for server in servers.iter().take(MAX_SOURCE_ENTRIES) {
        let Some(server) = known_object(server, STATUS_KEYS) else {
            result.mark_degraded("mcp-entry-invalid");
            continue;
        };
        let (Some(raw_name), Some(auth_status)) = (
            server.get("name").and_then(Value::as_str),
            server.get("authStatus").and_then(Value::as_str),
        ) else {
            result.mark_degraded("mcp-entry-invalid");
            continue;
        };
        let Some(id) = normalized_entry_id("mcp", raw_name) else {
            result.mark_degraded("mcp-entry-invalid");
            continue;
        };
        if !ids.insert(id.clone()) {
            result.mark_degraded("mcp-entry-duplicate");
            continue;
        }
        let info = server
            .get("serverInfo")
            .and_then(|value| known_object(value, INFO_KEYS));
        if server
            .get("serverInfo")
            .is_some_and(|value| !value.is_null())
            && info.is_none()
        {
            result.mark_degraded("mcp-entry-invalid");
            continue;
        }
        let display_name = info
            .and_then(|info| info.get("title"))
            .or_else(|| info.and_then(|info| info.get("name")))
            .and_then(Value::as_str)
            .unwrap_or(raw_name);
        let summary = info
            .and_then(|info| info.get("description"))
            .and_then(Value::as_str)
            .unwrap_or("MCP server discovered through Codex.");
        let authentication = match auth_status {
            "notLoggedIn" => IntegrationAuthenticationState::Required,
            "bearerToken" | "oAuth" => IntegrationAuthenticationState::Connected,
            "unsupported" => IntegrationAuthenticationState::NotApplicable,
            _ => {
                result.mark_degraded("mcp-entry-invalid");
                continue;
            }
        };
        let needs_auth = authentication == IntegrationAuthenticationState::Required;
        result.entries.push(IntegrationEntry {
            id,
            kind: IntegrationEntryKind::McpServer,
            display_name: normalized_display(display_name, 128, "MCP server"),
            summary: normalized_display(summary, 320, "MCP server discovered through Codex."),
            scope: IntegrationScope::User,
            source: IntegrationSource::Configuration,
            installation: IntegrationInstallationState::Installed,
            enablement: IntegrationEnablementState::Enabled,
            authentication,
            version: info
                .and_then(|info| info.get("version"))
                .and_then(Value::as_str)
                .and_then(normalized_version),
            publisher: None,
            capability_ids: vec!["mcp.health".to_owned(), "mcp.authorize".to_owned()],
            permissions: vec![IntegrationPermission {
                kind: IntegrationPermissionKind::Network,
                access: IntegrationPermissionAccess::Connect,
                target: "Configured MCP endpoint".to_owned(),
                required: true,
            }],
            requirements: if needs_auth {
                vec![IntegrationRequirement {
                    kind: IntegrationRequirementKind::Authentication,
                    name: "MCP authorization".to_owned(),
                    state: IntegrationRequirementState::Missing,
                    detail: Some("Authorization must use the official returned URL.".to_owned()),
                }]
            } else {
                Vec::new()
            },
            policy: IntegrationEntryPolicy {
                state: if needs_auth {
                    IntegrationPolicyState::ApprovalRequired
                } else {
                    IntegrationPolicyState::Allowed
                },
                managed: false,
                reason: needs_auth.then(|| "Authorization requires explicit review.".to_owned()),
            },
            health: if needs_auth {
                IntegrationHealth {
                    state: IntegrationAvailability::Degraded,
                    diagnostic_codes: vec!["authorization-required".to_owned()],
                }
            } else {
                IntegrationHealth {
                    state: IntegrationAvailability::Ready,
                    diagnostic_codes: Vec::new(),
                }
            },
        });
    }
    result
}

async fn discover_policy(process: &mut AppServerProcess) -> PolicyDiscovery {
    let requirements = process.request("configRequirements/read", json!({})).await;
    let requirements = requirements.ok().and_then(|value| {
        let object = known_object(&value, &["requirements"])?;
        Some(
            object
                .get("requirements")
                .is_some_and(|value| !value.is_null()),
        )
    });

    let profiles = paginated_request(
        process,
        "permissionProfile/list",
        json!({"limit": 128, "cwd": null}),
    )
    .await;
    let profiles_ok = profiles.as_ref().is_ok_and(|profiles| {
        profiles.iter().all(|profile| {
            known_object(profile, &["allowed", "description", "id"]).is_some_and(|profile| {
                profile.get("allowed").is_some_and(Value::is_boolean)
                    && profile.get("id").is_some_and(Value::is_string)
            })
        })
    });

    let config = process
        .request("config/read", json!({"cwd": null, "includeLayers": false}))
        .await;
    let apps_enabled = config.ok().and_then(apps_enabled_from_config);

    let requirements_ok = requirements.is_some() && apps_enabled.is_some();
    let requirements_present = requirements.unwrap_or(false);
    let requirements_state = if requirements_ok {
        IntegrationAvailability::Ready
    } else {
        IntegrationAvailability::Degraded
    };
    let profiles_state = if profiles_ok {
        IntegrationAvailability::Ready
    } else {
        IntegrationAvailability::Degraded
    };
    PolicyDiscovery {
        snapshot: IntegrationPolicySnapshot {
            state: if requirements_ok && profiles_ok {
                IntegrationAvailability::Ready
            } else {
                IntegrationAvailability::Degraded
            },
            source: if requirements_present {
                IntegrationPolicySource::ConfigRequirements
            } else if requirements_ok {
                IntegrationPolicySource::UserConfiguration
            } else {
                IntegrationPolicySource::Unknown
            },
            permission_profiles: profiles_state,
            managed_requirements_present: requirements_present,
            mutation_confirmation_required: true,
            installation: if apps_enabled == Some(false) {
                IntegrationPolicyState::Blocked
            } else if requirements_ok {
                IntegrationPolicyState::ApprovalRequired
            } else {
                IntegrationPolicyState::Unknown
            },
        },
        requirements_state,
        requirements_diagnostic: (!requirements_ok).then(|| "policy-discovery-failed".to_owned()),
        profiles_state,
        profiles_diagnostic: (!profiles_ok)
            .then(|| "permission-profile-discovery-failed".to_owned()),
    }
}

fn apps_enabled_from_config(value: Value) -> Option<bool> {
    let object = known_object(&value, &["config", "layers", "origins"])?;
    let config = object.get("config")?.as_object()?;
    let Some(apps) = config.get("apps") else {
        return Some(true);
    };
    if apps.is_null() {
        return Some(true);
    }
    let apps = apps.as_object()?;
    let Some(defaults) = apps.get("_default") else {
        return Some(true);
    };
    if defaults.is_null() {
        return Some(true);
    }
    let defaults = defaults.as_object()?;
    match defaults.get("enabled") {
        None | Some(Value::Null) => Some(true),
        Some(enabled) => enabled.as_bool(),
    }
}

async fn paginated_request(
    process: &mut AppServerProcess,
    method: &str,
    base_params: Value,
) -> Result<Vec<Value>, CodexAdapterError> {
    let mut params = base_params
        .as_object()
        .cloned()
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    let mut data = Vec::new();
    let mut cursors = HashSet::new();
    for _ in 0..MAX_PAGES {
        let response = process
            .request(method, Value::Object(params.clone()))
            .await?;
        let object = known_object(&response, &["data", "nextCursor"])
            .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
        let page = object
            .get("data")
            .and_then(Value::as_array)
            .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
        if data.len().saturating_add(page.len()) > MAX_SOURCE_ENTRIES {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        data.extend(page.iter().cloned());
        let Some(cursor) = object.get("nextCursor").and_then(Value::as_str) else {
            return Ok(data);
        };
        if cursor.is_empty()
            || cursor.len() > MAX_CURSOR_BYTES
            || cursor.chars().any(char::is_control)
            || !cursors.insert(cursor.to_owned())
        {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        params.insert("cursor".to_owned(), Value::String(cursor.to_owned()));
    }
    Err(CodexAdapterError::InvalidProtocolMessage)
}

fn template_snapshot(cli_version: &str) -> IntegrationCatalogSnapshot {
    let mut snapshot: IntegrationCatalogSnapshot =
        serde_json::from_str(include_str!("../../../fixtures/integration-catalog.json"))
            .expect("integration template fixture must remain valid");
    snapshot.cli_version = cli_version.to_owned();
    snapshot.entries.clear();
    snapshot.refresh_reasons.clear();
    snapshot.catalog_state = IntegrationAvailability::Ready;
    snapshot.policy = IntegrationPolicySnapshot {
        state: IntegrationAvailability::Unknown,
        source: IntegrationPolicySource::Unknown,
        permission_profiles: IntegrationAvailability::Unknown,
        managed_requirements_present: false,
        mutation_confirmation_required: true,
        installation: IntegrationPolicyState::Unknown,
    };
    for capability in &mut snapshot.capabilities {
        capability.availability = IntegrationAvailability::Ready;
        capability.diagnostic_code = None;
        if matches!(
            capability.id.as_str(),
            "connector.catalog"
                | "connector.installed"
                | "plugin.catalog"
                | "marketplace.catalog"
                | "skill.catalog"
                | "mcp.health"
                | "policy.requirements"
                | "permission.profiles"
        ) {
            capability.implementation = IntegrationImplementation::Ready;
        }
    }
    snapshot
}

fn unavailable_snapshot(cli_version: &str, code: &str) -> IntegrationCatalogSnapshot {
    let mut snapshot = template_snapshot(cli_version);
    snapshot.catalog_state = IntegrationAvailability::Unavailable;
    snapshot.entries.clear();
    for capability in &mut snapshot.capabilities {
        capability.availability = IntegrationAvailability::Unavailable;
        capability.diagnostic_code = Some(code.to_owned());
    }
    snapshot.policy = IntegrationPolicySnapshot {
        state: IntegrationAvailability::Unavailable,
        source: IntegrationPolicySource::Unknown,
        permission_profiles: IntegrationAvailability::Unavailable,
        managed_requirements_present: false,
        mutation_confirmation_required: true,
        installation: IntegrationPolicyState::Unknown,
    };
    snapshot.dynamic_tool = unavailable_dynamic_tool(snapshot.dynamic_tool, code);
    snapshot.refresh_reasons.clear();
    snapshot
}

fn unavailable_dynamic_tool(mut contract: DynamicToolContract, code: &str) -> DynamicToolContract {
    contract.state = IntegrationAvailability::Unavailable;
    contract.diagnostic_code = Some(code.to_owned());
    contract
}

fn validated_or_unavailable(
    mut snapshot: IntegrationCatalogSnapshot,
) -> IntegrationCatalogSnapshot {
    snapshot
        .entries
        .sort_by(|left, right| left.id.cmp(&right.id));
    snapshot.refresh_reasons.sort_by_key(|reason| match reason {
        IntegrationRefreshReason::AppListUpdated => 0,
        IntegrationRefreshReason::SkillsChanged => 1,
        IntegrationRefreshReason::McpStatusUpdated => 2,
        IntegrationRefreshReason::ConfigWarning => 3,
    });
    if snapshot.validate().is_ok() {
        snapshot
    } else {
        unavailable_snapshot(&snapshot.cli_version, "integration-contract-invalid")
    }
}

fn set_capabilities(
    snapshot: &mut IntegrationCatalogSnapshot,
    ids: &[&str],
    state: IntegrationAvailability,
    diagnostic: Option<&str>,
) {
    for capability in &mut snapshot.capabilities {
        if ids.contains(&capability.id.as_str()) {
            capability.implementation = IntegrationImplementation::Ready;
            capability.availability = state;
            capability.diagnostic_code = diagnostic.map(str::to_owned);
        }
    }
}

fn replace_entries(
    snapshot: &mut IntegrationCatalogSnapshot,
    kind: IntegrationEntryKind,
    mut entries: Vec<IntegrationEntry>,
) {
    snapshot.entries.retain(|entry| entry.kind != kind);
    snapshot.entries.append(&mut entries);
}

fn finalize_catalog_state(snapshot: &mut IntegrationCatalogSnapshot) {
    let read_capabilities = snapshot
        .capabilities
        .iter()
        .filter(|capability| capability.implementation == IntegrationImplementation::Ready)
        .collect::<Vec<_>>();
    snapshot.catalog_state = if read_capabilities.iter().any(|capability| {
        !matches!(
            capability.availability,
            IntegrationAvailability::Ready | IntegrationAvailability::Unknown
        )
    }) {
        IntegrationAvailability::Degraded
    } else {
        IntegrationAvailability::Ready
    };
}

fn known_object<'a>(value: &'a Value, allowed: &[&str]) -> Option<&'a Map<String, Value>> {
    let object = value.as_object()?;
    object
        .keys()
        .all(|key| allowed.contains(&key.as_str()))
        .then_some(object)
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

fn normalized_display(value: &str, maximum: usize, fallback: &str) -> String {
    let mut output = String::new();
    for character in value.trim().chars() {
        if is_unsafe_display_character(character) {
            continue;
        }
        if output.len() + character.len_utf8() > maximum {
            break;
        }
        output.push(character);
    }
    let output = output.trim();
    if output.is_empty() {
        fallback.to_owned()
    } else {
        output.to_owned()
    }
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

fn normalized_version(value: &str) -> Option<String> {
    (!value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')
        }))
    .then(|| value.to_owned())
}

fn supports_integration_routes(version: &str) -> bool {
    let core = version
        .split_once(['-', '+'])
        .map_or(version, |(core, _)| core);
    let segments = core
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>();
    matches!(segments.as_deref(), Ok([major, minor, _]) if (*major, *minor) == SUPPORTED_CLI_MINOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_identifiers_and_strips_directional_display_text() {
        assert_eq!(
            normalized_entry_id("plugin", "Review Tools / Stable"),
            Some("plugin:review-tools-stable".to_owned())
        );
        assert_eq!(
            normalized_display("Safe\u{202e}name\n", 128, "fallback"),
            "Safename"
        );
        assert!(normalized_entry_id("skill", "////").is_none());
    }

    #[test]
    fn gates_runtime_routes_to_the_reviewed_minor_version() {
        assert!(supports_integration_routes("0.145.0"));
        assert!(supports_integration_routes("0.145.7-dev"));
        assert!(!supports_integration_routes("0.144.6"));
        assert!(!supports_integration_routes("0.146.0"));
    }

    #[test]
    fn fails_closed_for_unreviewed_plugin_sources_and_enums() {
        let response = json!({
            "installed": [],
            "available": [{
                    "authPolicy": "ON_INSTALL",
                    "enabled": true,
                    "installPolicy": "AVAILABLE",
                    "installed": false,
                    "marketplaceName": "curated",
                    "name": "unsafe",
                    "pluginId": "unsafe@curated",
                    "source": {"source": "private", "token": "discard me"},
                    "version": "1.0.0"
            }]
        });

        let discovery = normalize_cli_plugins(&response);

        assert!(discovery.entries.is_empty());
        assert_eq!(discovery.state, IntegrationAvailability::Degraded);
        assert_eq!(
            discovery.diagnostic.as_deref(),
            Some("plugin-entry-invalid")
        );
        let encoded =
            serde_json::to_string(&discovery.entries).expect("normalized entries must serialize");
        assert!(!encoded.contains("discard me"));
    }

    #[test]
    fn treats_missing_app_defaults_as_enabled_and_rejects_invalid_overrides() {
        assert_eq!(
            apps_enabled_from_config(json!({"config": {}, "origins": {}})),
            Some(true)
        );
        assert_eq!(
            apps_enabled_from_config(json!({
                "config": {"apps": {"_default": {"enabled": false}}},
                "origins": {}
            })),
            Some(false)
        );
        assert_eq!(
            apps_enabled_from_config(json!({
                "config": {"apps": {"_default": {"enabled": "private"}}},
                "origins": {}
            })),
            None
        );
    }

    #[tokio::test]
    async fn discovers_sanitized_entries_and_refreshes_invalidated_skills() {
        let script = fixture_script(true);
        let service = IntegrationCatalogService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
            Some(plugin_cli_fixture()),
        );

        let snapshot = service.snapshot().await;

        assert_eq!(snapshot.catalog_state, IntegrationAvailability::Ready);
        assert!(snapshot
            .capabilities
            .iter()
            .filter(|capability| matches!(
                capability.id.as_str(),
                "connector.catalog"
                    | "plugin.catalog"
                    | "marketplace.catalog"
                    | "skill.catalog"
                    | "mcp.health"
                    | "policy.requirements"
                    | "permission.profiles"
            ))
            .all(|capability| capability.implementation == IntegrationImplementation::Ready));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.id == "connector:calendar"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.id == "plugin:review-curated"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.id == "skill:updated-checks"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.id == "mcp:knowledge"));
        assert_eq!(
            snapshot.refresh_reasons,
            vec![IntegrationRefreshReason::SkillsChanged]
        );
        let encoded = serde_json::to_string(&snapshot).expect("snapshot must serialize");
        assert!(!encoded.contains("/private/fixture"));
        assert!(!encoded.contains("secret argument"));
        snapshot
            .validate()
            .expect("runtime snapshot must be strict");
    }

    #[tokio::test]
    async fn preserves_other_sources_when_plugin_discovery_fails() {
        let script = fixture_script(false);
        let service = IntegrationCatalogService::with_command(
            AppServerCommand::test("sh", &["-c", &script]),
            "0.145.0",
            None,
        );

        let snapshot = service.snapshot().await;

        assert_eq!(snapshot.catalog_state, IntegrationAvailability::Degraded);
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.id == "connector:calendar"));
        assert!(!snapshot
            .entries
            .iter()
            .any(|entry| entry.kind == IntegrationEntryKind::Plugin));
        let capability = snapshot
            .capabilities
            .iter()
            .find(|capability| capability.id == "plugin.catalog")
            .expect("plugin capability must exist");
        assert_eq!(capability.availability, IntegrationAvailability::Degraded);
        assert_eq!(
            capability.diagnostic_code.as_deref(),
            Some("plugin-discovery-failed")
        );
        snapshot
            .validate()
            .expect("partial snapshot must be strict");
    }

    #[tokio::test]
    async fn fails_closed_for_unreviewed_cli_versions_without_starting_a_process() {
        let service = IntegrationCatalogService::with_command(
            AppServerCommand::test("sh", &["-c", "exit 91"]),
            "0.146.0",
            None,
        );

        let snapshot = service.snapshot().await;

        assert_eq!(snapshot.catalog_state, IntegrationAvailability::Unavailable);
        assert!(snapshot.entries.is_empty());
        assert!(snapshot.capabilities.iter().all(|capability| {
            capability.availability == IntegrationAvailability::Unavailable
                && capability.diagnostic_code.as_deref() == Some("integration-version-unsupported")
        }));
        snapshot
            .validate()
            .expect("unsupported snapshot must be strict");
    }

    fn plugin_cli_fixture() -> (Value, Value) {
        (
            json!({
                "installed": [],
                "available": [{
                    "authPolicy": "ON_INSTALL",
                    "enabled": false,
                    "installPolicy": "AVAILABLE",
                    "installed": false,
                    "marketplaceName": "curated",
                    "marketplaceSource": {
                        "sourceType": "git",
                        "source": "https://example.invalid/private-marketplace"
                    },
                    "name": "review",
                    "pluginId": "review@curated",
                    "source": {
                        "source": "git-subdir",
                        "url": "https://example.invalid/private-plugin",
                        "path": "/private/fixture/secret argument"
                    },
                    "version": "1.0.0"
                }]
            }),
            json!({
                "marketplaces": [{
                    "marketplaceSource": {
                        "sourceType": "git",
                        "source": "https://example.invalid/private-marketplace"
                    },
                    "name": "curated",
                    "root": "/private/fixture/marketplace"
                }]
            }),
        )
    }

    fn fixture_script(emit_skill_refresh: bool) -> String {
        let refresh = if emit_skill_refresh {
            r#"
printf '%s\n' '{"method":"skills/changed","params":{}}'
read -r _skills_refresh
printf '%s\n' '{"id":9,"result":{"data":[{"cwd":"/private/fixture","errors":[],"skills":[{"description":"Updated checks.","enabled":true,"name":"updated-checks","path":"/private/fixture/SKILL.md","scope":"repo"}]}]}}'
"#
        } else {
            ""
        };
        format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _apps
printf '%s\n' '{{"id":2,"result":{{"data":[{{"description":"Calendar access.","id":"calendar","isAccessible":true,"isEnabled":true,"name":"Calendar"}}],"nextCursor":null}}}}'
read -r _installed
printf '%s\n' '{{"id":3,"result":{{"apps":[{{"callable":true,"enabled":true,"id":"calendar","runtimeName":"calendar"}}]}}}}'
read -r _skills
printf '%s\n' '{{"id":4,"result":{{"data":[{{"cwd":"/private/fixture","errors":[],"skills":[{{"description":"Project checks.","enabled":true,"name":"project-checks","path":"/private/fixture/SKILL.md","scope":"repo"}}]}}]}}}}'
read -r _mcp
printf '%s\n' '{{"id":5,"result":{{"data":[{{"authStatus":"notLoggedIn","name":"knowledge","resourceTemplates":[],"resources":[],"serverInfo":{{"description":"Knowledge lookup.","name":"Knowledge","title":"Knowledge","version":"1.0.0"}},"tools":[]}}],"nextCursor":null}}}}'
read -r _requirements
printf '%s\n' '{{"id":6,"result":{{"requirements":{{"private":"discarded"}}}}}}'
read -r _profiles
printf '%s\n' '{{"id":7,"result":{{"data":[{{"allowed":true,"description":"Default profile","id":"default"}}],"nextCursor":null}}}}'
read -r _config
printf '%s\n' '{{"id":8,"result":{{"config":{{"apps":{{"_default":{{"enabled":true}}}},"instructions":"private instructions"}},"origins":{{}}}}}}'
{refresh}
read -r _keep_open
"#
        )
    }
}
