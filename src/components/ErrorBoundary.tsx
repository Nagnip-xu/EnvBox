import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertTriangle, RotateCcw } from "lucide-react";
import { useStore } from "../store/useStore";

interface Props {
  children: ReactNode;
}
interface State {
  error: Error | null;
}

/** 全局错误边界：捕获渲染期异常，避免整个界面白屏 */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("EnvBox 渲染错误:", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      const t = useStore.getState().t;
      return (
        <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
          <AlertTriangle size={40} className="text-amber-400" />
          <div>
            <h1 className="text-lg font-semibold text-neutral-100">{t("err.title")}</h1>
            <p className="mt-1 max-w-md text-sm text-neutral-400">{t("err.desc")}</p>
          </div>
          <pre className="max-h-40 max-w-lg overflow-auto rounded-lg border border-neutral-800 bg-neutral-950 p-3 text-left text-xs text-rose-300">
            {this.state.error.message}
          </pre>
          <button className="btn-primary" onClick={() => window.location.reload()}>
            <RotateCcw size={15} /> {t("err.reload")}
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
