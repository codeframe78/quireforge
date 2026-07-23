mod lifecycle;
mod presentation;
pub mod types;

pub use lifecycle::{ConversationContinueRequest, SessionLifecycleSnapshot, SessionListRequest};

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    attachment::{ResolvedConversationAttachment, MAX_CONVERSATION_ATTACHMENTS},
    project::{
        ConversationPendingSelection, ConversationReference, ConversationSelectionMetadata,
        ProjectExecutionError, ProjectService, StoredConversationReference,
    },
};

use super::{
    app_server::{
        validate_uuid_v7, AppServerCommand, AppServerNotification, AppServerProcess,
        ConversationActivityDeltaKind as WireActivityDeltaKind,
        ConversationErrorCode as WireErrorCode, ConversationItemDetail as WireItemDetail,
        ConversationItemKind as WireItemKind, ConversationItemStatus as WireItemStatus,
        ConversationNotification, ConversationPlanStepStatus as WirePlanStepStatus,
        ConversationServerDecision as WireServerDecision, ConversationServerRequest,
        ConversationTurnStatus as WireTurnStatus,
    },
    error::CodexAdapterError,
    integration_control::ResolvedIntegrationMention,
    model_selection::{
        current_time_millis, diagnostic_message, AgentSelectionRequest, ModelSelectionApplication,
        ModelSelectionAvailability, ModelSelectionChoice, ModelSelectionDiagnosticCode,
        ModelSelectionOwnership, ModelSelectionPolicy, ModelSelectionProvenance,
        ModelSelectionService, ModelSelectionSnapshot, ModelSelectionUpdateRequest,
        ModelSelectorArguments, PendingModelSelection, PendingSelectionAction,
        MODEL_SELECTOR_TOOL_NAME,
    },
    types::CodexModel,
};
use presentation::{present_path, sanitize_display_text, sanitize_label};
use types::{
    ConversationActivityKind, ConversationActivityStatus, ConversationApproval,
    ConversationApprovalDecision, ConversationApprovalDecisionRequest, ConversationApprovalDetail,
    ConversationApprovalKind, ConversationApprovalPolicy, ConversationApprovalResolution,
    ConversationDiagnosticCode, ConversationEvent, ConversationLifecyclePhase,
    ConversationPlanStep, ConversationPlanStepStatus, ConversationRegistrySnapshot,
    ConversationSandboxMode, ConversationSnapshot, ConversationStartRequest, ConversationState,
    ConversationStreamErrorCode, CONVERSATION_REGISTRY_SCHEMA_VERSION, CONVERSATION_SCHEMA_VERSION,
};

const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_POLL_EVENTS: usize = 32;
const FIRST_POLL_WAIT: Duration = Duration::from_millis(200);
const DRAIN_POLL_WAIT: Duration = Duration::from_millis(1);
const MAX_TRACKED_ACTIVITIES: usize = 256;
const MAX_ACTIVITY_DETAIL_BYTES: usize = 8 * 1024;
pub(crate) const MAX_ACTIVE_CONVERSATIONS: usize = 4;
const MAX_RECENT_CONVERSATIONS: usize = 256;

pub struct ConversationService {
    state: Mutex<ConversationServiceState>,
    command: AppServerCommand,
}

struct ConversationServiceState {
    active: HashMap<String, Arc<Mutex<ActiveConversation>>>,
    starting_projects: HashSet<String>,
    recent: HashMap<String, ConversationSnapshot>,
    last: ConversationSnapshot,
}

struct ActiveConversation {
    conversation_id: String,
    project_id: String,
    model_id: String,
    reasoning_effort: String,
    sandbox_mode: ConversationSandboxMode,
    approval_policy: ConversationApprovalPolicy,
    cwd: PathBuf,
    thread_id: String,
    turn_id: String,
    state: ConversationState,
    next_sequence: u64,
    activities: HashMap<String, ActiveActivity>,
    pending_approval: Option<PendingApproval>,
    model_catalog: Vec<CodexModel>,
    model_selection: ModelSelectionSnapshot,
    agent_selection_request: Option<(AgentSelectionRequest, ModelSelectionApplication)>,
    agent_selection_request_seen: bool,
    process: AppServerProcess,
    finished: bool,
}

#[derive(Clone)]
struct ActiveActivity {
    activity_id: String,
    kind: ConversationActivityKind,
    pending_output: String,
}

#[derive(Clone)]
struct PendingApproval {
    request: ConversationServerRequest,
    approval: ConversationApproval,
}

struct StartedConversation {
    conversation_id: String,
    thread_id: String,
    turn_id: String,
    model_catalog: Vec<CodexModel>,
    model_selection: ModelSelectionSnapshot,
}

#[derive(Clone, Copy)]
struct TerminalState {
    state: ConversationState,
    phase: ConversationLifecyclePhase,
    storage_status: &'static str,
    diagnostic_code: Option<ConversationDiagnosticCode>,
}

pub(crate) struct ConversationNotificationCandidate {
    pub(crate) key: String,
    pub(crate) state: ConversationState,
}

impl ConversationServiceState {
    fn empty() -> Self {
        Self {
            active: HashMap::new(),
            starting_projects: HashSet::new(),
            recent: HashMap::new(),
            last: ConversationSnapshot::empty(),
        }
    }

    fn begin_start(&mut self, project_id: &str) -> Result<(), ConversationDiagnosticCode> {
        if self.active.len() + self.starting_projects.len() >= MAX_ACTIVE_CONVERSATIONS {
            return Err(ConversationDiagnosticCode::ParallelCapacityReached);
        }
        if !self.starting_projects.insert(project_id.to_owned()) {
            return Err(ConversationDiagnosticCode::ProjectBusy);
        }
        Ok(())
    }

    fn finish_start(&mut self, project_id: &str) {
        self.starting_projects.remove(project_id);
    }

    fn remember(&mut self, snapshot: ConversationSnapshot) {
        if let Some(conversation_id) = snapshot.conversation_id.clone() {
            if self.recent.len() >= MAX_RECENT_CONVERSATIONS
                && !self.recent.contains_key(&conversation_id)
            {
                if let Some(oldest) = self.recent.keys().next().cloned() {
                    self.recent.remove(&oldest);
                }
            }
            let mut recent = snapshot.clone();
            recent.events.clear();
            self.recent.insert(conversation_id, recent);
        }
        self.last = snapshot;
    }

    fn recent_or_not_found(&self, conversation_id: &str) -> ConversationSnapshot {
        self.recent
            .get(conversation_id)
            .cloned()
            .unwrap_or_else(|| {
                ConversationSnapshot::unavailable(ConversationDiagnosticCode::ConversationNotFound)
            })
    }
}

impl Default for ConversationService {
    fn default() -> Self {
        Self {
            state: Mutex::new(ConversationServiceState::empty()),
            command: AppServerCommand::codex("codex"),
        }
    }
}

impl ConversationService {
    #[cfg(test)]
    fn with_command(command: AppServerCommand) -> Self {
        Self {
            state: Mutex::new(ConversationServiceState::empty()),
            command,
        }
    }

    pub async fn status(&self) -> ConversationSnapshot {
        let state = self.state.lock().await;
        let mut snapshot = state.last.clone();
        snapshot.events.clear();
        snapshot
    }

    pub async fn active(&self) -> ConversationRegistrySnapshot {
        let slots = {
            let state = self.state.lock().await;
            state.active.values().cloned().collect::<Vec<_>>()
        };
        let mut conversations = Vec::with_capacity(slots.len());
        for slot in slots {
            let active = slot.lock().await;
            if !active.finished {
                conversations.push(active.snapshot(Vec::new(), None));
            }
        }
        conversations.sort_by(|left, right| left.conversation_id.cmp(&right.conversation_id));
        ConversationRegistrySnapshot {
            schema_version: CONVERSATION_REGISTRY_SCHEMA_VERSION,
            capacity: MAX_ACTIVE_CONVERSATIONS as u8,
            conversations,
        }
    }

    pub(crate) async fn notification_candidate(
        &self,
        conversation_id: &str,
    ) -> Option<ConversationNotificationCandidate> {
        if validate_uuid_v7(conversation_id).is_err() {
            return None;
        }
        let state = self.state.lock().await;
        let snapshot = state.recent.get(conversation_id)?;
        let key = match snapshot.state {
            ConversationState::WaitingForApproval => format!(
                "approval:{}",
                snapshot.pending_approval.as_ref()?.approval_id
            ),
            ConversationState::Completed => format!("terminal:{conversation_id}:completed"),
            ConversationState::Blocked => format!("terminal:{conversation_id}:blocked"),
            ConversationState::Failed => format!("terminal:{conversation_id}:failed"),
            _ => return None,
        };
        Some(ConversationNotificationCandidate {
            key,
            state: snapshot.state,
        })
    }

    async fn active_slot(
        &self,
        conversation_id: &str,
    ) -> Result<Arc<Mutex<ActiveConversation>>, ConversationSnapshot> {
        let state = self.state.lock().await;
        state
            .active
            .get(conversation_id)
            .cloned()
            .ok_or_else(|| state.recent_or_not_found(conversation_id))
    }

    async fn remember_snapshot(&self, snapshot: ConversationSnapshot) {
        self.state.lock().await.remember(snapshot);
    }

    async fn complete_slot(
        &self,
        conversation_id: &str,
        slot: &Arc<Mutex<ActiveConversation>>,
        snapshot: ConversationSnapshot,
    ) {
        let mut state = self.state.lock().await;
        if state
            .active
            .get(conversation_id)
            .is_some_and(|registered| Arc::ptr_eq(registered, slot))
        {
            state.active.remove(conversation_id);
        }
        state.remember(snapshot);
    }

    #[cfg(test)]
    pub async fn start(
        &self,
        request: ConversationStartRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        self.start_with_mentions(request, projects, Vec::new(), Vec::new())
            .await
    }

