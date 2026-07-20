mod lifecycle;
mod presentation;
pub mod types;

pub use lifecycle::{ConversationContinueRequest, SessionLifecycleSnapshot, SessionListRequest};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::project::{ConversationReference, ProjectExecutionError, ProjectService};

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
};
use presentation::{present_path, sanitize_display_text, sanitize_label};
use types::{
    ConversationActivityKind, ConversationActivityStatus, ConversationApproval,
    ConversationApprovalDecision, ConversationApprovalDecisionRequest, ConversationApprovalDetail,
    ConversationApprovalKind, ConversationApprovalPolicy, ConversationApprovalResolution,
    ConversationDiagnosticCode, ConversationEvent, ConversationLifecyclePhase,
    ConversationPlanStep, ConversationPlanStepStatus, ConversationSandboxMode,
    ConversationSnapshot, ConversationStartRequest, ConversationState, ConversationStreamErrorCode,
    CONVERSATION_SCHEMA_VERSION,
};

const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_POLL_EVENTS: usize = 32;
const FIRST_POLL_WAIT: Duration = Duration::from_millis(200);
const DRAIN_POLL_WAIT: Duration = Duration::from_millis(1);
const MAX_TRACKED_ACTIVITIES: usize = 256;
const MAX_ACTIVITY_DETAIL_BYTES: usize = 8 * 1024;

pub struct ConversationService {
    state: Mutex<ConversationServiceState>,
    command: AppServerCommand,
}

struct ConversationServiceState {
    active: Option<ActiveConversation>,
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
    process: AppServerProcess,
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
}

#[derive(Clone, Copy)]
struct TerminalState {
    state: ConversationState,
    phase: ConversationLifecyclePhase,
    storage_status: &'static str,
    diagnostic_code: Option<ConversationDiagnosticCode>,
}

impl Default for ConversationService {
    fn default() -> Self {
        Self {
            state: Mutex::new(ConversationServiceState {
                active: None,
                last: ConversationSnapshot::empty(),
            }),
            command: AppServerCommand::codex("codex"),
        }
    }
}

impl ConversationService {
    #[cfg(test)]
    fn with_command(command: AppServerCommand) -> Self {
        Self {
            state: Mutex::new(ConversationServiceState {
                active: None,
                last: ConversationSnapshot::empty(),
            }),
            command,
        }
    }

    pub async fn status(&self) -> ConversationSnapshot {
        let state = self.state.lock().await;
        state
            .active
            .as_ref()
            .map(|active| active.snapshot(Vec::new(), None))
            .unwrap_or_else(|| state.last.clone())
    }

