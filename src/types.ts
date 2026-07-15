export type EnvScope = "system" | "user" | "process";

export interface EnvVar {
  name: string;
  value: string;
  scope: EnvScope;
  isExpandable: boolean;
  conflictsWith?: EnvScope;
}

export interface PathEntry {
  raw: string;
  resolved: string;
  scope: "system" | "user";
  exists: boolean;
  duplicate: boolean;
  sdkTag?: string;
  enabled: boolean;
}

export type SdkKind =
  | "jdk"
  | "python"
  | "node"
  | "go"
  | "rust"
  | "dotnet"
  | "ruby"
  | "php"
  | "git"
  | "maven"
  | "gradle"
  | "deno"
  | "bun";

export interface SdkVersion {
  kind: SdkKind;
  version: string;
  home: string;
  isCurrent: boolean;
  source: "scan" | "manual" | "envbox";
}

export interface InstallableVersion {
  kind: SdkKind;
  distro: string;
  version: string;
  isLts: boolean;
  engines: ("winget" | "scoop")[];
  wingetId?: string;
  scoopId?: string;
}

export interface JobProgress {
  jobId: string;
  action: "install" | "uninstall";
  target: string;
  phase: "downloading" | "installing" | "configuring" | "cleaning" | "done" | "error";
  logLine?: string;
}

export interface Snapshot {
  id: string;
  createdAt: string;
  description: string;
}

export interface AuditEntry {
  time: string;
  action: string;
  detail: string;
}

export interface HealthReport {
  totalVars: number;
  invalidPaths: number;
  duplicatePaths: number;
  conflicts: number;
  pathLength: number;
  issues: string[];
}

export interface EngineStatus {
  winget: boolean;
  scoop: boolean;
  elevated: boolean;
}

export interface ImportPreview {
  userCount: number;
  systemCount: number;
  sensitiveCount: number;
  requiresElevation: boolean;
}
