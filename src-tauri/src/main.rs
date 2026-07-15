// Windows 下始终使用 GUI 子系统，避免「以管理员重启」时弹出黑色命令行窗口
#![cfg_attr(windows, windows_subsystem = "windows")]
fn main() {
    envbox_lib::run()
}
