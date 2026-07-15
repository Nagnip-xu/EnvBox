use crate::env_registry;
use crate::models::{HealthReport, ImportPreview, ProjectInspection, ProjectVersionHint};
use crate::path_manager;
use crate::win;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

fn read_small_text(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub fn inspect_project(path: &str) -> Result<ProjectInspection, String> {
    let root = Path::new(path);
    if !root.is_absolute() || !root.is_dir() {
        return Err("项目目录不存在或不是绝对路径".into());
    }
    let canonical = root
        .canonicalize()
        .map_err(|e| format!("无法访问项目目录: {e}"))?;
    let mut hints = Vec::new();
    let markers = [
        (".nvmrc", "Node.js"),
        (".node-version", "Node.js"),
        (".python-version", "Python"),
        (".java-version", "JDK"),
        (".ruby-version", "Ruby"),
    ];
    for (file, tool) in markers {
        if let Some(version) = read_small_text(&canonical.join(file)) {
            let version = version.trim();
            if !version.is_empty() {
                hints.push(ProjectVersionHint {
                    tool: tool.into(),
                    version: version.into(),
                    source: file.into(),
                });
            }
        }
    }

    if let Some(text) = read_small_text(&canonical.join(".tool-versions")) {
        for line in text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let mut parts = line.split_whitespace();
            if let (Some(tool), Some(version)) = (parts.next(), parts.next()) {
                hints.push(ProjectVersionHint {
                    tool: tool.into(),
                    version: version.into(),
                    source: ".tool-versions".into(),
                });
            }
        }
    }

    if let Some(text) = read_small_text(&canonical.join("global.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(version) = json
                .pointer("/sdk/version")
                .and_then(|value| value.as_str())
            {
                hints.push(ProjectVersionHint {
                    tool: ".NET SDK".into(),
                    version: version.into(),
                    source: "global.json".into(),
                });
            }
        }
    }

    if let Some(text) = read_small_text(&canonical.join("gradle/wrapper/gradle-wrapper.properties"))
    {
        if let Some(url) = text
            .lines()
            .find_map(|line| line.strip_prefix("distributionUrl="))
        {
            let version = url
                .split("gradle-")
                .nth(1)
                .and_then(|rest| rest.split('-').next())
                .unwrap_or("wrapper");
            hints.push(ProjectVersionHint {
                tool: "Gradle".into(),
                version: version.into(),
                source: "gradle-wrapper.properties".into(),
            });
        }
    }

    if canonical.join("mvnw.cmd").is_file() || canonical.join("mvnw").is_file() {
        hints.push(ProjectVersionHint {
            tool: "Maven".into(),
            version: "Wrapper".into(),
            source: "mvnw".into(),
        });
    }

    hints.sort_by(|a, b| a.tool.cmp(&b.tool).then(a.source.cmp(&b.source)));
    hints.dedup_by(|a, b| a.tool == b.tool && a.version == b.version && a.source == b.source);
    Ok(ProjectInspection {
        path: canonical.to_string_lossy().to_string(),
        hints,
    })
}

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
    cmd.args([
        "/K",
        &format!("echo [EnvBox] 临时终端: {} @ {} & echo.", kind, home),
    ]);
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
    let invalid: Vec<_> = paths.iter().filter(|e| e.safe_to_clean).collect();
    let dups: Vec<_> = paths.iter().filter(|e| e.duplicate).collect();
    let conflicts = vars.iter().filter(|v| v.conflicts_with.is_some()).count();
    let path_len = paths.iter().map(|e| e.raw.len() + 1).sum::<usize>();

    let mut issues = Vec::new();
    if !invalid.is_empty() {
        issues.push(format!(
            "发现 {} 个指向不存在目录的无效 PATH 条目",
            invalid.len()
        ));
    }
    if !dups.is_empty() {
        issues.push(format!("发现 {} 个重复的 PATH 条目", dups.len()));
    }
    if conflicts > 0 {
        issues.push(format!(
            "有 {} 个变量在系统级与用户级同名（用户级覆盖）",
            conflicts
        ));
    }
    if path_len > 2048 {
        issues.push(format!(
            "PATH 总长度 {} 偏大，注意 Windows 长度限制",
            path_len
        ));
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
#[serde(deny_unknown_fields)]
struct ExportBundle {
    #[serde(default = "export_version")]
    version: u32,
    system: BTreeMap<String, String>,
    user: BTreeMap<String, String>,
}

fn export_version() -> u32 {
    1
}

fn parse_import(path: &str) -> Result<ExportBundle, String> {
    const MAX_IMPORT_BYTES: u64 = 5 * 1024 * 1024;
    let metadata = std::fs::metadata(path).map_err(|e| format!("无法读取导入文件: {e}"))?;
    if metadata.len() > MAX_IMPORT_BYTES {
        return Err("导入文件超过 5 MiB 安全上限".into());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("无法读取导入文件: {e}"))?;
    let bundle: ExportBundle =
        serde_json::from_str(&text).map_err(|e| format!("解析失败: {}", e))?;
    if bundle.version != 1 {
        return Err(format!("不支持的导入文件版本: {}", bundle.version));
    }
    for (name, value) in bundle.user.iter().chain(bundle.system.iter()) {
        env_registry::validate_name(name)?;
        env_registry::validate_value(value)?;
    }
    Ok(bundle)
}

fn is_sensitive_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "API_KEY",
        "API-KEY",
        "PRIVATE_KEY",
        "CONNECTION_STRING",
        "CREDENTIAL",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

pub fn preview_import(path: &str) -> Result<ImportPreview, String> {
    let bundle = parse_import(path)?;
    let sensitive_count = bundle
        .user
        .keys()
        .chain(bundle.system.keys())
        .filter(|name| is_sensitive_name(name))
        .count();
    Ok(ImportPreview {
        user_count: bundle.user.len(),
        system_count: bundle.system.len(),
        sensitive_count,
        requires_elevation: !bundle.system.is_empty() && !win::is_elevated(),
    })
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
    let bundle = ExportBundle {
        version: export_version(),
        system,
        user,
    };

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
    let bundle = parse_import(path)?;
    if !bundle.system.is_empty() && !win::is_elevated() {
        return Err("导入文件包含系统变量，请先以管理员身份运行后再导入".into());
    }
    let mut count = 0;
    for (k, v) in &bundle.user {
        env_registry::set_env_var("user", k, v)
            .map_err(|e| format!("导入用户变量 {k} 失败: {e}"))?;
        count += 1;
    }
    for (k, v) in &bundle.system {
        env_registry::set_env_var("system", k, v)
            .map_err(|e| format!("导入系统变量 {k} 失败: {e}"))?;
        count += 1;
    }
    win::broadcast_env_change();
    crate::snapshot::audit("import", &format!("导入 {count} 个环境变量"));
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_sensitive_variable_names() {
        assert!(is_sensitive_name("GITHUB_TOKEN"));
        assert!(is_sensitive_name("database_password"));
        assert!(is_sensitive_name("MY_API_KEY"));
        assert!(!is_sensitive_name("JAVA_HOME"));
    }

    #[test]
    fn small_text_reader_rejects_missing_files() {
        assert!(read_small_text(Path::new("definitely-missing-envbox-project-file")).is_none());
    }
}
