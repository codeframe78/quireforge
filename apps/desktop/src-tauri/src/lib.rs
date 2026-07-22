mod attachment;
mod codex;
mod contract;
mod desktop;
mod git;
mod preview;
mod project;
mod terminal;
mod worktree;

pub use codex::integration;

use attachment::{
    types::{
        ConversationAttachmentCancelRequest, ConversationAttachmentDropRequest,
        ConversationAttachmentSnapshot, ConversationAttachmentState,
    },
    ClaimedConversationAttachments, ConversationAttachmentService,
};
use codex::{
    types::CodexRuntimeSnapshot, AuthLoginMethod, CodexAuthService, CodexAuthSnapshot,
    CodexRuntimeService, ConversationApprovalDecisionRequest, ConversationContinueRequest,
    ConversationDiagnosticCode, ConversationRegistrySnapshot, ConversationService,
    ConversationSnapshot, ConversationStartRequest, IntegrationCatalogService,
    IntegrationControlService, IntegrationMutationService, SessionLifecycleSnapshot,
};
use contract::DesktopBootstrap;
use desktop::{
    DesktopNotificationRequest, DesktopNotificationResult, DesktopNotificationService,
    DesktopNotificationStatus,
};
use git::{
    types::{
        GitDiffRequest, GitDiffSnapshot, GitMutationConfirmRequest, GitMutationPreviewRequest,
        GitMutationPreviewSnapshot, GitMutationResultSnapshot, GitOpenFileRequest,
        GitRecoveryRequest, GitWorkspaceSnapshot,
    },
    GitService,
};
use preview::{
    types::{FilePreviewHandoffRequest, FilePreviewSnapshot},
    FilePreviewService,
};
use project::{
    types::{ProjectPreflightSnapshot, ProjectWorkspaceSnapshot},
    ProjectService,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use terminal::{
    types::{
        TerminalCloseRequest, TerminalPollRequest, TerminalRegistrySnapshot, TerminalResizeRequest,
        TerminalSnapshot, TerminalStartRequest, TerminalWriteRequest,
    },
    TerminalService,
};
use worktree::{
    types::{
        WorktreeCancelRequest, WorktreeConfirmRequest, WorktreeCreatePreviewRequest,
        WorktreePreviewSnapshot, WorktreeRecoverPreviewRequest, WorktreeRemovePreviewRequest,
        WorktreeResultSnapshot, WorktreeWorkspaceSnapshot,
    },
    WorktreeService,
};

#[tauri::command]
fn desktop_bootstrap() -> DesktopBootstrap {
    DesktopBootstrap::current()
}

#[tauri::command]
async fn codex_runtime_probe(
    service: tauri::State<'_, CodexRuntimeService>,
) -> Result<CodexRuntimeSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[tauri::command]
async fn integration_catalog_read(
    service: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationCatalogSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[tauri::command]
async fn integration_catalog_refresh(
    service: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationCatalogSnapshot, ()> {
    Ok(service.refresh().await)
}

#[tauri::command]
async fn integration_control_preview(
    request: integration::IntegrationControlPreviewRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlPreviewSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    Ok(service.preview(request, &snapshot).await)
}

#[tauri::command]
async fn integration_control_confirm(
    request: integration::IntegrationControlConfirmationRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    let result = service.confirm(request, &snapshot).await;
    if result.catalog_refresh_required {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn integration_control_open_browser(
    request: integration::IntegrationControlActionRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, IntegrationControlService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let (url, result) = service.claim_handoff(&request).await.map_err(|_| ())?;
    if app.opener().open_url(url, None::<&str>).is_err() {
        service.restore_handoff(&request).await;
        return Err(());
    }
    Ok(result)
}

#[tauri::command]
async fn integration_control_status(
    request: integration::IntegrationControlActionRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let result = service.status(request).await;
    if result.catalog_refresh_required {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn integration_mutation_preview(
    request: integration::IntegrationMutationPreviewRequest,
    service: tauri::State<'_, IntegrationMutationService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationMutationPreviewSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    Ok(service.preview(request, &snapshot).await)
}

#[tauri::command]
async fn integration_mutation_confirm(
    request: integration::IntegrationMutationConfirmRequest,
    service: tauri::State<'_, IntegrationMutationService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationMutationResultSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    let result = service.confirm(request, &snapshot).await;
    if result.state == integration::IntegrationMutationResultState::Applied {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn codex_auth_status(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn codex_auth_refresh(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.refresh().await)
}

#[tauri::command]
async fn codex_auth_start(
    method: AuthLoginMethod,
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.start_login(method).await)
}

#[tauri::command]
async fn codex_auth_cancel(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.cancel_login().await)
}

#[tauri::command]
async fn codex_auth_logout(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.logout().await)
}

#[tauri::command]
async fn codex_auth_open_browser(
    app: tauri::AppHandle,
    service: tauri::State<'_, CodexAuthService>,
) -> Result<(), ()> {
    let url = service.handoff_url().await.ok_or(())?;
    app.opener().open_url(url, None::<&str>).map_err(|_| ())
}

#[tauri::command]
fn project_workspace_status(service: tauri::State<'_, ProjectService>) -> ProjectWorkspaceSnapshot {
    service.status()
}

#[tauri::command]
async fn project_pick_directory(
    app: tauri::AppHandle,
    service: tauri::State<'_, ProjectService>,
) -> Result<ProjectWorkspaceSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach a local project")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.prepare_attachment(path),
            Err(_) => service.picker_unavailable(),
        },
        None => service.cancel_pending(),
    })
}

#[tauri::command]
async fn project_pick_relink(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, ProjectService>,
) -> Result<ProjectWorkspaceSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Relink the local project")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.prepare_relink(project_id, path),
            Err(_) => service.picker_unavailable(),
        },
        None => service.cancel_pending(),
    })
}

