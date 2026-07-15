import type {
  AuditEntry,
  EngineStatus,
  EnvVar,
  HealthReport,
  InstallableVersion,
  ImportPreview,
  JobProgress,
  PathEntry,
  SdkKind,
  SdkVersion,
  Snapshot,
} from "../types";
import { mockEnvVars, mockPathEntries, mockSdks, mockSnapshots } from "./mock";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

class PreviewError extends Error {
  constructor() {
    super("浏览器预览模式：写操作请在桌面应用中进行");
    this.name = "PreviewError";
  }
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export const api = {
  // 读取
  async listEnvVars(): Promise<EnvVar[]> {
    return isTauri() ? invoke("list_env_vars") : mockEnvVars;
  },
  async getPathEntries(): Promise<PathEntry[]> {
    return isTauri() ? invoke("get_path_entries") : mockPathEntries;
  },
  async scanOrphans(): Promise<PathEntry[]> {
    return isTauri() ? invoke("scan_orphans") : mockPathEntries.filter((entry) => !entry.exists);
  },
  async scanSdks(): Promise<SdkVersion[]> {
    return isTauri() ? invoke("scan_sdks") : mockSdks;
  },
  async listSnapshots(): Promise<Snapshot[]> {
    return isTauri() ? invoke("list_snapshots") : mockSnapshots;
  },
  async listAudit(): Promise<AuditEntry[]> {
    return isTauri() ? invoke("list_audit") : [];
  },
  async engineStatus(): Promise<EngineStatus> {
    return isTauri()
      ? invoke("engine_status")
      : { winget: true, scoop: false, elevated: false };
  },
  async isElevated(): Promise<boolean> {
    return isTauri() ? invoke("is_elevated") : false;
  },
  async listInstallable(kind: SdkKind): Promise<InstallableVersion[]> {
    return isTauri() ? invoke("list_installable", { kind }) : [];
  },
  async healthCheck(): Promise<HealthReport> {
    if (!isTauri())
      return {
        totalVars: mockEnvVars.length,
        invalidPaths: 1,
        duplicatePaths: 1,
        conflicts: 1,
        pathLength: 480,
        issues: ["浏览器预览：这是演示数据"],
      };
    return invoke("health_check");
  },

  // 写入
  async setEnvVar(scope: string, name: string, value: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("set_env_var", { scope, name, value });
  },
  async deleteEnvVar(scope: string, name: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("delete_env_var", { scope, name });
  },
  async savePath(scope: string, entries: string[]): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("save_path", { scope, entries });
  },
  async cleanInvalidPath(scope: string): Promise<number> {
    if (!isTauri()) throw new PreviewError();
    return invoke("clean_invalid_path", { scope });
  },
  async dedupePath(scope: string): Promise<number> {
    if (!isTauri()) throw new PreviewError();
    return invoke("dedupe_path", { scope });
  },
  async switchSdk(kind: SdkKind, home: string): Promise<string> {
    if (!isTauri()) throw new PreviewError();
    return invoke("switch_sdk", { kind, home });
  },
  async installSdk(
    kind: SdkKind,
    distro: string,
    version: string,
    engine: string,
    location = ""
  ): Promise<string> {
    if (!isTauri()) throw new PreviewError();
    return invoke("install_sdk", { kind, distro, version, engine, location });
  },
  async uninstallSdk(kind: SdkKind, home: string): Promise<string> {
    if (!isTauri()) throw new PreviewError();
    return invoke("uninstall_sdk", { kind, home });
  },
  async createSnapshot(description: string): Promise<Snapshot> {
    if (!isTauri()) throw new PreviewError();
    return invoke("create_snapshot", { description });
  },
  async restoreSnapshot(id: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("restore_snapshot", { id });
  },
  async deleteSnapshot(id: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("delete_snapshot", { id });
  },
  async pruneSnapshots(days: number): Promise<number> {
    if (!isTauri()) return 0;
    return invoke("prune_snapshots", { days });
  },
  async openTerminalWith(kind: SdkKind, home: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("open_terminal_with", { kind, home });
  },
  async relaunchAsAdmin(): Promise<boolean> {
    if (!isTauri()) throw new PreviewError();
    return invoke("relaunch_as_admin");
  },
  async exportVars(path: string): Promise<void> {
    if (!isTauri()) throw new PreviewError();
    return invoke("export_vars", { path });
  },
  async importVars(path: string): Promise<number> {
    if (!isTauri()) throw new PreviewError();
    return invoke("import_vars", { path });
  },
  async previewImport(path: string): Promise<ImportPreview> {
    if (!isTauri()) {
      return { userCount: 0, systemCount: 0, sensitiveCount: 0, requiresElevation: false };
    }
    return invoke("preview_import", { path });
  },
};

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error ?? "Unknown error");
}

/** 订阅安装/卸载进度事件，返回取消订阅函数 */
export async function onJobProgress(
  cb: (p: JobProgress) => void
): Promise<() => void> {
  if (!isTauri()) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen<JobProgress>("job://progress", (e) => cb(e.payload));
}

/** 文件保存对话框（导出用） */
export async function saveDialog(defaultName: string): Promise<string | null> {
  if (!isTauri()) return null;
  const { save } = await import("@tauri-apps/plugin-dialog");
  return save({
    defaultPath: defaultName,
    filters: [
      { name: "JSON", extensions: ["json"] },
      { name: "Env", extensions: ["env"] },
    ],
  });
}

/** 文件打开对话框（导入用） */
export async function openDialog(): Promise<string | null> {
  if (!isTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({
    multiple: false,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  return typeof res === "string" ? res : null;
}

/** 目录选择对话框（自定义安装路径用） */
export async function pickDirectory(): Promise<string | null> {
  if (!isTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({ directory: true, multiple: false });
  return typeof res === "string" ? res : null;
}
