import { useEffect, useMemo, useRef, useState } from "react";
import {
  Check,
  Download,
  Trash2,
  Terminal,
  Loader2,
  Package,
  CircleDot,
  Coffee,
  FileCode,
  Hexagon,
  Zap,
  Cog,
  Hash,
  Gem,
  FileCode2,
  GitBranch,
  Boxes,
  Croissant,
  type LucideIcon,
} from "lucide-react";
import type { EngineStatus, InstallableVersion, JobProgress, SdkKind, SdkVersion } from "../types";
import { api, errorMessage, onJobProgress } from "../lib/tauri";
import { useStore } from "../store/useStore";
import {
  getCachedEngines,
  getCachedSdks,
  refreshSdkScan,
} from "../store/sdkCache";
import Modal from "../components/Modal";
import ConfirmModal from "../components/ConfirmModal";

const KIND_META: Record<SdkKind, { label: string; homeVar: string; icon: LucideIcon }> = {
  jdk: { label: "Java (JDK)", homeVar: "JAVA_HOME", icon: Coffee },
  python: { label: "Python", homeVar: "—", icon: FileCode },
  node: { label: "Node.js", homeVar: "—", icon: Hexagon },
  go: { label: "Go", homeVar: "GOROOT", icon: Zap },
  rust: { label: "Rust", homeVar: "—", icon: Cog },
  dotnet: { label: ".NET SDK", homeVar: "DOTNET_ROOT", icon: Hash },
  ruby: { label: "Ruby", homeVar: "—", icon: Gem },
  php: { label: "PHP", homeVar: "—", icon: FileCode2 },
  git: { label: "Git", homeVar: "—", icon: GitBranch },
  maven: { label: "Apache Maven", homeVar: "MAVEN_HOME", icon: Boxes },
  gradle: { label: "Gradle", homeVar: "GRADLE_HOME", icon: Boxes },
  deno: { label: "Deno", homeVar: "—", icon: Zap },
  bun: { label: "Bun", homeVar: "—", icon: Croissant },
};

const ALL_KINDS: SdkKind[] = [
  "jdk", "python", "node", "go", "rust", "dotnet", "ruby",
  "php", "git", "maven", "gradle", "deno", "bun",
];

/** 自动选择可用的安装引擎：优先 winget，其次 scoop */
function pickEngine(
  item: InstallableVersion,
  engines: EngineStatus | null
): "winget" | "scoop" | null {
  if (engines?.winget && item.engines.includes("winget")) return "winget";
  if (engines?.scoop && item.engines.includes("scoop")) return "scoop";
  return null;
}

/** 切换确认弹窗中显示的版本名称，如 JDK 21 */
function formatSwitchTarget(kind: SdkKind, version: string): string {
  if (kind === "jdk") {
    const m = version.match(/version\s+(\d+)/i);
    return m ? `JDK ${m[1]}` : version;
  }
  return `${KIND_META[kind].label} ${version}`;
}