#[tauri::command]
fn project_confirm_attachment(
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.confirm_pending()
}

#[tauri::command]
fn project_cancel_attachment(
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.cancel_pending()
}

#[tauri::command]
fn project_detach(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.detach(project_id)
}

#[tauri::command]
fn project_archive(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.archive(project_id)
}

#[tauri::command]
fn project_preflight(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectPreflightSnapshot {
    service.preflight(project_id)
}

#[tauri::command]
async fn file_preview_pick(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, FilePreviewService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<FilePreviewSnapshot, ()> {
    if !preview::valid_project_id(&project_id) {
        return Ok(FilePreviewSnapshot::unavailable(
            None,
            preview::types::FilePreviewDiagnosticCode::InvalidRequest,
        ));
    }
    let selection = app
        .dialog()
        .file()
        .set_title("Preview a project file")
        .blocking_pick_file();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.preview_selected(project_id, path, &projects),
            Err(_) => {
                service.clear_project(&project_id);
                FilePreviewSnapshot::unavailable(
                    Some(project_id),
                    preview::types::FilePreviewDiagnosticCode::PickerUnavailable,
                )
            }
        },
        None => {
            service.clear_project(&project_id);
            FilePreviewSnapshot::empty(Some(project_id))
        }
    })
}

#[tauri::command]
async fn file_preview_open(
    request: FilePreviewHandoffRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, FilePreviewService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<(), preview::types::FilePreviewDiagnosticCode> {
    let claimed = service.claim_handoff(&request)?;
    let path = match claimed.path(&projects) {
        Ok(path) => path,
        Err(error) => return Err(error),
    };
    let Some(path) = path.to_str() else {
        return Err(preview::types::FilePreviewDiagnosticCode::UnsafePath);
    };
    if app.opener().open_path(path, None::<&str>).is_err() {
        service.restore_handoff(claimed);
        return Err(preview::types::FilePreviewDiagnosticCode::OpenFailed);
    }
    Ok(())
}

#[tauri::command]
fn file_preview_cancel(
    request: FilePreviewHandoffRequest,
    service: tauri::State<'_, FilePreviewService>,
) -> bool {
    service.cancel_handoff(&request)
}

#[tauri::command]
async fn conversation_notify(
    request: DesktopNotificationRequest,
    app: tauri::AppHandle,
    conversations: tauri::State<'_, ConversationService>,
    notifications: tauri::State<'_, DesktopNotificationService>,
) -> Result<DesktopNotificationResult, ()> {
    let Some(candidate) = conversations
        .notification_candidate(&request.conversation_id)
        .await
    else {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Ineligible,
        ));
    };
    let Some(window) = app.get_webview_window("main") else {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Unavailable,
        ));
    };
    if window.is_focused().unwrap_or(true) {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Foreground,
        ));
    }
    let prepared = match notifications.prepare(candidate) {
        Ok(Some(prepared)) => prepared,
        Ok(None) => {
            return Ok(DesktopNotificationResult::new(
                DesktopNotificationStatus::Duplicate,
            ));
        }
        Err(()) => {
            return Ok(DesktopNotificationResult::new(
                DesktopNotificationStatus::Unavailable,
            ));
        }
    };
    if app
        .notification()
        .builder()
        .title(prepared.title())
        .body(prepared.body())
        .show()
        .is_err()
    {
        notifications.restore(prepared);
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Unavailable,
        ));
    }
    notifications.complete(prepared);
    Ok(DesktopNotificationResult::new(
        DesktopNotificationStatus::Sent,
    ))
}

