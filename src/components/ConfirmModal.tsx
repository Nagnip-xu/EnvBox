import { AlertTriangle } from "lucide-react";
import { useEffect } from "react";
import Modal from "./Modal";
import { useStore } from "../store/useStore";

export default function ConfirmModal({
  open,
  title,
  message,
  danger = false,
  confirmLabel,
  onConfirm,
  onClose,
}: {
  open: boolean;
  title: string;
  message: string;
  danger?: boolean;
  confirmLabel?: string;
  onConfirm: () => void;
  onClose: () => void;
}) {
  const t = useStore((s) => s.t);
  function confirm() {
    onConfirm();
    onClose();
  }

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      // 回车确认（避免在输入框中误触发时才处理，这里弹窗内无输入框，可直接确认）
      if (e.key === "Enter") {
        e.preventDefault();
        confirm();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  return (
    <Modal open={open} title={title} onClose={onClose} width="max-w-md">
      <div className="flex gap-3">
        {danger && <AlertTriangle size={20} className="mt-0.5 shrink-0 text-amber-400" />}
        <p className="whitespace-pre-wrap text-sm leading-relaxed text-neutral-300">{message}</p>
      </div>
      <p className="mt-3 text-xs text-neutral-500">{t("confirm.snapNote")}</p>
      <div className="mt-5 flex justify-end gap-2">
        <button className="btn-ghost border border-neutral-800" onClick={onClose}>
          {t("common.cancel")}
        </button>
        <button
          data-autofocus
          className={danger ? "btn bg-rose-600 text-white hover:bg-rose-500" : "btn-primary"}
          onClick={confirm}
        >
          {confirmLabel ?? t("common.confirm")}
        </button>
      </div>
    </Modal>
  );
}