export default function SdkCenter() {
  const cached = getCachedSdks();
  const [sdks, setSdks] = useState<SdkVersion[]>(cached ?? []);
  const [loading, setLoading] = useState(!cached);
  const [refreshing, setRefreshing] = useState(false);
  const [engines, setEngines] = useState<EngineStatus | null>(getCachedEngines());
  const { page, refreshKey, bumpRefresh, pushToast, t } = useStore();

  const [installKind, setInstallKind] = useState<SdkKind | null>(null);
  const [switchTarget, setSwitchTarget] = useState<SdkVersion | null>(null);
  const [uninstallTarget, setUninstallTarget] = useState<SdkVersion | null>(null);
  const [uninstalling, setUninstalling] = useState<SdkVersion | null>(null);

  // 每次进入 SDK 页或点击全局刷新：先展示缓存，再后台重扫并更新 UI
  useEffect(() => {
    if (page !== "sdk") return;

    let cancelled = false;
    const cached = getCachedSdks();

    if (cached) {
      setSdks(cached);
      setEngines(getCachedEngines());
      setLoading(false);
      setRefreshing(true);
    } else {
      setLoading(true);
      setRefreshing(false);
    }

    refreshSdkScan((nextSdks, nextEngines) => {
      if (cancelled) return;
      setSdks(nextSdks);
      setEngines(nextEngines);
      setRefreshing(false);
      setLoading(false);
    }).catch((error) => {
      if (!cancelled) {
        setRefreshing(false);
        setLoading(false);
        pushToast(t("common.loadFail", { err: errorMessage(error) }), "error");
      }
    });

    return () => {
      cancelled = true;
    };
  }, [page, refreshKey, pushToast, t]);

  const byKind = useMemo(() => {
    const m = new Map<SdkKind, SdkVersion[]>();
    for (const s of sdks) {
      const arr = m.get(s.kind) ?? [];
      arr.push(s);
      m.set(s.kind, arr);
    }
    return m;
  }, [sdks]);
  const installedKinds = useMemo(() => ALL_KINDS.filter((k) => byKind.has(k)), [byKind]);
  const emptyKinds = useMemo(() => ALL_KINDS.filter((k) => !byKind.has(k)), [byKind]);

  async function doSwitch(s: SdkVersion) {
    try {
      const warn = await api.switchSdk(s.kind, s.home);
      pushToast(t("toast.switched", { version: s.version }), "success");
      if (warn && warn.trim()) {
        pushToast(t("toast.switchNote", { warn }), "error");
      }
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.switchFail", { err: errorMessage(e) }), "error");
    }
  }

  async function openTerminal(s: SdkVersion) {
    try {
      await api.openTerminalWith(s.kind, s.home);
      pushToast(t("toast.terminalOpened"), "success");
    } catch (e) {
      pushToast(t("toast.terminalFail", { err: errorMessage(e) }), "error");
    }
  }

  return (
    <div className="space-y-5">
      {loading && sdks.length === 0 && (
        <div className="flex items-center gap-2 text-sm text-neutral-400">
          <Loader2 size={16} className="animate-spin" /> {t("sdk.scanning")}
        </div>
      )}
      {refreshing && sdks.length > 0 && (
        <div className="flex items-center gap-2 text-xs text-neutral-500">
          <Loader2 size={13} className="animate-spin" /> {t("sdk.refreshing")}
        </div>
      )}

      {installedKinds.map((kind) => {
        const items = byKind.get(kind) ?? [];
        const meta = KIND_META[kind];
        const Icon = meta.icon;
        return (
          <section key={kind} className="card p-4">
            <div className="mb-3 flex items-center gap-2">
              <Icon size={16} className="text-brand-300" />
              <h2 className="text-sm font-semibold text-neutral-200">{meta.label}</h2>
              {meta.homeVar !== "—" && (
                <span className="text-xs text-neutral-500">{t("sdk.envVar", { var: meta.homeVar })}</span>
              )}
              <div className="flex-1" />
              <button className="btn-primary !py-1" onClick={() => setInstallKind(kind)}>
                <Download size={15} /> {t("sdk.installNew")}
              </button>
            </div>

            <div className="space-y-2">
              {items.map((s) => (
                <div
                  key={s.home}
                  className={`flex items-center gap-3 rounded-lg border px-3 py-2.5 ${
                    s.isCurrent
                      ? "border-brand-500/50 bg-brand-600/10"
                      : "border-neutral-800 bg-neutral-900/40"
                  }`}
                >
                  <div className="flex h-6 w-6 items-center justify-center">
                    {s.isCurrent ? (
                      <Check size={18} className="text-brand-300" />
                    ) : (
                      <CircleDot size={14} className="text-neutral-700" />
                    )}
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-neutral-100">{s.version}</span>
                      {s.isCurrent && <span className="tag tag-brand">{t("sdk.current")}</span>}
                      {s.manager && (
                        <span className="tag tag-amber">
                          {t("sdk.managedBy", { manager: s.manager })}
                        </span>
                      )}
                    </div>
                    <div className="truncate font-mono text-xs text-neutral-500">{s.home}</div>
                  </div>
                  <div className="flex-1" />
                  <button
                    className="btn-ghost !px-2"
                    title={t("sdk.tempTerminal")}
                    aria-label={t("sdk.tempTerminal")}
                    onClick={() => openTerminal(s)}
                  >
                    <Terminal size={15} />
                  </button>
                  {!s.isCurrent && (
                    <button
                      className="btn-ghost border border-neutral-800 !py-1"
                      onClick={() => setSwitchTarget(s)}
                    >
                      {t("sdk.setCurrent")}
                    </button>
                  )}
                  <button
                    className="btn-ghost !px-2 hover:!text-rose-400"
                    title={t("sdk.uninstall")}
                    aria-label={t("sdk.uninstall")}
                    onClick={() => setUninstallTarget(s)}
                  >
                    <Trash2 size={15} />
                  </button>
                </div>
              ))}
            </div>
          </section>
        );
      })}

      {emptyKinds.length > 0 && (
        <section className="card p-4">
          <div className="mb-3 flex items-center gap-2">
            <Package size={16} className="text-brand-300" />
            <h2 className="text-sm font-semibold text-neutral-200">{t("sdk.more")}</h2>
            <span className="text-xs text-neutral-500">{t("sdk.moreDesc")}</span>
          </div>
          <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
            {emptyKinds.map((kind) => {
              const meta = KIND_META[kind];
              const Icon = meta.icon;
              return (
                <button
                  key={kind}
                  onClick={() => setInstallKind(kind)}
                  className="group flex items-center gap-2.5 rounded-lg border border-neutral-800 bg-neutral-900/40 px-3 py-2.5 text-left transition hover:border-brand-500/50 hover:bg-brand-600/10"
                >
                  <Icon size={16} className="text-neutral-400 group-hover:text-brand-300" />
                  <span className="flex-1 truncate text-sm text-neutral-200">{meta.label}</span>
                  <Download size={14} className="text-neutral-600 group-hover:text-brand-300" />
                </button>
              );
            })}
          </div>
        </section>
      )}

      {installKind && (
        <InstallModal
          kind={installKind}
          engines={engines}
          onClose={() => setInstallKind(null)}
          onDone={() => bumpRefresh()}
        />
      )}

      <ConfirmModal
        open={!!switchTarget}
        title={t("sdk.switchTitle")}
        message={t("sdk.switchMsg", {
          target: switchTarget ? formatSwitchTarget(switchTarget.kind, switchTarget.version) : "",
        }) + (switchTarget?.manager ? `\n\n${t("sdk.managerSwitchTip", { manager: switchTarget.manager })}` : "")}
        danger={!!switchTarget?.manager}
        confirmLabel={t("sdk.setCurrent")}
        onConfirm={() => {
          if (switchTarget) doSwitch(switchTarget);
        }}
        onClose={() => setSwitchTarget(null)}
      />

      <ConfirmModal
        open={!!uninstallTarget}
        title={t("sdk.uninstallTitle")}
        message={t("sdk.uninstallMsg", {
          version: uninstallTarget?.version ?? "",
          home: uninstallTarget?.home ?? "",
        })}
        danger
        confirmLabel={t("sdk.uninstall")}
        onConfirm={() => {
          if (uninstallTarget) {
            setUninstalling(uninstallTarget);
            setUninstallTarget(null);
          }
        }}
        onClose={() => setUninstallTarget(null)}
      />

      {uninstalling && (
        <UninstallModal
          target={uninstalling}
          onClose={() => setUninstalling(null)}
          onDone={() => bumpRefresh()}
        />
      )}
    </div>
  );
}

