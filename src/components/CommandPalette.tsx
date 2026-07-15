import { useEffect, useMemo, useRef, useState } from "react";
import {
  LayoutDashboard,
  Route,
  Boxes,
  History,
  Settings as SettingsIcon,
  Plus,
  RefreshCw,
  Palette,
  Search,
  type LucideIcon,
} from "lucide-react";
import { useStore, type PageId } from "../store/useStore";

interface Command {
  id: string;
  label: string;
  group: "nav" | "action";
  icon: LucideIcon;
  run: () => void;
}

export default function CommandPalette() {
  const {
    paletteOpen,
    setPaletteOpen,
    setPage,
    requestNewVar,
    bumpRefresh,
    toggleTheme,
    pushToast,
    lang,
    t,
  } = useStore();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const commands = useMemo<Command[]>(() => {
    const nav: { id: PageId; key: string; icon: LucideIcon }[] = [
      { id: "dashboard", key: "nav.dashboard", icon: LayoutDashboard },
      { id: "path", key: "nav.path", icon: Route },
      { id: "sdk", key: "nav.sdk", icon: Boxes },
      { id: "snapshots", key: "nav.snapshots", icon: History },
      { id: "settings", key: "nav.settings", icon: SettingsIcon },
    ];
    return [
      ...nav.map<Command>((n) => ({
        id: `nav:${n.id}`,
        label: t(n.key),
        group: "nav",
        icon: n.icon,
        run: () => setPage(n.id),
      })),
      {
        id: "action:newVar",
        label: t("palette.action.newVar"),
        group: "action",
        icon: Plus,
        run: () => requestNewVar(),
      },
      {
        id: "action:refresh",
        label: t("palette.action.refresh"),
        group: "action",
        icon: RefreshCw,
        run: () => {
          bumpRefresh();
          pushToast(t("toast.refreshing"), "info");
        },
      },
      {
        id: "action:theme",
        label: t("palette.action.theme"),
        group: "action",
        icon: Palette,
        run: () => toggleTheme(),
      },
    ];
  }, [lang, t, setPage, requestNewVar, bumpRefresh, toggleTheme, pushToast]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return commands;
    return commands.filter((c) => c.label.toLowerCase().includes(q));
  }, [commands, query]);

  useEffect(() => {
    if (paletteOpen) {
      setQuery("");
      setActive(0);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [paletteOpen]);

  useEffect(() => {
    setActive(0);
  }, [query]);

  if (!paletteOpen) return null;

  function close() {
    setPaletteOpen(false);
  }

  function runAt(idx: number) {
    const cmd = filtered[idx];
    if (!cmd) return;
    close();
    cmd.run();
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      runAt(active);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 p-4 pt-[12vh]"
      onClick={close}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
        className="card w-full max-w-lg overflow-hidden bg-neutral-900 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b border-neutral-800 px-4 py-3">
          <Search size={16} className="text-neutral-500" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder={t("palette.placeholder")}
            className="flex-1 bg-transparent text-sm text-neutral-100 outline-none placeholder:text-neutral-600"
          />
        </div>

        <div className="max-h-80 overflow-y-auto py-1">
          {filtered.length === 0 && (
            <div className="px-4 py-6 text-center text-sm text-neutral-600">{t("palette.empty")}</div>
          )}
          {filtered.map((cmd, i) => {
            const Icon = cmd.icon;
            const prev = filtered[i - 1];
            const showHeader = !prev || prev.group !== cmd.group;
            return (
              <div key={cmd.id}>
                {showHeader && (
                  <div className="px-4 pb-1 pt-2 text-[11px] font-medium uppercase tracking-wide text-neutral-600">
                    {cmd.group === "nav" ? t("palette.group.nav") : t("palette.group.action")}
                  </div>
                )}
                <button
                  onMouseEnter={() => setActive(i)}
                  onClick={() => runAt(i)}
                  className={`flex w-full items-center gap-3 px-4 py-2 text-left text-sm transition-colors ${
                    i === active ? "bg-brand-600/15 text-brand-300" : "text-neutral-300 hover:bg-neutral-800/60"
                  }`}
                >
                  <Icon size={16} className={i === active ? "text-brand-300" : "text-neutral-500"} />
                  {cmd.label}
                </button>
              </div>
            );
          })}
        </div>

        <div className="border-t border-neutral-800 px-4 py-2 text-[11px] text-neutral-600">
          {t("palette.hint")}
        </div>
      </div>
    </div>
  );
}
