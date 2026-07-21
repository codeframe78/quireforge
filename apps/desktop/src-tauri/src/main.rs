#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
fn configure_linux_vfs() {
    const GIO_USE_VFS: &str = "GIO_USE_VFS";

    // QuireForge consumes filesystem paths and does not require the optional
    // GVFS D-Bus daemon. Selecting GIO's local backend prevents GTK/WebKit
    // startup from warning when distributions intentionally mask that daemon.
    // Keep an explicit caller override available for diagnostics.
    if std::env::var_os(GIO_USE_VFS).is_none() {
        std::env::set_var(GIO_USE_VFS, "local");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    configure_linux_vfs();

    quireforge_lib::run();
}