const DISTRO_NOTE_KEY: Record<string, string> = {
  Temurin: "distro.Temurin",
  Oracle: "distro.Oracle",
  Microsoft: "distro.Microsoft",
  Corretto: "distro.Corretto",
  Zulu: "distro.Zulu",
};

const UNINSTALL_PHASE_KEY: Record<string, string> = {
  downloading: "phase.uninstall.preparing",
  installing: "phase.uninstall.running",
  configuring: "phase.configuring",
  cleaning: "phase.uninstall.cleaning",
  done: "phase.done",
  error: "phase.error",
  cancelled: "phase.cancelled",
};

const INSTALL_PHASE_KEY: Record<string, string> = {
  downloading: "phase.downloading",
  installing: "phase.installing",
  configuring: "phase.configuring",
  cleaning: "phase.cleaning",
  done: "phase.done",
  error: "phase.error",
  cancelled: "phase.cancelled",
};

function UninstallModal({
  target,
  onClose,
  onDone,
}: {
  target: SdkVersion;
  onClose: () => void;
  onDone: () => void;
}) {
  const [phase, setPhase] = useState<string>("installing");
  const [lines, setLines] = useState<string[]>([]);
  const { pushToast, t } = useStore();
  const logRef = useRef<HTMLDivElement>(null);
  const started = useRef(false);
  const jobId = useRef<string | null>(null);
  const [cancelling, setCancelling] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      unlisten = await onJobProgress((p: JobProgress) => {
        if (p.action !== "uninstall" || p.jobId !== jobId.current) return;
        if (p.logLine) setLines((prev) => [...prev, p.logLine!]);
        setPhase(p.phase);
        if (p.phase === "done") {
          pushToast(t("toast.uninstalled", { target: target.version }), "success");
          onDone();
        } else if (p.phase === "error") {
          pushToast(t("toast.uninstallFail", { err: target.version }), "error");
        } else if (p.phase === "cancelled") {
          pushToast(t("toast.jobCancelled"), "info");
        }
      });
      if (!started.current) {
        started.current = true;
        setLines([
          t("sdk.log.uninstallStart", { version: target.version }),
          t("sdk.log.dir", { home: target.home }),
        ]);
        try {
          jobId.current = await api.uninstallSdk(target.kind, target.home);
        } catch (e) {
          const message = errorMessage(e);
          setLines((prev) => [...prev, t("sdk.log.startFail", { err: message })]);
          setPhase("error");
          pushToast(t("toast.uninstallStartFail", { err: message }), "error");
        }
      }
    })();
    return () => unlisten?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    logRef.current?.scrollTo(0, logRef.current.scrollHeight);
  }, [lines]);

  async function cancel() {
    if (!jobId.current) return;
    setCancelling(true);
    try {
      const requested = await api.cancelJob(jobId.current);
      if (!requested) pushToast(t("toast.jobNotRunning"), "info");
    } catch (error) {
      pushToast(t("toast.cancelFail", { err: errorMessage(error) }), "error");
    } finally {
      setCancelling(false);
    }
  }

  const done = phase === "done" || phase === "error" || phase === "cancelled";

  return (
    <Modal
      open
      title={t("sdk.uninstallTitle") + " · " + target.version}
      onClose={done ? onClose : () => {}}
      width="max-w-2xl"
    >
      <div>
        <div className="mb-2 flex items-center gap-2 text-sm" role="status" aria-live="polite">
          {!done && <Loader2 size={16} className="animate-spin text-brand-300" />}
          <span className="font-medium text-neutral-200">
            {UNINSTALL_PHASE_KEY[phase] ? t(UNINSTALL_PHASE_KEY[phase]) : phase}
          </span>
        </div>
        <div
          ref={logRef}
          role="log"
          aria-live="polite"
          aria-label={t("sdk.progressLog")}
          className="h-64 overflow-y-auto rounded-lg border border-neutral-800 bg-neutral-950 p-3 font-mono text-xs leading-relaxed text-neutral-400"
        >
          {lines.map((l, i) => (
            <div key={i} className="whitespace-pre-wrap">
              {l}
            </div>
          ))}
          {lines.length === 0 && <div className="text-neutral-600">{t("sdk.waiting")}</div>}
        </div>
        {done && (
          <div className="mt-4 flex justify-end">
            <button className="btn-primary" onClick={onClose}>
              {t("common.close")}
            </button>
          </div>
        )}
        {!done && (
          <div className="mt-3 flex items-center justify-between gap-3">
            <span className="text-xs text-neutral-500">{t("sdk.uacNote")}</span>
            <button className="btn-ghost border border-neutral-800" disabled={cancelling || !jobId.current} onClick={cancel}>
              {cancelling && <Loader2 size={14} className="animate-spin" aria-hidden="true" />}
              {t("sdk.cancelTask")}
            </button>
          </div>
        )}
      </div>
    </Modal>
  );
}

