use crate::models::{EngineStatus, InstallableVersion, JobProgress, ManagedInstall};
use crate::win;
use crate::{env_registry, snapshot};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

const MAX_JOB_LOG_LINES: usize = 2_000;
const MAX_JOB_LOG_BYTES: usize = 1024 * 1024;

fn running_jobs() -> &'static Mutex<HashMap<String, u32>> {
    static JOBS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cancelled_jobs() -> &'static Mutex<HashSet<String>> {
    static CANCELLED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    CANCELLED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn manifest_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallManifest {
    #[serde(default = "manifest_schema_version")]
    schema_version: u32,
    #[serde(default)]
    installs: Vec<ManagedInstall>,
}

fn manifest_schema_version() -> u32 {
    1
}

fn validate_manifest_schema(manifest: &InstallManifest) -> Result<(), String> {
    if manifest.schema_version != manifest_schema_version() {
        Err(format!("不支持的安装清单版本: {}", manifest.schema_version))
    } else {
        Ok(())
    }
}

fn manifest_path() -> Result<std::path::PathBuf, String> {
    let directory = snapshot::ensure_data_dir()?;
    Ok(directory.join("installs.json"))
}

fn load_manifest() -> Result<InstallManifest, String> {
    let path = manifest_path()?;
    if !path.exists() {
        return Ok(InstallManifest {
            schema_version: manifest_schema_version(),
            installs: Vec::new(),
        });
    }
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("无法读取安装清单: {error}"))?;
    let manifest: InstallManifest =
        serde_json::from_str(&text).map_err(|error| format!("安装清单损坏: {error}"))?;
    validate_manifest_schema(&manifest)?;
    Ok(manifest)
}

fn save_manifest(manifest: &InstallManifest) -> Result<(), String> {
    let path = manifest_path()?;
    let temporary = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(manifest).map_err(|error| error.to_string())?;
    {
        use std::io::Write;
        let mut output = std::fs::File::create(&temporary)
            .map_err(|error| format!("无法创建安装清单临时文件: {error}"))?;
        output
            .write_all(&json)
            .and_then(|_| output.sync_all())
            .map_err(|error| format!("无法写入安装清单: {error}"))?;
    }
    atomic_replace(&temporary, &path)
}

