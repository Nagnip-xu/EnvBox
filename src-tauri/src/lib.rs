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

fn require_snapshot(description: &str) -> Result<Snapshot, String> {
    snapshot::create_snapshot(description).map_err(|e| format!("安全快照创建失败，操作已取消: {e}"))
}

fn command<T>(result: Result<T, String>) -> CommandResult<T> {
    result.map_err(AppError::from)
}

// ── 变量读写 ──────────────────────────────────────────────
#[tauri::command]
fn list_env_vars() -> Vec<EnvVar> {
    env_registry::list_env_vars()
}

#[tauri::command]
fn set_env_var(scope: String, name: String, value: String) -> CommandResult<()> {
    require_snapshot(&format!("修改 {} 变量 {} 前", scope, name))?;
    env_registry::set_env_var(&scope, &name, &value)?;
    win::broadcast_env_change();
    Ok(())
}

#[tauri::command]
fn delete_env_var(scope: String, name: String) -> CommandResult<()> {
    require_snapshot(&format!("删除 {} 变量 {} 前", scope, name))?;
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
fn save_path(scope: String, entries: Vec<String>) -> CommandResult<()> {
    require_snapshot(&format!("保存 {} PATH 前", scope))?;
    command(path_manager::save_path(&scope, entries))
}

#[tauri::command]
fn clean_invalid_path(scope: String) -> CommandResult<usize> {
    require_snapshot(&format!("清理 {} 无效 PATH 前", scope))?;
    command(path_manager::clean_invalid(&scope))
}

#[tauri::command]
fn dedupe_path(scope: String) -> CommandResult<usize> {
    require_snapshot(&format!("PATH 去重 {} 前", scope))?;
    command(path_manager::dedupe(&scope))
}

#[tauri::command]
fn scan_orphans() -> Vec<PathEntry> {
    path_manager::scan_orphans()
}

// ── SDK ──────────────────────────────────────────────────
#[tauri::command]
async fn scan_sdks() -> CommandResult<Vec<SdkVersion>> {
    tauri::async_runtime::spawn_blocking(sdk_scanner::scan_all)
        .await
        .map_err(|error| AppError::from(format!("SDK 扫描任务异常结束: {error}")))
}

#[tauri::command]
fn switch_sdk(kind: String, home: String) -> CommandResult<String> {
    require_snapshot(&format!("切换 {} 到 {} 前", kind, home))?;
    command(sdk_scanner::switch_sdk(&kind, &home))
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
) -> CommandResult<String> {
    let backup = require_snapshot(&format!("安装 {} {} {} 前", kind, distro, version))?;
    command(installer::install_sdk(
        app, &kind, &distro, &version, &engine, &location, &backup.id,
    ))
}

#[tauri::command]
fn uninstall_sdk(app: AppHandle, kind: String, home: String) -> CommandResult<String> {
    require_snapshot(&format!("卸载 {} ({}) 前", kind, home))?;
    command(installer::uninstall_sdk(app, &kind, &home))
}

#[tauri::command]
fn cancel_job(job_id: String) -> CommandResult<bool> {
    command(installer::cancel_job(&job_id))
}

#[tauri::command]
fn list_managed_installs() -> CommandResult<Vec<ManagedInstall>> {
    command(installer::list_managed_installs())
}

// ── 快照 / 审计 ───────────────────────────────────────────
#[tauri::command]
fn create_snapshot(description: String) -> CommandResult<Snapshot> {
    command(snapshot::create_snapshot(&description))
}

#[tauri::command]
fn list_snapshots() -> CommandResult<Vec<Snapshot>> {
    command(snapshot::list_snapshots())
}

#[tauri::command]
fn preview_snapshot_restore(id: String) -> CommandResult<SnapshotPreview> {
    command(snapshot::preview_restore(&id))
}

#[tauri::command]
fn restore_snapshot(id: String) -> CommandResult<()> {
    command(snapshot::restore_snapshot(&id))
}

#[tauri::command]
fn restore_snapshot_selected(id: String, selections: Vec<SnapshotSelection>) -> CommandResult<()> {
    command(snapshot::restore_snapshot_selected(&id, selections))
}

#[tauri::command]
fn delete_snapshot(id: String) -> CommandResult<()> {
    command(snapshot::delete_snapshot(&id))
}

#[tauri::command]
fn prune_snapshots(days: u64) -> CommandResult<usize> {
    command(snapshot::prune_snapshots(days))
}

#[tauri::command]
fn list_audit() -> CommandResult<Vec<AuditEntry>> {
    command(snapshot::list_audit())
}

// ── 其它 ─────────────────────────────────────────────────
#[tauri::command]
fn open_terminal_with(kind: String, home: String) -> CommandResult<()> {
    command(misc::open_terminal_with(&kind, &home))
}

#[tauri::command]
fn health_check() -> HealthReport {
    misc::health_check()
}

#[tauri::command]
fn inspect_project(path: String) -> CommandResult<ProjectInspection> {
    command(misc::inspect_project(&path))
}

#[tauri::command]
fn export_vars(path: String) -> CommandResult<()> {
    command(misc::export_vars(&path))
}

#[tauri::command]
fn import_vars(path: String) -> CommandResult<usize> {
    let backup = require_snapshot("导入环境变量前")?;
    match misc::import_vars(&path) {
        Ok(count) => Ok(count),
        Err(error) => {
            let rollback = match snapshot::restore_snapshot(&backup.id) {
                Ok(()) => "已自动恢复导入前状态".to_string(),
                Err(rollback) => format!("自动回滚失败: {rollback}"),
            };
            Err(AppError::from(format!("导入失败: {error}；{rollback}")))
        }
    }
}

#[tauri::command]
fn import_vars_selected(path: String, selections: Vec<SnapshotSelection>) -> CommandResult<usize> {
    let backup = require_snapshot("选择性导入环境变量前")?;
    match misc::import_vars_selected(&path, selections) {
        Ok(count) => Ok(count),
        Err(error) => {
            let rollback = match snapshot::restore_snapshot(&backup.id) {
                Ok(()) => "已自动恢复导入前状态".to_string(),
                Err(rollback) => format!("自动回滚失败: {rollback}"),
            };
            Err(AppError::from(format!("导入失败: {error}；{rollback}")))
        }
    }
}

#[tauri::command]
fn preview_import(path: String) -> CommandResult<ImportPreview> {
    command(misc::preview_import(&path))
}

#[tauri::command]
fn is_elevated() -> bool {
    win::is_elevated()
}

#[tauri::command]
fn relaunch_as_admin(app: AppHandle) -> CommandResult<bool> {
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
            cancel_job,
            list_managed_installs,
            create_snapshot,
            list_snapshots,
            preview_snapshot_restore,
            restore_snapshot,
            restore_snapshot_selected,
            delete_snapshot,
            prune_snapshots,
            list_audit,
            open_terminal_with,
            health_check,
            inspect_project,
            export_vars,
            preview_import,
            import_vars,
            import_vars_selected,
            is_elevated,
            relaunch_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("error while running EnvBox");
}