    pub(crate) async fn start_with_mentions(
        &self,
        request: ConversationStartRequest,
        projects: &ProjectService,
        mentions: Vec<ResolvedIntegrationMention>,
        attachments: Vec<ResolvedConversationAttachment>,
    ) -> ConversationSnapshot {
        if let Err(code) = validate_start_request(&request) {
            return ConversationSnapshot::unavailable(code);
        }
        if mentions.len() != request.integration_entry_ids.len() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::IntegrationUnavailable,
            );
        }
        if attachments.len() != request.attachment_ids.len() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            );
        }

        {
            let mut state = self.state.lock().await;
            if let Err(code) = state.begin_start(&request.project_id) {
                return ConversationSnapshot::unavailable(code);
            }
        }

        if let Err(error) = projects.reserve_execution(&request.project_id) {
            self.state.lock().await.finish_start(&request.project_id);
            return ConversationSnapshot::unavailable(map_project_error(error));
        }
        let cwd = match projects.execution_cwd(&request.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                projects.release_execution(&request.project_id);
                self.state.lock().await.finish_start(&request.project_id);
                return ConversationSnapshot::unavailable(map_project_error(error));
            }
        };

        match self
            .start_reserved(&request, &cwd, projects, &mentions, &attachments)
            .await
        {
            Ok(active) => {
                let events = vec![
                    ConversationEvent::Lifecycle {
                        sequence: 1,
                        phase: ConversationLifecyclePhase::Starting,
                    },
                    ConversationEvent::Lifecycle {
                        sequence: 2,
                        phase: ConversationLifecyclePhase::Running,
                    },
                ];
                let snapshot = active.snapshot(events, None);
                let conversation_id = active.conversation_id.clone();
                let mut state = self.state.lock().await;
                state.finish_start(&request.project_id);
                state
                    .active
                    .insert(conversation_id, Arc::new(Mutex::new(active)));
                state.remember(snapshot.clone());
                snapshot
            }
            Err(code) => {
                projects.release_execution(&request.project_id);
                let snapshot = ConversationSnapshot::unavailable(code);
                let mut state = self.state.lock().await;
                state.finish_start(&request.project_id);
                state.remember(snapshot.clone());
                snapshot
            }
        }
    }

    async fn start_reserved(
        &self,
        request: &ConversationStartRequest,
        cwd: &Path,
        projects: &ProjectService,
        mentions: &[ResolvedIntegrationMention],
        attachments: &[ResolvedConversationAttachment],
    ) -> Result<ActiveConversation, ConversationDiagnosticCode> {
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let started =
            match start_on_process(&mut process, request, cwd, projects, mentions, attachments)
                .await
            {
                Ok(started) => started,
                Err(error) => {
                    let _ = process.shutdown().await;
                    return Err(error);
                }
            };

        Ok(ActiveConversation {
            conversation_id: started.conversation_id,
            project_id: request.project_id.clone(),
            model_id: request.model_id.clone(),
            reasoning_effort: request.reasoning_effort.clone(),
            sandbox_mode: request.sandbox_mode,
            approval_policy: request.approval_policy,
            cwd: cwd.to_path_buf(),
            thread_id: started.thread_id,
            turn_id: started.turn_id,
            state: ConversationState::Running,
            next_sequence: 3,
            activities: HashMap::new(),
            pending_approval: None,
            model_catalog: started.model_catalog,
            model_selection: started.model_selection,
            agent_selection_request: None,
            agent_selection_request_seen: false,
            process,
            finished: false,
        })
    }

    pub async fn update_model_selection(
        &self,
        request: ModelSelectionUpdateRequest,
        projects: &ProjectService,
    ) -> Result<ModelSelectionSnapshot, ModelSelectionDiagnosticCode> {
        if validate_uuid_v7(&request.conversation_id).is_err() {
            return Err(ModelSelectionDiagnosticCode::ConversationNotFound);
        }
        let reference = projects
            .conversation_reference(&request.conversation_id)
            .map_err(|_| ModelSelectionDiagnosticCode::ConversationNotFound)?;
        let current = model_selection_from_reference(&reference)
            .map_err(|_| ModelSelectionDiagnosticCode::MetadataUnavailable)?;

        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|_| ModelSelectionDiagnosticCode::CatalogUnavailable)?;
        let catalog = process
            .discover_models()
            .await
            .map(|value| value.0)
            .map_err(|_| ModelSelectionDiagnosticCode::CatalogUnavailable);
        let _ = process.shutdown().await;
        let catalog = catalog?;

        let accepted_pending = if request.pending_action == PendingSelectionAction::Accept {
            Some(
                current
                    .pending
                    .as_ref()
                    .ok_or(ModelSelectionDiagnosticCode::InvalidRequest)?
                    .choice
                    .clone(),
            )
        } else {
            None
        };
        let requested_choice = accepted_pending.unwrap_or_else(|| request.choice.clone());
        ModelSelectionService::validate_choice(&requested_choice, &catalog)?;
        ModelSelectionService::validate_policy(&request.policy, &catalog, &current.effective)?;
        ModelSelectionService::validate_policy_shape(&request.policy, &requested_choice)?;

        let effective = current.effective.clone();
        let mut pending = match request.pending_action {
            PendingSelectionAction::Keep => current.pending.clone(),
            PendingSelectionAction::Accept | PendingSelectionAction::Dismiss => None,
        };
        if requested_choice != effective {
            pending = Some(PendingModelSelection {
                choice: requested_choice,
                provenance: ModelSelectionProvenance::User,
                application: ModelSelectionApplication::Manual,
                rationale: "User selected this model and reasoning for the next turn.".to_owned(),
                requested_at_ms: current_time_millis(),
            });
        } else if pending.as_ref().is_some_and(|pending| {
            pending.provenance == ModelSelectionProvenance::Codex
                && (request.policy.user_locked
                    || request.policy.ownership == ModelSelectionOwnership::Manual)
        }) {
            pending = None;
        }

        let updated =
            ModelSelectionSnapshot::ready(current.availability, effective, pending, request.policy);
        persist_model_selection(projects, &request.conversation_id, &updated, None)
            .map_err(|_| ModelSelectionDiagnosticCode::MetadataUnavailable)?;

        let slot = {
            let state = self.state.lock().await;
            state.active.get(&request.conversation_id).cloned()
        };
        if let Some(slot) = slot {
            let mut active = slot.lock().await;
            active.model_selection = updated.clone();
            active.model_catalog = catalog;
            if active.model_selection.policy.user_locked
                || active.model_selection.policy.ownership == ModelSelectionOwnership::Manual
                || active
                    .model_selection
                    .pending
                    .as_ref()
                    .is_some_and(|pending| pending.provenance == ModelSelectionProvenance::User)
            {
                active.agent_selection_request = None;
            }
        }
        Ok(updated)
    }

    pub async fn poll(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let slot = match self.active_slot(&conversation_id).await {
            Ok(slot) => slot,
            Err(snapshot) => return snapshot,
        };
        let mut active = slot.lock().await;
        if active.finished {
            return active.snapshot(Vec::new(), None);
        }

        let mut events = Vec::new();
        let mut terminal = None;
        for index in 0..MAX_POLL_EVENTS {
            let wait = if index == 0 {
                FIRST_POLL_WAIT
            } else {
                DRAIN_POLL_WAIT
            };
            match active.process.next_notification_with_timeout(wait).await {
                Ok(Some(notification)) => {
                    if matches!(
                        &notification,
                        AppServerNotification::ConversationRequest(
                            ConversationServerRequest::DynamicTool { .. }
                        )
                    ) {
                        match handle_dynamic_tool_request(&mut active, notification).await {
                            Ok(Some(event)) => events.push(event),
                            Ok(None) => {}
                            Err(code) => {
                                terminal = Some(protocol_terminal(code));
                                break;
                            }
                        }
                        continue;
                    }
                    match apply_notification(&mut active, notification) {
                        Ok((event, completed)) => {
                            if let Some(event) = event {
                                events.push(event);
                            }
                            if completed.is_some() {
                                terminal = completed;
                                break;
                            }
                        }
                        Err(code) => {
                            terminal = Some(protocol_terminal(code));
                            break;
                        }
                    }
                }
                Ok(None) => break,
                Err(CodexAdapterError::UnexpectedServerRequest) => {
                    terminal = Some(TerminalState {
                        state: ConversationState::Blocked,
                        phase: ConversationLifecyclePhase::Blocked,
                        storage_status: "blocked",
                        diagnostic_code: Some(ConversationDiagnosticCode::ApprovalRequired),
                    });
                    break;
                }
                Err(error) => {
                    terminal = Some(protocol_terminal(map_adapter_error(&error)));
                    break;
                }
            }
        }

        if let Some(terminal) = terminal {
            let snapshot = finish_active(&mut active, projects, terminal, events).await;
            drop(active);
            self.complete_slot(&conversation_id, &slot, snapshot.clone())
                .await;
            return snapshot;
        }
        let snapshot = active.snapshot(events, None);
        drop(active);
        self.remember_snapshot(snapshot.clone()).await;
        snapshot
    }

    pub async fn interrupt(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let slot = match self.active_slot(&conversation_id).await {
            Ok(slot) => slot,
            Err(snapshot) => return snapshot,
        };
        let mut active = slot.lock().await;
        if active.finished {
            return active.snapshot(Vec::new(), None);
        }
        if active.state == ConversationState::Stopping {
            return active.snapshot(Vec::new(), None);
        }

        let mut events = Vec::new();
        if let Some(pending) = active.pending_approval.clone() {
            if let Err(error) = active
                .process
                .respond_server_request(
                    pending.request.request_id(),
                    approval_response(&pending.request, ConversationApprovalDecision::Cancel),
                )
                .await
            {
                let snapshot = finish_active(
                    &mut active,
                    projects,
                    protocol_terminal(map_adapter_error(&error)),
                    events,
                )
                .await;
                drop(active);
                self.complete_slot(&conversation_id, &slot, snapshot.clone())
                    .await;
                return snapshot;
            }
            active.pending_approval = None;
            events.push(ConversationEvent::ApprovalResolved {
                sequence: active.take_sequence(),
                approval_id: pending.approval.approval_id,
                resolution: ConversationApprovalResolution::Canceled,
            });
        }

        let thread_id = active.thread_id.clone();
        let turn_id = active.turn_id.clone();
        let result = active
            .process
            .request(
                "turn/interrupt",
                json!({
                    "threadId": thread_id,
                    "turnId": turn_id,
                }),
            )
            .await;
        if let Err(error) = result {
            let terminal = if matches!(error, CodexAdapterError::UnexpectedServerRequest) {
                TerminalState {
                    state: ConversationState::Blocked,
                    phase: ConversationLifecyclePhase::Blocked,
                    storage_status: "blocked",
                    diagnostic_code: Some(ConversationDiagnosticCode::ApprovalRequired),
                }
            } else {
                protocol_terminal(map_adapter_error(&error))
            };
            let snapshot = finish_active(&mut active, projects, terminal, events).await;
            drop(active);
            self.complete_slot(&conversation_id, &slot, snapshot.clone())
                .await;
            return snapshot;
        }

        events.push(active.lifecycle_event(ConversationLifecyclePhase::Stopping));
        active.state = ConversationState::Stopping;
        if projects
            .record_conversation_status(&active.conversation_id, "stopping")
            .is_err()
        {
            let snapshot = finish_active(
                &mut active,
                projects,
                protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable),
                events,
            )
            .await;
            drop(active);
            self.complete_slot(&conversation_id, &slot, snapshot.clone())
                .await;
            return snapshot;
        }
        let snapshot = active.snapshot(events, None);
        drop(active);
        self.remember_snapshot(snapshot.clone()).await;
        snapshot
    }

    pub async fn decide_approval(
        &self,
        request: ConversationApprovalDecisionRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if validate_uuid_v7(&request.conversation_id).is_err()
            || validate_uuid_v7(&request.approval_id).is_err()
        {
            return ConversationSnapshot::unavailable(ConversationDiagnosticCode::ApprovalNotFound);
        }
        let conversation_id = request.conversation_id.clone();
        let slot = match self.active_slot(&conversation_id).await {
            Ok(slot) => slot,
            Err(snapshot) => return snapshot,
        };
        let mut active = slot.lock().await;
        if active.finished {
            return active.snapshot(
                Vec::new(),
                Some(ConversationDiagnosticCode::ApprovalNotFound),
            );
        }
        let Some(pending) = active.pending_approval.clone() else {
            return active.snapshot(
                Vec::new(),
                Some(ConversationDiagnosticCode::ApprovalNotFound),
            );
        };
        if pending.approval.approval_id != request.approval_id {
            return active.snapshot(
                Vec::new(),
                Some(ConversationDiagnosticCode::ApprovalNotFound),
            );
        }
        if !pending.approval.decisions.contains(&request.decision) {
            return active.snapshot(
                Vec::new(),
                Some(ConversationDiagnosticCode::ApprovalDecisionUnavailable),
            );
        }

        if let Err(error) = active
            .process
            .respond_server_request(
                pending.request.request_id(),
                approval_response(&pending.request, request.decision),
            )
            .await
        {
            let snapshot = finish_active(
                &mut active,
                projects,
                protocol_terminal(map_adapter_error(&error)),
                Vec::new(),
            )
            .await;
            drop(active);
            self.complete_slot(&conversation_id, &slot, snapshot.clone())
                .await;
            return snapshot;
        }

        active.pending_approval = None;
        let resolution = match request.decision {
            ConversationApprovalDecision::Approve => ConversationApprovalResolution::Approved,
            ConversationApprovalDecision::Decline => ConversationApprovalResolution::Declined,
            ConversationApprovalDecision::Cancel => ConversationApprovalResolution::Canceled,
        };
        let mut events = vec![ConversationEvent::ApprovalResolved {
            sequence: active.take_sequence(),
            approval_id: pending.approval.approval_id,
            resolution,
        }];

        if request.decision == ConversationApprovalDecision::Cancel {
            let thread_id = active.thread_id.clone();
            let turn_id = active.turn_id.clone();
            let result = active
                .process
                .request(
                    "turn/interrupt",
                    json!({
                        "threadId": thread_id,
                        "turnId": turn_id,
                    }),
                )
                .await;
            if let Err(error) = result {
                let snapshot = finish_active(
                    &mut active,
                    projects,
                    protocol_terminal(map_adapter_error(&error)),
                    events,
                )
                .await;
                drop(active);
                self.complete_slot(&conversation_id, &slot, snapshot.clone())
                    .await;
                return snapshot;
            }
            events.push(active.lifecycle_event(ConversationLifecyclePhase::Stopping));
            active.state = ConversationState::Stopping;
            if projects
                .record_conversation_status(&active.conversation_id, "stopping")
                .is_err()
            {
                let snapshot = finish_active(
                    &mut active,
                    projects,
                    protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable),
                    events,
                )
                .await;
                drop(active);
                self.complete_slot(&conversation_id, &slot, snapshot.clone())
                    .await;
                return snapshot;
            }
        } else {
            active.state = ConversationState::Running;
        }

        let snapshot = active.snapshot(events, None);
        drop(active);
        self.remember_snapshot(snapshot.clone()).await;
        snapshot
    }
}

