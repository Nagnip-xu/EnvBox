use crate::env_registry;
use crate::models::SdkVersion;
use crate::win;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
fn no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn no_window(_cmd: &mut Command) {}

/// 每类 SDK/工具的检测与切换规格
pub struct KindSpec {
    pub kind: &'static str,
    /// 检测用的可执行文件名（含扩展名）
    pub exe: &'static str,
    /// true: exe 位于 home 的子目录（bin/cmd 等），home = 该目录的上一级；
    /// false: exe 直接位于该目录，home = 该目录本身
    pub exe_in_subdir: bool,
    /// 获取版本的参数
    pub version_args: &'static [&'static str],
    /// 切换时写入的 home 环境变量（如 JAVA_HOME）
    pub home_var: Option<&'static str>,
    /// 切换时加入 PATH 的相对目录（""=home 根目录）
    pub path_suffixes: &'static [&'static str],
}

/// 支持的所有工具类型（顺序即前端展示顺序）
pub const ALL_KINDS: &[&str] = &[
    "jdk", "python", "node", "go", "rust", "dotnet", "ruby", "php", "git", "maven", "gradle",
    "deno", "bun",
];

pub fn kind_spec(kind: &str) -> Option<KindSpec> {
    let s = match kind {
        "jdk" => KindSpec {
            kind: "jdk",
            exe: "java.exe",
            exe_in_subdir: true,
            version_args: &["-version"],
            home_var: Some("JAVA_HOME"),
            path_suffixes: &["bin"],
        },
        "python" => KindSpec {
            kind: "python",
            exe: "python.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &["", "Scripts"],
        },
        "node" => KindSpec {
            kind: "node",
            exe: "node.exe",
            exe_in_subdir: false,
            version_args: &["-v"],
            home_var: None,
            path_suffixes: &[""],
        },
        "go" => KindSpec {
            kind: "go",
            exe: "go.exe",
            exe_in_subdir: true,
            version_args: &["version"],
            home_var: Some("GOROOT"),
            path_suffixes: &["bin"],
        },
        "rust" => KindSpec {
            kind: "rust",
            exe: "cargo.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &[""],
        },
        "dotnet" => KindSpec {
            kind: "dotnet",
            exe: "dotnet.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: Some("DOTNET_ROOT"),
            path_suffixes: &[""],
        },
        "ruby" => KindSpec {
            kind: "ruby",
            exe: "ruby.exe",
            exe_in_subdir: true,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &["bin"],
        },
        "php" => KindSpec {
            kind: "php",
            exe: "php.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &[""],
        },
        "git" => KindSpec {
            kind: "git",
            exe: "git.exe",
            exe_in_subdir: true,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &["cmd", "bin"],
        },
        "maven" => KindSpec {
            kind: "maven",
            exe: "mvn.cmd",
            exe_in_subdir: true,
            version_args: &["-version"],
            home_var: Some("MAVEN_HOME"),
            path_suffixes: &["bin"],
        },
        "gradle" => KindSpec {
            kind: "gradle",
            exe: "gradle.bat",
            exe_in_subdir: true,
            version_args: &["-version"],
            home_var: Some("GRADLE_HOME"),
            path_suffixes: &["bin"],
        },
        "deno" => KindSpec {
            kind: "deno",
            exe: "deno.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &[""],
        },
        "bun" => KindSpec {
            kind: "bun",
            exe: "bun.exe",
            exe_in_subdir: false,
            version_args: &["--version"],
            home_var: None,
            path_suffixes: &[""],
        },
        _ => return None,
    };
    Some(s)
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var(var).ok().map(PathBuf::from)
}

/// 枚举所有磁盘上存在的 "Program Files" / "Program Files (x86)" 目录
fn drive_program_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for letter in b'A'..=b'Z' {
        let root = format!("{}:\\", letter as char);
        if !Path::new(&root).exists() {
            continue;
        }
        for sub in ["Program Files", "Program Files (x86)"] {
            let p = PathBuf::from(format!("{}{}", root, sub));
            if p.exists() {
                out.push(p);
            }
        }
    }
    out
}

