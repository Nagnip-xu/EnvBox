import { Search, Plus, Upload, Download, RefreshCw, Loader2, ShieldAlert } from "lucide-react";
import { useState } from "react";
import { useStore } from "../store/useStore";
import { api, errorMessage, isTauri, openDialog, saveDialog } from "../lib/tauri";
import type { ImportPreview } from "../types";
import ConfirmModal from "./ConfirmModal";
import Modal from "./Modal";
import { displaySensitiveValue } from "../lib/security";

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
  const [confirmExport, setConfirmExport] = useState(false);
  const [pendingImport, setPendingImport] = useState<{
    path: string;
    preview: ImportPreview;
  } | null>(null);
  const [selectedImports, setSelectedImports] = useState<Set<string>>(new Set());
  const [importing, setImporting] = useState(false);

  async function handleExport() {
    try {
      const path = await saveDialog("envbox-backup.json");
      if (!path) return;
      await api.exportVars(path);
      pushToast(t("toast.exported"), "success");
    } catch (e) {
      pushToast(t("toast.exportFail", { err: errorMessage(e) }), "error");
    }
  }

  async function handleImport() {
    try {
      const path = await openDialog();
      if (!path) return;
      const preview = await api.previewImport(path);
      setPendingImport({ path, preview });
      setSelectedImports(
        new Set(preview.changes.map((change) => `${change.scope}:${change.name.toLowerCase()}`))
      );
    } catch (e) {
      pushToast(t("toast.importFail", { err: errorMessage(e) }), "error");
    }
  }

  async function confirmImport() {
    if (!pendingImport) return;
    const selections = pendingImport.preview.changes
      .filter((change) => selectedImports.has(`${change.scope}:${change.name.toLowerCase()}`))
      .map((change) => ({ scope: change.scope, name: change.name }));
    if (selections.length === 0) return;
    setImporting(true);
    try {
      const n = await api.importVarsSelected(pendingImport.path, selections);
      pushToast(t("toast.imported", { n }), "success");
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.importFail", { err: errorMessage(e) }), "error");
    } finally {
      setImporting(false);
      setPendingImport(null);
    }
  }

  function toggleImport(scope: string, name: string) {
    const key = `${scope}:${name.toLowerCase()}`;
    setSelectedImports((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
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
            aria-label={t("topbar.search")}
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
        <RefreshCw size={16} aria-hidden="true" />
      </button>
      <button
        className="btn-ghost"
        title={t("topbar.import")}
        aria-label={t("topbar.import")}
        onClick={handleImport}
      >
        <Upload size={16} aria-hidden="true" />
      </button>
      <button
        className="btn-ghost"
        title={t("topbar.export")}
        aria-label={t("topbar.export")}
        onClick={() => setConfirmExport(true)}
      >
        <Download size={16} aria-hidden="true" />
      </button>
      <button className="btn-primary" onClick={requestNewVar}>
        <Plus size={16} /> {t("topbar.newVar")}
      </button>
      <ConfirmModal
        open={confirmExport}
        title={t("topbar.exportConfirmTitle")}
        message={t("topbar.exportConfirmMessage")}
        showSnapshotNote={false}
        confirmLabel={t("topbar.export")}
        onConfirm={handleExport}
        onClose={() => setConfirmExport(false)}
      />
      <Modal
        open={!!pendingImport}
        title={t("topbar.importConfirmTitle")}
        onClose={importing ? () => {} : () => setPendingImport(null)}
        width="max-w-3xl"
      >
        {pendingImport && (
          <div className="space-y-4">
            <p className="text-sm leading-relaxed text-neutral-400">
              {t("topbar.importConfirmMessage", {
                user: pendingImport.preview.userCount,
                system: pendingImport.preview.systemCount,
                sensitive: pendingImport.preview.sensitiveCount,
              })}
            </p>
            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                ref={(input) => {
                  if (input) {
                    input.indeterminate =
                      selectedImports.size > 0 &&
                      selectedImports.size < pendingImport.preview.changes.length;
                  }
                }}
                checked={
                  selectedImports.size === pendingImport.preview.changes.length &&
                  pendingImport.preview.changes.length > 0
                }
                onChange={(event) =>
                  setSelectedImports(
                    event.target.checked
                      ? new Set(
                          pendingImport.preview.changes.map(
                            (change) => `${change.scope}:${change.name.toLowerCase()}`
                          )
                        )
                      : new Set()
                  )
                }
              />
              {t("snap.diff.selected", {
                selected: selectedImports.size,
                total: pendingImport.preview.changes.length,
              })}
            </label>
            {pendingImport.preview.changes.length === 0 ? (
              <div className="rounded-lg border border-neutral-800 p-6 text-center text-sm text-neutral-500">
                {t("topbar.importNoChanges")}
              </div>
            ) : (
              <div className="max-h-80 overflow-y-auto rounded-lg border border-neutral-800">
                {pendingImport.preview.changes.map((change) => (
                  <label
                    key={`${change.scope}-${change.name}`}
                    className="flex cursor-pointer gap-3 border-b border-neutral-800 p-3 last:border-b-0 hover:bg-neutral-800/30"
                  >
                    <input
                      type="checkbox"
                      className="mt-1 shrink-0"
                      checked={selectedImports.has(`${change.scope}:${change.name.toLowerCase()}`)}
                      onChange={() => toggleImport(change.scope, change.name)}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="mb-2 flex items-center gap-2">
                        <span className="font-mono text-sm text-neutral-200">{change.name}</span>
                        <span className="tag bg-neutral-800 text-neutral-400">
                          {t(`scope.${change.scope}.short`)}
                        </span>
                        <span className={change.kind === "add" ? "tag tag-emerald" : "tag tag-amber"}>
                          {t(`snap.diff.${change.kind}`)}
                        </span>
                      </div>
                      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2 font-mono text-xs">
                        <span className="truncate rounded bg-neutral-950 px-2 py-1.5 text-neutral-500">
                          {displaySensitiveValue(change.before, change.sensitive)}
                        </span>
                        <span className="text-neutral-600">→</span>
                        <span className="truncate rounded bg-neutral-950 px-2 py-1.5 text-neutral-300">
                          {displaySensitiveValue(change.after, change.sensitive)}
                        </span>
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            )}
            {pendingImport.preview.requiresElevation &&
              pendingImport.preview.changes.some(
                (change) =>
                  change.scope === "system" &&
                  selectedImports.has(`${change.scope}:${change.name.toLowerCase()}`)
              ) && (
                <div className="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-300">
                  <ShieldAlert size={18} className="mt-0.5 shrink-0" aria-hidden="true" />
                  {t("topbar.importAdminRequired")}
                </div>
              )}
            <p className="text-xs text-neutral-500">{t("confirm.snapNote")}</p>
            <div className="flex justify-end gap-2">
              <button
                className="btn-ghost border border-neutral-800"
                disabled={importing}
                onClick={() => setPendingImport(null)}
              >
                {t("common.cancel")}
              </button>
              <button
                className="btn-primary"
                disabled={
                  importing ||
                  selectedImports.size === 0 ||
                  (pendingImport.preview.requiresElevation &&
                    pendingImport.preview.changes.some(
                      (change) =>
                        change.scope === "system" &&
                        selectedImports.has(`${change.scope}:${change.name.toLowerCase()}`)
                    ))
                }
                onClick={confirmImport}
              >
                {importing && <Loader2 size={15} className="animate-spin" aria-hidden="true" />}
                {t("topbar.import")}
              </button>
            </div>
          </div>
        )}
      </Modal>
    </header>
  );
}
