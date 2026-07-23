use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::project::{
    ConversationReference, ConversationSelectionMetadata, ProjectExecutionError, ProjectService,
    StoredConversationReference,
};

use super::{
    availability_storage_value, map_adapter_error, map_project_error,
    model_selection_from_reference, parse_turn_start, persist_model_selection, sandbox_policy,
    validate_attachment_ids, validate_protocol_choice, validate_user_text, ActiveConversation,
    ConversationService, ConversationState,
};
use crate::attachment::ResolvedConversationAttachment;
use crate::codex::{
    app_server::{validate_uuid_v7, AppServerProcess},
    conversation::types::{
        ConversationApprovalPolicy, ConversationDiagnosticCode, ConversationEvent,
        ConversationLifecyclePhase, ConversationSandboxMode, ConversationSnapshot,
    },
    model_selection::{
        ModelSelectionApplication, ModelSelectionDiagnosticCode, ModelSelectionService,
        ModelSelectionSnapshot,
    },
    types::CodexModel,
};

pub const SESSION_LIFECYCLE_SCHEMA_VERSION: u16 = 3;
const MAX_SESSION_REFERENCES: usize = 256;
const MAX_LIST_PAGES: usize = 8;
const LIST_PAGE_SIZE: u32 = 256;
const MAX_SESSION_TITLE_CHARS: usize = 256;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationContinueRequest {
    pub conversation_id: String,
    pub prompt: String,
    pub attachment_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionListRequest {
    pub project_id: Option<String>,
    pub search_term: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionLifecycleState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionReferenceState {
    Running,
    Completed,
    Interrupted,
    Blocked,
    Failed,
    Archived,
    Missing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReferenceSummary {
    pub conversation_id: String,
    pub project_id: String,
    pub parent_conversation_id: Option<String>,
    pub title: Option<String>,
    pub model_id: String,
    pub reasoning_effort: String,
    pub model_selection: ModelSelectionSnapshot,
    pub sandbox_mode: ConversationSandboxMode,
    pub approval_policy: ConversationApprovalPolicy,
    pub state: SessionReferenceState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLifecycleSnapshot {
    pub schema_version: u16,
    pub state: SessionLifecycleState,
    pub sessions: Vec<SessionReferenceSummary>,
    pub diagnostic_code: Option<ConversationDiagnosticCode>,
}

impl SessionLifecycleSnapshot {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: SESSION_LIFECYCLE_SCHEMA_VERSION,
            state: SessionLifecycleState::Empty,
            sessions: Vec::new(),
            diagnostic_code: None,
        }
    }

    fn unavailable(code: ConversationDiagnosticCode) -> Self {
        Self {
            state: SessionLifecycleState::Unavailable,
            diagnostic_code: Some(code),
            ..Self::empty()
        }
    }
}

#[derive(Clone, Copy)]
enum ContinueMode {
    Resume,
    Fork,
}

#[derive(Clone, Copy)]
struct ContinueTurnInputs<'a> {
    mode: ContinueMode,
    attachments: &'a [ResolvedConversationAttachment],
}

impl ConversationService {
    pub async fn sessions(
        &self,
        request: SessionListRequest,
        projects: &ProjectService,
    ) -> SessionLifecycleSnapshot {
        let search_term = request.search_term.as_deref().map(str::trim);
        if request
            .project_id
            .as_deref()
            .is_some_and(|value| validate_uuid_v7(value).is_err())
            || search_term.is_some_and(|value| !valid_session_text(value))
        {
            return SessionLifecycleSnapshot::unavailable(
                ConversationDiagnosticCode::InvalidRequest,
            );
        }

        let state = self.state.lock().await;
        let references = match projects.conversation_references(request.project_id.as_deref()) {
            Ok(references) => references,
            Err(error) => {
                return SessionLifecycleSnapshot::unavailable(map_project_error(error));
            }
        };
        if references.is_empty() {
            return SessionLifecycleSnapshot::empty();
        }
        if !state.active.is_empty() {
            return snapshot_from_references(references, None, None);
        }

        let reconciliation = match self
            .reconcile_references(&references, search_term, projects)
            .await
        {
            Ok(reconciliation) => reconciliation,
            Err(code) => return SessionLifecycleSnapshot::unavailable(code),
        };
        let references = match projects.conversation_references(request.project_id.as_deref()) {
            Ok(references) => references,
            Err(error) => {
                return SessionLifecycleSnapshot::unavailable(map_project_error(error));
            }
        };
        snapshot_from_references(
            references,
            Some(&reconciliation.authoritative),
            reconciliation.matches.as_ref(),
        )
    }

    #[cfg(test)]
    pub async fn resume(
        &self,
        request: ConversationContinueRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        self.resume_with_attachments(request, projects, Vec::new())
            .await
    }

    pub(crate) async fn resume_with_attachments(
        &self,
        request: ConversationContinueRequest,
        projects: &ProjectService,
        attachments: Vec<ResolvedConversationAttachment>,
    ) -> ConversationSnapshot {
        self.continue_conversation(request, projects, ContinueMode::Resume, attachments)
            .await
    }

    #[cfg(test)]
    pub async fn fork(
        &self,
        request: ConversationContinueRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        self.fork_with_attachments(request, projects, Vec::new())
            .await
    }

    pub(crate) async fn fork_with_attachments(
        &self,
        request: ConversationContinueRequest,
        projects: &ProjectService,
        attachments: Vec<ResolvedConversationAttachment>,
    ) -> ConversationSnapshot {
        self.continue_conversation(request, projects, ContinueMode::Fork, attachments)
            .await
    }

    pub async fn archive(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> SessionLifecycleSnapshot {
        self.set_archived(conversation_id, projects, true).await
    }

    pub async fn restore(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> SessionLifecycleSnapshot {
        self.set_archived(conversation_id, projects, false).await
    }

    async fn continue_conversation(
        &self,
        request: ConversationContinueRequest,
        projects: &ProjectService,
        mode: ContinueMode,
        attachments: Vec<ResolvedConversationAttachment>,
    ) -> ConversationSnapshot {
        if validate_continue_request(&request).is_err() {
            return ConversationSnapshot::unavailable(ConversationDiagnosticCode::InvalidRequest);
        }
        if attachments.len() != request.attachment_ids.len() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            );
        }

        let reference = match projects.conversation_reference(&request.conversation_id) {
            Ok(reference) => reference,
            Err(error) => return ConversationSnapshot::unavailable(map_reference_error(error)),
        };
        if reference.archived {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let controls = match StoredControls::try_from(&reference) {
            Ok(controls) => controls,
            Err(code) => return ConversationSnapshot::unavailable(code),
        };

        {
            let mut state = self.state.lock().await;
            if state.active.contains_key(&request.conversation_id) {
                return ConversationSnapshot::unavailable(
                    ConversationDiagnosticCode::ConversationActive,
                );
            }
            if let Err(code) = state.begin_start(&reference.project_id) {
                return ConversationSnapshot::unavailable(code);
            }
        }

        if let Err(error) = projects.reserve_execution(&reference.project_id) {
            self.state.lock().await.finish_start(&reference.project_id);
            return ConversationSnapshot::unavailable(map_project_error(error));
        }
        let cwd = match projects.execution_cwd(&reference.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                projects.release_execution(&reference.project_id);
                self.state.lock().await.finish_start(&reference.project_id);
                return ConversationSnapshot::unavailable(map_project_error(error));
            }
        };

        let active = self
            .continue_reserved(
                &request,
                &reference,
                &controls,
                &cwd,
                projects,
                ContinueTurnInputs {
                    mode,
                    attachments: &attachments,
                },
            )
            .await;
        match active {
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
                state.finish_start(&reference.project_id);
                state
                    .active
                    .insert(conversation_id, Arc::new(Mutex::new(active)));
                state.remember(snapshot.clone());
                snapshot
            }
            Err(code) => {
                projects.release_execution(&reference.project_id);
                let snapshot = ConversationSnapshot::unavailable(code);
                let mut state = self.state.lock().await;
                state.finish_start(&reference.project_id);
                state.remember(snapshot.clone());
                snapshot
            }
        }
    }

    async fn continue_reserved(
        &self,
        request: &ConversationContinueRequest,
        reference: &StoredConversationReference,
        controls: &StoredControls,
        cwd: &Path,
        projects: &ProjectService,
        inputs: ContinueTurnInputs<'_>,
    ) -> Result<ActiveConversation, ConversationDiagnosticCode> {
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let result = continue_on_process(
            &mut process,
            request,
            reference,
            controls,
            cwd,
            projects,
            inputs,
        )
        .await;
        match result {
            Ok(started) => Ok(ActiveConversation {
                conversation_id: started.conversation_id,
                project_id: reference.project_id.clone(),
                model_id: started.model_selection.effective.model_id.clone(),
                reasoning_effort: started.model_selection.effective.reasoning_effort.clone(),
                sandbox_mode: controls.sandbox_mode,
                approval_policy: controls.approval_policy,
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
            }),
            Err(code) => {
                let _ = process.shutdown().await;
                Err(code)
            }
        }
    }

    async fn set_archived(
        &self,
        conversation_id: String,
        projects: &ProjectService,
        archived: bool,
    ) -> SessionLifecycleSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return SessionLifecycleSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let state = self.state.lock().await;
        if state.active.contains_key(&conversation_id) {
            return SessionLifecycleSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationActive,
            );
        }
        drop(state);
        let reference = match projects.conversation_reference(&conversation_id) {
            Ok(reference) => reference,
            Err(error) => return SessionLifecycleSnapshot::unavailable(map_reference_error(error)),
        };
        if reference.archived == archived {
            return snapshot_from_references(vec![reference], None, None);
        }
        let _reservation =
            match ScopedProjectReservation::acquire(projects, reference.project_id.clone()) {
                Ok(reservation) => reservation,
                Err(error) => {
                    return SessionLifecycleSnapshot::unavailable(map_project_error(error));
                }
            };
        let cwd = match projects.execution_cwd(&reference.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                return SessionLifecycleSnapshot::unavailable(map_project_error(error));
            }
        };
        if let Err(code) = self
            .set_archived_on_process(&reference, &cwd, archived)
            .await
        {
            return SessionLifecycleSnapshot::unavailable(code);
        }
        if projects
            .record_conversation_archived(&conversation_id, archived)
            .is_err()
        {
            return SessionLifecycleSnapshot::unavailable(
                ConversationDiagnosticCode::MetadataUnavailable,
            );
        }
        match projects.conversation_reference(&conversation_id) {
            Ok(reference) => snapshot_from_references(vec![reference], None, None),
            Err(error) => SessionLifecycleSnapshot::unavailable(map_reference_error(error)),
        }
    }

    async fn set_archived_on_process(
        &self,
        reference: &StoredConversationReference,
        cwd: &Path,
        archived: bool,
    ) -> Result<(), ConversationDiagnosticCode> {
        validate_stored_reference(reference)?;
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let result = async {
            process
                .initialize()
                .await
                .map_err(|error| map_adapter_error(&error))?;
            read_exact_thread(&mut process, reference, cwd).await?;
            if archived {
                let response = process
                    .request(
                        "thread/archive",
                        json!({"threadId": reference.codex_thread_id}),
                    )
                    .await
                    .map_err(|error| map_adapter_error(&error))?;
                if !response.is_object() {
                    return Err(ConversationDiagnosticCode::ProtocolInvalid);
                }
            } else {
                let response = process
                    .request(
                        "thread/unarchive",
                        json!({"threadId": reference.codex_thread_id}),
                    )
                    .await
                    .map_err(|error| map_adapter_error(&error))?;
                parse_exact_thread(response, &reference.codex_thread_id, cwd)?;
            }
            Ok(())
        }
        .await;
        let _ = process.shutdown().await;
        result
    }

    async fn reconcile_references(
        &self,
        references: &[StoredConversationReference],
        search_term: Option<&str>,
        projects: &ProjectService,
    ) -> Result<SessionReconciliation, ConversationDiagnosticCode> {
        if references.len() > MAX_SESSION_REFERENCES {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        for reference in references {
            validate_stored_reference(reference)?;
        }

        let project_ids = references
            .iter()
            .map(|reference| reference.project_id.clone())
            .collect::<BTreeSet<_>>();
        let mut reservations = Vec::with_capacity(project_ids.len());
        let mut project_cwds = HashMap::with_capacity(project_ids.len());
        for project_id in project_ids {
            reservations.push(
                ScopedProjectReservation::acquire(projects, project_id.clone())
                    .map_err(map_project_error)?,
            );
            let cwd = projects
                .execution_cwd(&project_id)
                .map_err(map_project_error)?;
            project_cwds.insert(project_id, cwd);
        }

        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let result = async {
            process
                .initialize()
                .await
                .map_err(|error| map_adapter_error(&error))?;
            let mut authoritative = HashMap::new();
            let cwds = project_cwds.values().cloned().collect::<Vec<_>>();
            for archived in [false, true] {
                for thread in list_threads(&mut process, &cwds, archived, None).await? {
                    if authoritative
                        .insert(
                            thread.id,
                            AuthoritativeSession {
                                archived,
                                title: thread.title,
                            },
                        )
                        .is_some()
                    {
                        return Err(ConversationDiagnosticCode::ProtocolInvalid);
                    }
                }
            }
            let matches = if let Some(search_term) = search_term {
                let mut matches = BTreeSet::new();
                for archived in [false, true] {
                    for thread in
                        list_threads(&mut process, &cwds, archived, Some(search_term)).await?
                    {
                        if !authoritative.contains_key(&thread.id) || !matches.insert(thread.id) {
                            return Err(ConversationDiagnosticCode::ProtocolInvalid);
                        }
                    }
                }
                Some(matches)
            } else {
                None
            };
            Ok(SessionReconciliation {
                authoritative,
                matches,
            })
        }
        .await;
        let _ = process.shutdown().await;
        let reconciliation = result?;

        for reference in references {
            if let Some(authoritative) =
                reconciliation.authoritative.get(&reference.codex_thread_id)
            {
                if authoritative.archived != reference.archived {
                    projects
                        .record_conversation_archived(&reference.id, authoritative.archived)
                        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
                }
            }
        }
        Ok(reconciliation)
    }
}

