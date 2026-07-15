import { CheckCircle2, XCircle, Info, X } from "lucide-react";
import { useStore } from "../store/useStore";

export default function Toaster() {
  const { toasts, removeToast } = useStore();
  return (
    <div
      aria-live="polite"
      aria-atomic="false"
      className="pointer-events-none fixed bottom-5 right-5 z-50 flex flex-col gap-2"
    >
      {toasts.map((t) => {
        const Icon =
          t.type === "success" ? CheckCircle2 : t.type === "error" ? XCircle : Info;
        const color =
          t.type === "success"
            ? "text-emerald-400"
            : t.type === "error"
            ? "text-rose-400"
            : "text-brand-300";
        return (
          <div
            key={t.id}
            className="pointer-events-auto flex w-80 items-start gap-2.5 rounded-xl border border-neutral-800 bg-neutral-900 px-4 py-3 shadow-xl"
          >
            <Icon size={18} className={`mt-0.5 shrink-0 ${color}`} />
            <div className="min-w-0 flex-1">
              <div className="text-sm text-neutral-200">{t.message}</div>
              {t.action && (
                <button
                  onClick={() => {
                    t.action?.onClick();
                    removeToast(t.id);
                  }}
                  className="mt-2 rounded-md bg-brand-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-brand-500"
                >
                  {t.action.label}
                </button>
              )}
            </div>
            <button
              onClick={() => removeToast(t.id)}
              className="text-neutral-500 hover:text-neutral-200"
            >
              <X size={15} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