#[tauri::command]
fn conversation_attachment_status(
    project_id: String,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> ConversationAttachmentSnapshot {
    service.status(project_id, &projects)
}

#[tauri::command]
async fn conversation_attachment_pick(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationAttachmentSnapshot, ()> {
    let status = service.status(project_id.clone(), &projects);
    if status.state == ConversationAttachmentState::Unavailable {
        return Ok(status);
    }
    let selection = app
        .dialog()
        .file()
        .set_title("Attach images to the next Codex turn")
        .add_filter("Supported images", &["png", "jpg", "jpeg"])
        .blocking_pick_files();
    let Some(selection) = selection else {
        return Ok(status);
    };
    let selected_paths = selection
        .into_iter()
        .map(|path| path.into_path())
        .collect::<Result<Vec<_>, _>>();
    Ok(match selected_paths {
        Ok(paths) => service.stage_picker_paths(project_id, paths, &projects),
        Err(_) => ConversationAttachmentSnapshot::unavailable(
            Some(project_id),
            attachment::types::ConversationAttachmentDiagnosticCode::InvalidRequest,
        ),
    })
}

#[tauri::command]
fn conversation_attachment_stage_drop(
    request: ConversationAttachmentDropRequest,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> ConversationAttachmentSnapshot {
    service.stage_drop(request, &projects)
}

#[tauri::command]
fn conversation_attachment_cancel(
    request: ConversationAttachmentCancelRequest,
    service: tauri::State<'_, ConversationAttachmentService>,
) -> ConversationAttachmentSnapshot {
    service.cancel(request)
}

#[tauri::command]
async fn worktree_status(
    project_id: String,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreeWorkspaceSnapshot, ()> {
    Ok(service.status(project_id, &projects).await)
}

#[tauri::command]
async fn worktree_create_preview(
    request: WorktreeCreatePreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_create(request, &projects).await)
}

#[tauri::command]
async fn worktree_recover_preview(
    request: WorktreeRecoverPreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_recover(request, &projects).await)
}

#[tauri::command]
async fn worktree_remove_preview(
    request: WorktreeRemovePreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_remove(request, &projects).await)
}

#[tauri::command]
async fn worktree_pick_attach(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach an existing Git worktree")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.preview_attach(project_id, path, &projects).await,
            Err(_) => service.picker_unavailable(project_id),
        },
        None => service.picker_cancelled(project_id),
    })
}