fn approval_response(
    request: &ConversationServerRequest,
    decision: ConversationApprovalDecision,
) -> Value {
    match request {
        ConversationServerRequest::DynamicTool { .. } => json!({
            "success": false,
            "contentItems": [{
                "type": "inputText",
                "text": "The selector request was canceled before completion."
            }]
        }),
        ConversationServerRequest::CommandExecution { .. }
        | ConversationServerRequest::FileChange { .. } => json!({
            "decision": match decision {
                ConversationApprovalDecision::Approve => "accept",
                ConversationApprovalDecision::Decline => "decline",
                ConversationApprovalDecision::Cancel => "cancel",
            }
        }),
        ConversationServerRequest::Permissions { permissions, .. } => json!({
            "permissions": if decision == ConversationApprovalDecision::Approve {
                permissions.clone()
            } else {
                json!({})
            },
            "scope": "turn",
            "strictAutoReview": false,
        }),
    }
}

async fn start_on_process(
    process: &mut AppServerProcess,
    request: &ConversationStartRequest,
    cwd: &Path,
    projects: &ProjectService,
    mentions: &[ResolvedIntegrationMention],
    attachments: &[ResolvedConversationAttachment],
) -> Result<StartedConversation, ConversationDiagnosticCode> {
    let (models, _) = process
        .discover_models()
        .await
        .map_err(|error| map_adapter_error(&error))?;
    let Some(model) = models.iter().find(|model| model.id == request.model_id) else {
        return Err(ConversationDiagnosticCode::ModelUnavailable);
    };
    if !model
        .supported_reasoning_efforts
        .contains(&request.reasoning_effort)
    {
        return Err(ConversationDiagnosticCode::ReasoningUnavailable);
    }
    let effective = ModelSelectionChoice {
        model_id: request.model_id.clone(),
        reasoning_effort: request.reasoning_effort.clone(),
    };
    ModelSelectionService::validate_policy(&request.selection_policy, &models, &effective)
        .map_err(|_| ConversationDiagnosticCode::InvalidRequest)?;

    let thread_params = json!({
        "cwd": cwd,
        "model": request.model_id,
        "approvalPolicy": request.approval_policy.as_protocol_value(),
        "sandbox": request.sandbox_mode.as_protocol_value(),
        "dynamicTools": [ModelSelectionService::dynamic_tool_spec()],
    });
    let (thread_result, selection_availability) =
        match process.request("thread/start", thread_params).await {
            Ok(result) => (result, ModelSelectionAvailability::Ready),
            Err(CodexAdapterError::RpcRejected) => {
                let fallback = process
                    .request(
                        "thread/start",
                        json!({
                            "cwd": cwd,
                            "model": request.model_id,
                            "approvalPolicy": request.approval_policy.as_protocol_value(),
                            "sandbox": request.sandbox_mode.as_protocol_value(),
                        }),
                    )
                    .await
                    .map_err(|error| map_adapter_error(&error))?;
                (fallback, ModelSelectionAvailability::RecommendationOnly)
            }
            Err(error) => return Err(map_adapter_error(&error)),
        };
    let thread = parse_thread_start(thread_result, cwd)?;
    let conversation_id = Uuid::now_v7().to_string();
    let model_selection = ModelSelectionSnapshot::ready(
        selection_availability,
        effective,
        None,
        request.selection_policy.clone(),
    );
    let allowed_model_ids_json = serde_json::to_string(&model_selection.policy.allowed_model_ids)
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
    projects
        .record_conversation_reference(ConversationReference {
            conversation_id: &conversation_id,
            project_id: &request.project_id,
            codex_thread_id: &thread.thread.id,
            model_id: &request.model_id,
            reasoning_effort: &request.reasoning_effort,
            sandbox_mode: request.sandbox_mode.as_protocol_value(),
            approval_policy: request.approval_policy.as_protocol_value(),
            parent_conversation_id: None,
            selection: ConversationSelectionMetadata {
                availability: availability_storage_value(model_selection.availability),
                ownership: model_selection.policy.ownership.as_storage_value(),
                user_locked: model_selection.policy.user_locked,
                allowed_model_ids_json: &allowed_model_ids_json,
                reasoning_ceiling: model_selection.policy.reasoning_ceiling.as_deref(),
                pending: None,
            },
        })
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;

    let mut input = vec![json!({"type": "text", "text": request.prompt})];
    input.extend(
        attachments
            .iter()
            .map(ResolvedConversationAttachment::protocol_input),
    );
    input.extend(mentions.iter().map(|mention| {
        json!({
            "type": "mention",
            "name": mention.name,
            "path": mention.path,
        })
    }));
    let turn_result = process
        .request(
            "turn/start",
            json!({
                "threadId": thread.thread.id,
                "input": input,
                "cwd": cwd,
                "model": request.model_id,
                "effort": request.reasoning_effort,
                "approvalPolicy": request.approval_policy.as_protocol_value(),
                "sandboxPolicy": sandbox_policy(request.sandbox_mode, cwd),
            }),
        )
        .await
        .map_err(|error| {
            let _ = projects.record_conversation_status(&conversation_id, "failed");
            map_adapter_error(&error)
        })?;
    let turn = match parse_turn_start(turn_result) {
        Ok(turn) => turn,
        Err(error) => {
            let _ = projects.record_conversation_status(&conversation_id, "failed");
            return Err(error);
        }
    };
    if projects
        .record_conversation_turn(&conversation_id, &turn.turn.id)
        .is_err()
    {
        let _ = projects.record_conversation_status(&conversation_id, "failed");
        return Err(ConversationDiagnosticCode::MetadataUnavailable);
    }

    Ok(StartedConversation {
        conversation_id,
        thread_id: thread.thread.id,
        turn_id: turn.turn.id,
        model_catalog: models,
        model_selection,
    })
}

impl ActiveConversation {
    fn snapshot(
        &self,
        events: Vec<ConversationEvent>,
        diagnostic_code: Option<ConversationDiagnosticCode>,
    ) -> ConversationSnapshot {
        ConversationSnapshot {
            schema_version: CONVERSATION_SCHEMA_VERSION,
            state: self.state,
            conversation_id: Some(self.conversation_id.clone()),
            project_id: Some(self.project_id.clone()),
            model_id: Some(self.model_id.clone()),
            reasoning_effort: Some(self.reasoning_effort.clone()),
            model_selection: Some(self.model_selection.clone()),
            sandbox_mode: Some(self.sandbox_mode),
            approval_policy: Some(self.approval_policy),
            pending_approval: self
                .pending_approval
                .as_ref()
                .map(|pending| pending.approval.clone()),
            events,
            diagnostic_code,
        }
    }

    fn lifecycle_event(&mut self, phase: ConversationLifecyclePhase) -> ConversationEvent {
        let sequence = self.take_sequence();
        ConversationEvent::Lifecycle { sequence, phase }
    }

    fn take_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }
}

async fn finish_active(
    active: &mut ActiveConversation,
    projects: &ProjectService,
    mut terminal: TerminalState,
    mut events: Vec<ConversationEvent>,
) -> ConversationSnapshot {
    if active.finished {
        return active.snapshot(Vec::new(), terminal.diagnostic_code);
    }
    if persist_model_selection(
        projects,
        &active.conversation_id,
        &active.model_selection,
        None,
    )
    .is_err()
    {
        terminal = protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable);
    }
    if projects
        .record_conversation_status(&active.conversation_id, terminal.storage_status)
        .is_err()
    {
        terminal = protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable);
    }
    events.push(active.lifecycle_event(terminal.phase));
    active.state = terminal.state;
    active.pending_approval = None;
    let _ = active.process.shutdown().await;
    projects.release_execution(&active.project_id);
    active.finished = true;
    active.snapshot(events, terminal.diagnostic_code)
}

async fn handle_dynamic_tool_request(
    active: &mut ActiveConversation,
    notification: AppServerNotification,
) -> Result<Option<ConversationEvent>, ConversationDiagnosticCode> {
    let AppServerNotification::ConversationRequest(ConversationServerRequest::DynamicTool {
        request_id,
        thread_id,
        turn_id,
        call_id: _,
        namespace,
        tool,
        arguments,
    }) = notification
    else {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    };
    ensure_turn(active, &thread_id, &turn_id)?;

    let result = if active.state != ConversationState::Running
        || active.pending_approval.is_some()
        || namespace.is_some()
        || tool != MODEL_SELECTOR_TOOL_NAME
        || active.model_selection.availability != ModelSelectionAvailability::Ready
    {
        Err(ModelSelectionDiagnosticCode::ControlUnavailable)
    } else {
        ModelSelectionService::parse_arguments(arguments)
    };

    let mut event = None;
    let (success, text) = match result {
        Ok(ModelSelectorArguments::Inspect) => match ModelSelectionService::inspection_text(
            &active.model_selection,
            &active.model_catalog,
        ) {
            Ok(text) => (true, text),
            Err(code) => (false, diagnostic_message(code).to_owned()),
        },
        Ok(ModelSelectorArguments::Request(mut request)) => {
            if active.agent_selection_request_seen {
                (
                    false,
                    diagnostic_message(ModelSelectionDiagnosticCode::RequestAlreadyMade).to_owned(),
                )
            } else {
                active.agent_selection_request_seen = true;
                request.rationale = sanitize_display_text(&request.rationale, &active.cwd, 240);
                let evaluation = if request.rationale.is_empty() {
                    Err(ModelSelectionDiagnosticCode::InvalidRequest)
                } else {
                    ModelSelectionService::evaluate_agent_request(
                        request,
                        &active.model_selection.policy,
                        &active.model_catalog,
                        active.model_selection.pending.as_ref(),
                    )
                };
                match evaluation {
                    Ok((request, application)) => {
                        event = Some(ConversationEvent::ModelSelectionRequested {
                            sequence: active.take_sequence(),
                            choice: request.choice.clone(),
                            application,
                            rationale: request.rationale.clone(),
                        });
                        active.agent_selection_request = Some((request, application));
                        (
                            true,
                            "QuireForge accepted one bounded selector request for evaluation after this turn completes. The executing turn is unchanged.".to_owned(),
                        )
                    }
                    Err(code) => (false, diagnostic_message(code).to_owned()),
                }
            }
        }
        Err(code) => (false, diagnostic_message(code).to_owned()),
    };

    active
        .process
        .respond_server_request(
            &request_id,
            json!({
                "success": success,
                "contentItems": [{"type": "inputText", "text": text}],
            }),
        )
        .await
        .map_err(|error| map_adapter_error(&error))?;
    Ok(event)
}

