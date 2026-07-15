import { useEffect, useMemo, useState } from "react";
import {
  Trash2,
  CheckCircle2,
  XCircle,
  Copy,
  Tag,
  Sparkles,
  Layers,
  GripVertical,
} from "lucide-react";
import type { PathEntry } from "../types";
import { api } from "../lib/tauri";
import { useStore } from "../store/useStore";
import ConfirmModal from "../components/ConfirmModal";

export default function PathManager() {
  const [entries, setEntries] = useState<PathEntry[]>([]);
  const { search, refreshKey, bumpRefresh, pushToast, t } = useStore();
  const [delIdx, setDelIdx] = useState<number | null>(null);
  const [confirmAction, setConfirmAction] = useState<null | "clean" | "dedupe">(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [overIdx, setOverIdx] = useState<number | null>(null);

  useEffect(() => {
    api.getPathEntries().then(setEntries);
  }, [refreshKey]);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return entries
      .map((e, i) => ({ e, i }))
      .filter(({ e }) => !q || e.resolved.toLowerCase().includes(q));
  }, [entries, search]);

  const { invalid, dup, totalLen } = useMemo(() => {
    let invalid = 0;
    let dup = 0;
    let totalLen = 0;
    entries.forEach((e, i) => {
      if (!e.exists) invalid++;
      if (e.duplicate) dup++;
      totalLen += e.raw.length + (i > 0 ? 1 : 0);
    });
    return { invalid, dup, totalLen };
  }, [entries]);
  const canReorder = !search.trim();

  async function runBoth(
    fn: (scope: string) => Promise<number>,
    label: string
  ) {
    let total = 0;
    const errs: string[] = [];
    for (const scope of ["user", "system"]) {
      try {
        total += await fn(scope);
      } catch (e) {
        errs.push(`${scope}: ${e}`);
      }
    }
    if (total > 0) pushToast(t("path.toast.done", { label, n: total }), "success");
    else pushToast(t("path.toast.none", { label }), "info");
    if (errs.some((m) => isPermErr(m))) {
      pushToast(t("path.sysPermErrShort"), "error", {
        label: t("settings.perm.relaunch"),
        onClick: relaunchAdmin,
      });
    } else if (errs.length) {
      pushToast(t("path.toast.partFail", { n: errs.length }), "info");
    }
    bumpRefresh();
  }

  function isPermErr(msg: string): boolean {
    return /os error 5|拒绝访问|denied/i.test(msg);
  }

  async function relaunchAdmin() {
    try {
      await api.relaunchAsAdmin();
    } catch (e) {
      pushToast(t("toast.relaunchFail", { err: `${e}` }), "error");
    }
  }

  function reportScopeErr(scope: string, e: unknown, prefix: string) {
    const msg = `${e}`;
    if (scope === "system" && isPermErr(msg)) {
      pushToast(t("path.sysPermErr"), "error", {
        label: t("settings.perm.relaunch"),
        onClick: relaunchAdmin,
      });
    } else {
      pushToast(`${prefix}：${msg}`, "error");
    }
  }

  async function reorder(fromIdx: number, toIdx: number) {
    if (fromIdx === toIdx) return;
    const from = entries[fromIdx];
    const to = entries[toIdx];
    if (!from || !to || from.scope !== to.scope) return;
    const idxs = entries
      .map((e, i) => ({ e, i }))
      .filter((x) => x.e.scope === from.scope)
      .map((x) => x.i);
    const raws = idxs.map((i) => entries[i].raw);
    const fromPos = idxs.indexOf(fromIdx);
    const toPos = idxs.indexOf(toIdx);
    const [moved] = raws.splice(fromPos, 1);
    raws.splice(toPos, 0, moved);
    try {
      await api.savePath(from.scope, raws);
      bumpRefresh();
    } catch (e) {
      reportScopeErr(from.scope, e, t("path.reorderFail"));
    }
  }

  async function deleteEntry(globalIdx: number) {
    const target = entries[globalIdx];
    const remaining = entries
      .filter((_, i) => i !== globalIdx)
      .filter((e) => e.scope === target.scope)
      .map((e) => e.raw);
    try {
      await api.savePath(target.scope, remaining);
      pushToast(t("path.toast.deleted"), "success");
      bumpRefresh();
    } catch (e) {
      reportScopeErr(target.scope, e, t("path.delFail"));
    }
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-4 gap-3">
        <Stat label={t("path.stat.total")} value={entries.length} />
        <Stat label={t("path.stat.invalid")} value={invalid} tone={invalid ? "bad" : "ok"} />
        <Stat label={t("path.stat.dup")} value={dup} tone={dup ? "warn" : "ok"} />
        <Stat
          label={t("path.stat.length")}
          value={totalLen}
          suffix={t("path.unit.chars")}
          tone={totalLen > 2048 ? "warn" : "neutral"}
        />
      </div>

      <div className="flex gap-2">
        <button
          className="btn-ghost border border-neutral-800"
          onClick={() => setConfirmAction("clean")}
        >
          <Sparkles size={15} /> {t("path.clean")}
        </button>
        <button
          className="btn-ghost border border-neutral-800"
          onClick={() => setConfirmAction("dedupe")}
        >
          <Layers size={15} /> {t("path.dedupe")}
        </button>
      </div>

      {canReorder && (
        <p className="-mt-1 text-xs text-neutral-500">{t("path.reorderTip")}</p>
      )}

      <div className="card divide-y divide-neutral-800">
        {filtered.map(({ e, i }) => (
          <div
            key={i}
            draggable={canReorder}
            onDragStart={() => canReorder && setDragIdx(i)}
            onDragEnd={() => {
              setDragIdx(null);
              setOverIdx(null);
            }}
            onDragOver={(ev) => {
              if (dragIdx === null || entries[dragIdx]?.scope !== e.scope) return;
              ev.preventDefault();
              setOverIdx(i);
            }}
            onDrop={(ev) => {
              ev.preventDefault();
              if (dragIdx !== null) reorder(dragIdx, i);
              setDragIdx(null);
              setOverIdx(null);
            }}
            className={`group flex items-center gap-2 px-4 py-2.5 transition-colors hover:bg-neutral-800/40 ${
              dragIdx === i ? "opacity-40" : ""
            } ${overIdx === i && dragIdx !== i ? "bg-brand-600/10 ring-1 ring-inset ring-brand-500/50" : ""}`}
          >
            {canReorder ? (
              <GripVertical
                size={15}
                className="shrink-0 cursor-grab text-neutral-600 group-hover:text-neutral-400 active:cursor-grabbing"
              />
            ) : (
              <span className="w-[15px] shrink-0" />
            )}
            {e.exists ? (
              <CheckCircle2 size={16} className="shrink-0 text-emerald-400" />
            ) : (
              <XCircle size={16} className="shrink-0 text-rose-400" />
            )}
            <span
              className={`min-w-0 flex-1 truncate font-mono text-sm ${
                e.exists ? "text-neutral-300" : "text-rose-400/80 line-through"
              }`}
              title={e.resolved}
            >
              {e.raw}
            </span>

            {e.sdkTag && (
              <span className="tag tag-brand">
                <Tag size={11} /> {e.sdkTag}
              </span>
            )}
            {e.duplicate && (
              <span className="tag tag-amber">
                <Copy size={11} /> {t("path.dupBadge")}
              </span>
            )}
            <span className={`tag ${e.scope === "system" ? "tag-rose" : "tag-emerald"}`}>
              {e.scope === "system" ? t("scope.system.short") : t("scope.user.short")}
            </span>
            <button
              className="btn-ghost !px-2 opacity-0 transition-opacity hover:!text-rose-400 group-hover:opacity-100"
              title={t("common.delete")}
              aria-label={t("common.delete")}
              onClick={() => setDelIdx(i)}
            >
              <Trash2 size={15} />
            </button>
          </div>
        ))}
        {filtered.length === 0 && (
          <div className="px-4 py-6 text-center text-sm text-neutral-600">{t("path.noMatch")}</div>
        )}
      </div>

      <ConfirmModal
        open={confirmAction !== null}
        title={confirmAction === "clean" ? t("path.cleanTitle") : t("path.dedupeTitle")}
        message={
          confirmAction === "clean"
            ? t("path.cleanMsg", { n: invalid })
            : t("path.dedupeMsg", { n: dup })
        }
        confirmLabel={t("common.run")}
        onConfirm={() => {
          if (confirmAction === "clean") runBoth(api.cleanInvalidPath, t("path.clean"));
          else if (confirmAction === "dedupe") runBoth(api.dedupePath, t("path.dedupe"));
        }}
        onClose={() => setConfirmAction(null)}
      />

      <ConfirmModal
        open={delIdx !== null}
        title={t("path.delTitle")}
        message={t("path.delMsg", {
          scope:
            delIdx !== null && entries[delIdx]?.scope === "system"
              ? t("scope.system.short")
              : t("scope.user.short"),
          raw: delIdx !== null ? entries[delIdx]?.raw ?? "" : "",
        })}
        danger
        confirmLabel={t("common.delete")}
        onConfirm={() => delIdx !== null && deleteEntry(delIdx)}
        onClose={() => setDelIdx(null)}
      />
    </div>
  );
}

function Stat({
  label,
  value,
  suffix = "",
  tone = "neutral",
}: {
  label: string;
  value: number;
  suffix?: string;
  tone?: "neutral" | "ok" | "warn" | "bad";
}) {
  const color =
    tone === "bad"
      ? "text-rose-400"
      : tone === "warn"
      ? "text-amber-400"
      : tone === "ok"
      ? "text-emerald-400"
      : "text-neutral-100";
  return (
    <div className="card px-4 py-3">
      <div className="text-xs text-neutral-500">{label}</div>
      <div className={`mt-1 text-2xl font-semibold ${color}`}>
        {value.toLocaleString()}
        <span className="text-sm font-normal text-neutral-500">{suffix}</span>
      </div>
    </div>
  );
}
