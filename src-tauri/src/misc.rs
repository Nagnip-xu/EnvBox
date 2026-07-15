use crate::env_registry;
use crate::models::{
    HealthReport, ImportPreview, ProjectInspection, ProjectVersionHint, SnapshotSelection,
};
use crate::path_manager;
use crate::win;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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
                    status: String::new(),
                    installed_home: None,
                    current_version: None,
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
                    status: String::new(),
                    installed_home: None,
                    current_version: None,
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
                    status: String::new(),
                    installed_home: None,
                    current_version: None,
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
                status: "wrapper".into(),
                installed_home: None,
                current_version: None,
            });
        }
    }

    if canonical.join("mvnw.cmd").is_file() || canonical.join("mvnw").is_file() {
        hints.push(ProjectVersionHint {
            tool: "Maven".into(),
            version: "Wrapper".into(),
            source: "mvnw".into(),
            status: "wrapper".into(),
            installed_home: None,
            current_version: None,
        });
    }

    let mut sdk_cache = HashMap::new();
    for hint in &mut hints {
        if hint.status == "wrapper" {
            continue;
        }
        let tool = hint.tool.to_ascii_lowercase();
        let kind = match tool.as_str() {
            "node.js" | "node" | "nodejs" => Some("node"),
            "python" => Some("python"),
            "jdk" | "java" => Some("jdk"),
            "ruby" => Some("ruby"),
            "go" | "golang" => Some("go"),
            "rust" => Some("rust"),
            ".net sdk" | "dotnet" => Some("dotnet"),
            "deno" => Some("deno"),
            "bun" => Some("bun"),
            _ => None,
        };
        let Some(kind) = kind else {
            hint.status = "declared".into();
            continue;
        };
        let versions = sdk_cache
            .entry(kind)
            .or_insert_with(|| crate::sdk_scanner::scan_kind(kind));
        hint.current_version = versions
            .iter()
            .find(|version| version.is_current)
            .map(|version| version.version.clone());
        let requirement = hint
            .version
            .trim()
            .trim_start_matches('v')
            .to_ascii_lowercase();
        if let Some(found) = versions.iter().find(|version| {
            version
                .version
                .trim_start_matches('v')
                .to_ascii_lowercase()
                .contains(&requirement)
        }) {
            hint.installed_home = Some(found.home.clone());
            hint.status = if found.is_current {
                "current"
            } else {
                "installed"
            }
            .into();
        } else {
            hint.status = "missing".into();
        }
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
    let unresolved_paths = paths
        .iter()
        .filter(|entry| entry.status == "unresolved")
        .count();
    let network_paths = paths
        .iter()
        .filter(|entry| entry.status == "networkUnavailable")
        .count();
    let (snapshot_issues, snapshot_check_error) = match crate::snapshot::count_invalid_snapshots() {
        Ok(count) => (count, None),
        Err(error) => (0, Some(error)),
    };
    let (incomplete_installs, install_check_error) =
        match crate::installer::incomplete_install_count() {
            Ok(count) => (count, None),
            Err(error) => (0, Some(error)),
        };
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
    if unresolved_paths > 0 {
        issues.push(format!(
            "有 {unresolved_paths} 个 PATH 条目包含未解析的环境变量"
        ));
    }
    if network_paths > 0 {
        issues.push(format!(
            "有 {network_paths} 个网络 PATH 当前不可访问（不会自动清理）"
        ));
    }
    if snapshot_issues > 0 {
        issues.push(format!("发现 {snapshot_issues} 个损坏或不兼容的快照文件"));
    }
    if incomplete_installs > 0 {
        issues.push(format!(
            "发现 {incomplete_installs} 个失败、取消或未完成的受管安装记录"
        ));
    }
    if let Some(error) = snapshot_check_error {
        issues.push(format!("无法检查快照完整性: {error}"));
    }
    if let Some(error) = install_check_error {
        issues.push(format!("无法检查受管安装记录: {error}"));
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
        unresolved_paths,
        network_paths,
        snapshot_issues,
        incomplete_installs,
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
    let current_user = env_registry::scope_values_strict("user")?;
    let current_system = env_registry::scope_values_strict("system")?;
    let mut changes = import_changes("user", &current_user, &bundle.user);
    changes.extend(import_changes("system", &current_system, &bundle.system));
    Ok(ImportPreview {
        user_count: bundle.user.len(),
        system_count: bundle.system.len(),
        sensitive_count,
        requires_elevation: !bundle.system.is_empty() && !win::is_elevated(),
        changes,
    })
}

fn import_changes(
    scope: &str,
    current: &BTreeMap<String, String>,
    incoming: &BTreeMap<String, String>,
) -> Vec<crate::models::SnapshotChange> {
    incoming
        .iter()
        .filter_map(|(name, after)| {
            let before = current
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.clone());
            if before.as_ref() == Some(after) {
                return None;
            }
            Some(crate::models::SnapshotChange {
                scope: scope.into(),
                name: name.clone(),
                kind: if before.is_some() { "modify" } else { "add" }.into(),
                before,
                after: Some(after.clone()),
                sensitive: is_sensitive_name(name),
            })
        })
        .collect()
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

#[derive(Debug, PartialEq, Eq)]
struct ResolvedImport {
    scope: String,
    name: String,
    value: String,
}

fn resolve_import_selections(
    bundle: &ExportBundle,
    selections: &[SnapshotSelection],
) -> Result<Vec<ResolvedImport>, String> {
    if selections.is_empty() {
        return Err("请至少选择一个要导入的变量".into());
    }
    if selections.len() > 2_048 {
        return Err("单次导入的变量数量超过安全上限".into());
    }
    let mut seen = std::collections::HashSet::new();
    let mut resolved = Vec::with_capacity(selections.len());
    for selection in selections {
        env_registry::validate_scope(&selection.scope)?;
        env_registry::validate_name(&selection.name)?;
        let key = format!(
            "{}:{}",
            selection.scope,
            selection.name.to_ascii_lowercase()
        );
        if !seen.insert(key) {
            return Err(format!(
                "重复的导入项目: {} {}",
                selection.scope, selection.name
            ));
        }
        let source = if selection.scope == "system" {
            &bundle.system
        } else {
            &bundle.user
        };
        let (name, value) = source
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&selection.name))
            .ok_or_else(|| format!("导入文件中不存在 {} {}", selection.scope, selection.name))?;
        resolved.push(ResolvedImport {
            scope: selection.scope.clone(),
            name: name.clone(),
            value: value.clone(),
        });
    }
    Ok(resolved)
}

