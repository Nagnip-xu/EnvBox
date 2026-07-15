import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import ErrorBoundary from "./components/ErrorBoundary";
import { applyTheme, bindSystemThemeListener, loadThemeMode } from "./theme";
import { useStore } from "./store/useStore";

// 启动时先应用主题，避免闪烁
applyTheme(loadThemeMode());
bindSystemThemeListener(() => useStore.getState().theme);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
