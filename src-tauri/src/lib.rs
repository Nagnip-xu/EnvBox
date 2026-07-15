pub mod env_registry;
pub mod installer;
pub mod misc;
pub mod models;
pub mod path_manager;
pub mod sdk_scanner;
pub mod snapshot;
mod tray;
mod win;

use models::*;
use tauri::AppHandle;

// ── 变量读写 ──────────────────────────────────────────────
#[tauri::command]
fn list_env_vars() -> Vec<EnvVar> {
    env_registry::list_env_vars()
}

#[tauri::command]
fn set_env_var(scope: String, name: String, value: String) -> Result<(), String> {
    snapshot::create_snapshot(&format!("修改 {} 变量 {} 前", scope, name)).ok();
    env_registry::set_env_var(&scope, &name, &value)?;
    win::broadcast_env_change();
    Ok(())
}

#[tauri::command]
fn delete_env_var(scope: String, name: String) -> Result<(), String> {
    snapshot::create_snapshot(&format!("删除 {} 变量 {} 前", scope, name)).ok();
    env_registry::delete_env_var(&scope, &name)?;
    win::broadcast_env_change();
    Ok(())
}

// ── PATH ─────────────────────────────────────────────────
#[tauri::command]
fn get_path_entries() -> Vec<PathEntry> {
    path_manager::get_path_entries()
}

#[tauri::command]
fn save_path(scope: String, entries: Vec<String>) -> Result<(), String> {
    snapshot::create_snapshot(&format!("保存 {} PATH 前", scope)).ok();
    path_manager::save_path(&scope, entries)
}

#[tauri::command]
fn clean_invalid_path(scope: String) -> Result<usize, String> {
    snapshot::create_snapshot(&format!("清理 {} 无效 PATH 前", scope)).ok();
    path_manager::clean_invalid(&scope)
}

#[tauri::command]
fn dedupe_path(scope: String) -> Result<usize, String> {
    snapshot::create_snapshot(&format!("PATH 去重 {} 前", scope)).ok();
    path_manager::dedupe(&scope)
}

#[tauri::command]
fn scan_orphans() -> Vec<PathEntry> {
    path_manager::scan_orphans()
}

// ── SDK ──────────────────────────────────────────────────
#[tauri::command]
async fn scan_sdks() -> Vec<SdkVersion> {
    tauri::async_runtime::spawn_blocking(sdk_scanner::scan_all)
        .await
        .unwrap_or_default()
}

#[tauri::command]
fn switch_sdk(kind: String, home: String) -> Result<String, String> {
    snapshot::create_snapshot(&format!("切换 {} 到 {} 前", kind, home)).ok();
    sdk_scanner::switch_sdk(&kind, &home)
}

// ── 下载安装 / 卸载 ───────────────────────────────────────
#[tauri::command]
fn list_installable(kind: String) -> Vec<InstallableVersion> {
    installer::catalog(&kind)
}

#[tauri::command]
fn engine_status() -> EngineStatus {
    installer::engine_status()
}

#[tauri::command]
fn install_sdk(
    app: AppHandle,
    kind: String,
    distro: String,
    version: String,
    engine: String,
    location: String,
) -> Result<String, String> {
    installer::install_sdk(app, &kind, &distro, &version, &engine, &location)
}

#[tauri::command]
fn uninstall_sdk(app: AppHandle, kind: String, home: String) -> Result<String, String> {
    snapshot::create_snapshot(&format!("卸载 {} ({}) 前", kind, home)).ok();
    installer::uninstall_sdk(app, &kind, &home)
}

// ── 快照 / 审计 ───────────────────────────────────────────
#[tauri::command]
fn create_snapshot(description: String) -> Result<Snapshot, String> {
    snapshot::create_snapshot(&description)
}

#[tauri::command]
fn list_snapshots() -> Vec<Snapshot> {
    snapshot::list_snapshots()
}

#[tauri::command]
fn restore_snapshot(id: String) -> Result<(), String> {
    snapshot::restore_snapshot(&id)
}

#[tauri::command]
fn delete_snapshot(id: String) -> Result<(), String> {
    snapshot::delete_snapshot(&id)
}

#[tauri::command]
fn prune_snapshots(days: u64) -> usize {
    snapshot::prune_snapshots(days)
}

#[tauri::command]
fn list_audit() -> Vec<AuditEntry> {
    snapshot::list_audit()
}

// ── 其它 ─────────────────────────────────────────────────
#[tauri::command]
fn open_terminal_with(kind: String, home: String) -> Result<(), String> {
    misc::open_terminal_with(&kind, &home)
}

#[tauri::command]
fn health_check() -> HealthReport {
    misc::health_check()
}

#[tauri::command]
fn export_vars(path: String) -> Result<(), String> {
    misc::export_vars(&path)
}

#[tauri::command]
fn import_vars(path: String) -> Result<usize, String> {
    misc::import_vars(&path)
}

#[tauri::command]
fn is_elevated() -> bool {
    win::is_elevated()
}

#[tauri::command]
fn relaunch_as_admin(app: AppHandle) -> Result<bool, String> {
    if win::relaunch_as_admin()? {
        // 新实例已启动，关闭当前非管理员窗口，避免双开
        app.exit(0);
    }
    Ok(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            tray::on_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            list_env_vars,
            set_env_var,
            delete_env_var,
            get_path_entries,
            save_path,
            clean_invalid_path,
            dedupe_path,
            scan_orphans,
            scan_sdks,
            switch_sdk,
            list_installable,
            engine_status,
            install_sdk,
            uninstall_sdk,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            delete_snapshot,
            prune_snapshots,
            list_audit,
            open_terminal_with,
            health_check,
            export_vars,
            import_vars,
            is_elevated,
            relaunch_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("error while running EnvBox");
}