    pub async fn start(
        &self,
        request: ConversationStartRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if let Err(code) = validate_start_request(&request) {
            return ConversationSnapshot::unavailable(code);
        }

        let mut state = self.state.lock().await;
        if state.active.is_some() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationActive,
            );
        }

        if let Err(error) = projects.reserve_execution(&request.project_id) {
            return ConversationSnapshot::unavailable(map_project_error(error));
        }
        let cwd = match projects.execution_cwd(&request.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                projects.release_execution(&request.project_id);
                return ConversationSnapshot::unavailable(map_project_error(error));
            }
        };

        match self.start_reserved(&request, &cwd, projects).await {
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
                state.active = Some(active);
                state.last = snapshot.clone();
                snapshot
            }
            Err(code) => {
                projects.release_execution(&request.project_id);
                let snapshot = ConversationSnapshot::unavailable(code);
                state.last = snapshot.clone();
                snapshot
            }
        }
    }

    async fn start_reserved(
        &self,
        request: &ConversationStartRequest,
        cwd: &Path,
        projects: &ProjectService,
    ) -> Result<ActiveConversation, ConversationDiagnosticCode> {
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let started = match start_on_process(&mut process, request, cwd, projects).await {
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
            process,
        })
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
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return matching_last_or_not_found(&state.last, &conversation_id);
        };
        if active.conversation_id != conversation_id {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
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
                Ok(Some(notification)) => match apply_notification(active, notification) {
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
                },
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
            return finish_active(&mut state, projects, terminal, events).await;
        }
        let snapshot = state
            .active
            .as_ref()
            .expect("active conversation remains available")
            .snapshot(events, None);
        state.last = snapshot.clone();
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
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return matching_last_or_not_found(&state.last, &conversation_id);
        };
        if active.conversation_id != conversation_id {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
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
                return finish_active(
                    &mut state,
                    projects,
                    protocol_terminal(map_adapter_error(&error)),
                    events,
                )
                .await;
            }
            active.pending_approval = None;
            events.push(ConversationEvent::ApprovalResolved {
                sequence: active.take_sequence(),
                approval_id: pending.approval.approval_id,
                resolution: ConversationApprovalResolution::Canceled,
            });
        }

        let result = active
            .process
            .request(
                "turn/interrupt",
                json!({
                    "threadId": active.thread_id,
                    "turnId": active.turn_id,
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
            return finish_active(&mut state, projects, terminal, events).await;
        }

        events.push(active.lifecycle_event(ConversationLifecyclePhase::Stopping));
        active.state = ConversationState::Stopping;
        if projects
            .record_conversation_status(&active.conversation_id, "stopping")
            .is_err()
        {
            return finish_active(
                &mut state,
                projects,
                protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable),
                events,
            )
            .await;
        }
        let snapshot = active.snapshot(events, None);
        state.last = snapshot.clone();
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
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return matching_last_or_not_found(&state.last, &request.conversation_id);
        };
        if active.conversation_id != request.conversation_id {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
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
            return finish_active(
                &mut state,
                projects,
                protocol_terminal(map_adapter_error(&error)),
                Vec::new(),
            )
            .await;
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
            let result = active
                .process
                .request(
                    "turn/interrupt",
                    json!({
                        "threadId": active.thread_id,
                        "turnId": active.turn_id,
                    }),
                )
                .await;
            if let Err(error) = result {
                return finish_active(
                    &mut state,
                    projects,
                    protocol_terminal(map_adapter_error(&error)),
                    events,
                )
                .await;
            }
            events.push(active.lifecycle_event(ConversationLifecyclePhase::Stopping));
            active.state = ConversationState::Stopping;
            if projects
                .record_conversation_status(&active.conversation_id, "stopping")
                .is_err()
            {
                return finish_active(
                    &mut state,
                    projects,
                    protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable),
                    events,
                )
                .await;
            }
        } else {
            active.state = ConversationState::Running;
        }

        let snapshot = active.snapshot(events, None);
        state.last = snapshot.clone();
        snapshot
    }
}

fn approval_response(
    request: &ConversationServerRequest,
    decision: ConversationApprovalDecision,
) -> Value {
    match request {
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

    let thread_result = process
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
    let thread = parse_thread_start(thread_result, cwd)?;
    let conversation_id = Uuid::now_v7().to_string();
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
        })
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;

    let turn_result = process
        .request(
            "turn/start",
            json!({
                "threadId": thread.thread.id,
                "input": [{"type": "text", "text": request.prompt}],
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
    state: &mut ConversationServiceState,
    projects: &ProjectService,
    mut terminal: TerminalState,
    mut events: Vec<ConversationEvent>,
) -> ConversationSnapshot {
    let mut active = state
        .active
        .take()
        .expect("terminal transition requires an active conversation");
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
    let snapshot = active.snapshot(events, terminal.diagnostic_code);
    state.last = snapshot.clone();
    snapshot
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
        | AppServerNotification::AccountUpdated => return Ok((None, None)),
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
    validate_protocol_choice(&request.model_id, 128)?;
    validate_protocol_choice(&request.reasoning_effort, 32)?;
    if request.sandbox_mode == ConversationSandboxMode::DangerFullAccess
        && request.approval_policy == ConversationApprovalPolicy::Never
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
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

fn matching_last_or_not_found(
    last: &ConversationSnapshot,
    conversation_id: &str,
) -> ConversationSnapshot {
    if last.conversation_id.as_deref() == Some(conversation_id) {
        let mut snapshot = last.clone();
        snapshot.events.clear();
        snapshot
    } else {
        ConversationSnapshot::unavailable(ConversationDiagnosticCode::ConversationNotFound)
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
        assert_eq!(completed.state, ConversationState::Completed);
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
        let directory = std::env::temp_dir().join(format!(
            "quireforge-conversation-project-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&directory).expect("temporary project must be created");
        let projects = ProjectService::in_memory();
        let preview = projects.prepare_attachment(directory.clone());
        assert!(preview.pending_attachment.is_some());
        let snapshot = projects.confirm_pending();
        let project_id = snapshot
            .projects
            .first()
            .expect("project must attach")
            .id
            .clone();
        (projects, directory, project_id)
    }

    fn start_request(project_id: String) -> ConversationStartRequest {
        ConversationStartRequest {
            project_id,
            prompt: "Review the attached project.".to_owned(),
            model_id: "fixture-model".to_owned(),
            reasoning_effort: "medium".to_owned(),
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
}