pub(super) fn persist_model_selection(
    projects: &ProjectService,
    conversation_id: &str,
    snapshot: &ModelSelectionSnapshot,
    effective: Option<(&str, &str)>,
) -> Result<(), ConversationDiagnosticCode> {
    let allowed_model_ids_json = serde_json::to_string(&snapshot.policy.allowed_model_ids)
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
    let pending = snapshot
        .pending
        .as_ref()
        .map(|pending| ConversationPendingSelection {
            model_id: &pending.choice.model_id,
            reasoning_effort: &pending.choice.reasoning_effort,
            rationale: &pending.rationale,
            provenance: pending.provenance.as_storage_value(),
            application: pending.application.as_storage_value(),
            requested_at_ms: pending.requested_at_ms,
        });
    projects
        .record_model_selection(
            conversation_id,
            effective,
            ConversationSelectionMetadata {
                availability: availability_storage_value(snapshot.availability),
                ownership: snapshot.policy.ownership.as_storage_value(),
                user_locked: snapshot.policy.user_locked,
                allowed_model_ids_json: &allowed_model_ids_json,
                reasoning_ceiling: snapshot.policy.reasoning_ceiling.as_deref(),
                pending,
            },
        )
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)
}

pub(super) fn model_selection_from_reference(
    reference: &StoredConversationReference,
) -> Result<ModelSelectionSnapshot, ConversationDiagnosticCode> {
    let availability = match reference.selector_availability.as_str() {
        "ready" => ModelSelectionAvailability::Ready,
        "recommendation-only" => ModelSelectionAvailability::RecommendationOnly,
        "unavailable" => ModelSelectionAvailability::Unavailable,
        _ => return Err(ConversationDiagnosticCode::MetadataUnavailable),
    };
    let ownership = ModelSelectionOwnership::from_storage_value(&reference.selector_mode)
        .ok_or(ConversationDiagnosticCode::MetadataUnavailable)?;
    let allowed_model_ids: Vec<String> =
        serde_json::from_str(&reference.selector_allowed_model_ids_json)
            .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
    let effective = ModelSelectionChoice {
        model_id: reference.model_id.clone(),
        reasoning_effort: reference.reasoning_effort.clone(),
    };
    let policy = ModelSelectionPolicy {
        ownership,
        user_locked: reference.selector_user_locked,
        allowed_model_ids,
        reasoning_ceiling: reference.selector_reasoning_ceiling.clone(),
    };
    ModelSelectionService::validate_policy_shape(&policy, &effective)
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;

    let pending = match (
        reference.selector_pending_model_id.as_ref(),
        reference.selector_pending_reasoning_effort.as_ref(),
        reference.selector_pending_rationale.as_ref(),
        reference.selector_pending_provenance.as_deref(),
        reference.selector_pending_application.as_deref(),
        reference.selector_pending_requested_at_ms,
    ) {
        (None, None, None, None, None, None) => None,
        (
            Some(model_id),
            Some(reasoning_effort),
            Some(rationale),
            Some(provenance),
            Some(application),
            Some(requested_at_ms),
        ) if requested_at_ms >= 0 => {
            let parsed = ModelSelectionService::parse_arguments(json!({
                "action": "request",
                "modelId": model_id,
                "reasoningEffort": reasoning_effort,
                "rationale": rationale,
            }))
            .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
            let ModelSelectorArguments::Request(request) = parsed else {
                return Err(ConversationDiagnosticCode::MetadataUnavailable);
            };
            let pending = PendingModelSelection {
                choice: request.choice,
                provenance: ModelSelectionProvenance::from_storage_value(provenance)
                    .ok_or(ConversationDiagnosticCode::MetadataUnavailable)?,
                application: ModelSelectionApplication::from_storage_value(application)
                    .ok_or(ConversationDiagnosticCode::MetadataUnavailable)?,
                rationale: request.rationale,
                requested_at_ms,
            };
            if !matches!(
                (pending.provenance, pending.application),
                (
                    ModelSelectionProvenance::User,
                    ModelSelectionApplication::Manual
                ) | (
                    ModelSelectionProvenance::Codex,
                    ModelSelectionApplication::Recommendation
                ) | (
                    ModelSelectionProvenance::Codex,
                    ModelSelectionApplication::Automatic
                )
            ) {
                return Err(ConversationDiagnosticCode::MetadataUnavailable);
            }
            Some(pending)
        }
        _ => return Err(ConversationDiagnosticCode::MetadataUnavailable),
    };
    Ok(ModelSelectionSnapshot::ready(
        availability,
        effective,
        pending,
        policy,
    ))
}

pub(super) fn availability_storage_value(value: ModelSelectionAvailability) -> &'static str {
    match value {
        ModelSelectionAvailability::Ready => "ready",
        ModelSelectionAvailability::RecommendationOnly => "recommendation-only",
        ModelSelectionAvailability::Unavailable => "unavailable",
    }
}

fn apply_notification(
    active: &mut ActiveConversation,
    notification: AppServerNotification,
) -> Result<(Option<ConversationEvent>, Option<TerminalState>), ConversationDiagnosticCode> {
    let notification = match notification {
        AppServerNotification::Conversation(notification) => notification,
        AppServerNotification::ConversationRequest(request) => {
            return apply_server_request(active, request).map(|event| (Some(event), None));
        }
        AppServerNotification::AccountLoginCompleted { .. }
        | AppServerNotification::AccountUpdated
        | AppServerNotification::McpOauthLoginCompleted { .. }
        | AppServerNotification::IntegrationRefresh(_) => return Ok((None, None)),
    };
    match notification {
        ConversationNotification::ThreadStarted { thread_id } => {
            ensure_thread(active, &thread_id)?;
            Ok((None, None))
        }
        ConversationNotification::ThreadArchived { thread_id }
        | ConversationNotification::ThreadUnarchived { thread_id } => {
            ensure_thread(active, &thread_id)?;
            Ok((None, None))
        }
        ConversationNotification::TurnStarted { thread_id, turn_id } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((None, None))
        }
        ConversationNotification::AgentMessageDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::AgentMessageDelta {
                    sequence: active.take_sequence(),
                    delta,
                }),
                None,
            ))
        }
        ConversationNotification::ReasoningSummaryDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::ReasoningSummaryDelta {
                    sequence: active.take_sequence(),
                    delta,
                }),
                None,
            ))
        }
        ConversationNotification::PlanUpdated {
            thread_id,
            turn_id,
            explanation,
            steps,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::PlanUpdated {
                    sequence: active.take_sequence(),
                    explanation,
                    steps: steps
                        .into_iter()
                        .map(|step| ConversationPlanStep {
                            step: step.step,
                            status: match step.status {
                                WirePlanStepStatus::Pending => ConversationPlanStepStatus::Pending,
                                WirePlanStepStatus::InProgress => {
                                    ConversationPlanStepStatus::InProgress
                                }
                                WirePlanStepStatus::Completed => {
                                    ConversationPlanStepStatus::Completed
                                }
                            },
                        })
                        .collect(),
                }),
                None,
            ))
        }
        ConversationNotification::ItemLifecycle {
            thread_id,
            turn_id,
            item,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            let kind = map_item_kind(item.kind);
            let activity = active.activity_for_item(&item.item_id, kind)?;
            let (title, detail, exit_code) = present_activity(active, &item.detail, kind);
            if item.status == WireItemStatus::Completed {
                if let Some(activity) = active.activities.get_mut(&item.item_id) {
                    activity.pending_output.clear();
                }
            }
            Ok((
                Some(ConversationEvent::Activity {
                    sequence: active.take_sequence(),
                    activity_id: activity.activity_id,
                    kind,
                    status: match item.status {
                        WireItemStatus::Started => ConversationActivityStatus::Started,
                        WireItemStatus::Completed => ConversationActivityStatus::Completed,
                    },
                    title,
                    detail,
                    exit_code,
                }),
                None,
            ))
        }
        ConversationNotification::ActivityDelta {
            thread_id,
            turn_id,
            item_id,
            kind,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            let expected_kind = match kind {
                WireActivityDeltaKind::CommandOutput => ConversationActivityKind::CommandExecution,
                WireActivityDeltaKind::ToolProgress => ConversationActivityKind::ToolCall,
            };
            let activity = active
                .activities
                .get_mut(&item_id)
                .filter(|activity| activity.kind == expected_kind)
                .ok_or(ConversationDiagnosticCode::ProtocolInvalid)?;
            let activity_id = activity.activity_id.clone();
            let delta = match kind {
                WireActivityDeltaKind::CommandOutput => {
                    activity.pending_output.push_str(&delta);
                    if activity.pending_output.len() > 64 * 1024 {
                        activity.pending_output.clear();
                        "[Output omitted: incomplete line exceeded the safety limit.]".to_owned()
                    } else if let Some(boundary) = activity.pending_output.rfind(['\n', '\r']) {
                        let complete = activity.pending_output[..=boundary].to_owned();
                        activity.pending_output.drain(..=boundary);
                        sanitize_display_text(&complete, &active.cwd, MAX_ACTIVITY_DETAIL_BYTES)
                    } else {
                        return Ok((None, None));
                    }
                }
                WireActivityDeltaKind::ToolProgress => {
                    sanitize_display_text(&delta, &active.cwd, MAX_ACTIVITY_DETAIL_BYTES)
                }
            };
            if delta.is_empty() {
                return Ok((None, None));
            }
            Ok((
                Some(ConversationEvent::ActivityOutputDelta {
                    sequence: active.take_sequence(),
                    activity_id,
                    delta,
                }),
                None,
            ))
        }
        ConversationNotification::ServerRequestResolved {
            thread_id,
            request_id,
        } => {
            ensure_thread(active, &thread_id)?;
            let Some(pending) = active.pending_approval.as_ref() else {
                return Ok((None, None));
            };
            if pending.request.request_id() != &request_id {
                return Err(ConversationDiagnosticCode::ProtocolInvalid);
            }
            let approval_id = pending.approval.approval_id.clone();
            active.pending_approval = None;
            active.state = ConversationState::Running;
            Ok((
                Some(ConversationEvent::ApprovalResolved {
                    sequence: active.take_sequence(),
                    approval_id,
                    resolution: ConversationApprovalResolution::ResolvedExternally,
                }),
                None,
            ))
        }
        ConversationNotification::TurnCompleted {
            thread_id,
            turn_id,
            status,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            if status == WireTurnStatus::Completed {
                if let Some((request, application)) = active.agent_selection_request.take() {
                    active.model_selection.pending = Some(PendingModelSelection {
                        choice: request.choice,
                        provenance: ModelSelectionProvenance::Codex,
                        application,
                        rationale: request.rationale,
                        requested_at_ms: current_time_millis(),
                    });
                }
            } else {
                active.agent_selection_request = None;
            }
            let terminal = match status {
                WireTurnStatus::Completed => TerminalState {
                    state: ConversationState::Completed,
                    phase: ConversationLifecyclePhase::Completed,
                    storage_status: "completed",
                    diagnostic_code: None,
                },
                WireTurnStatus::Interrupted => TerminalState {
                    state: ConversationState::Interrupted,
                    phase: ConversationLifecyclePhase::Interrupted,
                    storage_status: "interrupted",
                    diagnostic_code: None,
                },
                WireTurnStatus::Failed => TerminalState {
                    state: ConversationState::Failed,
                    phase: ConversationLifecyclePhase::Failed,
                    storage_status: "failed",
                    diagnostic_code: Some(ConversationDiagnosticCode::RuntimeUnavailable),
                },
            };
            Ok((None, Some(terminal)))
        }
        ConversationNotification::Error {
            thread_id,
            turn_id,
            code,
            will_retry,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::Error {
                    sequence: active.take_sequence(),
                    code: map_stream_error(code),
                    will_retry,
                }),
                None,
            ))
        }
    }
}

