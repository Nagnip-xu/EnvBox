use crate::models::{EngineStatus, InstallableVersion, JobProgress};
use crate::win;
use crate::{env_registry, snapshot};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
fn no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn no_window(_cmd: &mut Command) {}

fn where_lookup(tool: &str) -> bool {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "where", tool]);
    no_window(&mut cmd);
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 用 cmd 的 `if exist` 判断文件是否存在：能正确处理 %VAR% 展开以及
/// winget 这类「应用执行别名」的重解析点（普通 fs::exists 对别名会误判）。
fn path_exists_shell(expr: &str) -> bool {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", &format!("if exist \"{}\" (exit 0) else (exit 1)", expr)]);
    no_window(&mut cmd);
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 检测某个工具是否可用。先查 PATH，再回退到已知安装位置——
/// 这样即使以管理员重启后 PATH 里缺少用户级 WindowsApps 目录，也能稳定识别到 winget。
fn tool_exists(tool: &str) -> bool {
    if where_lookup(tool) {
        return true;
    }
    let candidates: &[&str] = match tool {
        "winget" => &["%LOCALAPPDATA%\\Microsoft\\WindowsApps\\winget.exe"],
        "scoop" => &[
            "%USERPROFILE%\\scoop\\shims\\scoop.cmd",
            "%USERPROFILE%\\scoop\\shims\\scoop.ps1",
            "%USERPROFILE%\\scoop\\shims\\scoop",
        ],
        _ => &[],
    };
    candidates.iter().any(|c| path_exists_shell(c))
}

pub fn engine_status() -> EngineStatus {
    EngineStatus {
        winget: tool_exists("winget"),
        scoop: tool_exists("scoop"),
        elevated: win::is_elevated(),
    }
}

