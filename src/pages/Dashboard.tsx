import { useEffect, useMemo, useState } from "react";
import { Pencil, Trash2, AlertTriangle, Eye, EyeOff } from "lucide-react";
import type { EnvVar, EnvScope } from "../types";
import { api, errorMessage } from "../lib/tauri";
import { useStore } from "../store/useStore";
import Modal from "../components/Modal";
import ConfirmModal from "../components/ConfirmModal";
import { isSensitiveName } from "../lib/security";

const SCOPE_KEY: Record<EnvScope, string> = {
  system: "scope.system",
  user: "scope.user",
  process: "scope.process",
};

interface EditState {
  scope: "system" | "user";
  name: string;
  value: string;
  isNew: boolean;
}

export default function Dashboard() {
  const [vars, setVars] = useState<EnvVar[]>([]);
  const [loading, setLoading] = useState(true);
  const { search, refreshKey, bumpRefresh, pushToast, pendingNewVar, clearNewVar, t } = useStore();
  const [edit, setEdit] = useState<EditState | null>(null);
  const [del, setDel] = useState<EnvVar | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    api
      .listEnvVars()
      .then((items) => !cancelled && setVars(items))
      .catch((error) => !cancelled && pushToast(t("common.loadFail", { err: errorMessage(error) }), "error"))
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [refreshKey, pushToast, t]);

  useEffect(() => {
    if (pendingNewVar) {
      setEdit({ scope: "user", name: "", value: "", isNew: true });
      clearNewVar();
    }
  }, [pendingNewVar, clearNewVar]);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return vars;
    return vars.filter(
      (v) => v.name.toLowerCase().includes(q) || v.value.toLowerCase().includes(q)
    );
  }, [vars, search]);

  async function save() {
    if (!edit) return;
    if (!edit.name.trim()) {
      pushToast(t("toast.nameEmpty"), "error");
      return;
    }
    try {
      await api.setEnvVar(edit.scope, edit.name.trim(), edit.value);
      pushToast(t("toast.varSaved", { name: edit.name }), "success");
      setEdit(null);
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.saveFail", { err: errorMessage(e) }), "error");
    }
  }

  async function remove() {
    if (!del) return;
    try {
      await api.deleteEnvVar(del.scope, del.name);
      pushToast(t("toast.varDeleted", { name: del.name }), "success");
      setDel(null);
      bumpRefresh();
    } catch (e) {
      pushToast(t("toast.deleteFail", { err: errorMessage(e) }), "error");
    }
  }

  const groups: EnvScope[] = ["system", "user"];

  return (
    <div className="space-y-6">
      {groups.map((scope) => {
        const items = filtered.filter((v) => v.scope === scope);
        return (
          <section key={scope}>
            <div className="mb-2 flex items-center gap-2">
              <h2 className="text-sm font-semibold text-neutral-300">
                {t(SCOPE_KEY[scope])}
              </h2>
              <span className="rounded-full bg-neutral-800 px-2 py-0.5 text-xs text-neutral-400">
                {items.length}
              </span>
            </div>
            <div className="card divide-y divide-neutral-800">
              {loading && (
                <div className="px-4 py-6 text-center text-sm text-neutral-500">{t("common.loading")}</div>
              )}
              {!loading && items.length === 0 && (
                <div className="px-4 py-6 text-center text-sm text-neutral-600">{t("dash.noMatch")}</div>
              )}
              {items.map((v) => (
                <VarRow
                  key={`${v.scope}-${v.name}`}
                  v={v}
                  t={t}
                  onEdit={() =>
                    setEdit({ scope: v.scope as "system" | "user", name: v.name, value: v.value, isNew: false })
                  }
                  onDelete={() => setDel(v)}
                />
              ))}
            </div>
          </section>
        );
      })}

      <Modal
        open={!!edit}
        title={edit?.isNew ? t("dash.newTitle") : t("dash.editTitle", { name: edit?.name ?? "" })}
        onClose={() => setEdit(null)}
      >
        {edit && (
          <div className="space-y-4">
            <div>
              <label className="mb-1 block text-xs text-neutral-400">{t("dash.scopeLabel")}</label>
              <div className="flex gap-2">
                {(["user", "system"] as const).map((s) => (
                  <button
                    key={s}
                    disabled={!edit.isNew}
                    onClick={() => setEdit({ ...edit, scope: s })}
                    className={`btn border ${
                      edit.scope === s
                        ? "border-brand-500 bg-brand-600/15 text-brand-300"
                        : "border-neutral-800 text-neutral-400"
                    } ${!edit.isNew ? "opacity-60" : ""}`}
                  >
                    {s === "user" ? t("dash.scope.user") : t("dash.scope.system")}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label htmlFor="env-var-name" className="mb-1 block text-xs text-neutral-400">
                {t("dash.nameLabel")}
              </label>
              <input
                id="env-var-name"
                value={edit.name}
                disabled={!edit.isNew}
                onChange={(e) => setEdit({ ...edit, name: e.target.value })}
                className="w-full rounded-lg border border-neutral-800 bg-neutral-950 px-3 py-2 font-mono text-sm outline-none focus:border-brand-500 disabled:opacity-60"
                placeholder={t("dash.namePlaceholder")}
              />
            </div>
            <div>
              <label htmlFor="env-var-value" className="mb-1 block text-xs text-neutral-400">
                {t("dash.valueLabel")}
              </label>
              <textarea
                id="env-var-value"
                value={edit.value}
                onChange={(e) => setEdit({ ...edit, value: e.target.value })}
                rows={3}
                className="w-full resize-y rounded-lg border border-neutral-800 bg-neutral-950 px-3 py-2 font-mono text-sm outline-none focus:border-brand-500"
                placeholder={t("dash.valuePlaceholder")}
              />
            </div>
            <div className="flex justify-end gap-2 pt-1">
              <button className="btn-ghost border border-neutral-800" onClick={() => setEdit(null)}>
                {t("common.cancel")}
              </button>
              <button className="btn-primary" onClick={save}>
                {t("common.save")}
              </button>
            </div>
          </div>
        )}
      </Modal>

      <ConfirmModal
        open={!!del}
        title={t("dash.delTitle")}
        message={t("dash.delMsg", {
          scope: del?.scope === "system" ? t("scope.system.short") : t("scope.user.short"),
          name: del?.name ?? "",
        })}
        danger
        confirmLabel={t("common.delete")}
        onConfirm={remove}
        onClose={() => setDel(null)}
      />
    </div>
  );
}

function VarRow({
  v,
  t,
  onEdit,
  onDelete,
}: {
  v: EnvVar;
  t: (key: string, paramsOrFallback?: Record<string, string | number> | string, fallback?: string) => string;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const sensitive = isSensitiveName(v.name);
  const [revealed, setRevealed] = useState(false);
  return (
    <div className="group flex items-center gap-3 px-4 py-2.5 hover:bg-neutral-800/40">
      <div className="w-52 shrink-0 truncate font-mono text-sm text-brand-300">{v.name}</div>
      <div
        className="min-w-0 flex-1 truncate font-mono text-sm text-neutral-400"
        title={sensitive && !revealed ? undefined : v.value}
      >
        {sensitive && !revealed ? "••••••••••••" : v.value}
      </div>
      {sensitive && (
        <button
          className="btn-ghost !px-2"
          title={revealed ? t("common.hideSecret") : t("common.revealSecret")}
          aria-label={revealed ? t("common.hideSecret") : t("common.revealSecret")}
          aria-pressed={revealed}
          onClick={() => setRevealed((value) => !value)}
        >
          {revealed ? (
            <EyeOff size={15} aria-hidden="true" />
          ) : (
            <Eye size={15} aria-hidden="true" />
          )}
        </button>
      )}
      {v.conflictsWith && (
        <span className="tag tag-amber" title={t("dash.conflictTip")}>
          <AlertTriangle size={12} /> {t("dash.conflict")}
        </span>
      )}
      {v.isExpandable && (
        <span className="rounded-md bg-neutral-800 px-2 py-0.5 text-xs text-neutral-400">
          {t("dash.hasVar")}
        </span>
      )}
      <div className="flex items-center gap-1 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
        <button className="btn-ghost !px-2" title={t("common.edit")} aria-label={t("common.edit")} onClick={onEdit}>
          <Pencil size={15} />
        </button>
        <button
          className="btn-ghost !px-2 hover:!text-rose-400"
          title={t("common.delete")}
          aria-label={t("common.delete")}
          onClick={onDelete}
        >
          <Trash2 size={15} />
        </button>
      </div>
    </div>
  );
}
