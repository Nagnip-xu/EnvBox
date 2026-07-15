import type { EngineStatus, SdkVersion } from "../types";
import { api } from "../lib/tauri";

/** SDK 扫描结果缓存：秒开 + 后台静默更新（stale-while-revalidate） */
let cachedSdks: SdkVersion[] | null = null;
let cachedEngines: EngineStatus | null = null;
interface ScanTask {
  id: number;
  promise: Promise<{ sdks: SdkVersion[]; engines: EngineStatus | null }>;
}

let scanTask: ScanTask | null = null;
let scanGeneration = 0;
let lastScanAt = 0;

export function getCachedSdks(): SdkVersion[] | null {
  return cachedSdks;
}

export function getCachedEngines(): EngineStatus | null {
  return cachedEngines;
}

export function getLastScanAt(): number {
  return lastScanAt;
}

export function isSdkScanning(): boolean {
  return scanTask !== null;
}

export function invalidateSdkCache() {
  scanGeneration += 1;
  scanTask = null;
  cachedSdks = null;
  cachedEngines = null;
  lastScanAt = 0;
}

/** 执行一次扫描；并发调用复用同一 Promise */
export function refreshSdkScan(
  onUpdate?: (sdks: SdkVersion[], engines: EngineStatus | null) => void
): Promise<{ sdks: SdkVersion[]; engines: EngineStatus | null }> {
  if (scanTask) {
    return scanTask.promise.then((r) => {
      onUpdate?.(r.sdks, r.engines);
      return r;
    });
  }

  const id = ++scanGeneration;
  const promise = (async () => {
    const [sdks, engines] = await Promise.all([api.scanSdks(), api.engineStatus()]);
    if (id !== scanGeneration) return { sdks, engines };
    cachedSdks = sdks;
    cachedEngines = engines;
    lastScanAt = Date.now();
    const result = { sdks, engines };
    onUpdate?.(sdks, engines);
    return result;
  })();
  scanTask = { id, promise };

  return promise.finally(() => {
    if (scanTask?.id === id) scanTask = null;
  });
}