fn apply_server_request(
    active: &mut ActiveConversation,
    request: ConversationServerRequest,
) -> Result<ConversationEvent, ConversationDiagnosticCode> {
    if active.state != ConversationState::Running || active.pending_approval.is_some() {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    let (thread_id, turn_id, item_id, kind) = match &request {
        ConversationServerRequest::DynamicTool { .. } => {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        ConversationServerRequest::CommandExecution {
            thread_id,
            turn_id,
            item_id,
            ..
        } => (
            thread_id,
            turn_id,
            item_id,
            ConversationActivityKind::CommandExecution,
        ),
        ConversationServerRequest::FileChange {
            thread_id,
            turn_id,
            item_id,
            ..
        } => (
            thread_id,
            turn_id,
            item_id,
            ConversationActivityKind::FileChange,
        ),
        ConversationServerRequest::Permissions {
            thread_id,
            turn_id,
            item_id,
            ..
        } => (
            thread_id,
            turn_id,
            item_id,
            ConversationActivityKind::CommandExecution,
        ),
    };
    ensure_turn(active, thread_id, turn_id)?;
    let activity = active.activity_for_item(item_id, kind)?;
    let approval = present_approval(active, &request, activity.activity_id)?;
    let event = ConversationEvent::ApprovalRequested {
        sequence: active.take_sequence(),
        approval_id: approval.approval_id.clone(),
        activity_id: approval.activity_id.clone(),
        kind: approval.kind,
    };
    active.pending_approval = Some(PendingApproval { request, approval });
    active.state = ConversationState::WaitingForApproval;
    Ok(event)
}

impl ActiveConversation {
    fn activity_for_item(
        &mut self,
        item_id: &str,
        kind: ConversationActivityKind,
    ) -> Result<ActiveActivity, ConversationDiagnosticCode> {
        if let Some(activity) = self.activities.get(item_id) {
            if activity.kind != kind {
                return Err(ConversationDiagnosticCode::ProtocolInvalid);
            }
            return Ok(activity.clone());
        }
        if self.activities.len() >= MAX_TRACKED_ACTIVITIES {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        let activity = ActiveActivity {
            activity_id: Uuid::now_v7().to_string(),
            kind,
            pending_output: String::new(),
        };
        self.activities.insert(item_id.to_owned(), activity.clone());
        Ok(activity)
    }
}

fn present_activity(
    active: &ActiveConversation,
    detail: &WireItemDetail,
    kind: ConversationActivityKind,
) -> (String, Option<String>, Option<i32>) {
    match detail {
        WireItemDetail::CommandExecution {
            command,
            cwd,
            exit_code,
        } => {
            let command = sanitize_display_text(command, &active.cwd, MAX_ACTIVITY_DETAIL_BYTES);
            let cwd = present_path(cwd, &active.cwd);
            let detail = if cwd == "." {
                command
            } else {
                format!("{command}\nWorking directory: {cwd}")
            };
            ("Run command".to_owned(), nonempty(detail), *exit_code)
        }
        WireItemDetail::FileChange { paths } => {
            let paths = paths
                .iter()
                .map(|path| present_path(path, &active.cwd))
                .collect::<Vec<_>>()
                .join("\n");
            ("Apply file changes".to_owned(), nonempty(paths), None)
        }
        WireItemDetail::ToolCall {
            server,
            tool,
            app_name,
            action_name,
        } => {
            let tool = sanitize_label(tool, 128);
            let title = app_name
                .as_deref()
                .map(|name| sanitize_label(name, 128))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "Use tool".to_owned());
            let mut parts = Vec::new();
            if let Some(server) = server {
                parts.push(format!("Server: {}", sanitize_label(server, 128)));
            }
            parts.push(format!("Tool: {tool}"));
            if let Some(action_name) = action_name {
                parts.push(format!("Action: {}", sanitize_label(action_name, 128)));
            }
            (title, nonempty(parts.join("\n")), None)
        }
        WireItemDetail::WebSearch { query } => (
            "Search the web".to_owned(),
            nonempty(sanitize_display_text(
                query,
                &active.cwd,
                MAX_ACTIVITY_DETAIL_BYTES,
            )),
            None,
        ),
        WireItemDetail::None => (activity_title(kind).to_owned(), None, None),
    }
}

fn activity_title(kind: ConversationActivityKind) -> &'static str {
    match kind {
        ConversationActivityKind::UserMessage => "User message",
        ConversationActivityKind::AgentMessage => "Codex response",
        ConversationActivityKind::Plan => "Update plan",
        ConversationActivityKind::Reasoning => "Reasoning summary",
        ConversationActivityKind::CommandExecution => "Run command",
        ConversationActivityKind::FileChange => "Apply file changes",
        ConversationActivityKind::ToolCall => "Use tool",
        ConversationActivityKind::WebSearch => "Search the web",
        ConversationActivityKind::Image => "Inspect image",
        ConversationActivityKind::Other => "Codex activity",
    }
}

fn present_approval(
    active: &ActiveConversation,
    request: &ConversationServerRequest,
    activity_id: String,
) -> Result<ConversationApproval, ConversationDiagnosticCode> {
    let (kind, title, reason, details, decisions) = match request {
        ConversationServerRequest::DynamicTool { .. } => {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        ConversationServerRequest::CommandExecution {
            command,
            cwd,
            reason,
            additional_permissions,
            network_host,
            network_protocol,
            available_decisions,
            ..
        } => {
            let mut details = Vec::new();
            if let Some(command) = command {
                push_approval_detail(
                    &mut details,
                    "Command",
                    sanitize_display_text(command, &active.cwd, MAX_ACTIVITY_DETAIL_BYTES),
                );
            }
            if let Some(cwd) = cwd {
                push_approval_detail(
                    &mut details,
                    "Working directory",
                    present_path(cwd, &active.cwd),
                );
            }
            if let Some(permissions) = additional_permissions {
                push_approval_detail(
                    &mut details,
                    "Additional access",
                    permission_summary(permissions),
                );
            }
            if let (Some(host), Some(protocol)) = (network_host, network_protocol) {
                push_approval_detail(
                    &mut details,
                    "Network target",
                    format!(
                        "{} ({})",
                        sanitize_label(host, 253),
                        sanitize_label(protocol, 32)
                    ),
                );
            }
            (
                ConversationApprovalKind::CommandExecution,
                "Run this command?",
                reason,
                details,
                available_decisions
                    .iter()
                    .map(|decision| match decision {
                        WireServerDecision::Accept => ConversationApprovalDecision::Approve,
                        WireServerDecision::Decline => ConversationApprovalDecision::Decline,
                        WireServerDecision::Cancel => ConversationApprovalDecision::Cancel,
                    })
                    .collect(),
            )
        }
        ConversationServerRequest::FileChange {
            grant_root, reason, ..
        } => {
            let mut details = Vec::new();
            if let Some(root) = grant_root {
                push_approval_detail(&mut details, "Write root", present_path(root, &active.cwd));
            }
            (
                ConversationApprovalKind::FileChange,
                "Apply these file changes?",
                reason,
                details,
                if grant_root.is_some() {
                    vec![
                        ConversationApprovalDecision::Decline,
                        ConversationApprovalDecision::Cancel,
                    ]
                } else {
                    standard_decisions()
                },
            )
        }
        ConversationServerRequest::Permissions {
            cwd,
            permissions,
            reason,
            ..
        } => {
            let mut details = vec![ConversationApprovalDetail {
                label: "Working directory".to_owned(),
                value: present_path(cwd, &active.cwd),
            }];
            push_approval_detail(
                &mut details,
                "Requested access",
                permission_summary(permissions),
            );
            (
                ConversationApprovalKind::Permissions,
                "Grant additional permissions?",
                reason,
                details,
                standard_decisions(),
            )
        }
    };
    let reason = reason
        .as_deref()
        .map(|reason| sanitize_display_text(reason, &active.cwd, 4096))
        .filter(|reason| !reason.is_empty());
    if decisions.is_empty() {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(ConversationApproval {
        approval_id: Uuid::now_v7().to_string(),
        activity_id,
        kind,
        title: title.to_owned(),
        reason,
        details,
        decisions,
    })
}

fn standard_decisions() -> Vec<ConversationApprovalDecision> {
    vec![
        ConversationApprovalDecision::Approve,
        ConversationApprovalDecision::Decline,
        ConversationApprovalDecision::Cancel,
    ]
}

fn push_approval_detail(details: &mut Vec<ConversationApprovalDetail>, label: &str, value: String) {
    if !value.is_empty() {
        details.push(ConversationApprovalDetail {
            label: label.to_owned(),
            value,
        });
    }
}

fn permission_summary(permissions: &Value) -> String {
    let file_system = permissions.get("fileSystem").and_then(Value::as_object);
    let entries = file_system
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let reads = file_system
        .and_then(|value| value.get("read"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let writes = file_system
        .and_then(|value| value.get("write"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let network = permissions
        .get("network")
        .and_then(|value| value.get("enabled"))
        .and_then(Value::as_bool);
    let mut parts = Vec::new();
    if entries + reads + writes > 0 {
        parts.push(format!(
            "File system: {} scoped rule(s)",
            entries + reads + writes
        ));
    }
    if let Some(enabled) = network {
        parts.push(if enabled {
            "Network: requested".to_owned()
        } else {
            "Network: disabled".to_owned()
        });
    }
    if parts.is_empty() {
        "No additional access described".to_owned()
    } else {
        parts.join("; ")
    }
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn validate_start_request(
    request: &ConversationStartRequest,
) -> Result<(), ConversationDiagnosticCode> {
    validate_uuid_v7(&request.project_id)
        .map_err(|_| ConversationDiagnosticCode::InvalidRequest)?;
    validate_user_text(&request.prompt)?;
    validate_attachment_ids(&request.attachment_ids)?;
    if request.integration_entry_ids.len() > 8
        || request
            .integration_entry_ids
            .iter()
            .any(|entry_id| validate_integration_entry_id(entry_id).is_err())
        || request
            .integration_entry_ids
            .iter()
            .collect::<HashSet<_>>()
            .len()
            != request.integration_entry_ids.len()
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    validate_protocol_choice(&request.model_id, 128)?;
    validate_protocol_choice(&request.reasoning_effort, 32)?;
    ModelSelectionService::validate_policy_shape(
        &request.selection_policy,
        &ModelSelectionChoice {
            model_id: request.model_id.clone(),
            reasoning_effort: request.reasoning_effort.clone(),
        },
    )
    .map_err(|_| ConversationDiagnosticCode::InvalidRequest)?;
    if request.sandbox_mode == ConversationSandboxMode::DangerFullAccess
        && request.approval_policy == ConversationApprovalPolicy::Never
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

pub(super) fn validate_attachment_ids(
    attachment_ids: &[String],
) -> Result<(), ConversationDiagnosticCode> {
    if attachment_ids.len() > MAX_CONVERSATION_ATTACHMENTS
        || attachment_ids
            .iter()
            .any(|attachment_id| validate_uuid_v7(attachment_id).is_err())
        || attachment_ids.iter().collect::<HashSet<_>>().len() != attachment_ids.len()
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn validate_integration_entry_id(value: &str) -> Result<(), ()> {
    let Some(suffix) = value.strip_prefix("connector:") else {
        return Err(());
    };
    if value.len() > 128
        || suffix.is_empty()
        || !suffix
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
    {
        return Err(());
    }
    Ok(())
}

fn validate_user_text(value: &str) -> Result<(), ConversationDiagnosticCode> {
    if value.trim().is_empty()
        || value.len() > MAX_PROMPT_BYTES
        || value.chars().any(|character| {
            character == '\0'
                || (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
                || matches!(
                    character,
                    '\u{200B}'..='\u{200F}'
                        | '\u{202A}'..='\u{202E}'
                        | '\u{2060}'..='\u{206F}'
                        | '\u{FEFF}'
                )
        })
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn validate_protocol_choice(
    value: &str,
    max_bytes: usize,
) -> Result<(), ConversationDiagnosticCode> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn sandbox_policy(mode: ConversationSandboxMode, cwd: &Path) -> Value {
    match mode {
        ConversationSandboxMode::ReadOnly => {
            json!({"type": "readOnly", "networkAccess": false})
        }
        ConversationSandboxMode::WorkspaceWrite => json!({
            "type": "workspaceWrite",
            "writableRoots": [cwd],
            "networkAccess": false,
            "excludeSlashTmp": false,
            "excludeTmpdirEnvVar": false,
        }),
        ConversationSandboxMode::DangerFullAccess => json!({"type": "dangerFullAccess"}),
    }
}

#[derive(Deserialize)]
struct ProtocolId {
    id: String,
}

#[derive(Deserialize)]
struct ThreadStartResult {
    cwd: String,
    thread: ProtocolId,
}

#[derive(Deserialize)]
struct TurnStartResult {
    turn: TurnStartTurn,
}

#[derive(Deserialize)]
struct TurnStartTurn {
    id: String,
    status: String,
}

fn parse_thread_start(
    value: Value,
    expected_cwd: &Path,
) -> Result<ThreadStartResult, ConversationDiagnosticCode> {
    let result: ThreadStartResult =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.thread.id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if Path::new(&result.cwd) != expected_cwd {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(result)
}

fn parse_turn_start(value: Value) -> Result<TurnStartResult, ConversationDiagnosticCode> {
    let result: TurnStartResult =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.turn.id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if result.turn.status != "inProgress" {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(result)
}

fn ensure_thread(
    active: &ActiveConversation,
    thread_id: &str,
) -> Result<(), ConversationDiagnosticCode> {
    if active.thread_id != thread_id {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn ensure_turn(
    active: &ActiveConversation,
    thread_id: &str,
    turn_id: &str,
) -> Result<(), ConversationDiagnosticCode> {
    ensure_thread(active, thread_id)?;
    if active.turn_id != turn_id {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn map_project_error(error: ProjectExecutionError) -> ConversationDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            ConversationDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::MetadataUnavailable => {
            ConversationDiagnosticCode::MetadataUnavailable
        }
        ProjectExecutionError::DirectoryUnavailable => {
            ConversationDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::IdentityChanged => {
            ConversationDiagnosticCode::ProjectIdentityChanged
        }
        ProjectExecutionError::NotRepository => ConversationDiagnosticCode::ProjectUnavailable,
        ProjectExecutionError::NotWritable => ConversationDiagnosticCode::ProjectNotWritable,
        ProjectExecutionError::ProjectBusy => ConversationDiagnosticCode::ProjectBusy,
    }
}

fn map_adapter_error(error: &CodexAdapterError) -> ConversationDiagnosticCode {
    match error {
        CodexAdapterError::ProcessSpawnFailed | CodexAdapterError::CliNotFound => {
            ConversationDiagnosticCode::RuntimeUnavailable
        }
        CodexAdapterError::ProcessExited => ConversationDiagnosticCode::ProcessExited,
        CodexAdapterError::TransportTimeout | CodexAdapterError::TransportClosed => {
            ConversationDiagnosticCode::TransportFailed
        }
        CodexAdapterError::RpcRejected => ConversationDiagnosticCode::RpcRejected,
        CodexAdapterError::UnexpectedServerRequest => ConversationDiagnosticCode::ApprovalRequired,
        CodexAdapterError::MessageTooLarge
        | CodexAdapterError::InvalidProtocolMessage
        | CodexAdapterError::InvalidModelCatalog
        | CodexAdapterError::CliVersionInvalid => ConversationDiagnosticCode::ProtocolInvalid,
    }
}

fn protocol_terminal(code: ConversationDiagnosticCode) -> TerminalState {
    TerminalState {
        state: if code == ConversationDiagnosticCode::ApprovalRequired {
            ConversationState::Blocked
        } else {
            ConversationState::Failed
        },
        phase: if code == ConversationDiagnosticCode::ApprovalRequired {
            ConversationLifecyclePhase::Blocked
        } else {
            ConversationLifecyclePhase::Failed
        },
        storage_status: if code == ConversationDiagnosticCode::ApprovalRequired {
            "blocked"
        } else {
            "failed"
        },
        diagnostic_code: Some(code),
    }
}

fn map_item_kind(kind: WireItemKind) -> ConversationActivityKind {
    match kind {
        WireItemKind::UserMessage => ConversationActivityKind::UserMessage,
        WireItemKind::AgentMessage => ConversationActivityKind::AgentMessage,
        WireItemKind::Plan => ConversationActivityKind::Plan,
        WireItemKind::Reasoning => ConversationActivityKind::Reasoning,
        WireItemKind::CommandExecution => ConversationActivityKind::CommandExecution,
        WireItemKind::FileChange => ConversationActivityKind::FileChange,
        WireItemKind::ToolCall => ConversationActivityKind::ToolCall,
        WireItemKind::WebSearch => ConversationActivityKind::WebSearch,
        WireItemKind::Image => ConversationActivityKind::Image,
        WireItemKind::Other => ConversationActivityKind::Other,
    }
}

fn map_stream_error(code: WireErrorCode) -> ConversationStreamErrorCode {
    match code {
        WireErrorCode::ContextWindowExceeded => ConversationStreamErrorCode::ContextWindowExceeded,
        WireErrorCode::UsageLimitExceeded => ConversationStreamErrorCode::UsageLimitExceeded,
        WireErrorCode::Unauthorized => ConversationStreamErrorCode::Unauthorized,
        WireErrorCode::Sandbox => ConversationStreamErrorCode::Sandbox,
        WireErrorCode::Server => ConversationStreamErrorCode::Server,
        WireErrorCode::Other => ConversationStreamErrorCode::Other,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;
    use uuid::Uuid;

    use super::*;

    const THREAD_ID: &str = "018f0000-0000-7000-8000-000000000020";
    const TURN_ID: &str = "018f0000-0000-7000-8000-000000000030";

    #[test]
    fn serialized_empty_snapshot_matches_the_shared_frontend_fixture() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../../../fixtures/conversation.json"))
                .expect("conversation fixture must be JSON");
        let snapshot = serde_json::to_value(ConversationSnapshot::empty())
            .expect("conversation snapshot must serialize");

        assert_eq!(snapshot, fixture);
    }

    #[tokio::test]
    async fn serialized_empty_registry_matches_the_shared_frontend_fixture() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../fixtures/conversation-registry.json"
        ))
        .expect("conversation registry fixture must be JSON");
        let registry =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", "exit 91"]))
                .active()
                .await;

        assert_eq!(
            serde_json::to_value(registry).expect("registry must serialize"),
            fixture
        );
    }

    #[tokio::test]
    async fn notification_candidates_require_fresh_eligible_native_state() {
        let service = ConversationService::default();
        let conversation_id = Uuid::now_v7().to_string();
        service
            .remember_snapshot(ConversationSnapshot {
                state: ConversationState::Running,
                conversation_id: Some(conversation_id.clone()),
                ..ConversationSnapshot::empty()
            })
            .await;
        assert!(service
            .notification_candidate(&conversation_id)
            .await
            .is_none());

        service
            .remember_snapshot(ConversationSnapshot {
                state: ConversationState::Completed,
                conversation_id: Some(conversation_id.clone()),
                ..ConversationSnapshot::empty()
            })
            .await;
        let candidate = service
            .notification_candidate(&conversation_id)
            .await
            .expect("completed conversation must be eligible");
        assert_eq!(candidate.state, ConversationState::Completed);
        assert_eq!(
            candidate.key,
            format!("terminal:{conversation_id}:completed")
        );
        assert!(service
            .notification_candidate("raw-thread-id")
            .await
            .is_none());
    }

    #[test]
    fn provisional_starts_count_toward_the_parallel_capacity() {
        let mut state = ConversationServiceState::empty();
        for index in 0..MAX_ACTIVE_CONVERSATIONS {
            assert_eq!(state.begin_start(&format!("project-{index}")), Ok(()));
        }
        assert_eq!(
            state.begin_start("overflow-project"),
            Err(ConversationDiagnosticCode::ParallelCapacityReached)
        );

        state.finish_start("project-0");
        assert_eq!(state.begin_start("replacement-project"), Ok(()));
    }

    #[test]
    fn permission_decisions_are_turn_scoped_and_never_create_persistent_grants() {
        let request = ConversationServerRequest::Permissions {
            request_id: crate::codex::app_server::ServerRequestId::Integer(7),
            thread_id: THREAD_ID.to_owned(),
            turn_id: TURN_ID.to_owned(),
            item_id: "item-1".to_owned(),
            cwd: "/workspace/project".to_owned(),
            permissions: json!({"network": {"enabled": true}}),
            reason: None,
        };

        assert_eq!(
            approval_response(&request, ConversationApprovalDecision::Approve),
            json!({
                "permissions": {"network": {"enabled": true}},
                "scope": "turn",
                "strictAutoReview": false,
            })
        );
        assert_eq!(
            approval_response(&request, ConversationApprovalDecision::Decline),
            json!({
                "permissions": {},
                "scope": "turn",
                "strictAutoReview": false,
            })
        );
    }

    #[tokio::test]
    async fn starts_in_the_verified_cwd_and_normalizes_streamed_events() {
        let (projects, directory, project_id) = attached_project();
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"item/agentMessage/delta","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"item-1","delta":"Review complete."}}}}'
printf '%s\n' '{{"method":"turn/plan/updated","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","explanation":"Checked safely.","plan":[{{"step":"Inspect project","status":"completed"}}]}}}}'
printf '%s\n' '{{"method":"item/started","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","startedAtMs":1,"item":{{"id":"item-2","type":"commandExecution","command":"git status","commandActions":[],"cwd":{cwd_json},"status":"inProgress"}}}}}}'
printf '%s\n' '{{"method":"item/commandExecution/outputDelta","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"item-2","delta":"OPENAI_API_KEY="}}}}'
printf '%s\n' '{{"method":"item/commandExecution/outputDelta","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"item-2","delta":"stream-secret\n"}}}}'
printf '%s\n' '{{"method":"item/completed","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","completedAtMs":2,"item":{{"id":"item-2","type":"commandExecution","command":"git status","commandActions":[],"cwd":{cwd_json},"status":"completed","exitCode":0}}}}}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .start(start_request(project_id.clone()), &projects)
            .await;
        assert_eq!(started.state, ConversationState::Running);
        assert_eq!(
            projects.archive(project_id.clone()).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        let serialized = serde_json::to_string(&started).expect("snapshot must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(TURN_ID));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));

        let completed = service
            .poll(
                started
                    .conversation_id
                    .clone()
                    .expect("conversation ID must exist"),
                &projects,
            )
            .await;
        assert_eq!(
            completed.state,
            ConversationState::Completed,
            "unexpected selector completion: {completed:?}"
        );
        assert!(completed.events.iter().any(|event| matches!(
            event,
            ConversationEvent::AgentMessageDelta { delta, .. } if delta == "Review complete."
        )));
        let activity_ids = completed
            .events
            .iter()
            .filter_map(|event| match event {
                ConversationEvent::Activity { activity_id, .. }
                | ConversationEvent::ActivityOutputDelta { activity_id, .. } => Some(activity_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(activity_ids.len(), 3);
        assert!(activity_ids.windows(2).all(|ids| ids[0] == ids[1]));
        let serialized = serde_json::to_string(&completed).expect("snapshot must serialize");
        assert!(!serialized.contains("stream-secret"));
        assert!(completed.events.iter().any(|event| matches!(
            event,
            ConversationEvent::Activity {
                kind: ConversationActivityKind::CommandExecution,
                status: ConversationActivityStatus::Completed,
                ..
            }
        )));
        assert!(completed.diagnostic_code.is_none());
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn constructs_connector_mentions_natively_without_forwarding_catalog_ids() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing).replacen(
            "read -r _turn",
            r#"read -r _turn
case "$_turn" in
  *'"type":"mention"'*) ;;
  *) exit 81 ;;
esac
case "$_turn" in
  *'"name":"Fixture calendar connector"'*'"path":"app://fixture-calendar"'*) ;;
  *) exit 82 ;;
esac
case "$_turn" in
  *'connector:fixture-calendar'*) exit 83 ;;
  *) ;;
esac"#,
            1,
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut request = start_request(project_id);
        request.integration_entry_ids = vec!["connector:fixture-calendar".to_owned()];

        let started = service
            .start_with_mentions(
                request,
                &projects,
                vec![ResolvedIntegrationMention {
                    name: "Fixture calendar connector".to_owned(),
                    path: "app://fixture-calendar".to_owned(),
                }],
                Vec::new(),
            )
            .await;
        assert_eq!(started.state, ConversationState::Running);
        let completed = service
            .poll(
                started
                    .conversation_id
                    .expect("running conversation needs an ID"),
                &projects,
            )
            .await;
        assert_eq!(completed.state, ConversationState::Completed);
        let encoded = serde_json::to_string(&completed).expect("snapshot must serialize");
        assert!(!encoded.contains("app://fixture-calendar"));
        assert!(!encoded.contains("connector:fixture-calendar"));

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn constructs_local_image_inputs_from_native_resolved_attachments() {
        let (projects, directory, project_id) = attached_project();
        let attachment_id = "018f0000-0000-7000-8000-000000000099";
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing).replacen(
            "read -r _turn",
            &format!(
                r#"read -r _turn
case "$_turn" in
  *'"path":'*'"type":"localImage"'*) ;;
  *) exit 84 ;;
esac
case "$_turn" in
  *'{attachment_id}'*) exit 85 ;;
  *) ;;
esac"#
            ),
            1,
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut request = start_request(project_id);
        request.attachment_ids = vec![attachment_id.to_owned()];

        let started = service
            .start_with_mentions(
                request,
                &projects,
                Vec::new(),
                vec![ResolvedConversationAttachment::for_test(
                    directory.join("private-staged-image.png"),
                )],
            )
            .await;
        assert_eq!(started.state, ConversationState::Running);
        let completed = service
            .poll(
                started
                    .conversation_id
                    .expect("running conversation needs an ID"),
                &projects,
            )
            .await;
        assert_eq!(completed.state, ConversationState::Completed);

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn interrupts_only_the_exact_app_owned_conversation() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
read -r _interrupt
printf '%s\n' '{{"id":5,"result":null}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"interrupted"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;
        let conversation_id = started
            .conversation_id
            .clone()
            .expect("conversation ID must exist");

        let stopping = service.interrupt(conversation_id.clone(), &projects).await;
        assert_eq!(stopping.state, ConversationState::Stopping);
        let interrupted = service.poll(conversation_id, &projects).await;
        assert_eq!(interrupted.state, ConversationState::Interrupted);

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn runs_distinct_projects_concurrently_and_routes_interrupts_by_app_id() {
        let projects = ProjectService::in_memory();
        let (first_directory, first_project_id) = attach_project(&projects);
        let (second_directory, second_project_id) = attach_project(&projects);
        let script = concurrent_start_script();
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let first = service
            .start(start_request(first_project_id.clone()), &projects)
            .await;
        let second = service
            .start(start_request(second_project_id.clone()), &projects)
            .await;
        assert_eq!(first.state, ConversationState::Running);
        assert_eq!(second.state, ConversationState::Running, "{second:?}");
        let first_id = first.conversation_id.expect("first app ID must exist");
        let second_id = second.conversation_id.expect("second app ID must exist");
        assert_ne!(first_id, second_id);
        let registry = service.active().await;
        assert_eq!(registry.capacity, MAX_ACTIVE_CONVERSATIONS as u8);
        assert_eq!(registry.conversations.len(), 2);
        let serialized = serde_json::to_string(&registry).expect("registry must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(TURN_ID));

        let duplicate = service
            .start(start_request(first_project_id), &projects)
            .await;
        assert_eq!(
            duplicate.diagnostic_code,
            Some(ConversationDiagnosticCode::ProjectBusy)
        );

        assert_eq!(
            service.interrupt(first_id.clone(), &projects).await.state,
            ConversationState::Stopping
        );
        assert_eq!(
            service.poll(first_id, &projects).await.state,
            ConversationState::Interrupted
        );
        assert_eq!(
            service.poll(second_id.clone(), &projects).await.state,
            ConversationState::Running
        );
        assert_eq!(
            service.interrupt(second_id.clone(), &projects).await.state,
            ConversationState::Stopping
        );
        assert_eq!(
            service.poll(second_id, &projects).await.state,
            ConversationState::Interrupted
        );

        assert_ne!(
            projects.archive(second_project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        fs::remove_dir_all(first_directory).expect("first project must be removed");
        fs::remove_dir_all(second_directory).expect("second project must be removed");
    }

    #[tokio::test]
    async fn enforces_the_bounded_parallel_capacity_and_reaps_every_child() {
        let projects = ProjectService::in_memory();
        let attached = (0..=MAX_ACTIVE_CONVERSATIONS)
            .map(|_| attach_project(&projects))
            .collect::<Vec<_>>();
        let script = concurrent_start_script();
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut active_ids = Vec::new();
        for (_, project_id) in attached.iter().take(MAX_ACTIVE_CONVERSATIONS) {
            let started = service
                .start(start_request(project_id.clone()), &projects)
                .await;
            assert_eq!(started.state, ConversationState::Running);
            active_ids.push(started.conversation_id.expect("app ID must exist"));
        }
        let full = service
            .start(
                start_request(
                    attached
                        .last()
                        .expect("overflow project must exist")
                        .1
                        .clone(),
                ),
                &projects,
            )
            .await;
        assert_eq!(
            full.diagnostic_code,
            Some(ConversationDiagnosticCode::ParallelCapacityReached)
        );

        for conversation_id in active_ids {
            assert_eq!(
                service
                    .interrupt(conversation_id.clone(), &projects)
                    .await
                    .state,
                ConversationState::Stopping
            );
            assert_eq!(
                service.poll(conversation_id, &projects).await.state,
                ConversationState::Interrupted
            );
        }
        assert!(service.active().await.conversations.is_empty());
        for (directory, _) in attached {
            fs::remove_dir_all(directory).expect("temporary project must be removed");
        }
    }

    #[tokio::test]
    async fn waits_for_and_resolves_a_bounded_command_approval() {
        let (projects, directory, project_id) = attached_project();
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        let trailing = format!(
            r#"
printf '%s\n' '{{"id":99,"method":"item/commandExecution/requestApproval","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"approval-item","startedAtMs":1,"command":"curl --token topsecret /etc/passwd","cwd":{cwd_json},"reason":"Needs token topsecret","availableDecisions":["accept","acceptForSession","decline","cancel"]}}}}'
read -r approval
case "$approval" in
  *'"id":99'*'"decision":"accept"'*) ;;
  *) exit 89 ;;
esac
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service
            .start(start_request(project_id.clone()), &projects)
            .await;
        let conversation_id = started.conversation_id.expect("conversation ID must exist");

        let waiting = service.poll(conversation_id.clone(), &projects).await;
        assert_eq!(waiting.state, ConversationState::WaitingForApproval);
        let approval = waiting
            .pending_approval
            .expect("bounded approval must be present");
        assert_eq!(approval.kind, ConversationApprovalKind::CommandExecution);
        assert_eq!(
            approval.decisions,
            vec![
                ConversationApprovalDecision::Approve,
                ConversationApprovalDecision::Decline,
                ConversationApprovalDecision::Cancel,
            ]
        );
        let serialized = serde_json::to_string(&approval).expect("approval must serialize");
        assert!(!serialized.contains("topsecret"));
        assert!(!serialized.contains("/etc/passwd"));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));
        assert_eq!(
            projects.archive(project_id.clone()).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );

        let resumed = service
            .decide_approval(
                ConversationApprovalDecisionRequest {
                    conversation_id: conversation_id.clone(),
                    approval_id: approval.approval_id,
                    decision: ConversationApprovalDecision::Approve,
                },
                &projects,
            )
            .await;
        assert_eq!(resumed.state, ConversationState::Running);
        assert!(resumed.pending_approval.is_none());
        assert!(resumed.events.iter().any(|event| matches!(
            event,
            ConversationEvent::ApprovalResolved {
                resolution: ConversationApprovalResolution::Approved,
                ..
            }
        )));

        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn stages_one_policy_bounded_dynamic_selector_request_after_completion() {
        let projects = ProjectService::in_memory();
        let (directory, project_id) = attach_project(&projects);
        let trailing = r#"
printf '%s\n' '{"id":"inspect-1","method":"item/tool/call","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turnId":"018f0000-0000-7000-8000-000000000030","callId":"inspect-call","namespace":null,"tool":"quireforge_model_selector","arguments":{"action":"inspect"}}}'
read -r inspect_response
case "$inspect_response" in *'"id":"inspect-1"'*) ;; *) exit 91 ;; esac
case "$inspect_response" in *'"success":true'*) ;; *) exit 94 ;; esac
case "$inspect_response" in *'fixture-terra'*) ;; *) exit 95 ;; esac
printf '%s\n' '{"id":"request-1","method":"item/tool/call","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turnId":"018f0000-0000-7000-8000-000000000030","callId":"request-call","namespace":null,"tool":"quireforge_model_selector","arguments":{"action":"request","modelId":"fixture-terra","reasoningEffort":"medium","rationale":"Use the allowed model; token=topsecret; never retain /home/alice/private."}}}'
read -r request_response
case "$request_response" in *'"id":"request-1"'*'"success":true'*) ;; *) exit 92 ;; esac
printf '%s\n' '{"id":"request-2","method":"item/tool/call","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turnId":"018f0000-0000-7000-8000-000000000030","callId":"request-call-2","namespace":null,"tool":"quireforge_model_selector","arguments":{"action":"request","modelId":"fixture-model","reasoningEffort":"medium","rationale":"Oscillate back during the same turn."}}}'
read -r repeated_response
case "$repeated_response" in *'"id":"request-2"'*) ;; *) exit 93 ;; esac
case "$repeated_response" in *'"success":false'*) ;; *) exit 96 ;; esac
case "$repeated_response" in *'Only one model-selection request'*) ;; *) exit 97 ;; esac
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
"#;
        let script = successful_start_script(&directory, trailing).replace(
            r#"{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}"#,
            r#"{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]},{"model":"fixture-terra","displayName":"Fixture Terra","isDefault":false,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}"#,
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut request = start_request(project_id);
        request.selection_policy = ModelSelectionPolicy {
            ownership: ModelSelectionOwnership::Automatic,
            user_locked: false,
            allowed_model_ids: vec!["fixture-model".to_owned(), "fixture-terra".to_owned()],
            reasoning_ceiling: Some("medium".to_owned()),
        };

        let started = service.start(request, &projects).await;
        let conversation_id = started
            .conversation_id
            .clone()
            .expect("conversation must start");
        assert!(started
            .model_selection
            .as_ref()
            .is_some_and(|selection| selection.pending.is_none()));

        let completed = service.poll(conversation_id.clone(), &projects).await;
        assert_eq!(
            completed.state,
            ConversationState::Completed,
            "unexpected dynamic selector completion: {completed:?}"
        );
        let pending = completed
            .model_selection
            .as_ref()
            .and_then(|selection| selection.pending.as_ref())
            .expect("completed request must stage one pending choice");
        assert_eq!(pending.choice.model_id, "fixture-terra");
        assert_eq!(pending.provenance, ModelSelectionProvenance::Codex);
        assert_eq!(pending.application, ModelSelectionApplication::Automatic);
        assert_eq!(
            completed
                .events
                .iter()
                .filter(|event| matches!(event, ConversationEvent::ModelSelectionRequested { .. }))
                .count(),
            1
        );

        let stored = projects
            .conversation_reference(&conversation_id)
            .expect("pending selector metadata must persist");
        assert_eq!(
            stored.selector_pending_model_id.as_deref(),
            Some("fixture-terra")
        );
        assert_eq!(stored.model_id, "fixture-model");
        let rationale = stored
            .selector_pending_rationale
            .as_deref()
            .expect("bounded selector rationale must persist");
        assert!(!rationale.contains("topsecret"));
        assert!(!rationale.contains("/home/alice/private"));
        assert!(rationale.contains("[redacted]"));
        assert!(rationale.contains("[outside project]"));
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn degrades_to_visible_recommendation_only_when_registration_is_rejected() {
        let projects = ProjectService::in_memory();
        let (directory, project_id) = attach_project(&projects);
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        let script = r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r registration
case "$registration" in *'"dynamicTools"'*'"quireforge_model_selector"'*) ;; *) exit 98 ;; esac
printf '%s\n' '{"id":3,"error":{"code":-32602,"message":"unsupported field"}}'
read -r fallback
case "$fallback" in *'"dynamicTools"'*) exit 99 ;; *) ;; esac
printf '%s\n' '{"id":4,"result":{"cwd":__CWD__,"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r _turn
printf '%s\n' '{"id":5,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"completed"}}}'
"#
        .replace("__CWD__", &cwd_json);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service.start(start_request(project_id), &projects).await;
        assert_eq!(started.state, ConversationState::Running);
        let selection = started
            .model_selection
            .as_ref()
            .expect("selector availability must be visible");
        assert_eq!(
            selection.availability,
            ModelSelectionAvailability::RecommendationOnly
        );
        assert!(selection.pending.is_none());

        let conversation_id = started.conversation_id.expect("conversation must start");
        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn rejects_stale_or_unavailable_approval_decisions_without_mutating_the_turn() {
        let (projects, directory, project_id) = attached_project();
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        let trailing = format!(
            r#"
printf '%s\n' '{{"id":"request-1","method":"item/commandExecution/requestApproval","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"approval-item","startedAtMs":1,"command":"cargo test","cwd":{cwd_json},"availableDecisions":["decline"]}}}}'
read -r approval
case "$approval" in
  *'"id":"request-1"'*'"decision":"decline"'*) ;;
  *) exit 89 ;;
