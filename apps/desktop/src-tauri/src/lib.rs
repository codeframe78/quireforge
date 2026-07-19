mod codex;
mod contract;

use codex::{
    types::CodexRuntimeSnapshot, AuthLoginMethod, CodexAuthService, CodexAuthSnapshot,
    CodexRuntimeService,
};
use contract::DesktopBootstrap;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(CodexRuntimeService::default())
        .manage(CodexAuthService::default())
        .invoke_handler(tauri::generate_handler![
            desktop_bootstrap,
            codex_runtime_probe,
            codex_auth_status,
            codex_auth_refresh,
            codex_auth_start,
            codex_auth_cancel,
            codex_auth_logout,
            codex_auth_open_browser
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}