pub fn catalog(kind: &str) -> Vec<InstallableVersion> {
    let mk = |distro: &str, version: &str, lts: bool, winget: Option<&str>, scoop: Option<&str>| {
        let mut engines = Vec::new();
        if winget.is_some() {
            engines.push("winget".to_string());
        }
        if scoop.is_some() {
            engines.push("scoop".to_string());
        }
        InstallableVersion {
            kind: kind.to_string(),
            distro: distro.to_string(),
            version: version.to_string(),
            is_lts: lts,
            engines,
            winget_id: winget.map(|s| s.to_string()),
            scoop_id: scoop.map(|s| s.to_string()),
        }
    };

    match kind {
        "jdk" => vec![
            // Eclipse Temurin（Adoptium）—— 最主流的免费 OpenJDK
            mk("Temurin", "21", true, Some("EclipseAdoptium.Temurin.21.JDK"), Some("temurin21-jdk")),
            mk("Temurin", "17", true, Some("EclipseAdoptium.Temurin.17.JDK"), Some("temurin17-jdk")),
            mk("Temurin", "11", true, Some("EclipseAdoptium.Temurin.11.JDK"), Some("temurin11-jdk")),
            mk("Temurin", "8", true, Some("EclipseAdoptium.Temurin.8.JDK"), Some("temurin8-jdk")),
            // Oracle JDK —— 官方原厂（商用生产需订阅授权）
            mk("Oracle", "21", true, Some("Oracle.JDK.21"), None),
            mk("Oracle", "17", true, Some("Oracle.JDK.17"), None),
            // Microsoft Build of OpenJDK —— 免费，适合 Windows/Azure
            mk("Microsoft", "21", true, Some("Microsoft.OpenJDK.21"), None),
            mk("Microsoft", "17", true, Some("Microsoft.OpenJDK.17"), None),
            mk("Microsoft", "11", true, Some("Microsoft.OpenJDK.11"), None),
            // Amazon Corretto —— 免费，长期支持
            mk("Corretto", "21", true, Some("Amazon.Corretto.21.JDK"), None),
            mk("Corretto", "17", true, Some("Amazon.Corretto.17.JDK"), None),
            mk("Corretto", "11", true, Some("Amazon.Corretto.11.JDK"), None),
            mk("Corretto", "8", true, Some("Amazon.Corretto.8.JDK"), None),
            // Azul Zulu —— 免费社区版
            mk("Zulu", "21", true, Some("Azul.Zulu.21.JDK"), None),
            mk("Zulu", "17", true, Some("Azul.Zulu.17.JDK"), None),
            mk("Zulu", "11", true, Some("Azul.Zulu.11.JDK"), None),
        ],
        "node" => vec![
            // 官方 Node.js（nodejs.org）
            mk("官方 (LTS)", "LTS", true, Some("OpenJS.NodeJS.LTS"), Some("nodejs-lts")),
            mk("官方 (Current)", "Current", false, Some("OpenJS.NodeJS"), Some("nodejs")),
            // nvm-windows 多版本管理器
            mk("nvm-windows", "版本管理器", false, Some("CoreyButler.NVMforWindows"), Some("nvm")),
        ],
        "python" => vec![
            // 官方 CPython（python.org）
            mk("官方 CPython", "3.13", true, Some("Python.Python.3.13"), None),
            mk("官方 CPython", "3.12", true, Some("Python.Python.3.12"), Some("python")),
            mk("官方 CPython", "3.11", true, Some("Python.Python.3.11"), None),
            mk("官方 CPython", "3.10", false, Some("Python.Python.3.10"), None),
            // Astral uv（现代 Python 版本/依赖管理器）
            mk("uv (Astral)", "版本管理器", false, Some("astral-sh.uv"), Some("uv")),
        ],
        "go" => vec![
            // 官方 Go（go.dev）
            mk("官方 Go", "latest", true, Some("GoLang.Go"), Some("go")),
        ],
        "rust" => vec![
            // 官方 rustup（可管理 stable/beta/nightly 多工具链）
            mk("官方 rustup", "stable", true, Some("Rustlang.Rustup"), Some("rustup")),
        ],
        "dotnet" => vec![
            // 官方 .NET SDK（微软）
            mk("官方 .NET SDK", "10", true, Some("Microsoft.DotNet.SDK.10"), None),
            mk("官方 .NET SDK", "9", false, Some("Microsoft.DotNet.SDK.9"), None),
            mk("官方 .NET SDK", "8 (LTS)", true, Some("Microsoft.DotNet.SDK.8"), None),
            mk("官方 .NET SDK", "6 (LTS)", true, Some("Microsoft.DotNet.SDK.6"), None),
        ],
        "ruby" => vec![
            // 官方 RubyInstaller（含 DevKit，编译原生扩展需要）
            mk("官方 RubyInstaller", "3.3 (含 DevKit)", true, Some("RubyInstallerTeam.RubyWithDevKit.3.3"), None),
            mk("官方 RubyInstaller", "3.2 (含 DevKit)", true, Some("RubyInstallerTeam.RubyWithDevKit.3.2"), None),
            mk("官方 RubyInstaller", "3.3", true, Some("RubyInstallerTeam.Ruby.3.3"), None),
        ],
        "php" => vec![
            // 官方 PHP（php.net）
            mk("官方 PHP", "8.4", true, Some("PHP.PHP.8.4"), None),
            mk("官方 PHP", "8.3", true, Some("PHP.PHP.8.3"), None),
            mk("官方 PHP", "8.2", true, Some("PHP.PHP.8.2"), None),
        ],
        "git" => vec![
            // 官方 Git for Windows
            mk("官方 Git", "latest", true, Some("Git.Git"), Some("git")),
        ],
        "maven" => vec![
            // 官方 Apache Maven
            mk("官方 Apache Maven", "latest", true, Some("Apache.Maven"), Some("maven")),
        ],
        "gradle" => vec![
            // 官方 Gradle
            mk("官方 Gradle", "latest", true, Some("Gradle.Gradle"), Some("gradle")),
        ],
        "deno" => vec![
            // 官方 Deno
            mk("官方 Deno", "latest", true, Some("DenoLand.Deno"), Some("deno")),
        ],
        "bun" => vec![
            // 官方 Bun
            mk("官方 Bun", "latest", true, Some("Oven-sh.Bun"), Some("bun")),
        ],
        _ => vec![],
    }
}