#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winbase::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};

    let wide = |value: &OsStr| {
        value
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    };
    let source = wide(source.as_os_str());
    let destination = wide(destination.as_os_str());
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(format!(
            "无法原子提交安装清单: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::rename(source, destination).map_err(|error| format!("无法提交安装清单: {error}"))
}

fn record_install_started(record: ManagedInstall) -> Result<(), String> {
    let _guard = manifest_lock()
        .lock()
        .map_err(|_| "安装清单锁已损坏".to_string())?;
    let mut manifest = load_manifest()?;
    manifest.installs.retain(|item| item.id != record.id);
    manifest.installs.push(record);
    save_manifest(&manifest)
}

fn update_install_status(job_id: &str, status: &str) -> Result<(), String> {
    let _guard = manifest_lock()
        .lock()
        .map_err(|_| "安装清单锁已损坏".to_string())?;
    let mut manifest = load_manifest()?;
    let record = manifest
        .installs
        .iter_mut()
        .find(|item| item.job_id == job_id)
        .ok_or_else(|| "安装任务不在所有权清单中".to_string())?;
    record.status = status.into();
    if status == "installed" {
        let previous = record
            .previous_homes
            .iter()
            .map(|home| home.trim_end_matches('\\').to_ascii_lowercase())
            .collect::<HashSet<_>>();
        record.detected_homes = crate::sdk_scanner::scan_kind(&record.kind)
            .into_iter()
            .map(|version| version.home)
            .filter(|home| !previous.contains(&home.trim_end_matches('\\').to_ascii_lowercase()))
            .collect();
        if record.detected_homes.is_empty() && !record.requested_location.is_empty() {
            record
                .detected_homes
                .push(record.requested_location.clone());
        }
    }
    save_manifest(&manifest)
}

fn update_install_record_status(record_id: &str, status: &str) -> Result<(), String> {
    let _guard = manifest_lock()
        .lock()
        .map_err(|_| "安装清单锁已损坏".to_string())?;
    let mut manifest = load_manifest()?;
    let record = manifest
        .installs
        .iter_mut()
        .find(|item| item.id == record_id)
        .ok_or_else(|| "安装记录不存在".to_string())?;
    record.status = status.into();
    save_manifest(&manifest)
}

pub fn list_managed_installs() -> Result<Vec<ManagedInstall>, String> {
    let _guard = manifest_lock()
        .lock()
        .map_err(|_| "安装清单锁已损坏".to_string())?;
    Ok(load_manifest()?.installs)
}

pub fn incomplete_install_count() -> Result<usize, String> {
    Ok(list_managed_installs()?
        .iter()
        .filter(|record| matches!(record.status.as_str(), "running" | "failed" | "cancelled"))
        .count())
}

fn managed_install_for_home(kind: &str, home: &str) -> Result<Option<ManagedInstall>, String> {
    let key = home.trim_end_matches('\\').to_ascii_lowercase();
    Ok(list_managed_installs()?.into_iter().find(|record| {
        record.kind == kind
            && record.status == "installed"
            && record
                .detected_homes
                .iter()
                .any(|candidate| candidate.trim_end_matches('\\').to_ascii_lowercase() == key)
    }))
}

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
    cmd.args([
        "/C",
        &format!("if exist \"{}\" (exit 0) else (exit 1)", expr),
    ]);
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
            mk(
                "Temurin",
                "21",
                true,
                Some("EclipseAdoptium.Temurin.21.JDK"),
                Some("temurin21-jdk"),
            ),
            mk(
                "Temurin",
                "17",
                true,
                Some("EclipseAdoptium.Temurin.17.JDK"),
                Some("temurin17-jdk"),
            ),
            mk(
                "Temurin",
                "11",
                true,
                Some("EclipseAdoptium.Temurin.11.JDK"),
                Some("temurin11-jdk"),
            ),
            mk(
                "Temurin",
                "8",
                true,
                Some("EclipseAdoptium.Temurin.8.JDK"),
                Some("temurin8-jdk"),
            ),
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
            mk(
                "官方 (LTS)",
                "LTS",
                true,
                Some("OpenJS.NodeJS.LTS"),
                Some("nodejs-lts"),
            ),
            mk(
                "官方 (Current)",
                "Current",
                false,
                Some("OpenJS.NodeJS"),
                Some("nodejs"),
            ),
            // nvm-windows 多版本管理器
            mk(
                "nvm-windows",
                "版本管理器",
                false,
                Some("CoreyButler.NVMforWindows"),
                Some("nvm"),
            ),
        ],
        "python" => vec![
            // 官方 CPython（python.org）
            mk(
                "官方 CPython",
                "3.13",
                true,
                Some("Python.Python.3.13"),
                None,
            ),
            mk(
                "官方 CPython",
                "3.12",
                true,
                Some("Python.Python.3.12"),
                Some("python"),
            ),
            mk(
                "官方 CPython",
                "3.11",
                true,
                Some("Python.Python.3.11"),
                None,
            ),
            mk(
                "官方 CPython",
                "3.10",
                false,
                Some("Python.Python.3.10"),
                None,
            ),
            // Astral uv（现代 Python 版本/依赖管理器）
            mk(
                "uv (Astral)",
                "版本管理器",
                false,
                Some("astral-sh.uv"),
                Some("uv"),
            ),
        ],
        "go" => vec![
            // 官方 Go（go.dev）
            mk("官方 Go", "latest", true, Some("GoLang.Go"), Some("go")),
        ],
        "rust" => vec![
            // 官方 rustup（可管理 stable/beta/nightly 多工具链）
            mk(
                "官方 rustup",
                "stable",
                true,
                Some("Rustlang.Rustup"),
                Some("rustup"),
            ),
        ],
        "dotnet" => vec![
            // 官方 .NET SDK（微软）
            mk(
                "官方 .NET SDK",
                "10",
                true,
                Some("Microsoft.DotNet.SDK.10"),
                None,
            ),
            mk(
                "官方 .NET SDK",
                "9",
                false,
                Some("Microsoft.DotNet.SDK.9"),
                None,
            ),
            mk(
                "官方 .NET SDK",
                "8 (LTS)",
                true,
                Some("Microsoft.DotNet.SDK.8"),
                None,
            ),
            mk(
                "官方 .NET SDK",
                "6 (LTS)",
                true,
                Some("Microsoft.DotNet.SDK.6"),
                None,
            ),
        ],
        "ruby" => vec![
            // 官方 RubyInstaller（含 DevKit，编译原生扩展需要）
            mk(
                "官方 RubyInstaller",
                "3.3 (含 DevKit)",
                true,
                Some("RubyInstallerTeam.RubyWithDevKit.3.3"),
                None,
            ),
            mk(
                "官方 RubyInstaller",
                "3.2 (含 DevKit)",
                true,
                Some("RubyInstallerTeam.RubyWithDevKit.3.2"),
                None,
            ),
            mk(
                "官方 RubyInstaller",
                "3.3",
                true,
                Some("RubyInstallerTeam.Ruby.3.3"),
                None,
            ),
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
            mk(
                "官方 Apache Maven",
                "latest",
                true,
                Some("Apache.Maven"),
                Some("maven"),
            ),
        ],
        "gradle" => vec![
            // 官方 Gradle
            mk(
                "官方 Gradle",
                "latest",
                true,
                Some("Gradle.Gradle"),
                Some("gradle"),
            ),
        ],
        "deno" => vec![
            // 官方 Deno
            mk(
                "官方 Deno",
                "latest",
                true,
                Some("DenoLand.Deno"),
                Some("deno"),
            ),
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

#[derive(Clone)]
struct OutputStreamContext {
    app: AppHandle,
    job_id: String,
    action: String,
    target: String,
    lines_seen: Arc<AtomicUsize>,
    bytes_seen: Arc<AtomicUsize>,
}

fn stream_output<R: std::io::Read + Send + 'static>(
    reader: R,
    context: OutputStreamContext,
    source: &'static str,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut bytes = Vec::new();
        let mut truncation_emitted = false;
        loop {
            bytes.clear();
            let count = match reader.read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            let line_index = context.lines_seen.fetch_add(1, Ordering::Relaxed);
            let previous_bytes = context.bytes_seen.fetch_add(count, Ordering::Relaxed);
            if line_index >= MAX_JOB_LOG_LINES || previous_bytes >= MAX_JOB_LOG_BYTES {
                if !truncation_emitted {
                    truncation_emitted = true;
                    emit(
                        &context.app,
                        JobProgress {
                            job_id: context.job_id.clone(),
                            action: context.action.clone(),
                            target: context.target.clone(),
                            phase: "installing".into(),
                            log_line: Some("日志输出过多，后续内容已省略但任务仍在运行".into()),
                        },
                    );
                }
                continue;
            }
            let text = String::from_utf8_lossy(&bytes).trim_end().to_string();
            if text.is_empty() {
                continue;
            }
            emit(
                &context.app,
                JobProgress {
                    job_id: context.job_id.clone(),
                    action: context.action.clone(),
                    target: context.target.clone(),
                    phase: "installing".into(),
                    log_line: Some(if source == "stderr" {
                        format!("[stderr] {text}")
                    } else {
                        text
                    }),
                },
            );
        }
    })
}