struct AuthoritativeSession {
    archived: bool,
    title: Option<String>,
}

struct SessionReconciliation {
    authoritative: HashMap<String, AuthoritativeSession>,
    matches: Option<BTreeSet<String>>,
}

struct ScopedProjectReservation<'a> {
    projects: &'a ProjectService,
    project_id: String,
}

impl<'a> ScopedProjectReservation<'a> {
    fn acquire(
        projects: &'a ProjectService,
        project_id: String,
    ) -> Result<Self, ProjectExecutionError> {
        projects.reserve_execution(&project_id)?;
        Ok(Self {
            projects,
            project_id,
        })
    }
}

impl Drop for ScopedProjectReservation<'_> {
    fn drop(&mut self) {
        self.projects.release_execution(&self.project_id);
    }
}

struct ContinuedConversation {
    conversation_id: String,
    thread_id: String,
    turn_id: String,
    model_catalog: Vec<CodexModel>,
    model_selection: ModelSelectionSnapshot,
}

#[derive(Clone, Copy)]
struct StoredControls {
    sandbox_mode: ConversationSandboxMode,
    approval_policy: ConversationApprovalPolicy,
}

impl TryFrom<&StoredConversationReference> for StoredControls {
    type Error = ConversationDiagnosticCode;

    fn try_from(reference: &StoredConversationReference) -> Result<Self, Self::Error> {
        validate_stored_reference(reference)?;
        let sandbox_mode = match reference.sandbox_mode.as_str() {
            "read-only" => ConversationSandboxMode::ReadOnly,
            "workspace-write" => ConversationSandboxMode::WorkspaceWrite,
            "danger-full-access" => ConversationSandboxMode::DangerFullAccess,
            _ => return Err(ConversationDiagnosticCode::ProtocolInvalid),
        };
        let approval_policy = match reference.approval_policy.as_str() {
            "untrusted" => ConversationApprovalPolicy::Untrusted,
            "on-request" => ConversationApprovalPolicy::OnRequest,
            "never" => ConversationApprovalPolicy::Never,
            _ => return Err(ConversationDiagnosticCode::ProtocolInvalid),
        };
        if sandbox_mode == ConversationSandboxMode::DangerFullAccess
            && approval_policy == ConversationApprovalPolicy::Never
        {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        Ok(Self {
            sandbox_mode,
            approval_policy,
        })
    }
}

async fn continue_on_process(
    process: &mut AppServerProcess,
    request: &ConversationContinueRequest,
    reference: &StoredConversationReference,
    controls: &StoredControls,
    cwd: &Path,
    projects: &ProjectService,
    inputs: ContinueTurnInputs<'_>,
) -> Result<ContinuedConversation, ConversationDiagnosticCode> {
    let (model_catalog, _) = process
        .discover_models()
        .await
        .map_err(|error| map_adapter_error(&error))?;
    let mut model_selection = model_selection_from_reference(reference)?;
    ModelSelectionService::validate_choice(&model_selection.effective, &model_catalog)
        .map_err(map_selection_error)?;
    ModelSelectionService::validate_policy(
        &model_selection.policy,
        &model_catalog,
        &model_selection.effective,
    )
    .map_err(map_selection_error)?;
    let mut next_choice = model_selection.effective.clone();
    let mut consume_pending = false;
    if let Some(pending) = model_selection.pending.as_ref() {
        match pending.application {
            ModelSelectionApplication::Recommendation => {}
            ModelSelectionApplication::Manual => {
                ModelSelectionService::validate_choice(&pending.choice, &model_catalog)
                    .map_err(map_selection_error)?;
                next_choice = pending.choice.clone();
                consume_pending = true;
            }
            ModelSelectionApplication::Automatic => {
                if ModelSelectionService::pending_still_applicable(
                    pending,
                    &model_selection.policy,
                    &model_catalog,
                ) {
                    next_choice = pending.choice.clone();
                    consume_pending = true;
                } else {
                    model_selection.pending = None;
                    persist_model_selection(projects, &reference.id, &model_selection, None)?;
                }
            }
        }
    }
    read_exact_thread(process, reference, cwd).await?;

    let method = match inputs.mode {
        ContinueMode::Resume => "thread/resume",
        ContinueMode::Fork => "thread/fork",
    };
    let params = match inputs.mode {
        ContinueMode::Resume => json!({
            "threadId": reference.codex_thread_id,
            "cwd": cwd,
            "model": next_choice.model_id,
            "approvalPolicy": controls.approval_policy.as_protocol_value(),
            "sandbox": controls.sandbox_mode.as_protocol_value(),
            "excludeTurns": true,
        }),
        ContinueMode::Fork => json!({
            "threadId": reference.codex_thread_id,
            "cwd": cwd,
            "model": next_choice.model_id,
            "approvalPolicy": controls.approval_policy.as_protocol_value(),
            "sandbox": controls.sandbox_mode.as_protocol_value(),
            "excludeTurns": true,
            "ephemeral": false,
        }),
    };
    let response = process
        .request(method, params)
        .await
        .map_err(|error| map_adapter_error(&error))?;
    let expected_thread_id = match inputs.mode {
        ContinueMode::Resume => Some(reference.codex_thread_id.as_str()),
        ContinueMode::Fork => None,
    };
    let continued = parse_continue_response(
        response,
        expected_thread_id,
        reference,
        &next_choice.model_id,
        controls,
        cwd,
    )?;

    let conversation_id = match inputs.mode {
        ContinueMode::Resume => reference.id.clone(),
        ContinueMode::Fork => {
            let conversation_id = Uuid::now_v7().to_string();
            let allowed_model_ids_json =
                serde_json::to_string(&model_selection.policy.allowed_model_ids)
                    .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;
            if projects
                .record_conversation_reference(ConversationReference {
                    conversation_id: &conversation_id,
                    project_id: &reference.project_id,
                    codex_thread_id: &continued.thread.id,
                    model_id: &next_choice.model_id,
                    reasoning_effort: &next_choice.reasoning_effort,
                    sandbox_mode: controls.sandbox_mode.as_protocol_value(),
                    approval_policy: controls.approval_policy.as_protocol_value(),
                    parent_conversation_id: Some(&reference.id),
                    selection: ConversationSelectionMetadata {
                        availability: availability_storage_value(model_selection.availability),
                        ownership: model_selection.policy.ownership.as_storage_value(),
                        user_locked: model_selection.policy.user_locked,
                        allowed_model_ids_json: &allowed_model_ids_json,
                        reasoning_ceiling: model_selection.policy.reasoning_ceiling.as_deref(),
                        pending: None,
                    },
                })
                .is_err()
            {
                let _ = process
                    .request("thread/archive", json!({"threadId": continued.thread.id}))
                    .await;
                return Err(ConversationDiagnosticCode::MetadataUnavailable);
            }
            conversation_id
        }
    };

    let mut input = vec![json!({"type": "text", "text": request.prompt})];
    input.extend(
        inputs
            .attachments
            .iter()
            .map(ResolvedConversationAttachment::protocol_input),
    );
    let turn_result = process
        .request(
            "turn/start",
            json!({
                "threadId": continued.thread.id,
                "input": input,
                "cwd": cwd,
                "model": next_choice.model_id,
                "effort": next_choice.reasoning_effort,
                "approvalPolicy": controls.approval_policy.as_protocol_value(),
                "sandboxPolicy": sandbox_policy(controls.sandbox_mode, cwd),
            }),
        )
        .await
        .map_err(|error| {
            let _ = projects.record_conversation_status(&conversation_id, "failed");
            map_adapter_error(&error)
        })?;
    let turn = parse_turn_start(turn_result).inspect_err(|_| {
        let _ = projects.record_conversation_status(&conversation_id, "failed");
    })?;
    model_selection.effective = next_choice;
    if consume_pending || matches!(inputs.mode, ContinueMode::Fork) {
        model_selection.pending = None;
    }
    if matches!(inputs.mode, ContinueMode::Resume) {
        persist_model_selection(
            projects,
            &conversation_id,
            &model_selection,
            Some((
                &model_selection.effective.model_id,
                &model_selection.effective.reasoning_effort,
            )),
        )?;
    }
    projects
        .record_conversation_turn(&conversation_id, &turn.turn.id)
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;

    Ok(ContinuedConversation {
        conversation_id,
        thread_id: continued.thread.id,
        turn_id: turn.turn.id,
        model_catalog,
        model_selection,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContinueResponse {
    cwd: String,
    model: String,
    approval_policy: String,
    thread: ThreadReference,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadReference {
    id: String,
    cwd: String,
    forked_from_id: Option<String>,
    name: Option<String>,
}

fn map_selection_error(code: ModelSelectionDiagnosticCode) -> ConversationDiagnosticCode {
    match code {
        ModelSelectionDiagnosticCode::ModelUnavailable => {
            ConversationDiagnosticCode::ModelUnavailable
        }
        ModelSelectionDiagnosticCode::ReasoningUnavailable => {
            ConversationDiagnosticCode::ReasoningUnavailable
        }
        ModelSelectionDiagnosticCode::MetadataUnavailable => {
            ConversationDiagnosticCode::MetadataUnavailable
        }
        _ => ConversationDiagnosticCode::InvalidRequest,
    }
}

fn parse_continue_response(
    value: Value,
    expected_thread_id: Option<&str>,
    source: &StoredConversationReference,
    expected_model: &str,
    controls: &StoredControls,
    cwd: &Path,
) -> Result<ContinueResponse, ConversationDiagnosticCode> {
    let result: ContinueResponse =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.thread.id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if Path::new(&result.cwd) != cwd
        || Path::new(&result.thread.cwd) != cwd
        || result.model != expected_model
        || result.approval_policy != controls.approval_policy.as_protocol_value()
    {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    match expected_thread_id {
        Some(expected) if result.thread.id != expected => {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        None if result.thread.id == source.codex_thread_id
            || result.thread.forked_from_id.as_deref() != Some(source.codex_thread_id.as_str()) =>
        {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        _ => {}
    }
    Ok(result)
}

async fn read_exact_thread(
    process: &mut AppServerProcess,
    reference: &StoredConversationReference,
    cwd: &Path,
) -> Result<(), ConversationDiagnosticCode> {
    let response = process
        .request(
            "thread/read",
            json!({
                "threadId": reference.codex_thread_id,
                "includeTurns": false,
            }),
        )
        .await
        .map_err(|error| map_adapter_error(&error))?;
    parse_exact_thread(response, &reference.codex_thread_id, cwd)
}

#[derive(Deserialize)]
struct ReadResponse {
    thread: ThreadReference,
}

fn parse_exact_thread(
    value: Value,
    expected_thread_id: &str,
    cwd: &Path,
) -> Result<(), ConversationDiagnosticCode> {
    let response: ReadResponse =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&response.thread.id)
        .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if response.thread.id != expected_thread_id || Path::new(&response.thread.cwd) != cwd {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadListResponse {
    data: Vec<ThreadReference>,
    next_cursor: Option<String>,
}

async fn list_threads(
    process: &mut AppServerProcess,
    cwds: &[std::path::PathBuf],
    archived: bool,
    search_term: Option<&str>,
) -> Result<Vec<ListedThread>, ConversationDiagnosticCode> {
    if cwds.is_empty() || cwds.len() > MAX_SESSION_REFERENCES {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    let mut cursor: Option<String> = None;
    let mut threads = Vec::new();
    for _ in 0..MAX_LIST_PAGES {
        let response = process
            .request(
                "thread/list",
                json!({
                    "archived": archived,
                    "cursor": cursor,
                    "cwd": cwds,
                    "limit": LIST_PAGE_SIZE,
                    "searchTerm": search_term,
                    "useStateDbOnly": true,
                }),
            )
            .await
            .map_err(|error| map_adapter_error(&error))?;
        let page: ThreadListResponse = serde_json::from_value(response)
            .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
        if page.data.len() > LIST_PAGE_SIZE as usize {
            return Err(ConversationDiagnosticCode::ProtocolInvalid);
        }
        for thread in page.data {
            validate_uuid_v7(&thread.id)
                .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
            if !cwds.iter().any(|cwd| Path::new(&thread.cwd) == cwd)
                || threads
                    .iter()
                    .any(|listed: &ListedThread| listed.id == thread.id)
            {
                return Err(ConversationDiagnosticCode::ProtocolInvalid);
            }
            threads.push(ListedThread {
                id: thread.id,
                title: normalize_session_title(thread.name)?,
            });
        }
        match page.next_cursor {
            Some(next) if valid_cursor(&next) => cursor = Some(next),
            Some(_) => return Err(ConversationDiagnosticCode::ProtocolInvalid),
            None => return Ok(threads),
        }
    }
    Err(ConversationDiagnosticCode::ProtocolInvalid)
}

struct ListedThread {
    id: String,
    title: Option<String>,
}

fn normalize_session_title(
    title: Option<String>,
) -> Result<Option<String>, ConversationDiagnosticCode> {
    let Some(title) = title else {
        return Ok(None);
    };
    let title = title.trim();
    if title.is_empty() {
        return Ok(None);
    }
    if !valid_session_text(title) {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(Some(title.to_owned()))
}

fn valid_session_text(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= MAX_SESSION_TITLE_CHARS
        && !value.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '\u{200B}'..='\u{200F}'
                        | '\u{202A}'..='\u{202E}'
                        | '\u{2060}'..='\u{206F}'
                        | '\u{FEFF}'
                )
        })
}

fn valid_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && !value
            .chars()
            .any(|character| character.is_control() || character == '\0')
}

fn validate_continue_request(
    request: &ConversationContinueRequest,
) -> Result<(), ConversationDiagnosticCode> {
    validate_uuid_v7(&request.conversation_id)
        .map_err(|_| ConversationDiagnosticCode::InvalidRequest)?;
    validate_user_text(&request.prompt)?;
    validate_attachment_ids(&request.attachment_ids)
}

fn validate_stored_reference(
    reference: &StoredConversationReference,
) -> Result<(), ConversationDiagnosticCode> {
    for id in [
        Some(reference.id.as_str()),
        Some(reference.project_id.as_str()),
        Some(reference.codex_thread_id.as_str()),
        reference.active_turn_id.as_deref(),
        reference.parent_conversation_id.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_uuid_v7(id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    }
    validate_protocol_choice(&reference.model_id, 128)
        .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_protocol_choice(&reference.reasoning_effort, 32)
        .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    model_selection_from_reference(reference)
        .map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if reference.created_at_ms < 0 || reference.updated_at_ms < reference.created_at_ms {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    if !matches!(
        reference.status.as_str(),
        "thread-started"
            | "running"
            | "stopping"
            | "completed"
            | "interrupted"
            | "blocked"
            | "failed"
    ) {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn snapshot_from_references(
    references: Vec<StoredConversationReference>,
    authoritative: Option<&HashMap<String, AuthoritativeSession>>,
    matches: Option<&BTreeSet<String>>,
) -> SessionLifecycleSnapshot {
    let sessions = references
        .into_iter()
        .filter(|reference| {
            matches.is_none_or(|matches| matches.contains(&reference.codex_thread_id))
        })
        .map(|reference| summary_from_reference(reference, authoritative))
        .collect::<Result<Vec<_>, _>>();
    let sessions = match sessions {
        Ok(sessions) => sessions,
        Err(code) => return SessionLifecycleSnapshot::unavailable(code),
    };
    SessionLifecycleSnapshot {
        schema_version: SESSION_LIFECYCLE_SCHEMA_VERSION,
        state: if sessions.is_empty() {
            SessionLifecycleState::Empty
        } else {
            SessionLifecycleState::Ready
        },
        sessions,
        diagnostic_code: None,
    }
}

fn summary_from_reference(
    reference: StoredConversationReference,
    authoritative: Option<&HashMap<String, AuthoritativeSession>>,
) -> Result<SessionReferenceSummary, ConversationDiagnosticCode> {
    let controls = StoredControls::try_from(&reference)?;
    let model_selection = model_selection_from_reference(&reference)?;
    let authoritative_session =
        authoritative.and_then(|threads| threads.get(&reference.codex_thread_id));
    let state = match authoritative_session {
        None if authoritative.is_some() => SessionReferenceState::Missing,
        Some(session) if session.archived => SessionReferenceState::Archived,
        _ if reference.archived => SessionReferenceState::Archived,
        _ => match reference.status.as_str() {
            "thread-started" | "running" | "stopping" => SessionReferenceState::Running,
            "completed" => SessionReferenceState::Completed,
            "interrupted" => SessionReferenceState::Interrupted,
            "blocked" => SessionReferenceState::Blocked,
            "failed" => SessionReferenceState::Failed,
            _ => return Err(ConversationDiagnosticCode::ProtocolInvalid),
        },
    };
    Ok(SessionReferenceSummary {
        conversation_id: reference.id,
        project_id: reference.project_id,
        parent_conversation_id: reference.parent_conversation_id,
        title: authoritative_session.and_then(|session| session.title.clone()),
        model_id: reference.model_id,
        reasoning_effort: reference.reasoning_effort,
        model_selection,
        sandbox_mode: controls.sandbox_mode,
        approval_policy: controls.approval_policy,
        state,
        created_at_ms: reference.created_at_ms,
        updated_at_ms: reference.updated_at_ms,
    })
}

fn map_reference_error(error: ProjectExecutionError) -> ConversationDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            ConversationDiagnosticCode::ConversationNotFound
        }
        _ => map_project_error(error),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;
    use uuid::Uuid;

    use super::*;
    use crate::codex::app_server::AppServerCommand;

    const THREAD_ID: &str = "018f0000-0000-7000-8000-000000000020";
    const FORK_THREAD_ID: &str = "018f0000-0000-7000-8000-000000000021";
    const TURN_ID: &str = "018f0000-0000-7000-8000-000000000030";

    #[test]
    fn serialized_empty_snapshot_matches_the_shared_frontend_fixture() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../../../fixtures/session-lifecycle.json"))
                .expect("session lifecycle fixture must be JSON");
        let snapshot = serde_json::to_value(SessionLifecycleSnapshot::empty())
            .expect("session lifecycle snapshot must serialize");

        assert_eq!(snapshot, fixture);
    }

    #[tokio::test]
    async fn resumes_only_an_owned_reference_without_exposing_native_identity() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        let cwd_json = json_string(&directory);
        let script = lifecycle_start_script(&cwd_json, THREAD_ID, "thread/resume", "", THREAD_ID);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .resume(
                ConversationContinueRequest {
                    conversation_id: conversation_id.clone(),
                    prompt: "Continue safely.".to_owned(),
                    attachment_ids: Vec::new(),
                },
                &projects,
            )
            .await;

        assert_eq!(
            started.state,
            ConversationState::Running,
            "unexpected resume result: {started:?}"
        );
        assert_eq!(
            started.conversation_id.as_deref(),
            Some(conversation_id.as_str())
        );
        let serialized = serde_json::to_string(&started).expect("snapshot must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));

        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn revalidates_and_applies_an_automatic_choice_only_to_the_next_turn() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        projects
            .record_model_selection(
                &conversation_id,
                None,
                ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "automatic",
                    user_locked: false,
                    allowed_model_ids_json: r#"["fixture-model","fixture-terra"]"#,
                    reasoning_ceiling: Some("medium"),
                    pending: Some(crate::project::ConversationPendingSelection {
                        model_id: "fixture-terra",
                        reasoning_effort: "medium",
                        rationale: "Use the allowed model on the next turn.",
                        provenance: "codex",
                        application: "automatic",
                        requested_at_ms: 1,
                    }),
                },
            )
            .expect("pending choice must store");
        let stored_before = projects
            .conversation_reference(&conversation_id)
            .expect("selector reference must remain readable");
        assert!(
            model_selection_from_reference(&stored_before).is_ok(),
            "stored selector state must parse: {stored_before:?}"
        );
        let cwd_json = json_string(&directory);
        let script = lifecycle_start_script(&cwd_json, THREAD_ID, "thread/resume", "", THREAD_ID)
            .replace(
                r#"{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}"#,
                r#"{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]},{"model":"fixture-terra","displayName":"Fixture Terra","isDefault":false,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}"#,
            )
            .replace(
                r#""model":"fixture-model","approvalPolicy":"untrusted""#,
                r#""model":"fixture-terra","approvalPolicy":"untrusted""#,
            )
            .replace(
                "read -r _turn\nprintf",
                "read -r _turn\ncase \"$_turn\" in *'\"model\":\"fixture-terra\"'*) ;; *) exit 88 ;; esac\ncase \"$_turn\" in *'\"effort\":\"medium\"'*) ;; *) exit 89 ;; esac\nprintf",
            );
        assert!(
            script.contains(r#""model":"fixture-terra","approvalPolicy":"untrusted""#),
            "continue response must use the pending model: {script}"
        );
        assert!(
            script.contains(r#""model":"fixture-terra","displayName":"Fixture Terra""#),
            "fresh catalog must advertise the pending model: {script}"
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .resume(
                ConversationContinueRequest {
                    conversation_id: conversation_id.clone(),
                    prompt: "Continue on the revalidated next-turn choice.".to_owned(),
                    attachment_ids: Vec::new(),
                },
                &projects,
            )
            .await;
        assert_eq!(
            started.state,
            ConversationState::Running,
            "unexpected selector resume result: {started:?}"
        );
        assert_eq!(started.model_id.as_deref(), Some("fixture-terra"));
        assert!(started
            .model_selection
            .as_ref()
            .is_some_and(|selection| selection.pending.is_none()));
        let stored = projects
            .conversation_reference(&conversation_id)
            .expect("effective choice must update");
        assert_eq!(stored.model_id, "fixture-terra");
        assert!(stored.selector_pending_model_id.is_none());

        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn discards_a_stale_automatic_request_without_changing_effective_selection() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        projects
            .record_model_selection(
                &conversation_id,
                None,
                ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "automatic",
                    user_locked: false,
                    allowed_model_ids_json: r#"["fixture-model"]"#,
                    reasoning_ceiling: Some("medium"),
                    pending: Some(crate::project::ConversationPendingSelection {
                        model_id: "stale-model",
                        reasoning_effort: "medium",
                        rationale: "This catalog row disappeared.",
                        provenance: "codex",
                        application: "automatic",
                        requested_at_ms: 1,
                    }),
                },
            )
            .expect("stale pending choice must store");
        let cwd_json = json_string(&directory);
        let script = lifecycle_start_script(&cwd_json, THREAD_ID, "thread/resume", "", THREAD_ID);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .resume(
                ConversationContinueRequest {
                    conversation_id: conversation_id.clone(),
                    prompt: "Continue without the stale selector request.".to_owned(),
                    attachment_ids: Vec::new(),
                },
                &projects,
            )
            .await;
        assert_eq!(started.state, ConversationState::Running);
        assert_eq!(started.model_id.as_deref(), Some("fixture-model"));
        let stored = projects
            .conversation_reference(&conversation_id)
            .expect("stale request must clear");
        assert_eq!(stored.model_id, "fixture-model");
        assert!(stored.selector_pending_model_id.is_none());

        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn resume_sends_only_native_resolved_local_image_inputs() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        let attachment_id = "018f0000-0000-7000-8000-000000000099";
        let cwd_json = json_string(&directory);
        let script = lifecycle_start_script(&cwd_json, THREAD_ID, "thread/resume", "", THREAD_ID)
            .replacen(
                "read -r _turn",
                &format!(
                    r#"read -r _turn
case "$_turn" in *'"path":'*'"type":"localImage"'*) ;; *) exit 86 ;; esac
case "$_turn" in *'{attachment_id}'*) exit 87 ;; *) ;; esac"#
                ),
                1,
            );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .resume_with_attachments(
                ConversationContinueRequest {
                    conversation_id: conversation_id.clone(),
                    prompt: "Review the image safely.".to_owned(),
                    attachment_ids: vec![attachment_id.to_owned()],
                },
                &projects,
                vec![ResolvedConversationAttachment::for_test(
                    directory.join("private-staged-image.png"),
                )],
            )
            .await;
        assert_eq!(started.state, ConversationState::Running);
        let completed = service.poll(conversation_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn forks_to_a_new_app_reference_with_bounded_parent_lineage() {
        let (projects, directory, project_id) = attached_project();
        let parent_id = stored_reference(&projects, &project_id, None);
        let cwd_json = json_string(&directory);
        let fork_fields = format!(",\"forkedFromId\":\"{THREAD_ID}\"");
        let script = lifecycle_start_script(
            &cwd_json,
            FORK_THREAD_ID,
            "thread/fork",
            &fork_fields,
            THREAD_ID,
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .fork(
                ConversationContinueRequest {
                    conversation_id: parent_id.clone(),
                    prompt: "Try a separate approach.".to_owned(),
                    attachment_ids: Vec::new(),
                },
                &projects,
            )
            .await;
        let fork_id = started
            .conversation_id
            .clone()
            .expect("fork must have an app conversation ID");
        assert_ne!(fork_id, parent_id);
        let stored = projects
            .conversation_reference(&fork_id)
            .expect("fork reference must be stored");
        assert_eq!(
            stored.parent_conversation_id.as_deref(),
            Some(parent_id.as_str())
        );
        assert_eq!(stored.codex_thread_id, FORK_THREAD_ID);

        let completed = service.poll(fork_id, &projects).await;
        assert_eq!(completed.state, ConversationState::Completed);
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn archives_and_restores_without_deleting_the_reference() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        projects
            .record_conversation_status(&conversation_id, "completed")
            .expect("fixture status must update");
        let cwd_json = json_string(&directory);
        let archive_script = lifecycle_mutation_script(&cwd_json, "thread/archive", "{}");
        let archive_service = ConversationService::with_command(AppServerCommand::test(
            "sh",
            &["-c", &archive_script],
        ));

        let archive_operation = archive_service.archive(conversation_id.clone(), &projects);
        tokio::pin!(archive_operation);
        tokio::select! {
            _ = &mut archive_operation => panic!("archive operation must remain pending"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(20)) => {}
        }
        assert_eq!(
            projects.archive(project_id.clone()).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        let archived = archive_operation.await;
        assert_eq!(archived.sessions.len(), 1);
        assert_eq!(archived.sessions[0].state, SessionReferenceState::Archived);
        assert!(
            projects
                .conversation_reference(&conversation_id)
                .expect("reference must remain stored")
                .archived
        );

        let restore_response = format!(r#"{{"thread":{{"id":"{THREAD_ID}","cwd":{cwd_json}}}}}"#);
        let restore_script =
            lifecycle_mutation_script(&cwd_json, "thread/unarchive", &restore_response);
        let restore_service = ConversationService::with_command(AppServerCommand::test(
            "sh",
            &["-c", &restore_script],
        ));
        let restored = restore_service
            .restore(conversation_id.clone(), &projects)
            .await;

        assert_eq!(restored.sessions.len(), 1);
        assert_eq!(restored.sessions[0].state, SessionReferenceState::Completed);
        assert!(
            !projects
                .conversation_reference(&conversation_id)
                .expect("reference must remain stored")
                .archived
        );
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn reconciles_only_owned_references_from_exact_cwd_lists() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        projects
            .record_conversation_status(&conversation_id, "completed")
            .expect("fixture status must update");
        let cwd_json = json_string(&directory);
        let script = format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _current
printf '%s\n' '{{"id":2,"result":{{"data":[{{"id":"{THREAD_ID}","cwd":{cwd_json}}}]}}}}'
read -r _archived
printf '%s\n' '{{"id":3,"result":{{"data":[]}}}}'
"#
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let sessions = service
            .sessions(
                SessionListRequest {
                    project_id: Some(project_id),
                    search_term: None,
                },
                &projects,
            )
            .await;
        assert_eq!(sessions.state, SessionLifecycleState::Ready);
        assert_eq!(sessions.sessions.len(), 1);
        assert_eq!(sessions.sessions[0].state, SessionReferenceState::Completed);
        let serialized = serde_json::to_string(&sessions).expect("snapshot must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn searches_authoritative_titles_without_exposing_native_identity() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        projects
            .record_conversation_status(&conversation_id, "completed")
            .expect("fixture status must update");
        let cwd_json = json_string(&directory);
        let script = format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _current
printf '%s\n' '{{"id":2,"result":{{"data":[{{"id":"{THREAD_ID}","cwd":{cwd_json},"name":"Review lifecycle boundaries"}}]}}}}'
read -r _archived
printf '%s\n' '{{"id":3,"result":{{"data":[]}}}}'
read -r filtered_current
case "$filtered_current" in
  *'"searchTerm":"lifecycle"'*) ;;
  *) exit 1 ;;
esac
printf '%s\n' '{{"id":4,"result":{{"data":[{{"id":"{THREAD_ID}","cwd":{cwd_json},"name":"Review lifecycle boundaries"}}]}}}}'
read -r filtered_archived
case "$filtered_archived" in
  *'"searchTerm":"lifecycle"'*) ;;
  *) exit 1 ;;
esac
printf '%s\n' '{{"id":5,"result":{{"data":[]}}}}'
"#
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let sessions = service
            .sessions(
                SessionListRequest {
                    project_id: Some(project_id),
                    search_term: Some("lifecycle".to_owned()),
                },
                &projects,
            )
            .await;

        assert_eq!(sessions.state, SessionLifecycleState::Ready);
        assert_eq!(sessions.sessions.len(), 1);
        assert_eq!(
            sessions.sessions[0].title.as_deref(),
            Some("Review lifecycle boundaries")
        );
        let serialized = serde_json::to_string(&sessions).expect("snapshot must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn rejects_a_resumed_thread_from_a_different_cwd_and_releases_the_project() {
        let (projects, directory, project_id) = attached_project();
        let conversation_id = stored_reference(&projects, &project_id, None);
        let wrong_cwd = json_string(&directory.join("different"));
        let script = format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _read
printf '%s\n' '{{"id":2,"result":{{"thread":{{"id":"{THREAD_ID}","cwd":{wrong_cwd}}}}}}}'
"#
        );
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let failed = service
            .resume(
                ConversationContinueRequest {
                    conversation_id,
                    prompt: "Continue safely.".to_owned(),
                    attachment_ids: Vec::new(),
                },
                &projects,
            )
            .await;

        assert_eq!(
            failed.diagnostic_code,
            Some(ConversationDiagnosticCode::ProtocolInvalid)
        );
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    fn attached_project() -> (ProjectService, PathBuf, String) {
        let directory = std::env::temp_dir().join(format!(
            "quireforge-session-lifecycle-project-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&directory).expect("temporary project must be created");
        let projects = ProjectService::in_memory();
        let preview = projects.prepare_attachment(directory.clone());
        assert!(preview.pending_attachment.is_some());
        let project_id = projects
            .confirm_pending()
            .projects
            .first()
            .expect("project must attach")
            .id
            .clone();
        (projects, directory, project_id)
    }

    fn stored_reference(
        projects: &ProjectService,
        project_id: &str,
        parent_conversation_id: Option<&str>,
    ) -> String {
        let conversation_id = Uuid::now_v7().to_string();
        projects
            .record_conversation_reference(ConversationReference {
                conversation_id: &conversation_id,
                project_id,
                codex_thread_id: THREAD_ID,
                model_id: "fixture-model",
                reasoning_effort: "medium",
                sandbox_mode: "read-only",
                approval_policy: "untrusted",
                parent_conversation_id,
                selection: ConversationSelectionMetadata {
                    availability: "ready",
                    ownership: "manual",
                    user_locked: false,
                    allowed_model_ids_json: "[]",
                    reasoning_ceiling: None,
                    pending: None,
                },
            })
            .expect("conversation reference must store");
        conversation_id
    }

    fn json_string(path: &Path) -> String {
        serde_json::to_string(&path.to_string_lossy()).expect("temporary cwd must serialize")
    }

    fn lifecycle_start_script(
        cwd_json: &str,
        response_thread_id: &str,
        method: &str,
        fork_fields: &str,
        expected_thread_id: &str,
    ) -> String {
        format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _models
case "$_models" in *'"method":"model/list"'*) ;; *) exit 80 ;; esac
printf '%s\n' '{{"id":2,"result":{{"data":[{{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{{"reasoningEffort":"medium"}}]}}]}}}}'
read -r _read
case "$_read" in *'"includeTurns":false'*'"threadId":"{expected_thread_id}"'*) ;; *) exit 81 ;; esac
printf '%s\n' '{{"id":3,"result":{{"thread":{{"id":"{expected_thread_id}","cwd":{cwd_json}}}}}}}'
read -r _continue
case "$_continue" in *'"method":"{method}"'*'"excludeTurns":true'*) ;; *) exit 82 ;; esac
case "$_continue" in *'"path"'*|*'"history"'*|*'"runtimeWorkspaceRoots"'*) exit 83 ;; esac
printf '%s\n' '{{"id":4,"result":{{"cwd":{cwd_json},"model":"fixture-model","approvalPolicy":"untrusted","thread":{{"id":"{response_thread_id}","cwd":{cwd_json}{fork_fields}}}}}}}'
read -r _turn
printf '%s\n' '{{"id":5,"result":{{"turn":{{"id":"{TURN_ID}","status":"inProgress"}}}}}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{response_thread_id}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        )
    }

    fn lifecycle_mutation_script(cwd_json: &str, method: &str, response: &str) -> String {
        let notification = if method == "thread/archive" {
            "thread/archived"
        } else {
            "thread/unarchived"
        };
        format!(
            r#"
read -r _initialize
printf '%s\n' '{{"id":1,"result":{{}}}}'
read -r _read
printf '%s\n' '{{"id":2,"result":{{"thread":{{"id":"{THREAD_ID}","cwd":{cwd_json}}}}}}}'
read -r _mutation
case "$_mutation" in *'"method":"{method}"'*'"threadId":"{THREAD_ID}"'*) ;; *) exit 84 ;; esac
printf '%s\n' '{{"method":"{notification}","params":{{"threadId":"{THREAD_ID}"}}}}'
sleep 0.1
printf '%s\n' '{{"id":3,"result":{response}}}'
"#
        )
    }
}
