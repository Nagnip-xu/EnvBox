use crate::env_registry;
use crate::models::PathEntry;
use crate::win;
use std::collections::HashSet;
use std::path::Path;

const MAX_PATH_VALUE_LEN: usize = 32_767;

pub fn validate_path_entries(scope: &str, entries: &[String]) -> Result<(), String> {
    env_registry::validate_scope(scope)?;
    for entry in entries {
        if entry.contains('\0') || entry.contains(';') || entry.chars().any(|c| c.is_control()) {
            return Err(format!("PATH 条目包含非法字符: {entry}"));
        }
    }
    let total = entries
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.encode_utf16().count() + 1)
        .sum::<usize>();
    if total > MAX_PATH_VALUE_LEN {
        return Err(format!(
            "PATH 总长度超过 Windows 上限（{MAX_PATH_VALUE_LEN} 个 UTF-16 字符）"
        ));
    }
    Ok(())
}

fn sdk_tag_for(dir: &str) -> Option<String> {
    let p = Path::new(dir);
    let has = |exe: &str| p.join(exe).exists();
    if has("java.exe") {
        Some("JDK".into())
    } else if has("node.exe") {
        Some("Node".into())
    } else if has("python.exe") {
        Some("Python".into())
    } else if has("go.exe") {
        Some("Go".into())
    } else if has("cargo.exe") || has("rustc.exe") {
        Some("Rust".into())
    } else {
        None
    }
}

fn raw_path_of(scope: &str) -> Vec<String> {
    env_registry::get_scope_value(scope, "Path")
        .unwrap_or_default()
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn get_path_entries() -> Vec<PathEntry> {
    let vars = env_registry::all_vars_map();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();

    for scope in ["system", "user"] {
        for raw in raw_path_of(scope) {
            let resolved = win::expand_vars(&raw, &vars);
            let key = resolved.to_lowercase().trim_end_matches('\\').to_string();
            let duplicate = !seen.insert(key);
            let exists = Path::new(&resolved).is_dir();
            let sdk_tag = if exists { sdk_tag_for(&resolved) } else { None };
            out.push(PathEntry {
                raw,
                resolved,
                scope: scope.into(),
                exists,
                duplicate,
                sdk_tag,
                enabled: true,
            });
        }
    }
    out
}

/// 用给定的原始条目列表覆盖某作用域的 PATH，并广播。
pub fn save_path(scope: &str, entries: Vec<String>) -> Result<(), String> {
    validate_path_entries(scope, &entries)?;
    let cleaned: Vec<String> = entries
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let joined = cleaned.join(";");
    env_registry::set_env_var(scope, "Path", &joined)?;
    win::broadcast_env_change();
    Ok(())
}

/// 删除某作用域中所有指向不存在目录的条目，返回删除数量。
pub fn clean_invalid(scope: &str) -> Result<usize, String> {
    env_registry::validate_scope(scope)?;
    let vars = env_registry::all_vars_map();
    let before = raw_path_of(scope);
    let after: Vec<String> = before
        .iter()
        .filter(|raw| {
            let resolved = win::expand_vars(raw, &vars);
            // 未解析变量、UNC/网络位置可能只是暂时不可访问，不自动删除。
            resolved.contains('%') || resolved.starts_with("\\\\") || Path::new(&resolved).is_dir()
        })
        .cloned()
        .collect();
    let removed = before.len() - after.len();
    if removed > 0 {
        save_path(scope, after)?;
    }
    Ok(removed)
}

/// 按解析后的路径 key 去重（保留首次出现），返回 (去重后列表, 删除数量)
pub fn dedupe_by_resolved_key(
    before: &[String],
    resolve: impl Fn(&str) -> String,
) -> (Vec<String>, usize) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut after = Vec::new();
    for raw in before {
        let key = resolve(raw)
            .to_lowercase()
            .trim_end_matches('\\')
            .to_string();
        if seen.insert(key) {
            after.push(raw.clone());
        }
    }
    let removed = before.len().saturating_sub(after.len());
    (after, removed)
}

/// 去除某作用域中重复条目（保留首次出现），返回删除数量。
pub fn dedupe(scope: &str) -> Result<usize, String> {
    env_registry::validate_scope(scope)?;
    let vars = env_registry::all_vars_map();
    let before = raw_path_of(scope);
    let (after, removed) = dedupe_by_resolved_key(&before, |raw| win::expand_vars(raw, &vars));
    if removed > 0 {
        save_path(scope, after)?;
    }
    Ok(removed)
}

/// 扫描两个作用域中指向已不存在目录的残留条目。
pub fn scan_orphans() -> Vec<PathEntry> {
    get_path_entries()
        .into_iter()
        .filter(|e| !e.exists)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_removes_case_insensitive_duplicates() {
        let before = vec![
            "C:\\Tools\\bin".into(),
            "c:\\tools\\bin".into(),
            "D:\\Other".into(),
        ];
        let (after, removed) = dedupe_by_resolved_key(&before, |s| s.to_string());
        assert_eq!(removed, 1);
        assert_eq!(after.len(), 2);
    }

    #[test]
    fn dedupe_trims_trailing_backslash() {
        let before = vec!["C:\\Tools\\bin\\".into(), "C:\\Tools\\bin".into()];
        let (after, removed) = dedupe_by_resolved_key(&before, |s| s.to_string());
        assert_eq!(removed, 1);
        assert_eq!(after.len(), 1);
    }

    #[test]
    fn validates_path_entries() {
        assert!(validate_path_entries("user", &["C:\\Tools\\bin".into()]).is_ok());
        assert!(validate_path_entries("process", &[]).is_err());
        assert!(validate_path_entries("user", &["C:\\A;C:\\B".into()]).is_err());
    }
}