fn emit(app: &AppHandle, p: JobProgress) {
    let _ = app.emit("job://progress", p);
}

/// 在后台线程执行命令并把输出以事件流式回传
fn spawn_stream(
    app: AppHandle,
    job_id: String,
    action: String,
    target: String,
    program: String,
    args: Vec<String>,
    after_success: Option<Box<dyn FnOnce() + Send>>,
) {
    std::thread::spawn(move || {
        emit(
            &app,
            JobProgress {
                job_id: job_id.clone(),
                action: action.clone(),
                target: target.clone(),
                phase: if action == "install" { "downloading" } else { "cleaning" }.into(),
                log_line: Some(format!("$ {} {}", program, args.join(" "))),
            },
        );

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        no_window(&mut cmd);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                emit(
                    &app,
                    JobProgress {
                        job_id: job_id.clone(),
                        action: action.clone(),
                        target: target.clone(),
                        phase: "error".into(),
                        log_line: Some(format!("无法启动: {}", e)),
                    },
                );
                return;
            }
        };

        if let Some(out) = child.stdout.take() {
            let reader = BufReader::new(out);
            for line in reader.lines().map_while(Result::ok) {
                emit(
                    &app,
                    JobProgress {
                        job_id: job_id.clone(),
                        action: action.clone(),
                        target: target.clone(),
                        phase: "installing".into(),
                        log_line: Some(line),
                    },
                );
            }
        }

        let status = child.wait();
        let mut err_text = String::new();
        if let Some(mut e) = child.stderr.take() {
            use std::io::Read;
            let _ = e.read_to_string(&mut err_text);
        }

        match status {
            Ok(s) if s.success() => {
                if action == "install" {
                    emit(
                        &app,
                        JobProgress {
                            job_id: job_id.clone(),
                            action: action.clone(),
                            target: target.clone(),
                            phase: "configuring".into(),
                            log_line: Some("正在配置环境变量...".into()),
                        },
                    );
                }
                if let Some(cb) = after_success {
                    cb();
                }
                win::broadcast_env_change();
                emit(
                    &app,
                    JobProgress {
                        job_id,
                        action,
                        target,
                        phase: "done".into(),
                        log_line: Some("完成".into()),
                    },
                );
            }
            other => {
                let msg = match other {
                    Ok(s) => format!("退出码 {:?}", s.code()),
                    Err(e) => e.to_string(),
                };
                let detail = if err_text.trim().is_empty() {
                    msg
                } else {
                    format!("{} | {}", msg, err_text.trim())
                };
                emit(
                    &app,
                    JobProgress {
                        job_id,
                        action,
                        target,
                        phase: "error".into(),
                        log_line: Some(detail),
                    },
                );
            }
        }
    });
}

pub fn install_sdk(
    app: AppHandle,
    kind: &str,
    distro: &str,
    version: &str,
    engine: &str,
    location: &str,
) -> Result<String, String> {
    let item = catalog(kind)
        .into_iter()
        .find(|i| i.version == version && (distro.is_empty() || i.distro == distro))
        .ok_or_else(|| "目录中找不到该版本".to_string())?;
    let job_id = win::timestamp_id();
    let target = format!("{} {}", item.distro, item.version);

    let (program, args) = match engine {
        "scoop" => {
            let id = item.scoop_id.ok_or("该版本不支持 scoop")?;
            ("scoop".to_string(), vec!["install".to_string(), id])
        }
        _ => {
            let id = item.winget_id.ok_or("该版本不支持 winget")?;
            let mut args = vec![
                "install".to_string(),
                "--id".to_string(),
                id,
                "-e".to_string(),
                "--accept-source-agreements".to_string(),
                "--accept-package-agreements".to_string(),
            ];
            // 自定义安装路径：仅对支持 --location 的 winget 包生效
            if !location.trim().is_empty() {
                args.push("--location".to_string());
                args.push(location.trim().to_string());
            }
            ("winget".to_string(), args)
        }
    };

    snapshot::audit("install", &target);
    spawn_stream(app, job_id.clone(), "install".into(), target, program, args, None);
    Ok(job_id)
}

