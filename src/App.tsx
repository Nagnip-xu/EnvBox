import { useEffect, useState } from "react";
import Sidebar from "./components/Sidebar";
import TopBar from "./components/TopBar";
import Toaster from "./components/Toaster";
import CommandPalette from "./components/CommandPalette";
import Dashboard from "./pages/Dashboard";
import PathManager from "./pages/PathManager";
import SdkCenter from "./pages/SdkCenter";
import Snapshots from "./pages/Snapshots";
import Settings from "./pages/Settings";
import { useStore } from "./store/useStore";
import type { PageId } from "./store/useStore";

export default function App() {
  const page = useStore((s) => s.page);
  const [mountedPages, setMountedPages] = useState<Set<PageId>>(
    () => new Set<PageId>(["dashboard"])
  );

  useEffect(() => {
    setMountedPages((current) => {
      if (current.has(page)) return current;
      const next = new Set(current);
      next.add(page);
      return next;
    });
  }, [page]);

  useEffect(() => {
    function focusSearch() {
      const st = useStore.getState();
      if (st.page !== "dashboard" && st.page !== "path") st.setPage("dashboard");
      // 等待搜索框渲染后再聚焦
      requestAnimationFrame(() => {
        const el = document.getElementById("envbox-search") as HTMLInputElement | null;
        el?.focus();
        el?.select();
      });
    }

    function onKeyDown(e: KeyboardEvent) {
      const mod = e.ctrlKey || e.metaKey;
      if (mod && e.key.toLowerCase() === "k") {
        e.preventDefault();
        const st = useStore.getState();
        st.setPaletteOpen(!st.paletteOpen);
      } else if (mod && e.key.toLowerCase() === "f") {
        e.preventDefault();
        focusSearch();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <div className="flex h-full">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-6">
          <div hidden={page !== "dashboard"}>
            <Dashboard />
          </div>
          {mountedPages.has("path") && (
            <div hidden={page !== "path"}>
              <PathManager />
            </div>
          )}
          {mountedPages.has("sdk") && (
            <div hidden={page !== "sdk"}>
              <SdkCenter />
            </div>
          )}
          {mountedPages.has("snapshots") && (
            <div hidden={page !== "snapshots"}>
              <Snapshots />
            </div>
          )}
          {mountedPages.has("settings") && (
            <div hidden={page !== "settings"}>
              <Settings />
            </div>
          )}
        </main>
      </div>
      <Toaster />
      <CommandPalette />
    </div>
  );
}
