import { create } from "zustand";
import { invalidateSdkCache, refreshSdkScan } from "./sdkCache";
import { type Lang, loadLang, translate } from "../i18n";
import { type ThemeMode, applyTheme, loadThemeMode, resolveTheme } from "../theme";

export type PageId = "dashboard" | "path" | "sdk" | "snapshots" | "settings";

type TParams = Record<string, string | number>;

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  action?: ToastAction;
}

interface AppState {
  page: PageId;
  search: string;
  toasts: Toast[];
  refreshKey: number;
  pendingNewVar: boolean;
  paletteOpen: boolean;
  lang: Lang;
  theme: ThemeMode;
  installPath: string;
  setPage: (p: PageId) => void;
  setSearch: (s: string) => void;
  pushToast: (message: string, type?: Toast["type"], action?: ToastAction) => void;
  removeToast: (id: number) => void;
  bumpRefresh: () => void;
  requestNewVar: () => void;
  clearNewVar: () => void;
  setPaletteOpen: (open: boolean) => void;
  toggleTheme: () => void;
  setLang: (l: Lang) => void;
  setTheme: (t: ThemeMode) => void;
  setInstallPath: (p: string) => void;
  t: (key: string, paramsOrFallback?: TParams | string, fallback?: string) => string;
}

let toastId = 0;

export const useStore = create<AppState>((set, get) => ({
  page: "dashboard",
  search: "",
  toasts: [],
  refreshKey: 0,
  pendingNewVar: false,
  paletteOpen: false,
  lang: loadLang(),
  theme: loadThemeMode(),
  installPath: localStorage.getItem("envbox.installPath") ?? "",
  setPage: (p) => set({ page: p, search: "" }),
  setSearch: (s) => set({ search: s }),
  pushToast: (message, type = "info", action) => {
    const id = ++toastId;
    set((st) => {
      // 最多同时显示 4 条，超出时丢弃最旧的
      const next = [...st.toasts, { id, message, type, action }];
      return { toasts: next.slice(-4) };
    });
    setTimeout(
      () => {
        set((st) => ({ toasts: st.toasts.filter((t) => t.id !== id) }));
      },
      action ? 8000 : 4000
    );
  },
  removeToast: (id) =>
    set((st) => ({ toasts: st.toasts.filter((t) => t.id !== id) })),
  bumpRefresh: () => {
    invalidateSdkCache();
    // 即使不在 SDK 页也后台重扫，保证下次进入或点刷新时数据最新
    void refreshSdkScan();
    set((st) => ({ refreshKey: st.refreshKey + 1 }));
  },
  requestNewVar: () => set({ page: "dashboard", pendingNewVar: true }),
  clearNewVar: () => set({ pendingNewVar: false }),
  setPaletteOpen: (open) => set({ paletteOpen: open }),
  toggleTheme: () => {
    const next = resolveTheme(get().theme) === "dark" ? "light" : "dark";
    localStorage.setItem("envbox.theme", next);
    applyTheme(next);
    set({ theme: next });
  },
  setLang: (l) => {
    localStorage.setItem("envbox.lang", l);
    set({ lang: l });
  },
  setTheme: (t) => {
    localStorage.setItem("envbox.theme", t);
    applyTheme(t);
    set({ theme: t });
  },
  setInstallPath: (p) => {
    localStorage.setItem("envbox.installPath", p);
    set({ installPath: p });
  },
  t: (key, paramsOrFallback, fallback) =>
    translate(get().lang, key, paramsOrFallback, fallback),
}));