#[tauri::command]
async fn worktree_confirm(
    request: WorktreeConfirmRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreeResultSnapshot, ()> {
    Ok(service.confirm(request, &projects).await)
}

#[tauri::command]
fn worktree_cancel(
    request: WorktreeCancelRequest,
    service: tauri::State<'_, WorktreeService>,
) -> bool {
    service.cancel(request)
}

#[tauri::command]
async fn git_status(
    project_id: String,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitWorkspaceSnapshot, ()> {
    Ok(service.status(project_id, &projects).await)
}

#[tauri::command]
async fn git_diff(
    request: GitDiffRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitDiffSnapshot, ()> {
    Ok(service.diff(request, &projects).await)
}

#[tauri::command]
async fn git_open_file(
    request: GitOpenFileRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<(), git::types::GitDiagnosticCode> {
    let path = service.review_file(request, &projects).await?;
    let path = path
        .to_str()
        .ok_or(git::types::GitDiagnosticCode::InvalidPath)?;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|_| git::types::GitDiagnosticCode::DiffUnavailable)
}

#[tauri::command]
async fn git_mutation_preview(
    request: GitMutationPreviewRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationPreviewSnapshot, ()> {
    Ok(service.preview_mutation(request, &projects).await)
}

#[tauri::command]
async fn git_mutation_confirm(
    request: GitMutationConfirmRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationResultSnapshot, ()> {
    Ok(service.confirm_mutation(request, &projects).await)
}

#[tauri::command]
async fn git_mutation_recover(
    request: GitRecoveryRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationResultSnapshot, ()> {
    Ok(service.recover_mutation(request, &projects).await)
}

#[tauri::command]
async fn conversation_status(
    service: tauri::State<'_, ConversationService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn conversation_active(
    service: tauri::State<'_, ConversationService>,
) -> Result<ConversationRegistrySnapshot, ()> {
    Ok(service.active().await)
}

#[tauri::command]
async fn conversation_start(
    request: ConversationStartRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    integrations: tauri::State<'_, IntegrationControlService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let claimed =
        match attachment_service.claim(&request.project_id, &request.attachment_ids, &projects) {
            Ok(claimed) => claimed,
            Err(_) => {
                return Ok(ConversationSnapshot::unavailable(
                    ConversationDiagnosticCode::AttachmentUnavailable,
                ));
            }
        };
    let mentions = match integrations
        .resolve_mentions(&request.integration_entry_ids)
        .await
    {
        Ok(mentions) => mentions,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::IntegrationUnavailable,
            ))
        }
    };
    let snapshot = service
        .start_with_mentions(request, &projects, mentions, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_poll(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.poll(conversation_id, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_interrupt(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.interrupt(conversation_id, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_approval_decide(
    request: ConversationApprovalDecisionRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.decide_approval(request, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_sessions(
    request: codex::SessionListRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.sessions(request, &projects).await)
}

#[tauri::command]
async fn conversation_resume(
    request: ConversationContinueRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let project_id = match projects.conversation_reference(&request.conversation_id) {
        Ok(reference) => reference.project_id,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            ))
        }
    };
    let claimed = match attachment_service.claim(&project_id, &request.attachment_ids, &projects) {
        Ok(claimed) => claimed,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            ))
        }
    };
    let snapshot = service
        .resume_with_attachments(request, &projects, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_fork(
    request: ConversationContinueRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let project_id = match projects.conversation_reference(&request.conversation_id) {
        Ok(reference) => reference.project_id,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            ))
        }
    };
    let claimed = match attachment_service.claim(&project_id, &request.attachment_ids, &projects) {
        Ok(claimed) => claimed,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            ))
        }
    };
    let snapshot = service
        .fork_with_attachments(request, &projects, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

fn retain_claimed_attachments(
    service: &ConversationAttachmentService,
    claimed: ClaimedConversationAttachments,
    snapshot: &ConversationSnapshot,
) -> Result<(), attachment::types::ConversationAttachmentDiagnosticCode> {
    if !snapshot.turn_in_flight() {
        return Ok(());
    }
    let conversation_id = snapshot
        .conversation_id
        .as_deref()
        .ok_or(attachment::types::ConversationAttachmentDiagnosticCode::InvalidRequest)?;
    service.retain_for_conversation(conversation_id, claimed)
}

fn cleanup_terminal_attachments(
    service: &ConversationAttachmentService,
    snapshot: &ConversationSnapshot,
) {
    if snapshot.turn_in_flight() {
        return;
    }
    if let Some(conversation_id) = snapshot.conversation_id.as_deref() {
        let _ = service.cleanup_conversation(conversation_id);
    }
}

#[tauri::command]
async fn conversation_archive(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.archive(conversation_id, &projects).await)
}

#[tauri::command]
async fn conversation_restore(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.restore(conversation_id, &projects).await)
}

