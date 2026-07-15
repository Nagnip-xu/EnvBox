//! 只读 Smoke 测试：验证核心模块能在本机正常调用（不修改系统状态）

#[test]
fn smoke_list_env_vars() {
    let vars = envbox_lib::env_registry::list_env_vars();
    // 正常 Windows 机器至少应有 Path 等变量
    assert!(!vars.is_empty(), "环境变量列表不应为空");
}

#[test]
fn smoke_get_path_entries() {
    let entries = envbox_lib::path_manager::get_path_entries();
    assert!(!entries.is_empty(), "PATH 条目不应为空");
}

#[test]
fn smoke_health_check() {
    let report = envbox_lib::misc::health_check();
    assert!(report.total_vars > 0);
    assert!(!report.issues.is_empty());
}

#[test]
fn smoke_engine_status() {
    let status = envbox_lib::installer::engine_status();
    // 只验证结构可读，winget/scoop 因环境而异
    let _ = (status.winget, status.scoop, status.elevated);
}

#[test]
fn smoke_list_snapshots() {
    let _ = envbox_lib::snapshot::list_snapshots().expect("快照目录应可读取");
}

#[test]
fn smoke_scan_sdks() {
    let sdks = envbox_lib::sdk_scanner::scan_all();
    // 不断言数量：不同机器安装不同
    let _ = sdks.len();
}
