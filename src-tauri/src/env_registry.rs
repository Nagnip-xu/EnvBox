use crate::models::EnvVar;
use std::collections::HashMap;

#[cfg(windows)]
const SYSTEM_SUBKEY: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
#[cfg(windows)]
const USER_SUBKEY: &str = "Environment";

/// 读取某作用域下的原始变量（保留大小写的变量名 -> (值, 是否含%VAR%)）
#[cfg(windows)]
fn read_scope(scope: &str) -> HashMap<String, (String, bool)> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut map = HashMap::new();
    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        _ => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
    };
    if let Ok(key) = root.open_subkey(sub) {
        for (name, value) in key.enum_values().flatten() {
            let s = value.to_string();
            let expandable = value.vtype == REG_EXPAND_SZ || s.contains('%');
            map.insert(name, (s, expandable));
        }
    }
    map
}

#[cfg(not(windows))]
fn read_scope(_scope: &str) -> HashMap<String, (String, bool)> {
    HashMap::new()
}

/// 所有变量合并成一个大写键的映射，供 %VAR% 展开使用（用户覆盖系统）。
pub fn all_vars_map() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, (v, _)) in read_scope("system") {
        map.insert(k.to_uppercase(), v);
    }
    for (k, (v, _)) in read_scope("user") {
        map.insert(k.to_uppercase(), v);
    }
    map
}

pub fn get_scope_value(scope: &str, name: &str) -> Option<String> {
    read_scope(scope)
        .into_iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, (v, _))| v)
}

#[cfg(windows)]
pub fn list_env_vars() -> Vec<EnvVar> {
    let user = read_scope("user");
    let system = read_scope("system");
    let mut out = Vec::new();

    let mut sys_sorted: Vec<_> = system.iter().collect();
    sys_sorted.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    for (name, (value, expandable)) in sys_sorted {
        let conflicts = if user.keys().any(|k| k.eq_ignore_ascii_case(name)) {
            Some("user".to_string())
        } else {
            None
        };
        out.push(EnvVar {
            name: name.clone(),
            value: value.clone(),
            scope: "system".into(),
            is_expandable: *expandable,
            conflicts_with: conflicts,
        });
    }

    let mut user_sorted: Vec<_> = user.iter().collect();
    user_sorted.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    for (name, (value, expandable)) in user_sorted {
        let conflicts = if system.keys().any(|k| k.eq_ignore_ascii_case(name)) {
            Some("system".to_string())
        } else {
            None
        };
        out.push(EnvVar {
            name: name.clone(),
            value: value.clone(),
            scope: "user".into(),
            is_expandable: *expandable,
            conflicts_with: conflicts,
        });
    }
    out
}

#[cfg(not(windows))]
pub fn list_env_vars() -> Vec<EnvVar> {
    std::env::vars()
        .map(|(name, value)| EnvVar {
            name,
            value,
            scope: "process".into(),
            is_expandable: false,
            conflicts_with: None,
        })
        .collect()
}

#[cfg(windows)]
fn to_utf16_bytes(s: &str) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity((s.len() + 1) * 2);
    for u in s.encode_utf16() {
        v.extend_from_slice(&u.to_le_bytes());
    }
    v.extend_from_slice(&[0, 0]); // 结尾 NUL
    v
}

/// 写入变量。含 %VAR% 时写为 REG_EXPAND_SZ，否则 REG_SZ。
#[cfg(windows)]
pub fn set_env_var(scope: &str, name: &str, value: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::{RegKey, RegValue};

    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        _ => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
    };
    let key = root
        .open_subkey_with_flags(sub, KEY_SET_VALUE)
        .map_err(|e| format!("无法打开注册表({}): {}", scope, e))?;

    let vtype = if value.contains('%') {
        REG_EXPAND_SZ
    } else {
        REG_SZ
    };
    let rv = RegValue {
        vtype,
        bytes: to_utf16_bytes(value),
    };
    key.set_raw_value(name, &rv)
        .map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

#[cfg(not(windows))]
pub fn set_env_var(_scope: &str, _name: &str, _value: &str) -> Result<(), String> {
    Err("仅支持 Windows".into())
}

#[cfg(windows)]
pub fn delete_env_var(scope: &str, name: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        _ => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
    };
    let key = root
        .open_subkey_with_flags(sub, KEY_SET_VALUE)
        .map_err(|e| format!("无法打开注册表({}): {}", scope, e))?;
    key.delete_value(name)
        .map_err(|e| format!("删除失败: {}", e))?;
    Ok(())
}

#[cfg(not(windows))]
pub fn delete_env_var(_scope: &str, _name: &str) -> Result<(), String> {
    Err("仅支持 Windows".into())
}
