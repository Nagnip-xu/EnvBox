// Windows 平台相关的底层辅助：变更广播、提权检测、以管理员身份重启、%VAR% 展开。

#[cfg(windows)]
pub fn broadcast_env_change() {
    use winapi::shared::minwindef::{LPARAM, WPARAM};
    use winapi::um::winuser::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let param: Vec<u16> = "Environment\0".encode_utf16().collect();
    let mut result: usize = 0;
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            param.as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result as *mut usize as *mut _,
        );
    }
}

#[cfg(not(windows))]
pub fn broadcast_env_change() {}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    use std::mem;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

    unsafe {
        let mut token: HANDLE = mem::zeroed();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = mem::zeroed();
        let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    true
}

/// 以管理员身份重启当前程序。返回 Ok(true) 表示已发起提权重启。
#[cfg(windows)]
pub fn relaunch_as_admin() -> Result<bool, String> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };
    if (result as usize) > 32 {
        Ok(true)
    } else {
        Err("提权失败（用户可能取消了 UAC）".into())
    }
}

#[cfg(not(windows))]
pub fn relaunch_as_admin() -> Result<bool, String> {
    Ok(false)
}

/// 以管理员身份运行一个程序并等待其结束，返回退出码。
/// 用于调用官方卸载程序 / msiexec 等需要提权的操作。
#[cfg(windows)]
pub fn run_elevated_wait(program: &str, params: &str) -> Result<u32, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetExitCodeProcess;
    use winapi::um::shellapi::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::INFINITE;

    let to_w = |s: &str| -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };
    let verb = to_w("runas");
    let file = to_w(program);
    let par = to_w(params);

    unsafe {
        let mut sei: SHELLEXECUTEINFOW = std::mem::zeroed();
        sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.lpVerb = verb.as_ptr();
        sei.lpFile = file.as_ptr();
        sei.lpParameters = par.as_ptr();
        sei.nShow = 1; // SW_SHOWNORMAL
        if ShellExecuteExW(&mut sei) == 0 || sei.hProcess.is_null() {
            return Err("提权启动失败（用户可能取消了 UAC）".into());
        }
        WaitForSingleObject(sei.hProcess, INFINITE);
        let mut code: u32 = 0;
        GetExitCodeProcess(sei.hProcess, &mut code);
        CloseHandle(sei.hProcess);
        Ok(code)
    }
}

#[cfg(not(windows))]
pub fn run_elevated_wait(_program: &str, _params: &str) -> Result<u32, String> {
    Err("仅支持 Windows".into())
}

/// 本地时间字符串 "YYYY-MM-DD HH:MM:SS"
#[cfg(windows)]
pub fn now_string() -> String {
    use winapi::um::minwinbase::SYSTEMTIME;
    use winapi::um::sysinfoapi::GetLocalTime;
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        )
    }
}

#[cfg(not(windows))]
pub fn now_string() -> String {
    "".into()
}

/// 紧凑时间戳，用作快照 id
pub fn timestamp_id() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("snap-{}", secs)
}

/// 展开字符串中的 %VAR%，使用给定的变量映射（键不区分大小写）。
pub fn expand_vars(input: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut name = String::new();
            let mut closed = false;
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc == '%' {
                    closed = true;
                    break;
                }
                name.push(nc);
            }
            if closed {
                let key = name.to_uppercase();
                if let Some(v) = vars.get(&key) {
                    out.push_str(v);
                } else {
                    out.push('%');
                    out.push_str(&name);
                    out.push('%');
                }
            } else {
                out.push('%');
                out.push_str(&name);
            }
        } else {
            out.push(c);
        }
    }
    out
}
