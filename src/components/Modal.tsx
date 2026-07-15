import { X } from "lucide-react";
import { useEffect, useRef, type ReactNode } from "react";
import { useStore } from "../store/useStore";

export default function Modal({
  open,
  title,
  onClose,
  children,
  width = "max-w-lg",
}: {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  width?: string;
}) {
  const panelRef = useRef<HTMLDivElement>(null);
  const t = useStore((s) => s.t);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    // 打开时把焦点移入弹窗，方便键盘操作与可访问性
    const focusTarget = panelRef.current?.querySelector<HTMLElement>(
      "input, textarea, select, [data-autofocus]"
    );
    (focusTarget ?? panelRef.current)?.focus();
    return () => document.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4"
      onClick={onClose}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        className={`card w-full ${width} bg-neutral-900 shadow-2xl outline-none`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-neutral-800 px-5 py-3">
          <h3 className="text-sm font-semibold text-neutral-100">{title}</h3>
          <button
            onClick={onClose}
            aria-label={t("common.close")}
            className="text-neutral-500 hover:text-neutral-200"
          >
            <X size={18} />
          </button>
        </div>
        <div className="p-5">{children}</div>
      </div>
    </div>
  );
}
