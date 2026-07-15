use crate::env_registry;
use crate::models::{AuditEntry, Snapshot};
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

pub fn create_snapshot(description: &str) -> Result<Snapshot, String> {
    let id = win::timestamp_id();
    let created_at = win::now_string();
    let full = SnapshotFull {
        id: id.clone(),
        created_at: created_at.clone(),
        description: description.to_string(),
        system: scope_map("system"),
        user: scope_map("user"),
    };
    let file = snap_dir().join(format!("{}.json", id));
    let json = serde_json::to_string_pretty(&full).map_err(|e| e.to_string())?;
    std::fs::write(&file, json).map_err(|e| e.to_string())?;
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
        .checked_sub(std::time::Duration::from_secs(days * 86400));
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
        audit("snapshot_prune", &format!("清理 {} 个超过 {} 天的快照", removed, days));
    }
    removed
}

pub fn restore_snapshot(id: &str) -> Result<(), String> {
    let file = snap_dir().join(format!("{}.json", id));
    let text = std::fs::read_to_string(&file).map_err(|e| format!("找不到快照: {}", e))?;
    let full: SnapshotFull = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    // 恢复用户变量：写入快照中的所有值，删除快照中不存在的多余变量
    let current_user = scope_map("user");
    for (k, v) in &full.user {
        let _ = env_registry::set_env_var("user", k, v);
    }
    for k in current_user.keys() {
        if !full.user.contains_key(k) {
            let _ = env_registry::delete_env_var("user", k);
        }
    }

    // 系统变量仅在已提权时恢复
    if win::is_elevated() {
        let current_sys = scope_map("system");
        for (k, v) in &full.system {
            let _ = env_registry::set_env_var("system", k, v);
        }
        for k in current_sys.keys() {
            if !full.system.contains_key(k) {
                let _ = env_registry::delete_env_var("system", k);
            }
        }
    }

    win::broadcast_env_change();
    audit("restore", id);
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
