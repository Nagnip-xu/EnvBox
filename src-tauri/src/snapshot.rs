use crate::env_registry;
use crate::models::{AuditEntry, Snapshot, SnapshotChange, SnapshotPreview};
use crate::win;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotFull {
    id: String,
    created_at: String,
    description: String,
    system: BTreeMap<String, String>,
    user: BTreeMap<String, String>,
}

fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("EnvBox")
}

fn snap_dir() -> PathBuf {
    let d = data_dir().join("snapshots");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn scope_map(scope: &str) -> BTreeMap<String, String> {
    env_registry::list_env_vars()
        .into_iter()
        .filter(|v| v.scope == scope)
        .map(|v| (v.name, v.value))
        .collect()
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
    let file = snap_dir().join(format!("{id}.json"));
    let text = std::fs::read_to_string(&file).map_err(|e| format!("找不到快照: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("快照文件损坏或格式不兼容: {e}"))
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
    let current_user = scope_map("user");
    let current_system = scope_map("system");
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
    let base_id = win::timestamp_id();
    let created_at = win::now_string();
    let system = scope_map("system");
    let user = scope_map("user");
    let mut saved_id = None;
    for suffix in 0..100 {
        let id = if suffix == 0 {
            base_id.clone()
        } else {
            format!("{base_id}-{suffix}")
        };
        let full = SnapshotFull {
            id: id.clone(),
            created_at: created_at.clone(),
            description: description.to_string(),
            system: system.clone(),
            user: user.clone(),
        };
        let json = serde_json::to_string_pretty(&full).map_err(|e| e.to_string())?;
        let file = snap_dir().join(format!("{id}.json"));
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

pub fn list_snapshots() -> Vec<Snapshot> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(snap_dir()) {
        for e in rd.flatten() {
            if e.path().extension().map(|x| x == "json").unwrap_or(false) {
                if let Ok(text) = std::fs::read_to_string(e.path()) {
                    if let Ok(full) = serde_json::from_str::<SnapshotFull>(&text) {
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
    out
}

pub fn delete_snapshot(id: &str) -> Result<(), String> {
    validate_snapshot_id(id)?;
    let file = snap_dir().join(format!("{}.json", id));
    std::fs::remove_file(&file).map_err(|e| format!("删除快照失败: {}", e))?;
    audit("snapshot_delete", id);
    Ok(())
}

/// 删除超过 `days` 天的快照（基于文件修改时间）。days 为 0 表示永不删除。
/// 返回删除的数量。
pub fn prune_snapshots(days: u64) -> usize {
    if days == 0 {
        return 0;
    }
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(days.saturating_mul(86400)));
    let cutoff = match cutoff {
        Some(c) => c,
        None => return 0,
    };
    let mut removed = 0;
    if let Ok(rd) = std::fs::read_dir(snap_dir()) {
        for e in rd.flatten() {
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
    }
    if removed > 0 {
        audit(
            "snapshot_prune",
            &format!("清理 {} 个超过 {} 天的快照", removed, days),
        );
    }
    removed
}

pub fn restore_snapshot(id: &str) -> Result<(), String> {
    let full = load_snapshot(id)?;

    let current_sys = scope_map("system");
    if current_sys != full.system && !win::is_elevated() {
        return Err(
            "该快照包含系统变量差异，请先以管理员身份运行后再恢复；尚未修改任何变量".into(),
        );
    }

    // 所有前置校验通过后再保存当前状态，避免无效恢复制造多余快照。
    create_snapshot(&format!("恢复快照 {id} 前"))
        .map_err(|e| format!("恢复前安全快照创建失败，操作已取消: {e}"))?;

    let current_user = scope_map("user");
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

fn replace_scope(scope: &str, target: &BTreeMap<String, String>) -> Result<(), String> {
    let current = scope_map(scope);
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
        let file = data_dir().join("audit.jsonl");
        let _ = std::fs::create_dir_all(data_dir());
        if std::fs::metadata(&file)
            .map(|metadata| metadata.len() > 5 * 1024 * 1024)
            .unwrap_or(false)
        {
            let rotated = data_dir().join("audit.previous.jsonl");
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

pub fn list_audit() -> Vec<AuditEntry> {
    let file = data_dir().join("audit.jsonl");
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
    out
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
}
