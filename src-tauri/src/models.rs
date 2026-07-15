use serde::{Deserialize, Serialize};

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