pub fn import_vars_selected(
    path: &str,
    selections: Vec<SnapshotSelection>,
) -> Result<usize, String> {
    let bundle = parse_import(path)?;
    let resolved = resolve_import_selections(&bundle, &selections)?;
    if resolved.iter().any(|item| item.scope == "system") && !win::is_elevated() {
        return Err("所选导入项目包含系统变量，请先以管理员身份运行".into());
    }
    for item in &resolved {
        env_registry::set_env_var(&item.scope, &item.name, &item.value)
            .map_err(|error| format!("导入 {} 变量 {} 失败: {error}", item.scope, item.name))?;
    }
    win::broadcast_env_change();
    let count = resolved.len();
    crate::snapshot::audit("import_selected", &format!("导入 {count} 个环境变量"));
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

    #[test]
    fn selected_imports_are_resolved_from_the_file_case_insensitively() {
        let bundle = ExportBundle {
            version: export_version(),
            system: BTreeMap::from([("JAVA_HOME".into(), "C:\\Java\\21".into())]),
            user: BTreeMap::from([("Node_Home".into(), "C:\\Node".into())]),
        };
        let selections = vec![SnapshotSelection {
            scope: "user".into(),
            name: "node_home".into(),
        }];

        let resolved = resolve_import_selections(&bundle, &selections).unwrap();
        assert_eq!(
            resolved,
            vec![ResolvedImport {
                scope: "user".into(),
                name: "Node_Home".into(),
                value: "C:\\Node".into(),
            }]
        );
    }

    #[test]
    fn selected_imports_reject_duplicates_and_unknown_entries() {
        let bundle = ExportBundle {
            version: export_version(),
            system: BTreeMap::new(),
            user: BTreeMap::from([("JAVA_HOME".into(), "C:\\Java".into())]),
        };
        let duplicate = vec![
            SnapshotSelection {
                scope: "user".into(),
                name: "JAVA_HOME".into(),
            },
            SnapshotSelection {
                scope: "user".into(),
                name: "java_home".into(),
            },
        ];
        assert!(resolve_import_selections(&bundle, &duplicate).is_err());
        assert!(resolve_import_selections(
            &bundle,
            &[SnapshotSelection {
                scope: "user".into(),
                name: "MISSING".into(),
            }]
        )
        .is_err());
    }

    #[test]
    fn import_diff_omits_unchanged_values_and_masks_sensitive_names() {
        let current = BTreeMap::from([("TOKEN".into(), "old".into())]);
        let incoming = BTreeMap::from([
            ("token".into(), "new".into()),
            ("UNCHANGED".into(), "same".into()),
        ]);
        let current = current
            .into_iter()
            .chain([("UNCHANGED".into(), "same".into())])
            .collect();

        let changes = import_changes("user", &current, &incoming);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, "modify");
        assert!(changes[0].sensitive);
    }
}