#[tauri::command]
async fn terminal_status(
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalRegistrySnapshot, ()> {
    Ok(service.status(&projects).await)
}

#[tauri::command]
async fn terminal_start(
    request: TerminalStartRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.start(request, &projects).await)
}

#[tauri::command]
async fn terminal_poll(
    request: TerminalPollRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.poll(request, &projects).await)
}

#[tauri::command]
async fn terminal_write(
    request: TerminalWriteRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.write(request, &projects).await)
}

#[tauri::command]
async fn terminal_resize(
    request: TerminalResizeRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.resize(request, &projects).await)
}

#[tauri::command]
async fn terminal_close(
    request: TerminalCloseRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalRegistrySnapshot, ()> {
    Ok(service.close(request.terminal_id, &projects).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(CodexRuntimeService::default())
        .manage(CodexAuthService::default())
        .manage(IntegrationCatalogService::default())
        .manage(IntegrationControlService::default())
        .manage(IntegrationMutationService::default())
        .manage(ConversationService::default())
        .manage(DesktopNotificationService::default())
        .manage(GitService::default())
        .manage(FilePreviewService::default())
        .manage(TerminalService::default())
        .setup(|app| {
            match app.path().app_data_dir() {
                Ok(directory) => {
                    app.manage(ProjectService::open(&directory.join("metadata.sqlite3")));
                    app.manage(WorktreeService::open(&directory.join("worktrees")));
                    app.manage(ConversationAttachmentService::open(
                        directory.join("conversation-attachments"),
                    ));
                }
                Err(_) => {
                    app.manage(ProjectService::unavailable());
                    app.manage(WorktreeService::unavailable());
                    app.manage(ConversationAttachmentService::unavailable());
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_bootstrap,
            codex_runtime_probe,
            integration_catalog_read,
            integration_catalog_refresh,
            integration_control_preview,
            integration_control_confirm,
            integration_control_open_browser,
            integration_control_status,
            integration_mutation_preview,
            integration_mutation_confirm,
            codex_auth_status,
            codex_auth_refresh,
            codex_auth_start,
            codex_auth_cancel,
            codex_auth_logout,
            codex_auth_open_browser,
            project_workspace_status,
            project_pick_directory,
            project_pick_relink,
            project_confirm_attachment,
            project_cancel_attachment,
            project_detach,
            project_archive,
            project_preflight,
            file_preview_pick,
            file_preview_open,
            file_preview_cancel,
            conversation_attachment_status,
            conversation_attachment_pick,
            conversation_attachment_stage_drop,
            conversation_attachment_cancel,
            worktree_status,
            worktree_create_preview,
            worktree_recover_preview,
            worktree_remove_preview,
            worktree_pick_attach,
            worktree_confirm,
            worktree_cancel,
            git_status,
            git_diff,
            git_open_file,
            git_mutation_preview,
            git_mutation_confirm,
            git_mutation_recover,
            conversation_status,
            conversation_active,
            conversation_notify,
            conversation_start,
            conversation_poll,
            conversation_interrupt,
            conversation_approval_decide,
            conversation_sessions,
            conversation_resume,
            conversation_fork,
            conversation_archive,
            conversation_restore,
            terminal_status,
            terminal_start,
            terminal_poll,
            terminal_write,
            terminal_resize,
            terminal_close
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}