fn subdirs(root: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                v.push(e.path());
            }
        }
    }
    v
}

/// 运行可执行文件（自动处理 .cmd/.bat）并返回首个非空输出行
fn run_first_line(exe: &Path, args: &[&str]) -> Option<String> {
    let ext = exe
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut cmd = if ext == "cmd" || ext == "bat" {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(exe).args(args);
        c
    } else {
        let mut c = Command::new(exe);
        c.args(args);
        c
    };
    no_window(&mut cmd);
    let out = cmd.output().ok()?;
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&out.stderr).to_string();
    }
    text.lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .map(|l| l.to_string())
}

/// PATH 中解析后的有序目录列表（system 在前，user 在后）
fn resolved_path_dirs() -> Vec<String> {
    let vars = env_registry::all_vars_map();
    let mut dirs = Vec::new();
    for scope in ["system", "user"] {
        if let Some(p) = env_registry::get_scope_value(scope, "Path") {
            for raw in p.split(';').filter(|s| !s.trim().is_empty()) {
                dirs.push(win::expand_vars(raw.trim(), &vars));
            }
        }
    }
    dirs
}

/// 找出某类工具在 PATH 中第一个生效的安装目录（home）
fn current_home_from_path(spec: &KindSpec) -> Option<String> {
    for dir in resolved_path_dirs() {
        let p = Path::new(&dir);
        if p.join(spec.exe).exists() {
            if spec.exe_in_subdir {
                return p.parent().map(|x| x.to_string_lossy().to_string());
            }
            return Some(dir);
        }
    }
    None
}

fn candidate_roots(kind: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let pf = env_path("ProgramFiles");
    let local = env_path("LocalAppData");
    let userprofile = env_path("USERPROFILE");

    match kind {
        "jdk" => {
            // 扫描所有磁盘的 Program Files 下常见 JDK 发行商目录
            for base in drive_program_files() {
                for sub in [
                    "Java",
                    "Eclipse Adoptium",
                    "Eclipse Foundation",
                    "Zulu",
                    "Microsoft",
                    "Amazon Corretto",
                    "BellSoft",
                    "Semeru",
                    "AdoptOpenJDK",
                    "RedHat",
                ] {
                    roots.extend(subdirs(&base.join(sub)));
                }
            }
            // 一些人直接把 JDK 放在盘符根目录，如 D:\jdk-21
            for letter in b'C'..=b'F' {
                let root = format!("{}:\\", letter as char);
                if Path::new(&root).exists() {
                    roots.extend(subdirs(Path::new(&root)).into_iter().filter(|p| {
                        p.file_name()
                            .map(|n| {
                                let n = n.to_string_lossy().to_lowercase();
                                n.starts_with("jdk")
                                    || n.starts_with("java")
                                    || n.starts_with("openjdk")
                            })
                            .unwrap_or(false)
                    }));
                }
            }
            if let Some(up) = &userprofile {
                roots.extend(subdirs(&up.join("scoop").join("apps")));
            }
        }
        "node" => {
            if let Some(p) = &pf {
                roots.push(p.join("nodejs"));
            }
            if let Some(nvm) = env_path("NVM_HOME") {
                roots.extend(subdirs(&nvm));
            }
            if let Some(app) = env_path("APPDATA") {
                roots.extend(subdirs(&app.join("nvm")));
            }
            if let Some(up) = &userprofile {
                roots.extend(subdirs(&up.join("scoop").join("apps").join("nodejs")));
            }
        }
        "python" => {
            if let Some(l) = &local {
                roots.extend(subdirs(&l.join("Programs").join("Python")));
            }
            for c in ["C:\\", "D:\\"] {
                roots.extend(subdirs(Path::new(c)).into_iter().filter(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().to_lowercase().starts_with("python"))
                        .unwrap_or(false)
                }));
            }
            if let Some(up) = &userprofile {
                roots.extend(subdirs(&up.join("scoop").join("apps").join("python")));
            }
        }
        "go" => {
            for base in drive_program_files() {
                roots.push(base.join("Go"));
            }
            for letter in b'C'..=b'F' {
                let p = PathBuf::from(format!("{}:\\Go", letter as char));
                if p.exists() {
                    roots.push(p);
                }
            }
            if let Some(gr) = env_path("GOROOT") {
                roots.push(gr);
            }
        }
        "rust" => {
            if let Some(up) = &userprofile {
                roots.push(up.join(".cargo"));
            }
            if let Some(ch) = env_path("CARGO_HOME") {
                roots.push(ch);
            }
        }
        "dotnet" => {
            if let Some(p) = &pf {
                roots.push(p.join("dotnet"));
            }
            if let Some(l) = &local {
                roots.push(l.join("Microsoft").join("dotnet"));
            }
        }
        "ruby" => {
            for c in ["C:\\", "D:\\"] {
                roots.extend(subdirs(Path::new(c)).into_iter().filter(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().to_lowercase().starts_with("ruby"))
                        .unwrap_or(false)
                }));
            }
        }
        "git" => {
            for base in drive_program_files() {
                roots.push(base.join("Git"));
            }
        }
        "deno" => {
            if let Some(up) = &userprofile {
                roots.push(up.join(".deno").join("bin"));
            }
        }
        "bun" => {
            if let Some(up) = &userprofile {
                roots.push(up.join(".bun").join("bin"));
            }
        }
        _ => {}
    }
    roots
}