esac
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;
        let conversation_id = started.conversation_id.expect("conversation ID must exist");
        let waiting = service.poll(conversation_id.clone(), &projects).await;
        let approval = waiting
            .pending_approval
            .expect("approval must remain pending");

        let unavailable = service
            .decide_approval(
                ConversationApprovalDecisionRequest {
                    conversation_id: conversation_id.clone(),
                    approval_id: approval.approval_id.clone(),
                    decision: ConversationApprovalDecision::Approve,
                },
                &projects,
            )
            .await;
        assert_eq!(unavailable.state, ConversationState::WaitingForApproval);
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ConversationDiagnosticCode::ApprovalDecisionUnavailable)
        );
        assert_eq!(
            unavailable
                .pending_approval
                .as_ref()
                .map(|value| &value.approval_id),
            Some(&approval.approval_id)
        );

        let stale = service
            .decide_approval(
                ConversationApprovalDecisionRequest {
                    conversation_id: conversation_id.clone(),
                    approval_id: Uuid::now_v7().to_string(),
                    decision: ConversationApprovalDecision::Decline,
                },
                &projects,
            )
            .await;
        assert_eq!(stale.state, ConversationState::WaitingForApproval);
        assert_eq!(
            stale.diagnostic_code,
            Some(ConversationDiagnosticCode::ApprovalNotFound)
        );

        let resumed = service
            .decide_approval(
                ConversationApprovalDecisionRequest {
                    conversation_id: conversation_id.clone(),
                    approval_id: approval.approval_id,
                    decision: ConversationApprovalDecision::Decline,
                },
                &projects,
            )
            .await;
        assert_eq!(resumed.state, ConversationState::Running);
        assert_eq!(
            service.poll(conversation_id, &projects).await.state,
            ConversationState::Completed
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn interrupt_cancels_a_pending_approval_before_stopping_the_exact_turn() {
        let (projects, directory, project_id) = attached_project();
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        let trailing = format!(
            r#"
printf '%s\n' '{{"id":99,"method":"item/commandExecution/requestApproval","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"approval-item","startedAtMs":1,"command":"cargo test","cwd":{cwd_json}}}}}'
read -r approval
case "$approval" in
  *'"id":99'*'"decision":"cancel"'*) ;;
  *) exit 89 ;;
esac
read -r interrupt
case "$interrupt" in
  *'"id":5'*'"method":"turn/interrupt"'*) ;;
  *) exit 90 ;;
