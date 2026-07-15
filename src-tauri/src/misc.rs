use crate::env_registry;
use crate::models::HealthReport;
use crate::path_manager;
use crate::win;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// 用指定 SDK 版本打开一个临时终端（仅该会话生效，不改全局）
#[cfg(windows)]
pub fn open_terminal_with(kind: &str, home: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let cur_path = std::env::var("PATH").unwrap_or_default();
    let home_trim = home.trim_end_matches('\\');
    let (prepend, extra_env): (Vec<String>, Vec<(String, String)>) =
        match crate::sdk_scanner::kind_spec(kind) {
            Some(spec) => {
                let prepend: Vec<String> = spec
                    .path_suffixes
                    .iter()
                    .map(|s| {
                        if s.is_empty() {
                            home_trim.to_string()
                        } else {
                            format!("{}\\{}", home_trim, s)
                        }
                    })
                    .collect();
                let extra = match spec.home_var {
                    Some(v) => vec![(v.to_string(), home.to_string())],
                    None => vec![],
                };
                (prepend, extra)
            }
            None => (vec![home.to_string()], vec![]),
        };
    let new_path = format!("{};{}", prepend.join(";"), cur_path);

    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/K", &format!("echo [EnvBox] 临时终端: {} @ {} & echo.", kind, home)]);
    cmd.env("PATH", new_path);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.creation_flags(0x00000010); // CREATE_NEW_CONSOLE
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn open_terminal_with(_kind: &str, _home: &str) -> Result<(), String> {
    Err("仅支持 Windows".into())
}

pub fn health_check() -> HealthReport {
    let vars = env_registry::list_env_vars();
    let paths = path_manager::get_path_entries();
    let invalid: Vec<_> = paths.iter().filter(|e| !e.exists).collect();
    let dups: Vec<_> = paths.iter().filter(|e| e.duplicate).collect();
    let conflicts = vars.iter().filter(|v| v.conflicts_with.is_some()).count();
    let path_len = paths.iter().map(|e| e.raw.len() + 1).sum::<usize>();

    let mut issues = Vec::new();
    if !invalid.is_empty() {
        issues.push(format!("发现 {} 个指向不存在目录的无效 PATH 条目", invalid.len()));
    }
    if !dups.is_empty() {
        issues.push(format!("发现 {} 个重复的 PATH 条目", dups.len()));
    }
    if conflicts > 0 {
        issues.push(format!("有 {} 个变量在系统级与用户级同名（用户级覆盖）", conflicts));
    }
    if path_len > 2048 {
        issues.push(format!("PATH 总长度 {} 偏大，注意 Windows 长度限制", path_len));
    }
    if issues.is_empty() {
        issues.push("未发现明显问题，环境很健康 ✨".into());
    }

    HealthReport {
        total_vars: vars.len(),
        invalid_paths: invalid.len(),
        duplicate_paths: dups.len(),
        conflicts,
        path_length: path_len,
        issues,
    }
}

#[derive(Serialize, Deserialize)]
struct ExportBundle {
    system: BTreeMap<String, String>,
    user: BTreeMap<String, String>,
}

pub fn export_vars(path: &str) -> Result<(), String> {
    let mut system = BTreeMap::new();
    let mut user = BTreeMap::new();
    for v in env_registry::list_env_vars() {
        match v.scope.as_str() {
            "system" => {
                system.insert(v.name, v.value);
            }
            "user" => {
                user.insert(v.name, v.value);
            }
            _ => {}
        }
    }
    let bundle = ExportBundle { system, user };

    if Path::new(path)
        .extension()
        .map(|e| e.eq_ignore_ascii_case("env"))
        .unwrap_or(false)
    {
        let mut text = String::new();
        for (k, v) in bundle.system.iter().chain(bundle.user.iter()) {
            text.push_str(&format!("{}={}\n", k, v));
        }
        std::fs::write(path, text).map_err(|e| e.to_string())?;
    } else {
        let json = serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 从 JSON 导入变量（写入用户作用域；系统作用域需已提权），返回导入条数
pub fn import_vars(path: &str) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let bundle: ExportBundle = serde_json::from_str(&text).map_err(|e| format!("解析失败: {}", e))?;
    let mut count = 0;
    for (k, v) in &bundle.user {
        if env_registry::set_env_var("user", k, v).is_ok() {
            count += 1;
        }
    }
    if win::is_elevated() {
        for (k, v) in &bundle.system {
            if env_registry::set_env_var("system", k, v).is_ok() {
                count += 1;
            }
        }
    }
    win::broadcast_env_change();
    Ok(count)
}