/// 返回某个 home 目录中存在的可执行文件路径（若存在）
fn home_has_exe(spec: &KindSpec, home: &Path) -> Option<PathBuf> {
    if spec.exe_in_subdir {
        for sub in spec.path_suffixes.iter().filter(|s| !s.is_empty()) {
            let p = home.join(sub).join(spec.exe);
            if p.exists() {
                return Some(p);
            }
        }
        // 兜底：尝试 bin 目录
        let p = home.join("bin").join(spec.exe);
        if p.exists() {
            return Some(p);
        }
    } else {
        let p = home.join(spec.exe);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn version_of(spec: &KindSpec, exe: &Path) -> String {
    run_first_line(exe, spec.version_args)
        .unwrap_or_else(|| "已安装".into())
        .replace('"', "")
}

fn manager_for(kind: &str, home: &str) -> Option<String> {
    let path = home.replace('/', "\\").to_ascii_lowercase();
    let manager = match kind {
        "node" if path.contains("\\nvm\\") || path.contains("\\nvm-windows\\") => "nvm",
        "node" if path.contains("\\.fnm\\") || path.contains("\\fnm\\node-versions\\") => "fnm",
        "node" if path.contains("\\.volta\\") || path.contains("\\volta\\tools\\") => "Volta",
        "python"
            if path.contains("\\conda\\")
                || path.contains("\\anaconda")
                || path.contains("\\miniconda") =>
        {
            "Conda"
        }
        "rust" if path.contains("\\.cargo") || path.contains("\\.rustup") => "rustup",
        "jdk" if path.contains("\\.jabba\\") => "Jabba",
        _ => return None,
    };
    Some(manager.into())
}

/// 汇总所有可能的 home 目录：固定候选位置 + home 环境变量 + PATH 中实际生效目录
fn collect_home_candidates(spec: &KindSpec) -> Vec<PathBuf> {
    let mut homes: Vec<PathBuf> = candidate_roots(spec.kind);

    // 同时纳入用户级与系统级的 home 变量（例如 JAVA_HOME 两处可能指向不同 JDK）
    if let Some(var) = spec.home_var {
        for scope in ["user", "system"] {
            if let Some(h) = env_registry::get_scope_value(scope, var) {
                homes.push(PathBuf::from(h));
            }
        }
    }

    for dir in resolved_path_dirs() {
        let d = PathBuf::from(&dir);
        if spec.exe_in_subdir {
            if d.join(spec.exe).exists() {
                if let Some(par) = d.parent() {
                    homes.push(par.to_path_buf());
                }
            } else {
                homes.push(d);
            }
        } else {
            homes.push(d);
        }
    }
    homes
}

pub fn scan_kind(kind: &str) -> Vec<SdkVersion> {
    let spec = match kind_spec(kind) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let current =
        current_home_from_path(&spec).map(|s| s.to_lowercase().trim_end_matches('\\').to_string());
    let home_var_val = spec.home_var.and_then(|v| {
        env_registry::get_scope_value("user", v)
            .or_else(|| env_registry::get_scope_value("system", v))
    });

    let mut out: Vec<SdkVersion> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for home in collect_home_candidates(&spec) {
        if let Some(exe) = home_has_exe(&spec, &home) {
            let home_str = home.to_string_lossy().to_string();
            let key = home_str.to_lowercase().trim_end_matches('\\').to_string();
            if !seen.insert(key.clone()) {
                continue;
            }
            let version = version_of(&spec, &exe);
            let manager = manager_for(kind, &home_str);
            let is_current = if let Some(hv) = &home_var_val {
                hv.to_lowercase().trim_end_matches('\\') == key
            } else {
                current.as_ref().map(|c| c == &key).unwrap_or(false)
            };
            out.push(SdkVersion {
                kind: kind.into(),
                version,
                home: home_str,
                is_current,
                source: "scan".into(),
                manager,
            });
        }
    }
    out
}

pub fn scan_all() -> Vec<SdkVersion> {
    let mut all = Vec::new();
    for kind in ALL_KINDS {
        all.extend(scan_kind(kind));
    }
    all
}

/// 从用户 PATH 移除所有指向某类工具 exe 的条目
fn strip_kind_from_user_path(spec: &KindSpec) -> Vec<String> {
    let vars = env_registry::all_vars_map();
    env_registry::get_scope_value("user", "Path")
        .unwrap_or_default()
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|raw| {
            let resolved = win::expand_vars(raw, &vars);
            !Path::new(&resolved).join(spec.exe).exists()
        })
        .collect()
}

/// 检测切换后可能仍存在的"隐藏覆盖/写死路径"冲突，返回给前端提醒用户
fn detect_switch_conflicts(spec: &KindSpec, home: &str) -> String {
    let mut warns: Vec<String> = Vec::new();
    let home_key = home.to_lowercase().trim_end_matches('\\').to_string();

    // 1) 系统级同名 home 变量（会被用户级覆盖，但仍是隐藏值）
    if let Some(var) = spec.home_var {
        if let Some(sys) = env_registry::get_scope_value("system", var) {
            if sys.trim().to_lowercase().trim_end_matches('\\') != home_key {
                warns.push(format!(
                    "系统级 {} 仍指向 {}（已被你的用户级设置覆盖，但属隐藏值，建议清理）",
                    var, sys
                ));
            }
        }
    }

    // 2) 系统 PATH 里写死的该类工具绝对路径（优先级高于用户级，可能让切换不生效）
    let vars = env_registry::all_vars_map();
    if let Some(sp) = env_registry::get_scope_value("system", "Path") {
        for raw in sp.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let is_var_ref = spec
                .home_var
                .map(|v| {
                    raw.to_uppercase()
                        .contains(&format!("%{}%", v.to_uppercase()))
                })
                .unwrap_or(false);
            if is_var_ref {
                continue;
            }
            let resolved = win::expand_vars(raw, &vars);
            if Path::new(&resolved).join(spec.exe).exists() {
                warns.push(format!(
                    "系统 PATH 存在写死的绝对路径「{}」，优先级高于用户级，可能导致切换不生效（需管理员清理）",
                    raw
                ));
            }
        }
    }

    warns.join("；")
}