esac
printf '%s\n' '{{"id":5,"result":null}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"interrupted"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;
        let conversation_id = started.conversation_id.expect("conversation ID must exist");
        assert_eq!(
            service.poll(conversation_id.clone(), &projects).await.state,
            ConversationState::WaitingForApproval
        );

        let stopping = service.interrupt(conversation_id.clone(), &projects).await;
        assert_eq!(stopping.state, ConversationState::Stopping);
        assert!(stopping.pending_approval.is_none());
        assert!(stopping.events.iter().any(|event| matches!(
            event,
            ConversationEvent::ApprovalResolved {
                resolution: ConversationApprovalResolution::Canceled,
                ..
            }
        )));
        assert_eq!(
            service.poll(conversation_id, &projects).await.state,
            ConversationState::Interrupted
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn rejects_unadvertised_reasoning_and_never_spawns_for_a_missing_project() {
        let projects = ProjectService::in_memory();
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", "exit 91"]));
        let missing = service
            .start(
                start_request("018f0000-0000-7000-8000-000000000001".to_owned()),
                &projects,
            )
            .await;
        assert_eq!(
            missing.diagnostic_code,
            Some(ConversationDiagnosticCode::ProjectUnavailable)
        );

        let (projects, directory, project_id) = attached_project();
        let shutdown_marker = directory.join("shutdown-marker");
        let script = r#"
trap 'printf reaped > __SHUTDOWN_MARKER__' EXIT
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r _wait_until_shutdown || true
"#
        .replace("__SHUTDOWN_MARKER__", shutdown_marker.to_string_lossy().as_ref());
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut request = start_request(project_id.clone());
        request.reasoning_effort = "xhigh".to_owned();
        let unavailable = service.start(request, &projects).await;
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ConversationDiagnosticCode::ReasoningUnavailable)
        );
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        assert_eq!(
            fs::read_to_string(&shutdown_marker).expect("child exit trap must run before return"),
            "reaped"
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn fails_closed_when_stream_identity_does_not_match_the_active_turn() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"item/agentMessage/delta","params":{{"threadId":"018f0000-0000-7000-8000-000000000099","turnId":"{TURN_ID}","itemId":"item-1","delta":"wrong thread"}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;

        let failed = service
            .poll(
                started.conversation_id.expect("conversation ID must exist"),
                &projects,
            )
            .await;
        assert_eq!(failed.state, ConversationState::Failed);
        assert_eq!(
            failed.diagnostic_code,
            Some(ConversationDiagnosticCode::ProtocolInvalid)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    fn attached_project() -> (ProjectService, PathBuf, String) {
        let projects = ProjectService::in_memory();
        let (directory, project_id) = attach_project(&projects);
        (projects, directory, project_id)
    }

    fn attach_project(projects: &ProjectService) -> (PathBuf, String) {
        let directory = std::env::temp_dir().join(format!(
            "quireforge-conversation-project-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&directory).expect("temporary project must be created");
        let preview = projects.prepare_attachment(directory.clone());
        assert!(preview.pending_attachment.is_some());
        let snapshot = projects.confirm_pending();
        let project_id = snapshot
            .projects
            .iter()
            .find(|project| {
                project.directory.as_ref().is_some_and(|attached| {
                    attached.display_path == directory.to_string_lossy().as_ref()
                })
            })
            .expect("project must attach")
            .id
            .clone();
        (directory, project_id)
    }

    fn start_request(project_id: String) -> ConversationStartRequest {
        ConversationStartRequest {
            project_id,
            prompt: "Review the attached project.".to_owned(),
            attachment_ids: Vec::new(),
            integration_entry_ids: Vec::new(),
            model_id: "fixture-model".to_owned(),
            reasoning_effort: "medium".to_owned(),
            selection_policy: ModelSelectionPolicy::default(),
            sandbox_mode: ConversationSandboxMode::ReadOnly,
            approval_policy: ConversationApprovalPolicy::Untrusted,
        }
    }

    fn successful_start_script(directory: &Path, trailing: &str) -> String {
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r _thread
printf '%s\n' '{"method":"thread/started","params":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
printf '%s\n' '{"id":3,"result":{"cwd":__CWD__,"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r _turn
printf '%s\n' '{"method":"turn/started","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030"}}}'
printf '%s\n' '{"id":4,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
__TRAILING__
"#
        .replace("__CWD__", &cwd_json)
        .replace("__TRAILING__", trailing)
    }

    fn concurrent_start_script() -> String {
        r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r thread_request
cwd=$(printf '%s' "$thread_request" | sed -n 's/.*"cwd":"\([^"]*\)".*/\1/p')
test -n "$cwd"
thread_suffix=$(printf '%012x' "$$")
turn_suffix=$(printf '%012x' "$(( $$ + 1 ))")
thread_id="018f0000-0000-7000-8000-$thread_suffix"
turn_id="018f0000-0000-7000-8000-$turn_suffix"
printf '{"method":"thread/started","params":{"thread":{"id":"%s"}}}\n' "$thread_id"
printf '{"id":3,"result":{"cwd":"%s","thread":{"id":"%s"}}}\n' "$cwd" "$thread_id"
read -r _turn
printf '{"method":"turn/started","params":{"threadId":"%s","turn":{"id":"%s"}}}\n' "$thread_id" "$turn_id"
printf '{"id":4,"result":{"turn":{"id":"%s","status":"inProgress"}}}\n' "$turn_id"
read -r interrupt
case "$interrupt" in
  *'"id":5'*'"method":"turn/interrupt"'*) ;;
  *) exit 90 ;;
esac
printf '%s\n' '{"id":5,"result":null}'
printf '{"method":"turn/completed","params":{"threadId":"%s","turn":{"id":"%s","status":"interrupted"}}}\n' "$thread_id" "$turn_id"
"#
        .to_owned()
    }
}
