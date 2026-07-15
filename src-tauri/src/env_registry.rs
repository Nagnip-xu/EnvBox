use crate::models::EnvVar;
use std::collections::BTreeMap;
use std::collections::HashMap;

const MAX_ENV_NAME_LEN: usize = 255;
const MAX_ENV_VALUE_LEN: usize = 32_767;

pub fn validate_scope(scope: &str) -> Result<(), String> {
    match scope {
        "user" | "system" => Ok(()),
        _ => Err(format!("不支持的环境变量作用域: {scope}")),
    }
}

pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("环境变量名不能为空".into());
    }
    if trimmed.len() > MAX_ENV_NAME_LEN {
        return Err(format!("环境变量名不能超过 {MAX_ENV_NAME_LEN} 个字节"));
    }
    if trimmed != name
        || name.contains('=')
        || name.contains('\0')
        || name.chars().any(char::is_control)
    {
        return Err("环境变量名不能包含首尾空格、等号、NUL 或控制字符".into());
    }
    Ok(())
}

pub fn validate_value(value: &str) -> Result<(), String> {
    if value.contains('\0') {
        return Err("环境变量值不能包含 NUL 字符".into());
    }
    if value.encode_utf16().count() > MAX_ENV_VALUE_LEN {
        return Err(format!(
            "环境变量值超过 Windows 上限（{MAX_ENV_VALUE_LEN} 个 UTF-16 字符）"
        ));
    }
    Ok(())
}

#[cfg(windows)]
const SYSTEM_SUBKEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
#[cfg(windows)]
const USER_SUBKEY: &str = "Environment";

/// 读取某作用域下的原始变量（保留大小写的变量名 -> (值, 是否含%VAR%)）
#[cfg(windows)]
fn read_scope_strict(scope: &str) -> Result<HashMap<String, (String, bool)>, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    validate_scope(scope)?;
    let mut map = HashMap::new();
    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        "user" => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
        _ => unreachable!("scope validated above"),
    };
    let key = root
        .open_subkey(sub)
        .map_err(|error| format!("无法读取 {scope} 环境变量注册表: {error}"))?;
    for value in key.enum_values() {
        let (name, value) = value.map_err(|error| format!("枚举 {scope} 环境变量失败: {error}"))?;
        let text = value.to_string();
        let expandable = value.vtype == REG_EXPAND_SZ || text.contains('%');
        map.insert(name, (text, expandable));
    }
    Ok(map)
}

#[cfg(not(windows))]
fn read_scope_strict(_scope: &str) -> Result<HashMap<String, (String, bool)>, String> {
    Err("严格作用域读取仅支持 Windows".into())
}

fn read_scope(scope: &str) -> HashMap<String, (String, bool)> {
    read_scope_strict(scope).unwrap_or_default()
}

pub fn scope_values_strict(scope: &str) -> Result<BTreeMap<String, String>, String> {
    read_scope_strict(scope).map(|values| {
        values
            .into_iter()
            .map(|(name, (value, _))| (name, value))
            .collect()
    })
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
    sys_sorted.sort_by_key(|a| a.0.to_lowercase());
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
    user_sorted.sort_by_key(|a| a.0.to_lowercase());
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

    validate_scope(scope)?;
    validate_name(name)?;
    validate_value(value)?;
    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        "user" => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
        _ => unreachable!("scope validated above"),
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

    validate_scope(scope)?;
    validate_name(name)?;
    let (root, sub) = match scope {
        "system" => (RegKey::predef(HKEY_LOCAL_MACHINE), SYSTEM_SUBKEY),
        "user" => (RegKey::predef(HKEY_CURRENT_USER), USER_SUBKEY),
        _ => unreachable!("scope validated above"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_scope() {
        assert!(validate_scope("process").is_err());
        assert!(validate_scope("unexpected").is_err());
    }

    #[test]
    fn rejects_invalid_variable_names() {
        for name in ["", " NAME", "NAME ", "A=B", "A\0B", "A\nB"] {
            assert!(
                validate_name(name).is_err(),
                "accepted invalid name: {name:?}"
            );
        }
        assert!(validate_name("JAVA_HOME").is_ok());
    }

    #[test]
    fn rejects_nul_in_value() {
        assert!(validate_value("hello\0world").is_err());
        assert!(validate_value("%JAVA_HOME%\\bin").is_ok());
    }
}