/// 移除两个作用域 PATH 中位于 home 目录下的条目，并清理 JAVA_HOME/GOROOT
fn cleanup_env_for_home(kind: &str, home: &str) {
    let vars = env_registry::all_vars_map();
    let home_l = home.to_lowercase().trim_end_matches('\\').to_string();
    for scope in ["user", "system"] {
        if scope == "system" && !win::is_elevated() {
            continue;
        }
        if let Some(p) = env_registry::get_scope_value(scope, "Path") {
            let kept: Vec<String> = p
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .filter(|raw| {
                    let resolved = win::expand_vars(raw, &vars).to_lowercase();
                    !resolved.trim_end_matches('\\').starts_with(&home_l)
                })
                .collect();
            let _ = env_registry::set_env_var(scope, "Path", &kept.join(";"));
        }
    }
    let home_var = crate::sdk_scanner::kind_spec(kind).and_then(|s| s.home_var);
    if let Some(hv) = home_var {
        for scope in ["user", "system"] {
            if scope == "system" && !win::is_elevated() {
                continue;
            }
            if let Some(v) = env_registry::get_scope_value(scope, hv) {
                if v.to_lowercase().trim_end_matches('\\') == home_l {
                    let _ = env_registry::delete_env_var(scope, hv);
                }
            }
        }
    }
}

