import {
  LayoutDashboard,
  Route,
  Boxes,
  History,
  Settings as SettingsIcon,
  type LucideIcon,
} from "lucide-react";
import { useStore, type PageId } from "../store/useStore";
import { APP_VERSION } from "../version";

const NAV: { id: PageId; key: string; icon: LucideIcon }[] = [
  { id: "dashboard", key: "nav.dashboard", icon: LayoutDashboard },
  { id: "path", key: "nav.path", icon: Route },
  { id: "sdk", key: "nav.sdk", icon: Boxes },
  { id: "snapshots", key: "nav.snapshots", icon: History },
  { id: "settings", key: "nav.settings", icon: SettingsIcon },
];

export default function Sidebar() {
  const { page, setPage, t } = useStore();
  return (
    <aside className="flex w-52 flex-col border-r border-neutral-800 bg-neutral-900/40">
      <div className="flex items-center gap-2 px-5 py-5">
        <img src="/app-icon.png" alt="EnvBox" className="h-8 w-8 rounded-lg" />
        <span className="text-lg font-semibold tracking-tight">EnvBox</span>
      </div>

      <nav className="flex flex-1 flex-col gap-1 px-3">
        {NAV.map(({ id, key, icon: Icon }) => {
          const active = page === id;
          return (
            <button
              key={id}
              onClick={() => setPage(id)}
              className={`flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
                active
                  ? "bg-brand-600/15 text-brand-300"
                  : "text-neutral-400 hover:bg-neutral-800 hover:text-neutral-100"
              }`}
            >
              <Icon size={18} />
              {t(key)}
            </button>
          );
        })}
      </nav>

      <div className="px-5 py-4 text-xs text-neutral-600">
        v{APP_VERSION} · {t("app.edition")}
      </div>
    </aside>
  );
}
