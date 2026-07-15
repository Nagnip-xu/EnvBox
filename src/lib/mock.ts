import type { EnvVar, PathEntry, SdkVersion, Snapshot } from "../types";

// 仅用于浏览器预览(未接 Rust 后端时)的示例数据
export const mockEnvVars: EnvVar[] = [
  { name: "JAVA_HOME", value: "D:\\DevTools\\jdk-17", scope: "system", isExpandable: false },
  { name: "GOROOT", value: "C:\\Program Files\\Go", scope: "system", isExpandable: false },
  { name: "GOPATH", value: "%USERPROFILE%\\go", scope: "user", isExpandable: true },
  { name: "CARGO_HOME", value: "D:\\DevTools\\Rust\\.cargo", scope: "user", isExpandable: false },
  { name: "RUSTUP_HOME", value: "D:\\DevTools\\Rust\\.rustup", scope: "user", isExpandable: false },
  { name: "NODE_OPTIONS", value: "--max-old-space-size=4096", scope: "user", isExpandable: false },
  { name: "SystemRoot", value: "C:\\Windows", scope: "system", isExpandable: false },
  { name: "TEMP", value: "%USERPROFILE%\\AppData\\Local\\Temp", scope: "user", isExpandable: true, conflictsWith: "system" },
];

export const mockPathEntries: PathEntry[] = [
  { raw: "%JAVA_HOME%\\bin", resolved: "D:\\DevTools\\jdk-17\\bin", scope: "system", exists: true, status: "available", safeToClean: false, duplicate: false, sdkTag: "JDK 17", enabled: true },
  { raw: "C:\\Program Files\\nodejs", resolved: "C:\\Program Files\\nodejs", scope: "system", exists: true, status: "available", safeToClean: false, duplicate: false, sdkTag: "Node 24", enabled: true },
  { raw: "D:\\DevTools\\Rust\\.cargo\\bin", resolved: "D:\\DevTools\\Rust\\.cargo\\bin", scope: "user", exists: true, status: "available", safeToClean: false, duplicate: false, sdkTag: "Rust", enabled: true },
  { raw: "C:\\Python39\\Scripts", resolved: "C:\\Python39\\Scripts", scope: "user", exists: false, status: "missing", safeToClean: true, duplicate: false, enabled: true },
  { raw: "%UNSET_SDK_HOME%\\bin", resolved: "%UNSET_SDK_HOME%\\bin", scope: "user", exists: false, status: "unresolved", safeToClean: false, duplicate: false, enabled: true },
  { raw: "C:\\Program Files\\Go\\bin", resolved: "C:\\Program Files\\Go\\bin", scope: "system", exists: true, status: "available", safeToClean: false, duplicate: false, sdkTag: "Go", enabled: true },
  { raw: "C:\\Program Files\\nodejs", resolved: "C:\\Program Files\\nodejs", scope: "user", exists: true, status: "available", safeToClean: false, duplicate: true, sdkTag: "Node 24", enabled: true },
  { raw: "C:\\Windows\\System32", resolved: "C:\\Windows\\System32", scope: "system", exists: true, status: "available", safeToClean: false, duplicate: false, enabled: true },
];

export const mockSdks: SdkVersion[] = [
  { kind: "jdk", version: "17.0.10 (Temurin)", home: "D:\\DevTools\\jdk-17", isCurrent: true, source: "scan" },
  { kind: "jdk", version: "8u402 (Temurin)", home: "C:\\Program Files\\Java\\jdk-8", isCurrent: false, source: "scan" },
  { kind: "jdk", version: "21.0.2 (GraalVM)", home: "D:\\DevTools\\graalvm-21", isCurrent: false, source: "envbox" },
  { kind: "node", version: "24.18.0", home: "C:\\Program Files\\nodejs", isCurrent: true, source: "scan" },
  { kind: "node", version: "18.19.1", home: "C:\\Users\\me\\nvm\\v18.19.1", isCurrent: false, source: "scan", manager: "nvm" },
  { kind: "python", version: "3.11.8", home: "C:\\Python311", isCurrent: true, source: "scan" },
  { kind: "python", version: "3.9.13", home: "C:\\Python39", isCurrent: false, source: "scan" },
  { kind: "go", version: "1.22.1", home: "C:\\Program Files\\Go", isCurrent: true, source: "scan" },
];

export const mockSnapshots: Snapshot[] = [
  { id: "snap-3", createdAt: "2026-07-12 19:45", description: "切换 JDK 17 前自动快照" },
  { id: "snap-2", createdAt: "2026-07-12 18:02", description: "清理无效 PATH 前自动快照" },
  { id: "snap-1", createdAt: "2026-07-12 17:30", description: "首次启动初始快照" },
];
