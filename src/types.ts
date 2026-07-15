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
  status: "available" | "missing" | "unresolved" | "networkUnavailable";
  safeToClean: boolean;
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
  manager?: string;
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
  phase:
    | "downloading"
    | "installing"
    | "configuring"
    | "cleaning"
    | "done"
    | "error"
    | "cancelled";
  logLine?: string;
}

export interface Snapshot {
  id: string;
  createdAt: string;
  description: string;
}

export interface SnapshotChange {
  scope: "system" | "user";
  name: string;
  kind: "add" | "modify" | "delete";
  before?: string;
  after?: string;
  sensitive: boolean;
}

export interface SnapshotPreview {
  snapshotId: string;
  description: string;
  createdAt: string;
  changes: SnapshotChange[];
  userChanges: number;
  systemChanges: number;
  requiresElevation: boolean;
}

export interface SnapshotSelection {
  scope: "system" | "user";
  name: string;
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
  unresolvedPaths: number;
  networkPaths: number;
  snapshotIssues: number;
  incompleteInstalls: number;
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
  changes: SnapshotChange[];
}

export interface ProjectVersionHint {
  tool: string;
  version: string;
  source: string;
  status: "current" | "installed" | "missing" | "wrapper" | "declared";
  installedHome?: string;
  currentVersion?: string;
}

export interface ProjectInspection {
  path: string;
  hints: ProjectVersionHint[];
}

export interface ManagedInstall {
  id: string;
  jobId: string;
  snapshotId: string;
  kind: SdkKind;
  distro: string;
  version: string;
  engine: "winget" | "scoop";
  packageId: string;
  requestedLocation: string;
  previousHomes: string[];
  detectedHomes: string[];
  status: "running" | "installed" | "failed" | "cancelled" | "uninstalled";
  installedAt: string;
}
