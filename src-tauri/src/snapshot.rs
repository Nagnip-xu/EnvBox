use crate::env_registry;
use crate::models::{AuditEntry, Snapshot, SnapshotChange, SnapshotPreview, SnapshotSelection};
use crate::win;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

fn snapshot_schema_version() -> u32 {
    SNAPSHOT_SCHEMA_VERSION
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotFull {
    #[serde(default = "snapshot_schema_version")]
    schema_version: u32,
    #[serde(default)]
    app_version: String,
    id: String,
    created_at: String,
    description: String,
    system: BTreeMap<String, String>,
    user: BTreeMap<String, String>,
}

pub fn data_dir() -> Result<PathBuf, String> {
    let base = std::env::var("APPDATA")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(|value| PathBuf::from(value).join("AppData").join("Roaming"))
        })
        .ok_or_else(|| {
            "无法确定 Windows 应用数据目录（APPDATA/USERPROFILE 均不可用）".to_string()
        })?;
    Ok(base.join("EnvBox"))
}

fn ensure_directory(directory: &Path, label: &str) -> Result<(), String> {
    match std::fs::create_dir_all(directory) {
        Ok(()) => Ok(()),
        // Windows 上两个并发的初始化操作可能同时首次创建目录。若另一方已成功，
        // CreateDirectoryW 会返回 ERROR_ALREADY_EXISTS；重新确认类型即可安全继续。
        Err(_) if directory.is_dir() => Ok(()),
        Err(_) if directory.exists() => Err(format!(
            "{label}路径已存在，但不是目录: {}",
            directory.display()
        )),
        Err(error) => Err(format!("无法创建{label}: {error}")),
    }
}

pub(crate) fn ensure_data_dir() -> Result<PathBuf, String> {
    let directory = data_dir()?;
    ensure_directory(&directory, "应用数据目录")?;
    Ok(directory)
}

fn snap_dir() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("snapshots"))
}

fn ensure_snap_dir() -> Result<PathBuf, String> {
    let directory = ensure_data_dir()?.join("snapshots");
    ensure_directory(&directory, "快照目录")?;
    Ok(directory)
}

fn existing_snap_dir() -> Result<Option<PathBuf>, String> {
    let directory = snap_dir()?;
    match std::fs::metadata(&directory) {
        Ok(metadata) if metadata.is_dir() => Ok(Some(directory)),
        Ok(_) => Err(format!(
            "快照目录路径已存在，但不是目录: {}",
            directory.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("无法访问快照目录 {}: {error}", directory.display())),
    }
}

#[cfg(windows)]
fn secure_data_directory() -> Result<(), String> {
    static ACL_RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    ACL_RESULT
        .get_or_init(|| {
            use std::os::windows::process::CommandExt;
            use std::process::{Command, Stdio};

            let directory = ensure_data_dir()?;

            let mut whoami = Command::new("whoami.exe");
            whoami.args(["/user", "/fo", "csv", "/nh"]);
            whoami.creation_flags(0x08000000);
            let output = whoami
                .output()
                .map_err(|error| format!("无法查询当前用户 SID: {error}"))?;
            if !output.status.success() {
                return Err("查询当前用户 SID 失败".into());
            }
            let text = String::from_utf8_lossy(&output.stdout);
            let sid = text
                .split(|character: char| {
                    character == ',' || character == '"' || character.is_whitespace()
                })
                .find(|part| part.starts_with("S-1-"))
                .ok_or_else(|| "无法解析当前用户 SID".to_string())?;

            let grants = [
                format!("*{sid}:(OI)(CI)F"),
                "*S-1-5-18:(OI)(CI)F".to_string(),
                "*S-1-5-32-544:(OI)(CI)F".to_string(),
            ];
            let mut icacls = Command::new("icacls.exe");
            icacls
                .arg(&directory)
                .arg("/inheritance:r")
                .arg("/grant:r")
                .args(grants)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            icacls.creation_flags(0x08000000);
            let status = icacls
                .status()
                .map_err(|error| format!("无法设置快照目录权限: {error}"))?;
            if !status.success() {
                return Err("设置快照目录私有权限失败".into());
            }
            Ok(())
        })
        .clone()
}

#[cfg(not(windows))]
fn secure_data_directory() -> Result<(), String> {
    Ok(())
}

fn scope_map(scope: &str) -> Result<BTreeMap<String, String>, String> {
    env_registry::scope_values_strict(scope)
}

fn validate_snapshot_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("无效的快照 ID".into());
    }
    Ok(())
}

