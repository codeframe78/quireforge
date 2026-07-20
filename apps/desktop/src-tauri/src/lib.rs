mod codex;
mod contract;
mod project;

use codex::{
    types::CodexRuntimeSnapshot, AuthLoginMethod, CodexAuthService, CodexAuthSnapshot,
    CodexRuntimeService, ConversationContinueRequest, ConversationService, ConversationSnapshot,
    ConversationStartRequest, SessionLifecycleSnapshot,
};
use contract::DesktopBootstrap;
use project::{
    types::{ProjectPreflightSnapshot, ProjectWorkspaceSnapshot},
    ProjectService,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

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
async fn conversation_status(
    service: tauri::State<'_, ConversationService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn conversation_start(
    request: ConversationStartRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.start(request, &projects).await)
}

#[tauri::command]
async fn conversation_poll(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.poll(conversation_id, &projects).await)
}

#[tauri::command]
async fn conversation_interrupt(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.interrupt(conversation_id, &projects).await)
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
) -> Result<ConversationSnapshot, ()> {
    Ok(service.resume(request, &projects).await)
}

#[tauri::command]
async fn conversation_fork(
    request: ConversationContinueRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.fork(request, &projects).await)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(CodexRuntimeService::default())
        .manage(CodexAuthService::default())
        .manage(ConversationService::default())
        .setup(|app| {
            let service = app
                .path()
                .app_data_dir()
                .map(|directory| ProjectService::open(&directory.join("metadata.sqlite3")))
                .unwrap_or_else(|_| ProjectService::unavailable());
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_bootstrap,
            codex_runtime_probe,
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
            conversation_status,
            conversation_start,
            conversation_poll,
            conversation_interrupt,
            conversation_sessions,
            conversation_resume,
            conversation_fork,
            conversation_archive,
            conversation_restore
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}