fn validate_job_id(job_id: &str) -> Result<(), String> {
    if job_id.is_empty()
        || job_id.len() > 80
        || !job_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        Err("无效的任务 ID".into())
    } else {
        Ok(())
    }
}

pub fn cancel_job(job_id: &str) -> Result<bool, String> {
    validate_job_id(job_id)?;
    let pid = running_jobs()
        .lock()
        .map_err(|_| "任务表锁已损坏".to_string())?
        .get(job_id)
        .copied();
    let Some(pid) = pid else {
        return Ok(false);
    };
    cancelled_jobs()
        .lock()
        .map_err(|_| "取消状态锁已损坏".to_string())?
        .insert(job_id.to_string());
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill.exe");
        command.args(["/PID", &pid.to_string(), "/T", "/F"]);
        no_window(&mut command);
        let status = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| format!("无法终止任务: {error}"));
        let status = match status {
            Ok(status) => status,
            Err(error) => {
                if let Ok(mut jobs) = cancelled_jobs().lock() {
                    jobs.remove(job_id);
                }
                return Err(error);
            }
        };
        if !status.success() {
            if let Ok(mut jobs) = cancelled_jobs().lock() {
                jobs.remove(job_id);
            }
            return Err("任务终止命令执行失败".into());
        }
    }
    #[cfg(not(windows))]
    return Err("任务取消仅支持 Windows".into());
    Ok(true)
}