/// 一键切换某类 SDK 到指定 home（调用方负责先创建快照）
/// 返回值为可能的冲突提示（空字符串表示无冲突）
pub fn switch_sdk(kind: &str, home: &str) -> Result<String, String> {
    let spec = kind_spec(kind).ok_or_else(|| format!("不支持的类型: {}", kind))?;
    let home_path = Path::new(home);
    if !home_path.is_absolute() || !home_path.is_dir() {
        return Err(format!("目录不存在: {}", home));
    }
    if home_has_exe(&spec, home_path).is_none() {
        return Err(format!("目录中未找到 {}，已取消切换", spec.exe));
    }
    if let Some(manager) = manager_for(kind, home) {
        return Err(format!(
            "该版本由 {manager} 管理。为避免破坏其链接和状态，请使用 {manager} 完成版本切换"
        ));
    }

    let home_trim = home.trim_end_matches('\\');
    let mut new_front: Vec<String> = Vec::new();
    let old_home = spec
        .home_var
        .and_then(|var| env_registry::get_scope_value("user", var));
    let old_path = env_registry::get_scope_value("user", "Path").unwrap_or_default();

    if let Some(var) = spec.home_var {
        env_registry::set_env_var("user", var, home)?;
        for suf in spec.path_suffixes {
            if suf.is_empty() {
                new_front.push(format!("%{}%", var));
            } else {
                new_front.push(format!("%{}%\\{}", var, suf));
            }
        }
    } else {
        for suf in spec.path_suffixes {
            if suf.is_empty() {
                new_front.push(home_trim.to_string());
            } else {
                new_front.push(format!("{}\\{}", home_trim, suf));
            }
        }
    }

    let rest = strip_kind_from_user_path(&spec);
    let mut combined = new_front;
    for r in rest {
        if !combined.iter().any(|x| x.eq_ignore_ascii_case(&r)) {
            combined.push(r);
        }
    }
    if let Err(error) = env_registry::set_env_var("user", "Path", &combined.join(";")) {
        let mut rollback_errors = Vec::new();
        if let Some(var) = spec.home_var {
            match old_home {
                Some(value) => {
                    if let Err(e) = env_registry::set_env_var("user", var, &value) {
                        rollback_errors.push(format!("恢复 {var} 失败: {e}"));
                    }
                }
                None => {
                    if let Err(e) = env_registry::delete_env_var("user", var) {
                        rollback_errors.push(format!("删除新建的 {var} 失败: {e}"));
                    }
                }
            }
        }
        if let Err(e) = env_registry::set_env_var("user", "Path", &old_path) {
            rollback_errors.push(format!("恢复 PATH 失败: {e}"));
        }
        let rollback = if rollback_errors.is_empty() {
            "原设置已恢复".to_string()
        } else {
            format!("回滚不完整：{}", rollback_errors.join("；"))
        };
        return Err(format!("写入 PATH 失败（{error}）；{rollback}"));
    }
    win::broadcast_env_change();
    Ok(detect_switch_conflicts(&spec, home))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kinds_have_spec() {
        for kind in ALL_KINDS {
            assert!(kind_spec(kind).is_some(), "missing spec for {kind}");
        }
    }

    #[test]
    fn jdk_spec_fields() {
        let spec = kind_spec("jdk").unwrap();
        assert_eq!(spec.home_var, Some("JAVA_HOME"));
        assert_eq!(spec.exe, "java.exe");
    }

    #[test]
    fn detects_external_version_managers() {
        assert_eq!(
            manager_for("node", "C:\\Users\\me\\nvm\\v22"),
            Some("nvm".into())
        );
        assert_eq!(
            manager_for("rust", "C:\\Users\\me\\.cargo"),
            Some("rustup".into())
        );
        assert_eq!(manager_for("jdk", "C:\\Java\\jdk-21"), None);
    }
}
