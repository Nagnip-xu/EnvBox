import type { EngineStatus, SdkVersion } from "../types";
import { api } from "../lib/tauri";

/** SDK 扫描结果缓存：秒开 + 后台静默更新（stale-while-revalidate） */
let cachedSdks: SdkVersion[] | null = null;
let cachedEngines: EngineStatus | null = null;
let scanPromise: Promise<{ sdks: SdkVersion[]; engines: EngineStatus | null }> | null =
  null;
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
  return scanPromise !== null;
}

export function invalidateSdkCache() {
  cachedSdks = null;
  cachedEngines = null;
  lastScanAt = 0;
}

/** 执行一次扫描；并发调用复用同一 Promise */
export function refreshSdkScan(
  onUpdate?: (sdks: SdkVersion[], engines: EngineStatus | null) => void
): Promise<{ sdks: SdkVersion[]; engines: EngineStatus | null }> {
  if (scanPromise) {
    return scanPromise.then((r) => {
      onUpdate?.(r.sdks, r.engines);
      return r;
    });
  }

  scanPromise = (async () => {
    const [sdks, engines] = await Promise.all([api.scanSdks(), api.engineStatus()]);
    cachedSdks = sdks;
    cachedEngines = engines;
    lastScanAt = Date.now();
    const result = { sdks, engines };
    onUpdate?.(sdks, engines);
    return result;
  })();

  return scanPromise.finally(() => {
    scanPromise = null;
  });
}