/// 在后台线程执行命令并把输出以事件流式回传
fn spawn_stream(
    app: AppHandle,
    job_id: String,
    action: String,
    target: String,
    program: String,
    args: Vec<String>,
    after_success: Option<Box<dyn FnOnce() -> Result<(), String> + Send>>,
) {
    std::thread::spawn(move || {
        emit(
            &app,
            JobProgress {
                job_id: job_id.clone(),
                action: action.clone(),
                target: target.clone(),
                phase: if action == "install" {
                    "downloading"
                } else {
                    "cleaning"
                }
                .into(),
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
                if action == "install" {
                    let _ = update_install_status(&job_id, "failed");
                }
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
        if let Ok(mut jobs) = running_jobs().lock() {
            jobs.insert(job_id.clone(), child.id());
        }
        let output_context = OutputStreamContext {
            app: app.clone(),
            job_id: job_id.clone(),
            action: action.clone(),
            target: target.clone(),
            lines_seen: Arc::new(AtomicUsize::new(0)),
            bytes_seen: Arc::new(AtomicUsize::new(0)),
        };
        let stdout_reader = child
            .stdout
            .take()
            .map(|stdout| stream_output(stdout, output_context.clone(), "stdout"));
        let stderr_reader = child
            .stderr
            .take()
            .map(|stderr| stream_output(stderr, output_context, "stderr"));
        let status = child.wait();
        if let Some(reader) = stdout_reader {
            let _ = reader.join();
        }
        if let Some(reader) = stderr_reader {
            let _ = reader.join();
        }
        if let Ok(mut jobs) = running_jobs().lock() {
            jobs.remove(&job_id);
        }
        let cancelled = cancelled_jobs()
            .lock()
            .map(|mut jobs| jobs.remove(&job_id))
            .unwrap_or(false);

        if cancelled {
            if action == "install" {
                let _ = update_install_status(&job_id, "cancelled");
            }
            emit(
                &app,
                JobProgress {
                    job_id,
                    action,
                    target,
                    phase: "cancelled".into(),
                    log_line: Some("任务已由用户取消；请运行环境体检确认没有残留".into()),
                },
            );
            return;
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
                    if let Err(error) = cb() {
                        emit(
                            &app,
                            JobProgress {
                                job_id,
                                action,
                                target,
                                phase: "error".into(),
                                log_line: Some(format!("主程序已完成，但环境清理失败: {error}")),
                            },
                        );
                        return;
                    }
                }
                if action == "install" {
                    if let Err(error) = update_install_status(&job_id, "installed") {
                        emit(
                            &app,
                            JobProgress {
                                job_id,
                                action,
                                target,
                                phase: "error".into(),
                                log_line: Some(format!(
                                    "安装已完成，但所有权清单写入失败: {error}"
                                )),
                            },
                        );
                        return;
                    }
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
                if action == "install" {
                    let _ = update_install_status(&job_id, "failed");
                }
                let msg = match other {
                    Ok(s) => format!("退出码 {:?}", s.code()),
                    Err(e) => e.to_string(),
                };
                emit(
                    &app,
                    JobProgress {
                        job_id,
                        action,
                        target,
                        phase: "error".into(),
                        log_line: Some(msg),
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
    snapshot_id: &str,
) -> Result<String, String> {
    if crate::sdk_scanner::kind_spec(kind).is_none() {
        return Err(format!("不支持的 SDK 类型: {kind}"));
    }
    if !matches!(engine, "winget" | "scoop") {
        return Err(format!("不支持的安装引擎: {engine}"));
    }
    let status = engine_status();
    if (engine == "winget" && !status.winget) || (engine == "scoop" && !status.scoop) {
        return Err(format!("安装引擎 {engine} 当前不可用"));
    }
    if location.contains('\0') {
        return Err("安装路径包含非法字符".into());
    }
    if !location.trim().is_empty() && !Path::new(location.trim()).is_absolute() {
        return Err("自定义安装路径必须是绝对路径".into());
    }
    let item = catalog(kind)
        .into_iter()
        .find(|i| i.version == version && (distro.is_empty() || i.distro == distro))
        .ok_or_else(|| "目录中找不到该版本".to_string())?;
    let job_id = win::timestamp_id();
    let target = format!("{} {}", item.distro, item.version);

    let (program, args, package_id) = match engine {
        "scoop" => {
            let id = item.scoop_id.clone().ok_or("该版本不支持 scoop")?;
            (
                "scoop".to_string(),
                vec!["install".to_string(), id.clone()],
                id.clone(),
            )
        }
        "winget" => {
            let id = item.winget_id.clone().ok_or("该版本不支持 winget")?;
            let mut args = vec![
                "install".to_string(),
                "--id".to_string(),
                id.clone(),
                "-e".to_string(),
                "--accept-source-agreements".to_string(),
                "--accept-package-agreements".to_string(),
            ];
            // 自定义安装路径：仅对支持 --location 的 winget 包生效
            if !location.trim().is_empty() {
                args.push("--location".to_string());
                args.push(location.trim().to_string());
            }
            ("winget".to_string(), args, id)
        }
        _ => unreachable!("engine validated above"),
    };

    let previous_homes = crate::sdk_scanner::scan_kind(kind)
        .into_iter()
        .map(|version| version.home)
        .collect();
    record_install_started(ManagedInstall {
        id: format!("install-{}", job_id.trim_start_matches("snap-")),
        job_id: job_id.clone(),
        snapshot_id: snapshot_id.into(),
        kind: kind.into(),
        distro: item.distro.clone(),
        version: item.version.clone(),
        engine: engine.into(),
        package_id,
        requested_location: location.trim().into(),
        previous_homes,
        detected_homes: Vec::new(),
        status: "running".into(),
        installed_at: win::now_string(),
    })?;

    snapshot::audit("install", &target);
    spawn_stream(
        app,
        job_id.clone(),
        "install".into(),
        target,
        program,
        args,
        None,
    );
    Ok(job_id)
}

/// 移除两个作用域 PATH 中位于 home 目录下的条目，并清理 JAVA_HOME/GOROOT
fn cleanup_env_for_home(kind: &str, home: &str) -> Result<(), String> {
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
            env_registry::set_env_var(scope, "Path", &kept.join(";"))
                .map_err(|e| format!("清理 {scope} PATH 失败: {e}"))?;
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
                    env_registry::delete_env_var(scope, hv)
                        .map_err(|e| format!("清理 {scope} {hv} 失败: {e}"))?;
                }
            }
        }
    }
    Ok(())
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
            let loc_l = install_loc
                .to_lowercase()
                .trim_end_matches('\\')
                .to_string();
            let icon_l = display_icon.to_lowercase();
            let matched = (!loc_l.is_empty() && loc_l == home_key)
                || (!home_key.is_empty() && icon_l.starts_with(&home_key));
            if !matched {
                continue;
            }
            let name: String = app.get_value("DisplayName").unwrap_or_else(|_| sub.clone());
            let quiet: String = app.get_value("QuietUninstallString").unwrap_or_default();
            let normal: String = app.get_value("UninstallString").unwrap_or_default();
            let cmd = if !quiet.trim().is_empty() {
                quiet
            } else {
                normal
            };
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
    let spec =
        crate::sdk_scanner::kind_spec(kind).ok_or_else(|| format!("不支持的 SDK 类型: {kind}"))?;
    let home_path = Path::new(home);
    if !home_path.is_absolute() || !home_path.is_dir() {
        return Err("SDK 目录不存在或不是绝对路径".into());
    }
    if home_path.parent().is_none() {
        return Err("拒绝卸载磁盘根目录".into());
    }
    let contains_expected_exe = if spec.exe_in_subdir {
        spec.path_suffixes
            .iter()
            .filter(|s| !s.is_empty())
            .any(|s| home_path.join(s).join(spec.exe).is_file())
            || home_path.join("bin").join(spec.exe).is_file()
    } else {
        home_path.join(spec.exe).is_file()
    };
    if !contains_expected_exe {
        return Err(format!("目录中未找到 {}，为避免误删已取消卸载", spec.exe));
    }
    let job_id = win::timestamp_id();
    let target = home.to_string();
    let home_owned = home.to_string();
    let kind_owned = kind.to_string();
    snapshot::audit("uninstall", home);

    if let Some(record) = managed_install_for_home(kind, home)? {
        let program = record.engine.clone();
        let args = if record.engine == "winget" {
            vec![
                "uninstall".into(),
                "--id".into(),
                record.package_id.clone(),
                "-e".into(),
                "--accept-source-agreements".into(),
            ]
        } else if record.engine == "scoop" {
            vec!["uninstall".into(), record.package_id.clone()]
        } else {
            return Err(format!("安装记录包含不支持的引擎: {}", record.engine));
        };
        let record_id = record.id;
        let callback: Box<dyn FnOnce() -> Result<(), String> + Send> = Box::new(move || {
            cleanup_env_for_home(&kind_owned, &home_owned)?;
            update_install_record_status(&record_id, "uninstalled")
        });
        spawn_stream(
            app,
            job_id.clone(),
            "uninstall".into(),
            target,
            program,
            args,
            Some(callback),
        );
        return Ok(job_id);
    }

    let home_l = home.to_lowercase();
    if home_l.contains("\\scoop\\apps\\") {
        // Scoop 应用：交给 scoop 卸载（本就干净）
        let app_name = Path::new(home)
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let cb: Box<dyn FnOnce() -> Result<(), String> + Send> =
            Box::new(move || cleanup_env_for_home(&kind_owned, &home_owned));
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
        let worker_job_id = job_id.clone();
        std::thread::spawn(move || {
            emit_uninstall(
                &app,
                &worker_job_id,
                &home_owned,
                "installing",
                &format!("调用官方卸载程序：{}", name),
            );
            emit_uninstall(
                &app,
                &worker_job_id,
                &home_owned,
                "installing",
                &format!("$ {} {}", program, params),
            );

            match win::run_elevated_wait(&program, &params) {
                Ok(code) if code == 0 => {
                    emit_uninstall(
                        &app,
                        &worker_job_id,
                        &home_owned,
                        "cleaning",
                        &format!("官方卸载完成(退出码 {})，正在清理残留...", code),
                    );
                    // 官方卸载器结束后不递归删除外部安装目录，避免误伤共享内容。
                    if Path::new(&home_owned).exists() {
                        emit_uninstall(
                            &app,
                            &worker_job_id,
                            &home_owned,
                            "cleaning",
                            "检测到安装目录仍存在；为避免误删共享文件，EnvBox 已保留该目录",
                        );
                    }
                    if let Err(error) = cleanup_env_for_home(&kind_owned, &home_owned) {
                        emit_uninstall(&app, &worker_job_id, &home_owned, "error", &error);
                        return;
                    }
                    win::broadcast_env_change();
                    emit_uninstall(
                        &app,
                        &worker_job_id,
                        &home_owned,
                        "done",
                        "已彻底卸载并清理环境变量",
                    );
                }
                Ok(code) => {
                    emit_uninstall(
                        &app,
                        &worker_job_id,
                        &home_owned,
                        "error",
                        &format!("官方卸载程序返回非零退出码 {code}，未清理环境变量"),
                    );
                }
                Err(e) => {
                    emit_uninstall(
                        &app,
                        &worker_job_id,
                        &home_owned,
                        "error",
                        &format!("卸载失败：{}", e),
                    );
                }
            }
        });
        return Ok(job_id);
    }

    Err("未找到可信的官方卸载程序。为避免误删外部或便携式目录，EnvBox 已取消卸载；你仍可手动删除目录后使用 PATH 清理功能。".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jdk_catalog_has_temurin() {
        let items = catalog("jdk");
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.distro == "Temurin"));
        assert!(items
            .iter()
            .any(|i| i.engines.contains(&"winget".to_string())));
    }

    #[test]
    fn unknown_kind_returns_empty_catalog() {
        assert!(catalog("unknown-tool").is_empty());
    }

    #[test]
    fn job_ids_are_strictly_validated_before_cancellation() {
        assert!(validate_job_id("snap-20260716_123456-7").is_ok());
        assert!(validate_job_id("").is_err());
        assert!(validate_job_id("../other-process").is_err());
        assert!(validate_job_id("job id").is_err());
        assert!(validate_job_id(&"a".repeat(81)).is_err());
    }

    #[test]
    fn install_manifest_schema_is_stable_and_rejects_future_versions() {
        let current: InstallManifest = serde_json::from_str(r#"{"installs":[]}"#).unwrap();
        assert_eq!(current.schema_version, manifest_schema_version());

        let future: InstallManifest =
            serde_json::from_str(r#"{"schemaVersion":99,"installs":[]}"#).unwrap();
        assert!(validate_manifest_schema(&future).is_err());
    }
}
