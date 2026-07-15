import { useEffect, useState } from "react";
import {
  History,
  RotateCcw,
  Camera,
  ScrollText,
  Trash2,
  Clock,
  Loader2,
  ShieldAlert,
} from "lucide-react";
import type { AuditEntry, Snapshot, SnapshotPreview } from "../types";
import { api, errorMessage } from "../lib/tauri";
import { useStore } from "../store/useStore";
import ConfirmModal from "../components/ConfirmModal";
import Modal from "../components/Modal";
import { displaySensitiveValue } from "../lib/security";

export default function Snapshots() {
  const [snaps, setSnaps] = useState<Snapshot[]>([]);
  const [audit, setAudit] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<"snaps" | "audit">("snaps");
  const [restorePreview, setRestorePreview] = useState<SnapshotPreview | null>(null);
  const [previewingId, setPreviewingId] = useState<string | null>(null);
  const [restoring, setRestoring] = useState(false);
  const [selectedChanges, setSelectedChanges] = useState<Set<string>>(new Set());
  const [del, setDel] = useState<Snapshot | null>(null);
  const [creating, setCreating] = useState(false);
  const [snapName, setSnapName] = useState("");
  const [retention, setRetention] = useState<number>(() => {
    const stored = Number(localStorage.getItem("envbox.snapRetention") ?? "0");
    return [0, 7, 30, 90].includes(stored) ? stored : 0;
  });
  const { refreshKey, bumpRefresh, pushToast, t } = useStore();

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    Promise.all([api.listSnapshots(), api.listAudit()])
      .then(([nextSnaps, nextAudit]) => {
        if (cancelled) return;
        setSnaps(nextSnaps);
        setAudit(nextAudit);
      })
      .catch((error) => !cancelled && pushToast(t("common.loadFail", { err: errorMessage(error) }), "error"))
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [refreshKey, pushToast, t]);

  useEffect(() => {
    if (retention > 0) {
      api.pruneSnapshots(retention).then((n) => {
        if (n > 0) {
          pushToast(t("toast.snapPruned", { n }), "info");
          bumpRefresh();
        }
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [retention]);

  function changeRetention(days: number) {
    setRetention(days);
    localStorage.setItem("envbox.snapRetention", String(days));
  }

  async function doDelete() {
    if (!del) return;
    try {
      await api.deleteSnapshot(del.id);
      pushToast(t("toast.snapDeleted"), "success");
      setDel(null);
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.snapDeleteFail", { err: errorMessage(e) }), "error");
    }
  }

  function openCreate() {
    const now = new Date();
    const stamp = `${String(now.getMonth() + 1).padStart(2, "0")}-${String(
      now.getDate()
    ).padStart(2, "0")} ${String(now.getHours()).padStart(2, "0")}:${String(
      now.getMinutes()
    ).padStart(2, "0")}`;
    setSnapName(t("snap.defaultName", { stamp }));
    setCreating(true);
  }

  async function createSnap() {
    const name = snapName.trim() || t("snap.create");
    try {
      await api.createSnapshot(name);
      pushToast(t("toast.snapCreated"), "success");
      setCreating(false);
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.snapCreateFail", { err: errorMessage(e) }), "error");
    }
  }

  async function doRestore() {
    if (!restorePreview) return;
    const selections = restorePreview.changes
      .filter((change) => selectedChanges.has(`${change.scope}:${change.name.toLowerCase()}`))
      .map((change) => ({ scope: change.scope, name: change.name }));
    if (selections.length === 0) return;
    setRestoring(true);
    try {
      await api.restoreSnapshotSelected(restorePreview.snapshotId, selections);
      pushToast(t("toast.snapRestored"), "success");
      setRestorePreview(null);
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.snapRestoreFail", { err: errorMessage(e) }), "error");
    } finally {
      setRestoring(false);
    }
  }

  async function openRestore(snapshot: Snapshot) {
    setPreviewingId(snapshot.id);
    try {
      const preview = await api.previewSnapshotRestore(snapshot.id);
      setRestorePreview(preview);
      setSelectedChanges(
        new Set(preview.changes.map((change) => `${change.scope}:${change.name.toLowerCase()}`))
      );
    } catch (e) {
      pushToast(t("toast.snapRestoreFail", { err: errorMessage(e) }), "error");
    } finally {
      setPreviewingId(null);
    }
  }

  async function relaunchAdmin() {
    try {
      await api.relaunchAsAdmin();
    } catch (e) {
      pushToast(t("toast.relaunchFail", { err: errorMessage(e) }), "error");
    }
  }

  function toggleChange(scope: string, name: string) {
    const key = `${scope}:${name.toLowerCase()}`;
    setSelectedChanges((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <button className="btn-primary" onClick={openCreate}>
          <Camera size={16} /> {t("snap.create")}
        </button>
        <p className="flex-1 text-xs text-neutral-500">{t("snap.autoTip")}</p>
        <div className="flex items-center gap-2 rounded-lg border border-neutral-800 px-3 py-1.5">
          <Clock size={14} className="text-neutral-500" />
          <span className="text-xs text-neutral-400">{t("snap.autoClean")}</span>
          <select
            value={retention}
            onChange={(e) => changeRetention(Number(e.target.value))}
            className="bg-transparent text-xs text-neutral-200 outline-none"
          >
            <option value={0}>{t("snap.retention.never")}</option>
            <option value={7}>{t("snap.retention.days", { n: 7 })}</option>
            <option value={30}>{t("snap.retention.days", { n: 30 })}</option>
            <option value={90}>{t("snap.retention.days", { n: 90 })}</option>
          </select>
        </div>
      </div>

      <div className="flex gap-1 border-b border-neutral-800">
        <TabBtn active={tab === "snaps"} onClick={() => setTab("snaps")} icon={<History size={15} />}>
          {t("snap.tab.snaps", { n: snaps.length })}
        </TabBtn>
        <TabBtn active={tab === "audit"} onClick={() => setTab("audit")} icon={<ScrollText size={15} />}>
          {t("snap.tab.audit", { n: audit.length })}
        </TabBtn>
      </div>

      {tab === "snaps" ? (
        <div className="card divide-y divide-neutral-800">
          {loading && (
            <div className="px-4 py-6 text-center text-sm text-neutral-500">{t("common.loading")}</div>
          )}
          {!loading && snaps.length === 0 && (
            <div className="px-4 py-6 text-center text-sm text-neutral-600">{t("snap.empty")}</div>
          )}
          {snaps.map((s) => (
            <div key={s.id} className="flex items-center gap-3 px-4 py-3 hover:bg-neutral-800/40">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-neutral-800">
                <History size={16} className="text-neutral-400" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm text-neutral-200">{s.description}</div>
                <div className="text-xs text-neutral-500">{s.createdAt}</div>
              </div>
              <button
                className="btn-ghost border border-neutral-800 !py-1"
                disabled={previewingId === s.id}
                onClick={() => openRestore(s)}
              >
                {previewingId === s.id ? (
                  <Loader2 size={15} className="animate-spin" aria-hidden="true" />
                ) : (
                  <RotateCcw size={15} aria-hidden="true" />
                )}
                {t("snap.restore")}
              </button>
              <button
                className="btn-ghost !px-2 !py-1 hover:!text-rose-400"
                title={t("snap.delTip")}
                aria-label={t("snap.delTip")}
                onClick={() => setDel(s)}
              >
                <Trash2 size={15} />
              </button>
            </div>
          ))}
        </div>
      ) : (
        <div className="card divide-y divide-neutral-800">
          {loading && (
            <div className="px-4 py-6 text-center text-sm text-neutral-500">{t("common.loading")}</div>
          )}
          {!loading && audit.length === 0 && (
            <div className="px-4 py-6 text-center text-sm text-neutral-600">{t("snap.auditEmpty")}</div>
          )}
          {audit.map((a, i) => (
            <div key={i} className="flex items-center gap-3 px-4 py-2.5">
              <span className="w-40 shrink-0 font-mono text-xs text-neutral-500">{a.time}</span>
              <span className="w-24 shrink-0 rounded bg-neutral-800 px-2 py-0.5 text-center text-xs text-neutral-300">
                {a.action}
              </span>
              <span className="min-w-0 flex-1 truncate text-sm text-neutral-400">{a.detail}</span>
            </div>
          ))}
        </div>
      )}

      <Modal open={creating} title={t("snap.createTitle")} onClose={() => setCreating(false)}>
        <div className="space-y-4">
          <div>
            <label className="mb-1 block text-xs text-neutral-400">{t("snap.nameLabel")}</label>
            <input
              autoFocus
              value={snapName}
              onChange={(e) => setSnapName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && createSnap()}
              placeholder={t("snap.namePlaceholder")}
              className="w-full rounded-lg border border-neutral-800 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-brand-500"
            />
            <p className="mt-1.5 text-xs text-neutral-500">{t("snap.nameDesc")}</p>
          </div>
          <div className="flex justify-end gap-2 pt-1">
            <button className="btn-ghost border border-neutral-800" onClick={() => setCreating(false)}>
              {t("common.cancel")}
            </button>
            <button className="btn-primary" onClick={createSnap}>
              <Camera size={15} /> {t("snap.createBtn")}
            </button>
          </div>
        </div>
      </Modal>

      <Modal
        open={!!restorePreview}
        title={t("snap.restoreTitle")}
        onClose={restoring ? () => {} : () => setRestorePreview(null)}
        width="max-w-3xl"
      >
        {restorePreview && (
          <div className="space-y-4">
            <div>
              <p className="text-sm font-medium text-neutral-200">{restorePreview.description}</p>
              <p className="mt-1 text-xs text-neutral-500">{restorePreview.createdAt}</p>
            </div>

            <div className="flex flex-wrap gap-2 text-xs">
              <span className="tag bg-neutral-800 text-neutral-300">
                {t("snap.diff.total", { n: restorePreview.changes.length })}
              </span>
              <span className="tag tag-brand">
                {t("snap.diff.user", { n: restorePreview.userChanges })}
              </span>
              <span className="tag tag-amber">
                {t("snap.diff.system", { n: restorePreview.systemChanges })}
              </span>
            </div>

            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                ref={(input) => {
                  if (input) {
                    input.indeterminate =
                      selectedChanges.size > 0 &&
                      selectedChanges.size < restorePreview.changes.length;
                  }
                }}
                checked={selectedChanges.size === restorePreview.changes.length && restorePreview.changes.length > 0}
                onChange={(event) =>
                  setSelectedChanges(
                    event.target.checked
                      ? new Set(
                          restorePreview.changes.map(
                            (change) => `${change.scope}:${change.name.toLowerCase()}`
                          )
                        )
                      : new Set()
                  )
                }
              />
              {t("snap.diff.selected", { selected: selectedChanges.size, total: restorePreview.changes.length })}
            </label>

            {restorePreview.requiresElevation &&
              restorePreview.changes.some(
                (change) =>
                  change.scope === "system" &&
                  selectedChanges.has(`${change.scope}:${change.name.toLowerCase()}`)
              ) && (
              <div className="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-300">
                <ShieldAlert size={18} className="mt-0.5 shrink-0" aria-hidden="true" />
                <span>{t("snap.diff.adminRequired")}</span>
              </div>
            )}

            <div className="max-h-80 overflow-y-auto rounded-lg border border-neutral-800">
              {restorePreview.changes.length === 0 ? (
                <div className="p-6 text-center text-sm text-neutral-500">{t("snap.diff.none")}</div>
              ) : (
                restorePreview.changes.map((change) => (
                  <label
                    key={`${change.scope}-${change.name}`}
                    className="flex cursor-pointer gap-3 border-b border-neutral-800 p-3 last:border-b-0 hover:bg-neutral-800/30"
                  >
                    <input
                      type="checkbox"
                      className="mt-1 shrink-0"
                      checked={selectedChanges.has(`${change.scope}:${change.name.toLowerCase()}`)}
                      onChange={() => toggleChange(change.scope, change.name)}
                    />
                    <div className="min-w-0 flex-1">
                    <div className="mb-2 flex items-center gap-2">
                      <span className="font-mono text-sm text-neutral-200">{change.name}</span>
                      <span className="tag bg-neutral-800 text-neutral-400">
                        {t(`scope.${change.scope}.short`)}
                      </span>
                      <span
                        className={`tag ${
                          change.kind === "delete"
                            ? "tag-rose"
                            : change.kind === "add"
                              ? "tag-emerald"
                              : "tag-amber"
                        }`}
                      >
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
                ))
              )}
            </div>

            <p className="text-xs text-neutral-500">{t("confirm.snapNote")}</p>
            <div className="flex justify-end gap-2">
              <button
                className="btn-ghost border border-neutral-800"
                disabled={restoring}
                onClick={() => setRestorePreview(null)}
              >
                {t("common.cancel")}
              </button>
              {restorePreview.requiresElevation &&
              restorePreview.changes.some(
                (change) =>
                  change.scope === "system" &&
                  selectedChanges.has(`${change.scope}:${change.name.toLowerCase()}`)
              ) ? (
                <button className="btn-primary" onClick={relaunchAdmin}>
                  <ShieldAlert size={15} aria-hidden="true" />
                  {t("settings.perm.relaunch")}
                </button>
              ) : (
                <button
                  className="btn bg-rose-600 text-white hover:bg-rose-500"
                  disabled={restoring || selectedChanges.size === 0}
                  onClick={doRestore}
                >
                  {restoring ? (
                    <Loader2 size={15} className="animate-spin" aria-hidden="true" />
                  ) : (
                    <RotateCcw size={15} aria-hidden="true" />
                  )}
                  {t("snap.restore")}
                </button>
              )}
            </div>
          </div>
        )}
      </Modal>

      <ConfirmModal
        open={!!del}
        title={t("snap.delTitle")}
        message={t("snap.delMsg", {
          desc: del?.description ?? "",
          time: del?.createdAt ?? "",
        })}
        danger
        confirmLabel={t("common.delete")}
        onConfirm={doDelete}
        onClose={() => setDel(null)}
      />
    </div>
  );
}

function TabBtn({
  active,
  onClick,
  icon,
  children,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 border-b-2 px-3 py-2 text-sm font-medium transition-colors ${
        active
          ? "border-brand-500 text-brand-300"
          : "border-transparent text-neutral-500 hover:text-neutral-300"
      }`}
    >
      {icon}
      {children}
    </button>
  );
}
