export type ThemeMode = "light" | "dark" | "system";

export function resolveTheme(mode: ThemeMode): "light" | "dark" {
  if (mode === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

export function applyTheme(mode: ThemeMode) {
  const eff = resolveTheme(mode);
  const root = document.documentElement;
  root.classList.remove("theme-dark", "theme-light");
  root.classList.add(eff === "dark" ? "theme-dark" : "theme-light");
  root.style.colorScheme = eff;
}

let mediaBound = false;
/** 监听系统主题变化，仅在「跟随系统」时重新应用 */
export function bindSystemThemeListener(getMode: () => ThemeMode) {
  if (mediaBound) return;
  mediaBound = true;
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  mq.addEventListener("change", () => {
    if (getMode() === "system") applyTheme("system");
  });
}

export function loadThemeMode(): ThemeMode {
  const v = localStorage.getItem("envbox.theme");
  return v === "light" || v === "dark" || v === "system" ? v : "system";
}
