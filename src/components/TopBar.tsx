import { Search, Plus, Upload, Download, RefreshCw } from "lucide-react";
import { useStore } from "../store/useStore";
import { api, isTauri, openDialog, saveDialog } from "../lib/tauri";

const TITLE_KEYS: Record<string, string> = {
  dashboard: "nav.dashboard",
  path: "nav.path",
  sdk: "nav.sdk",
  snapshots: "nav.snapshots",
  settings: "nav.settings",
};

export default function TopBar() {
  const { page, search, setSearch, bumpRefresh, pushToast, requestNewVar, t } = useStore();
  const showSearch = page === "dashboard" || page === "path";

  async function handleExport() {
    try {
      const path = await saveDialog("envbox-backup.json");
      if (!path) return;
      await api.exportVars(path);
      pushToast(t("toast.exported"), "success");
    } catch (e) {
      pushToast(t("toast.exportFail", { err: `${e}` }), "error");
    }
  }

  async function handleImport() {
    try {
      const path = await openDialog();
      if (!path) return;
      const n = await api.importVars(path);
      pushToast(t("toast.imported", { n }), "success");
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.importFail", { err: `${e}` }), "error");
    }
  }

  return (
    <header className="flex items-center gap-3 border-b border-neutral-800 px-6 py-3">
      <h1 className="text-sm font-semibold text-neutral-200">{t(TITLE_KEYS[page])}</h1>

      {!isTauri() && (
        <span className="rounded-md bg-amber-500/15 px-2 py-0.5 text-xs font-medium text-amber-400">
          {t("topbar.browserPreview")}
        </span>
      )}

      <div className="flex-1" />

      {showSearch && (
        <div className="relative w-72">
          <Search
            size={15}
            className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-neutral-500"
          />
          <input
            id="envbox-search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t("topbar.search")}
            className="w-full rounded-lg border border-neutral-800 bg-neutral-900 py-1.5 pl-9 pr-3 text-sm outline-none transition-colors placeholder:text-neutral-600 focus:border-brand-500"
          />
        </div>
      )}

      <button
        className="btn-ghost"
        title={t("topbar.refresh")}
        aria-label={t("topbar.refresh")}
        onClick={() => {
          bumpRefresh();
          pushToast(t("toast.refreshing"), "info");
        }}
      >
        <RefreshCw size={16} />
      </button>
      <button
        className="btn-ghost"
        title={t("topbar.import")}
        aria-label={t("topbar.import")}
        onClick={handleImport}
      >
        <Upload size={16} />
      </button>
      <button
        className="btn-ghost"
        title={t("topbar.export")}
        aria-label={t("topbar.export")}
        onClick={handleExport}
      >
        <Download size={16} />
      </button>
      <button className="btn-primary" onClick={requestNewVar}>
        <Plus size={16} /> {t("topbar.newVar")}
      </button>
    </header>
  );
}
