import { useEffect, useState } from "react";
import { History, RotateCcw, Camera, ScrollText, Trash2, Clock } from "lucide-react";
import type { AuditEntry, Snapshot } from "../types";
import { api, errorMessage } from "../lib/tauri";
import { useStore } from "../store/useStore";
import ConfirmModal from "../components/ConfirmModal";
import Modal from "../components/Modal";

export default function Snapshots() {
  const [snaps, setSnaps] = useState<Snapshot[]>([]);
  const [audit, setAudit] = useState<AuditEntry[]>([]);
  const [tab, setTab] = useState<"snaps" | "audit">("snaps");
  const [restore, setRestore] = useState<Snapshot | null>(null);
  const [del, setDel] = useState<Snapshot | null>(null);
  const [creating, setCreating] = useState(false);
  const [snapName, setSnapName] = useState("");
  const [retention, setRetention] = useState<number>(() =>
    Number(localStorage.getItem("envbox.snapRetention") ?? "0")
  );
  const { refreshKey, bumpRefresh, pushToast, t } = useStore();

  useEffect(() => {
    api.listSnapshots().then(setSnaps);
    api.listAudit().then(setAudit);
  }, [refreshKey]);

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
    if (!restore) return;
    try {
      await api.restoreSnapshot(restore.id);
      pushToast(t("toast.snapRestored"), "success");
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.snapRestoreFail", { err: errorMessage(e) }), "error");
    }
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
          {snaps.length === 0 && (
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
              <button className="btn-ghost border border-neutral-800 !py-1" onClick={() => setRestore(s)}>
                <RotateCcw size={15} /> {t("snap.restore")}
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
          {audit.length === 0 && (
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

      <ConfirmModal
        open={!!restore}
        title={t("snap.restoreTitle")}
        message={t("snap.restoreMsg", {
          desc: restore?.description ?? "",
          time: restore?.createdAt ?? "",
        })}
        danger
        confirmLabel={t("snap.restore")}
        onConfirm={doRestore}
        onClose={() => setRestore(null)}
      />

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