/// 在 Windows「卸载」注册表中查找与某安装目录匹配的卸载信息。
/// 返回 (显示名, 卸载命令原文)
#[cfg(windows)]
fn find_uninstall_command(home: &str) -> Option<(String, String)> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let home_key = home.to_lowercase().trim_end_matches('\\').to_string();
    let roots: [(RegKey, &str); 3] = [
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
    ];

    for (root, path) in roots {
        let key = match root.open_subkey(path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub in key.enum_keys().flatten() {
            let app = match key.open_subkey(&sub) {
                Ok(a) => a,
                Err(_) => continue,
            };
            let install_loc: String = app.get_value("InstallLocation").unwrap_or_default();
            let display_icon: String = app.get_value("DisplayIcon").unwrap_or_default();
            let loc_l = install_loc.to_lowercase().trim_end_matches('\\').to_string();
            let icon_l = display_icon.to_lowercase();
            let matched = (!loc_l.is_empty() && loc_l == home_key)
                || (!home_key.is_empty() && icon_l.starts_with(&home_key));
            if !matched {
                continue;
            }
            let name: String = app.get_value("DisplayName").unwrap_or_else(|_| sub.clone());
            let quiet: String = app.get_value("QuietUninstallString").unwrap_or_default();
            let normal: String = app.get_value("UninstallString").unwrap_or_default();
            let cmd = if !quiet.trim().is_empty() { quiet } else { normal };
            if !cmd.trim().is_empty() {
                return Some((name, cmd));
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn find_uninstall_command(_home: &str) -> Option<(String, String)> {
    None
}

/// 把注册表里的卸载命令拆成 (程序, 参数)，并把 MSI 转成静默卸载
fn to_uninstall_invocation(cmd: &str) -> (String, String) {
    let lower = cmd.to_lowercase();
    if lower.contains("msiexec") {
        if let (Some(s), Some(e)) = (cmd.find('{'), cmd.find('}')) {
            if e > s {
                let guid = &cmd[s..=e];
                return (
                    "msiexec.exe".to_string(),
                    format!("/x {} /quiet /norestart", guid),
                );
            }
        }
    }
    // 带引号的可执行路径
    let c = cmd.trim();
    if let Some(rest) = c.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            let prog = rest[..end].to_string();
            let params = rest[end + 1..].trim().to_string();
            return (prog, params);
        }
    }
    // 无引号：按第一个空格拆分
    match c.find(' ') {
        Some(i) => (c[..i].to_string(), c[i + 1..].trim().to_string()),
        None => (c.to_string(), String::new()),
    }
}

fn emit_uninstall(app: &AppHandle, job_id: &str, home: &str, phase: &str, line: &str) {
    emit(
        app,
        JobProgress {
            job_id: job_id.to_string(),
            action: "uninstall".into(),
            target: home.to_string(),
            phase: phase.into(),
            log_line: Some(line.to_string()),
        },
    );
}

pub fn uninstall_sdk(app: AppHandle, kind: &str, home: &str) -> Result<String, String> {
    let job_id = win::timestamp_id();
    let target = home.to_string();
    let home_owned = home.to_string();
    let kind_owned = kind.to_string();
    snapshot::audit("uninstall", home);

    let home_l = home.to_lowercase();
    if home_l.contains("\\scoop\\apps\\") {
        // Scoop 应用：交给 scoop 卸载（本就干净）
        let app_name = Path::new(home)
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let cb: Box<dyn FnOnce() + Send> = Box::new(move || {
            cleanup_env_for_home(&kind_owned, &home_owned);
        });
        spawn_stream(
            app,
            job_id.clone(),
            "uninstall".into(),
            target,
            "scoop".into(),
            vec!["uninstall".into(), app_name],
            Some(cb),
        );
        return Ok(job_id);
    }

    // 优先：调用官方卸载程序（清注册表/添加删除程序/开始菜单等）
    if let Some((name, raw_cmd)) = find_uninstall_command(home) {
        let (program, params) = to_uninstall_invocation(&raw_cmd);
        std::thread::spawn(move || {
            emit_uninstall(
                &app,
                &job_id,
                &home_owned,
                "installing",
                &format!("调用官方卸载程序：{}", name),
            );
            emit_uninstall(&app, &job_id, &home_owned, "installing", &format!("$ {} {}", program, params));

            match win::run_elevated_wait(&program, &params) {
                Ok(code) => {
                    emit_uninstall(
                        &app,
                        &job_id,
                        &home_owned,
                        "cleaning",
                        &format!("官方卸载完成(退出码 {})，正在清理残留...", code),
                    );
                    // 残留目录（官方卸载器通常已清除；若仍在则尝试删除，必要时提权）
                    if Path::new(&home_owned).exists() {
                        if std::fs::remove_dir_all(&home_owned).is_err()
                            && Path::new(&home_owned).exists()
                        {
                            let _ = win::run_elevated_wait(
                                "cmd.exe",
                                &format!("/c rmdir /s /q \"{}\"", home_owned),
                            );
                        }
                    }
                    cleanup_env_for_home(&kind_owned, &home_owned);
                    win::broadcast_env_change();
                    emit_uninstall(
                        &app,
                        &job_id,
                        &home_owned,
                        "done",
                        "已彻底卸载并清理环境变量",
                    );
                }
                Err(e) => {
                    emit_uninstall(
                        &app,
                        &job_id,
                        &home_owned,
                        "error",
                        &format!("卸载失败：{}", e),
                    );
                }
            }
        });
        return Ok(String::new());
    }

    // 兜底：绿色/便携式安装（无注册表卸载项）→ 删除目录 + 清理变量
    std::thread::spawn(move || {
        emit_uninstall(
            &app,
            &job_id,
            &home_owned,
            "cleaning",
            &format!("未发现官方卸载程序，按便携式处理：删除目录 {}", home_owned),
        );
        if std::fs::remove_dir_all(&home_owned).is_err() && Path::new(&home_owned).exists() {
            // 可能位于 Program Files，需要管理员
            if let Err(e) =
                win::run_elevated_wait("cmd.exe", &format!("/c rmdir /s /q \"{}\"", home_owned))
                    .map(|_| ())
            {
                emit_uninstall(
                    &app,
                    &job_id,
                    &home_owned,
                    "error",
                    &format!("删除失败(可能需要管理员权限)：{}", e),
                );
                return;
            }
        }
        cleanup_env_for_home(&kind_owned, &home_owned);
        win::broadcast_env_change();
        emit_uninstall(&app, &job_id, &home_owned, "done", "已卸载并清理环境变量");
    });
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jdk_catalog_has_temurin() {
        let items = catalog("jdk");
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.distro == "Temurin"));
        assert!(items.iter().any(|i| i.engines.contains(&"winget".to_string())));
    }

    #[test]
    fn unknown_kind_returns_empty_catalog() {
        assert!(catalog("unknown-tool").is_empty());
    }
}
