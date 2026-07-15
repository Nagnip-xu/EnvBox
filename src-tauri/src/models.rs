use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        let lower = message.to_lowercase();
        let code = if message.contains("快照")
            && (message.contains("失败") || message.contains("损坏"))
        {
            "SNAPSHOT_FAILED"
        } else if message.contains("拒绝访问")
            || lower.contains("permission")
            || lower.contains("denied")
            || lower.contains("os error 5")
            || message.contains("管理员权限")
        {
            "PERMISSION_DENIED"
        } else if message.contains("管理。为避免") || message.contains("版本管理器") {
            "EXTERNAL_MANAGER"
        } else if message.contains("找不到") || message.contains("不存在") {
            "NOT_FOUND"
        } else if message.contains("不支持")
            || message.contains("无效")
            || message.contains("不能为空")
            || message.contains("非法")
            || message.contains("超过")
            || message.contains("必须")
        {
            "INVALID_INPUT"
        } else {
            "OPERATION_FAILED"
        };
        Self {
            code: code.into(),
            message,
        }
    }
}

pub type CommandResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub scope: String, // "system" | "user" | "process"
    pub is_expandable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts_with: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathEntry {
    pub raw: String,
    pub resolved: String,
    pub scope: String, // "system" | "user"
    pub exists: bool,
    pub status: String, // "available" | "missing" | "unresolved" | "networkUnavailable"
    pub safe_to_clean: bool,
    pub duplicate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_tag: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkVersion {
    pub kind: String, // "jdk" | "python" | "node" | "go"
    pub version: String,
    pub home: String,
    pub is_current: bool,
    pub source: String, // "scan" | "manual" | "envbox"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallableVersion {
    pub kind: String,
    pub distro: String,
    pub version: String,
    pub is_lts: bool,
    pub engines: Vec<String>, // "winget" | "scoop"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winget_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoop_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub job_id: String,
    pub action: String, // "install" | "uninstall"
    pub target: String,
    pub phase: String, // downloading|installing|configuring|cleaning|done|error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub created_at: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotChange {
    pub scope: String,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotPreview {
    pub snapshot_id: String,
    pub description: String,
    pub created_at: String,
    pub changes: Vec<SnapshotChange>,
    pub user_changes: usize,
    pub system_changes: usize,
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub time: String,
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub total_vars: usize,
    pub invalid_paths: usize,
    pub duplicate_paths: usize,
    pub conflicts: usize,
    pub path_length: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub winget: bool,
    pub scoop: bool,
    pub elevated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub user_count: usize,
    pub system_count: usize,
    pub sensitive_count: usize,
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersionHint {
    pub tool: String,
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInspection {
    pub path: String,
    pub hints: Vec<ProjectVersionHint>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_command_errors() {
        assert_eq!(
            AppError::from("拒绝访问".to_string()).code,
            "PERMISSION_DENIED"
        );
        assert_eq!(
            AppError::from("安全快照创建失败".to_string()).code,
            "SNAPSHOT_FAILED"
        );
        assert_eq!(AppError::from("目录不存在".to_string()).code, "NOT_FOUND");
    }
}