fn load_snapshot(id: &str) -> Result<SnapshotFull, String> {
    validate_snapshot_id(id)?;
    let file = snap_dir()?.join(format!("{id}.json"));
    let text = std::fs::read_to_string(&file).map_err(|e| format!("找不到快照: {e}"))?;
    let full: SnapshotFull =
        serde_json::from_str(&text).map_err(|e| format!("快照文件损坏或格式不兼容: {e}"))?;
    validate_loaded_snapshot(&full, id)?;
    Ok(full)
}

fn validate_loaded_snapshot(full: &SnapshotFull, expected_id: &str) -> Result<(), String> {
    if full.schema_version != SNAPSHOT_SCHEMA_VERSION {
        return Err(format!(
            "不支持的快照 schema 版本: {}（当前支持 {}）",
            full.schema_version, SNAPSHOT_SCHEMA_VERSION
        ));
    }
    if full.id != expected_id {
        return Err("快照文件名与内部 ID 不一致".into());
    }
    Ok(())
}

fn sensitive_name(name: &str) -> bool {
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

fn normalized(map: &BTreeMap<String, String>) -> BTreeMap<String, (String, String)> {
    map.iter()
        .map(|(name, value)| (name.to_ascii_lowercase(), (name.clone(), value.clone())))
        .collect()
}

fn diff_scope(
    scope: &str,
    current: &BTreeMap<String, String>,
    target: &BTreeMap<String, String>,
) -> Vec<SnapshotChange> {
    let current = normalized(current);
    let target = normalized(target);
    let mut keys = current
        .keys()
        .chain(target.keys())
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter_map(|key| {
            let before = current.get(&key);
            let after = target.get(&key);
            if before.map(|(_, value)| value) == after.map(|(_, value)| value) {
                return None;
            }
            let name = after
                .or(before)
                .map(|(name, _)| name.clone())
                .unwrap_or(key);
            let kind = match (before, after) {
                (None, Some(_)) => "add",
                (Some(_), None) => "delete",
                _ => "modify",
            };
            Some(SnapshotChange {
                scope: scope.into(),
                sensitive: sensitive_name(&name),
                name,
                kind: kind.into(),
                before: before.map(|(_, value)| value.clone()),
                after: after.map(|(_, value)| value.clone()),
            })
        })
        .collect()
}

pub fn preview_restore(id: &str) -> Result<SnapshotPreview, String> {
    let full = load_snapshot(id)?;
    let current_user = scope_map("user")?;
    let current_system = scope_map("system")?;
    let mut changes = diff_scope("user", &current_user, &full.user);
    changes.extend(diff_scope("system", &current_system, &full.system));
    let user_changes = changes
        .iter()
        .filter(|change| change.scope == "user")
        .count();
    let system_changes = changes.len() - user_changes;
    Ok(SnapshotPreview {
        snapshot_id: full.id,
        description: full.description,
        created_at: full.created_at,
        requires_elevation: system_changes > 0 && !win::is_elevated(),
        changes,
        user_changes,
        system_changes,
    })
}

pub fn create_snapshot(description: &str) -> Result<Snapshot, String> {
    let description = description.trim();
    if description.is_empty() {
        return Err("快照说明不能为空".into());
    }
    if description.chars().count() > 200 || description.chars().any(char::is_control) {
        return Err("快照说明不能超过 200 个字符或包含控制字符".into());
    }
    secure_data_directory()?;
    let base_id = win::timestamp_id();
    let created_at = win::now_string();
    let system = scope_map("system")?;
    let user = scope_map("user")?;
    let mut saved_id = None;
    for suffix in 0..100 {
        let id = if suffix == 0 {
            base_id.clone()
        } else {
            format!("{base_id}-{suffix}")
        };
        let full = SnapshotFull {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            app_version: env!("CARGO_PKG_VERSION").into(),
            id: id.clone(),
            created_at: created_at.clone(),
            description: description.to_string(),
            system: system.clone(),
            user: user.clone(),
        };
        let json = serde_json::to_string_pretty(&full).map_err(|e| e.to_string())?;
        let file = ensure_snap_dir()?.join(format!("{id}.json"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
        {
            Ok(mut output) => {
                use std::io::Write;
                if let Err(error) = output
                    .write_all(json.as_bytes())
                    .and_then(|_| output.sync_all())
                {
                    drop(output);
                    let _ = std::fs::remove_file(&file);
                    return Err(format!("写入快照失败: {error}"));
                }
                saved_id = Some(id);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("创建快照文件失败: {error}")),
        }
    }
    let id = saved_id.ok_or_else(|| "无法生成唯一快照 ID".to_string())?;
    audit("snapshot", description);
    Ok(Snapshot {
        id,
        created_at,
        description: description.to_string(),
    })
}

pub fn list_snapshots() -> Result<Vec<Snapshot>, String> {
    let mut out = Vec::new();
    let Some(directory) = existing_snap_dir()? else {
        return Ok(out);
    };
    let entries =
        std::fs::read_dir(directory).map_err(|error| format!("无法读取快照目录: {error}"))?;
    for entry in entries.flatten() {
        if entry
            .path()
            .extension()
            .map(|x| x == "json")
            .unwrap_or(false)
        {
            if let Ok(text) = std::fs::read_to_string(entry.path()) {
                if let Ok(full) = serde_json::from_str::<SnapshotFull>(&text) {
                    if full.schema_version == SNAPSHOT_SCHEMA_VERSION {
                        out.push(Snapshot {
                            id: full.id,
                            created_at: full.created_at,
                            description: full.description,
                        });
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(out)
}

pub fn count_invalid_snapshots() -> Result<usize, String> {
    let mut invalid = 0;
    let Some(directory) = existing_snap_dir()? else {
        return Ok(invalid);
    };
    let entries =
        std::fs::read_dir(directory).map_err(|error| format!("无法读取快照目录: {error}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .map(|extension| extension == "json")
            .unwrap_or(false)
        {
            let id = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if load_snapshot(id).is_err() {
                invalid += 1;
            }
        }
    }
    Ok(invalid)
}

pub fn delete_snapshot(id: &str) -> Result<(), String> {
    validate_snapshot_id(id)?;
    let file = snap_dir()?.join(format!("{}.json", id));
    std::fs::remove_file(&file).map_err(|e| format!("删除快照失败: {}", e))?;
    audit("snapshot_delete", id);
    Ok(())
}

/// 删除超过 `days` 天的快照（基于文件修改时间）。days 为 0 表示永不删除。
/// 返回删除的数量。
pub fn prune_snapshots(days: u64) -> Result<usize, String> {
    if days == 0 {
        return Ok(0);
    }
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(days.saturating_mul(86400)));
    let cutoff = match cutoff {
        Some(c) => c,
        None => return Ok(0),
    };
    let mut removed = 0;
    let Some(directory) = existing_snap_dir()? else {
        return Ok(removed);
    };
    let entries =
        std::fs::read_dir(directory).map_err(|error| format!("无法读取快照目录: {error}"))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().map(|x| x == "json").unwrap_or(false) {
            if let Ok(meta) = e.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff && std::fs::remove_file(&p).is_ok() {
                        removed += 1;
                    }
                }
            }
        }
    }
    if removed > 0 {
        audit(
            "snapshot_prune",
            &format!("清理 {} 个超过 {} 天的快照", removed, days),
        );
    }
    Ok(removed)
}

pub fn restore_snapshot(id: &str) -> Result<(), String> {
    let full = load_snapshot(id)?;

    let current_sys = scope_map("system")?;
    if current_sys != full.system && !win::is_elevated() {
        return Err(
            "该快照包含系统变量差异，请先以管理员身份运行后再恢复；尚未修改任何变量".into(),
        );
    }

    // 所有前置校验通过后再保存当前状态，避免无效恢复制造多余快照。
    create_snapshot(&format!("恢复快照 {id} 前"))
        .map_err(|e| format!("恢复前安全快照创建失败，操作已取消: {e}"))?;

    let current_user = scope_map("user")?;
    let result = replace_scope("user", &full.user).and_then(|_| {
        if win::is_elevated() {
            replace_scope("system", &full.system)
        } else {
            Ok(())
        }
    });
    if let Err(error) = result {
        let mut rollback_errors = Vec::new();
        if let Err(rollback) = replace_scope("user", &current_user) {
            rollback_errors.push(format!("用户变量回滚失败: {rollback}"));
        }
        if win::is_elevated() {
            if let Err(rollback) = replace_scope("system", &current_sys) {
                rollback_errors.push(format!("系统变量回滚失败: {rollback}"));
            }
        }
        win::broadcast_env_change();
        let rollback = if rollback_errors.is_empty() {
            "已自动恢复执行前状态".to_string()
        } else {
            format!("自动回滚不完整：{}", rollback_errors.join("；"))
        };
        audit("restore_failed", &format!("{id}: {error}; {rollback}"));
        return Err(format!("恢复失败: {error}；{rollback}"));
    }

    win::broadcast_env_change();
    audit("restore", id);
    Ok(())
}

fn find_case_insensitive<'a>(
    map: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<(&'a String, &'a String)> {
    map.iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
}

fn selected_target(
    current: &BTreeMap<String, String>,
    snapshot: &BTreeMap<String, String>,
    selections: &[&SnapshotSelection],
) -> BTreeMap<String, String> {
    let mut target = current.clone();
    for selection in selections {
        let existing_key = target
            .keys()
            .find(|name| name.eq_ignore_ascii_case(&selection.name))
            .cloned();
        if let Some(key) = existing_key {
            target.remove(&key);
        }
        if let Some((name, value)) = find_case_insensitive(snapshot, &selection.name) {
            target.insert(name.clone(), value.clone());
        }
    }
    target
}

pub fn restore_snapshot_selected(
    id: &str,
    selections: Vec<SnapshotSelection>,
) -> Result<(), String> {
    if selections.is_empty() {
        return Err("请至少选择一个要恢复的变量".into());
    }
    if selections.len() > 2_048 {
        return Err("单次恢复的变量数量超过安全上限".into());
    }
    let mut seen = std::collections::HashSet::new();
    for selection in &selections {
        env_registry::validate_scope(&selection.scope)?;
        env_registry::validate_name(&selection.name)?;
        let key = format!(
            "{}:{}",
            selection.scope,
            selection.name.to_ascii_lowercase()
        );
        if !seen.insert(key) {
            return Err(format!(
                "重复的恢复项目: {} {}",
                selection.scope, selection.name
            ));
        }
    }

    let full = load_snapshot(id)?;
    let current_user = scope_map("user")?;
    let current_system = scope_map("system")?;
    let user_selections = selections
        .iter()
        .filter(|selection| selection.scope == "user")
        .collect::<Vec<_>>();
    let system_selections = selections
        .iter()
        .filter(|selection| selection.scope == "system")
        .collect::<Vec<_>>();
    if !system_selections.is_empty() && !win::is_elevated() {
        return Err("所选恢复项目包含系统变量，请先以管理员身份运行".into());
    }
    let target_user = selected_target(&current_user, &full.user, &user_selections);
    let target_system = selected_target(&current_system, &full.system, &system_selections);
    create_snapshot(&format!("选择性恢复快照 {id} 前"))
        .map_err(|error| format!("恢复前安全快照创建失败，操作已取消: {error}"))?;

    let result = (if user_selections.is_empty() {
        Ok(())
    } else {
        replace_scope("user", &target_user)
    })
    .and_then(|_| {
        if system_selections.is_empty() {
            Ok(())
        } else {
            replace_scope("system", &target_system)
        }
    });
    if let Err(error) = result {
        let mut rollback_errors = Vec::new();
        if !user_selections.is_empty() {
            if let Err(rollback) = replace_scope("user", &current_user) {
                rollback_errors.push(format!("用户变量回滚失败: {rollback}"));
            }
        }
        if !system_selections.is_empty() {
            if let Err(rollback) = replace_scope("system", &current_system) {
                rollback_errors.push(format!("系统变量回滚失败: {rollback}"));
            }
        }
        win::broadcast_env_change();
        let rollback = if rollback_errors.is_empty() {
            "已自动恢复执行前状态".to_string()
        } else {
            format!("自动回滚不完整：{}", rollback_errors.join("；"))
        };
        audit(
            "restore_selected_failed",
            &format!("{id}: {error}; {rollback}"),
        );
        return Err(format!("选择性恢复失败: {error}；{rollback}"));
    }
    win::broadcast_env_change();
    audit(
        "restore_selected",
        &format!("{id}: 恢复 {} 个变量", selections.len()),
    );
    Ok(())
}

fn replace_scope(scope: &str, target: &BTreeMap<String, String>) -> Result<(), String> {
    let current = scope_map(scope)?;
    for (name, value) in target {
        env_registry::set_env_var(scope, name, value)
            .map_err(|e| format!("写入 {scope} 变量 {name} 失败: {e}"))?;
    }
    let target_keys = target
        .keys()
        .map(|name| name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    for name in current.keys() {
        if !target_keys.contains(&name.to_ascii_lowercase()) {
            env_registry::delete_env_var(scope, name)
                .map_err(|e| format!("删除 {scope} 变量 {name} 失败: {e}"))?;
        }
    }
    Ok(())
}

pub fn audit(action: &str, detail: &str) {
    let entry = AuditEntry {
        time: win::now_string(),
        action: action.to_string(),
        detail: detail.to_string(),
    };
    if let Ok(line) = serde_json::to_string(&entry) {
        let Ok(directory) = data_dir() else {
            return;
        };
        let file = directory.join("audit.jsonl");
        let _ = std::fs::create_dir_all(&directory);
        if std::fs::metadata(&file)
            .map(|metadata| metadata.len() > 5 * 1024 * 1024)
            .unwrap_or(false)
        {
            let rotated = directory.join("audit.previous.jsonl");
            let _ = std::fs::remove_file(&rotated);
            let _ = std::fs::rename(&file, rotated);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
        {
            let _ = writeln!(f, "{}", line);
        }
    }
}

pub fn list_audit() -> Result<Vec<AuditEntry>, String> {
    let file = data_dir()?.join("audit.jsonl");
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string(file) {
        for line in text.lines() {
            if let Ok(e) = serde_json::from_str::<AuditEntry>(line) {
                out.push(e);
            }
        }
    }
    out.reverse();
    out.truncate(200);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_id_cannot_escape_data_directory() {
        assert!(validate_snapshot_id("snap-123456").is_ok());
        assert!(validate_snapshot_id("../audit").is_err());
        assert!(validate_snapshot_id("C:\\temp").is_err());
    }

    #[test]
    fn snapshot_diff_is_case_insensitive_and_classifies_changes() {
        let current = BTreeMap::from([
            ("Path".into(), "C:\\Old".into()),
            ("REMOVE_ME".into(), "1".into()),
        ]);
        let target = BTreeMap::from([
            ("PATH".into(), "C:\\New".into()),
            ("ADD_ME".into(), "2".into()),
        ]);
        let changes = diff_scope("user", &current, &target);
        assert_eq!(changes.len(), 3);
        assert!(changes.iter().any(|change| change.kind == "add"));
        assert!(changes.iter().any(|change| change.kind == "modify"));
        assert!(changes.iter().any(|change| change.kind == "delete"));
    }

    #[test]
    fn snapshot_description_is_bounded() {
        assert!(create_snapshot("").is_err());
        assert!(create_snapshot("line\nbreak").is_err());
    }

    #[test]
    fn selected_target_only_changes_selected_variables() {
        let current = BTreeMap::from([
            ("JAVA_HOME".into(), "17".into()),
            ("KEEP".into(), "current".into()),
        ]);
        let snapshot = BTreeMap::from([("java_home".into(), "21".into())]);
        let selection = SnapshotSelection {
            scope: "user".into(),
            name: "JAVA_HOME".into(),
        };
        let target = selected_target(&current, &snapshot, &[&selection]);
        assert_eq!(target.get("java_home"), Some(&"21".to_string()));
        assert_eq!(target.get("KEEP"), Some(&"current".to_string()));
    }

    #[test]
    fn legacy_snapshot_without_schema_remains_compatible() {
        let legacy = r#"{
            "id":"snap-legacy",
            "createdAt":"2026-07-16 00:00:00",
            "description":"legacy",
            "system":{},
            "user":{"JAVA_HOME":"C:\\\\Java"}
        }"#;
        let snapshot: SnapshotFull = serde_json::from_str(legacy).unwrap();
        assert_eq!(snapshot.schema_version, SNAPSHOT_SCHEMA_VERSION);
        assert!(snapshot.app_version.is_empty());
        assert!(validate_loaded_snapshot(&snapshot, "snap-legacy").is_ok());
    }

    #[test]
    fn snapshot_header_rejects_future_schema_and_id_mismatch() {
        let mut snapshot = SnapshotFull {
            schema_version: SNAPSHOT_SCHEMA_VERSION + 1,
            app_version: "future".into(),
            id: "snap-one".into(),
            created_at: "2026-07-16 00:00:00".into(),
            description: "future".into(),
            system: BTreeMap::new(),
            user: BTreeMap::new(),
        };
        assert!(validate_loaded_snapshot(&snapshot, "snap-one").is_err());

        snapshot.schema_version = SNAPSHOT_SCHEMA_VERSION;
        assert!(validate_loaded_snapshot(&snapshot, "snap-other").is_err());
    }
}
