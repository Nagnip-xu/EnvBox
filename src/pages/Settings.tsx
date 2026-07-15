import { useEffect, useState } from "react";
import {
  Globe,
  ShieldCheck,
  ShieldAlert,
  Stethoscope,
  Loader2,
  Terminal,
  Languages,
  Palette,
  Sun,
  Moon,
  Monitor,
  FolderOpen,
  FolderCog,
  Keyboard,
} from "lucide-react";
import type { EngineStatus, HealthReport } from "../types";
import { api, pickDirectory } from "../lib/tauri";
import { useStore } from "../store/useStore";
import { LANGS, type Lang } from "../i18n";
import type { ThemeMode } from "../theme";
import { APP_AUTHOR, APP_CONTACT_EMAIL, APP_VERSION } from "../version";

export default function Settings() {
  const [engines, setEngines] = useState<EngineStatus | null>(null);
  const [report, setReport] = useState<HealthReport | null>(null);
  const [checking, setChecking] = useState(false);
  const { pushToast, lang, setLang, theme, setTheme, installPath, setInstallPath, t, refreshKey } =
    useStore();
  const [mirror, setMirror] = useState(() => localStorage.getItem("envbox.mirror") ?? "rsproxy");
  const [pathDraft, setPathDraft] = useState(installPath);

  useEffect(() => {
    api.engineStatus().then(setEngines);
  }, [refreshKey]);

  async function runHealth() {
    setChecking(true);
    try {
      setReport(await api.healthCheck());
    } finally {
      setChecking(false);
    }
  }

  async function elevate() {
    try {
      const ok = await api.relaunchAsAdmin();
      if (ok) pushToast(t("toast.relaunching"), "info");
    } catch (e) {
      pushToast(t("toast.relaunchFail", { err: `${e}` }), "error");
    }
  }

  async function browsePath() {
    const dir = await pickDirectory();
    if (dir) {
      setPathDraft(dir);
      setInstallPath(dir);
      pushToast(t("toast.saved"), "success");
    }
  }

  const themeOptions: { id: ThemeMode; label: string; icon: typeof Sun }[] = [
    { id: "light", label: t("settings.theme.light"), icon: Sun },
    { id: "dark", label: t("settings.theme.dark"), icon: Moon },
    { id: "system", label: t("settings.theme.system"), icon: Monitor },
  ];

  const shortcuts = [
    { keys: "Ctrl + K", desc: t("settings.shortcuts.palette") },
    { keys: "Ctrl + F", desc: t("settings.shortcuts.search") },
    { keys: "Esc", desc: t("settings.shortcuts.close") },
  ];

  return (
    <div className="grid w-full grid-cols-1 gap-4 xl:grid-cols-2">
      {/* 权限 */}
      <div className="card p-4 xl:col-span-2">
        <div className="flex items-center gap-3">
          {engines?.elevated ? (
            <ShieldCheck size={20} className="text-emerald-400" />
          ) : (
            <ShieldAlert size={20} className="text-amber-400" />
          )}
          <div className="flex-1">
            <div className="text-sm font-medium text-neutral-200">
              {engines?.elevated ? t("settings.perm.title.admin") : t("settings.perm.title.user")}
            </div>
            <div className="text-xs text-neutral-500">{t("settings.perm.desc")}</div>
          </div>
          {!engines?.elevated && (
            <button className="btn-primary" onClick={elevate}>
              <ShieldCheck size={15} /> {t("settings.perm.relaunch")}
            </button>
          )}
        </div>
        <div className="mt-3 flex gap-4 border-t border-neutral-800 pt-3 text-xs">
          <span className={engines?.winget ? "text-emerald-400" : "text-neutral-500"}>
            winget：{engines?.winget ? "✓" : "✗"}
          </span>
          <span className={engines?.scoop ? "text-emerald-400" : "text-neutral-500"}>
            scoop：{engines?.scoop ? "✓" : "✗"}
          </span>
        </div>
      </div>

      {/* 环境体检 */}
      <div className="card p-4 xl:col-span-2">
        <div className="flex items-center gap-3">
          <Stethoscope size={20} className="text-brand-300" />
          <div className="flex-1">
            <div className="text-sm font-medium text-neutral-200">{t("settings.health.title")}</div>
            <div className="text-xs text-neutral-500">{t("settings.health.desc")}</div>
          </div>
          <button className="btn-primary" onClick={runHealth} disabled={checking}>
            {checking ? <Loader2 size={15} className="animate-spin" /> : <Stethoscope size={15} />}{" "}
            {t("settings.health.run")}
          </button>
        </div>
        {report && (
          <div className="mt-4 space-y-2 border-t border-neutral-800 pt-3">
            <div className="grid grid-cols-4 gap-2 text-center">
              <Metric label={t("settings.health.totalVars")} value={report.totalVars} />
              <Metric label={t("settings.health.invalid")} value={report.invalidPaths} bad={report.invalidPaths > 0} />
              <Metric label={t("settings.health.dup")} value={report.duplicatePaths} warn={report.duplicatePaths > 0} />
              <Metric label={t("settings.health.conflict")} value={report.conflicts} warn={report.conflicts > 0} />
            </div>
            <ul className="mt-2 space-y-1">
              {report.issues.map((issue, i) => (
                <li key={i} className="text-sm text-neutral-300">
                  • {issue}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* 默认安装路径 */}
      <div className="card p-4 xl:col-span-2">
        <div className="mb-3 flex items-center gap-2">
          <FolderCog size={18} className="text-brand-300" />
          <div>
            <div className="text-sm font-medium text-neutral-200">
              {t("settings.installPath.title")}
            </div>
            <div className="text-xs text-neutral-500">{t("settings.installPath.desc")}</div>
          </div>
        </div>
        <div className="flex gap-2">
          <input
            value={pathDraft}
            onChange={(e) => setPathDraft(e.target.value)}
            onBlur={() => setInstallPath(pathDraft.trim())}
            placeholder={t("settings.installPath.placeholder")}
            className="flex-1 rounded-lg border border-neutral-800 bg-neutral-900 px-3 py-1.5 font-mono text-sm text-neutral-100 outline-none focus:border-brand-500"
          />
          <button className="btn-ghost border border-neutral-800" onClick={browsePath}>
            <FolderOpen size={15} /> {t("common.browse")}
          </button>
          {pathDraft && (
            <button
              className="btn-ghost border border-neutral-800"
              onClick={() => {
                setPathDraft("");
                setInstallPath("");
              }}
            >
              {t("common.reset")}
            </button>
          )}
        </div>
      </div>

      {/* 下载镜像源 */}
      <Row
        className="xl:col-span-2"
        icon={<Globe size={18} />}
        title={t("settings.mirror.title")}
        desc={t("settings.mirror.desc")}
      >
        <select
          value={mirror}
          onChange={(e) => {
            setMirror(e.target.value);
            localStorage.setItem("envbox.mirror", e.target.value);
            pushToast(t("toast.saved"), "success");
          }}
          className="rounded-lg border border-neutral-800 bg-neutral-900 px-3 py-1.5 text-sm text-neutral-100 outline-none focus:border-brand-500"
        >
          <option value="official">{t("settings.mirror.official")}</option>
          <option value="tuna">清华 TUNA</option>
          <option value="ustc">中科大 USTC</option>
          <option value="rsproxy">rsproxy.cn</option>
        </select>
      </Row>

      {/* 界面语言 */}
      <div className="card p-4">
        <div className="mb-3 flex items-center gap-2">
          <Languages size={18} className="text-brand-300" />
          <div>
            <div className="text-sm font-medium text-neutral-200">{t("settings.lang.title")}</div>
            <div className="text-xs text-neutral-500">{t("settings.lang.desc")}</div>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          {LANGS.map((l) => (
            <SegBtn key={l.id} active={lang === l.id} onClick={() => setLang(l.id as Lang)}>
              {l.label}
            </SegBtn>
          ))}
        </div>
      </div>

      {/* 外观主题 */}
      <div className="card p-4">
        <div className="mb-3 flex items-center gap-2">
          <Palette size={18} className="text-brand-300" />
          <div>
            <div className="text-sm font-medium text-neutral-200">{t("settings.theme.title")}</div>
            <div className="text-xs text-neutral-500">{t("settings.theme.desc")}</div>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          {themeOptions.map((o) => {
            const Icon = o.icon;
            return (
              <SegBtn key={o.id} active={theme === o.id} onClick={() => setTheme(o.id)}>
                <Icon size={15} /> {o.label}
              </SegBtn>
            );
          })}
        </div>
      </div>

      {/* 快捷键 */}
      <div className="card p-4 xl:col-span-2">
        <div className="mb-3 flex items-center gap-2">
          <Keyboard size={18} className="text-brand-300" />
          <div>
            <div className="text-sm font-medium text-neutral-200">{t("settings.shortcuts.title")}</div>
            <div className="text-xs text-neutral-500">{t("settings.shortcuts.desc")}</div>
          </div>
        </div>
        <div className="divide-y divide-neutral-800 rounded-lg border border-neutral-800">
          {shortcuts.map((s) => (
            <div key={s.keys} className="flex items-center justify-between gap-4 px-4 py-3">
              <span className="text-sm text-neutral-300">{s.desc}</span>
              <kbd className="shrink-0 rounded-md border border-neutral-700 bg-neutral-950 px-2.5 py-1 font-mono text-xs text-neutral-200">
                {s.keys}
              </kbd>
            </div>
          ))}
        </div>
      </div>

      {/* 关于 */}
      <div className="card p-4 xl:col-span-2">
        <div className="mb-3 flex items-center gap-2">
          <Terminal size={18} className="text-brand-300" />
          <div className="flex-1">
            <div className="text-sm font-medium text-neutral-200">{t("settings.about.title")}</div>
            <div className="text-xs text-neutral-500">
              {t("settings.about.desc", { version: APP_VERSION })}
            </div>
          </div>
          <span className="shrink-0 text-xs text-neutral-600">MIT</span>
        </div>
        <div className="rounded-lg border border-neutral-800 px-4 py-3">
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-neutral-500">{t("settings.about.author")}</span>
            <span className="text-sm text-neutral-200">
              {APP_AUTHOR}
              <span className="mx-2 text-neutral-600">·</span>
              <a
                href={`mailto:${APP_CONTACT_EMAIL}`}
                className="text-brand-400 transition hover:text-brand-300 hover:underline"
              >
                {APP_CONTACT_EMAIL}
              </a>
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

function SegBtn({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={
        "inline-flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-medium transition " +
        (active
          ? "bg-brand-600 text-white"
          : "border border-neutral-800 bg-neutral-900 text-neutral-300 hover:bg-neutral-800")
      }
    >
      {children}
    </button>
  );
}

function Metric({ label, value, bad, warn }: { label: string; value: number; bad?: boolean; warn?: boolean }) {
  const color = bad ? "text-rose-400" : warn ? "text-amber-400" : "text-neutral-100";
  return (
    <div className="rounded-lg bg-neutral-950 px-2 py-2">
      <div className={`text-xl font-semibold ${color}`}>{value}</div>
      <div className="text-xs text-neutral-500">{label}</div>
    </div>
  );
}

function Row({
  icon,
  title,
  desc,
  children,
  className = "",
}: {
  icon: React.ReactNode;
  title: string;
  desc: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={`card flex items-center gap-4 px-4 py-3 ${className}`}>
      <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-neutral-800 text-neutral-300">
        {icon}
      </div>
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium text-neutral-200">{title}</div>
        <div className="text-xs text-neutral-500">{desc}</div>
      </div>
      {children}
    </div>
  );
}