function InstallModal({
  kind,
  engines,
  onClose,
  onDone,
}: {
  kind: SdkKind;
  engines: EngineStatus | null;
  onClose: () => void;
  onDone: () => void;
}) {
  const [list, setList] = useState<InstallableVersion[]>([]);
  const [distro, setDistro] = useState<string>("");
  const [running, setRunning] = useState(false);
  const [phase, setPhase] = useState<string>("");
  const [lines, setLines] = useState<string[]>([]);
  const { pushToast, installPath, t } = useStore();
  const logRef = useRef<HTMLDivElement>(null);
  const activeJobId = useRef<string | null>(null);
  const [cancelling, setCancelling] = useState(false);

  useEffect(() => {
    api.listInstallable(kind).then((l) => {
      setList(l);
      setDistro(l[0]?.distro ?? "");
    });
  }, [kind]);

  const distros = Array.from(new Set(list.map((i) => i.distro)));
  const shown = list.filter((i) => i.distro === distro);

  useEffect(() => {
    logRef.current?.scrollTo({ top: logRef.current.scrollHeight });
  }, [lines]);

  async function install(item: InstallableVersion, engine: string) {
    setRunning(true);
    setPhase("downloading");
    setLines([]);
    const unlisten = await onJobProgress((p: JobProgress) => {
      if (p.action !== "install" || p.jobId !== activeJobId.current) return;
      if (p.logLine) setLines((prev) => [...prev, p.logLine!]);
      setPhase(p.phase);
      if (p.phase === "done") {
        pushToast(t("toast.installed", { target: p.target }), "success");
        onDone();
        setTimeout(() => {
          unlisten();
          onClose();
        }, 1200);
      } else if (p.phase === "error") {
        pushToast(t("toast.installFail", { target: p.target }), "error");
      } else if (p.phase === "cancelled") {
        pushToast(t("toast.jobCancelled"), "info");
      }
    });
    try {
      activeJobId.current = await api.installSdk(
        kind,
        item.distro,
        item.version,
        engine,
        engine === "winget" ? installPath : ""
      );
    } catch (e) {
      const message = errorMessage(e);
      setLines((prev) => [...prev, t("sdk.log.startFail", { err: message })]);
      setPhase("error");
      pushToast(t("toast.installStartFail", { err: message }), "error");
      unlisten();
    }
  }

  async function cancelInstall() {
    if (!activeJobId.current) return;
    setCancelling(true);
    try {
      const requested = await api.cancelJob(activeJobId.current);
      if (!requested) pushToast(t("toast.jobNotRunning"), "info");
    } catch (error) {
      pushToast(t("toast.cancelFail", { err: errorMessage(error) }), "error");
    } finally {
      setCancelling(false);
    }
  }

  return (
    <Modal open title={t("sdk.installTitle", { label: KIND_META[kind].label })} onClose={running ? () => {} : onClose} width="max-w-2xl">
      {engines && !engines.winget && !engines.scoop && (
        <div className="mb-3 rounded-lg bg-amber-500/10 px-3 py-2 text-xs text-amber-400">
          {t("sdk.noEngine")}
        </div>
      )}

      {!running ? (
        <div className="space-y-3">
          <div className="rounded-lg border border-neutral-800 bg-neutral-950 px-3 py-2.5 text-xs leading-relaxed text-neutral-400">
            <span className="text-neutral-300">{t("sdk.installHelp.lead")}</span>
            {t("sdk.installHelp.body")}
            <span className="mx-1 tag tag-brand">winget</span>
            {t("sdk.installHelp.winget")}
            <span className="mx-1 rounded bg-neutral-800 px-1.5 py-0.5 text-neutral-300">scoop</span>
            {t("sdk.installHelp.scoop")}
          </div>

          {distros.length > 1 && (
            <div className="flex flex-wrap gap-1.5">
              {distros.map((d) => (
                <button
                  key={d}
                  onClick={() => setDistro(d)}
                  className={
                    "rounded-lg px-3 py-1.5 text-xs font-medium transition " +
                    (d === distro
                      ? "bg-brand-600 text-white"
                      : "bg-neutral-800 text-neutral-300 hover:bg-neutral-700")
                  }
                >
                  {d}
                </button>
              ))}
            </div>
          )}

          {DISTRO_NOTE_KEY[distro] && (
            <div className="rounded-lg bg-neutral-800/50 px-3 py-2 text-xs leading-relaxed text-neutral-400">
              {t(DISTRO_NOTE_KEY[distro])}
            </div>
          )}

          {shown.map((item) => {
            const picked = pickEngine(item, engines);
            return (
              <div
                key={item.version}
                className="flex items-center gap-3 rounded-lg border border-neutral-800 px-3 py-2.5"
              >
                <Package size={16} className="text-brand-300" />
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-neutral-100">
                      {item.distro} {item.version}
                    </span>
                    {item.isLts && <span className="tag tag-emerald">LTS</span>}
                  </div>
                  <div className="text-xs text-neutral-500">
                    {picked ? t("sdk.willUse", { engine: picked }) : t("sdk.noEngineShort")}
                  </div>
                </div>
                <div className="flex-1" />
                <button
                  disabled={!picked}
                  onClick={() => picked && install(item, picked)}
                  className="btn-primary !py-1.5 disabled:opacity-40"
                >
                  <Download size={14} /> {t("sdk.install")}
                </button>
              </div>
            );
          })}
        </div>
      ) : (
        <div>
          <div className="mb-2 flex items-center gap-2 text-sm" role="status" aria-live="polite">
            {phase !== "done" && phase !== "error" && phase !== "cancelled" && (
              <Loader2 size={16} className="animate-spin text-brand-300" />
            )}
            <span className="font-medium text-neutral-200">
              {INSTALL_PHASE_KEY[phase] ? t(INSTALL_PHASE_KEY[phase]) : phase}
            </span>
          </div>
          <div
            ref={logRef}
            role="log"
            aria-live="polite"
            aria-label={t("sdk.progressLog")}
            className="h-64 overflow-y-auto rounded-lg border border-neutral-800 bg-neutral-950 p-3 font-mono text-xs leading-relaxed text-neutral-400"
          >
            {lines.map((l, i) => (
              <div key={i} className="whitespace-pre-wrap">
                {l}
              </div>
            ))}
            {lines.length === 0 && <div className="text-neutral-600">{t("sdk.waiting")}</div>}
          </div>
          {(phase === "done" || phase === "error" || phase === "cancelled") && (
            <div className="mt-4 flex justify-end">
              <button className="btn-primary" onClick={onClose}>
                {t("common.close")}
              </button>
            </div>
          )}
          {phase !== "done" && phase !== "error" && phase !== "cancelled" && (
            <div className="mt-4 flex justify-end">
              <button
                className="btn-ghost border border-neutral-800"
                disabled={cancelling || !activeJobId.current}
                onClick={cancelInstall}
              >
                {cancelling && <Loader2 size={14} className="animate-spin" aria-hidden="true" />}
                {t("sdk.cancelTask")}
              </button>
            </div>
          )}
        </div>
      )}
    </Modal>
  );
}
