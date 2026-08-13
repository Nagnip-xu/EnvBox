import { useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/bricolage-grotesque";
import "@fontsource-variable/source-sans-3";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/500.css";
import {
  ArrowDownToLine,
  ArrowRight,
  Boxes,
  Camera,
  Check,
  CircleGauge,
  LayoutDashboard,
  Menu,
  MonitorCheck,
  Moon,
  Route,
  ScanSearch,
  Search,
  Settings,
  ShieldCheck,
  Sun,
  X,
} from "lucide-react";
import iconUrl from "./assets/envbox-icon.png";
import movementUrl from "./assets/movement-macro.png";
import genevaUrl from "./assets/geneva-stripes.png";
import "./styles.css";

const DOWNLOAD_PATH = import.meta.env.DEV
  ? "/downloads/EnvBox.exe"
  : "https://github.com/Nagnip-xu/EnvBox/releases/latest/download/EnvBox.exe";

const SYSTEM_PATH = [
  "%JAVA_HOME%\\bin",
  "C:\\Program Files\\Git\\cmd",
  "C:\\Windows\\System32",
  "C:\\Program Files\\Go\\bin",
  "C:\\Program Files\\dotnet",
  "%USERPROFILE%\\.cargo\\bin",
];

const USER_PATH = [
  "C:\\Tools\\node-v22",
  "C:\\Tools\\apache-maven\\bin",
  "%LOCALAPPDATA%\\Programs\\Python",
];

const JEWELS = [
  { name: "JDK", version: "21" },
  { name: "Node", version: "22" },
  { name: "Go", version: "1.22" },
  { name: "Python", version: "3.12" },
];

function useTheme() {
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("envbox.website.theme");
    if (saved === "dark" || saved === "light") return saved;
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("envbox.website.theme", theme);
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.setAttribute("content", theme === "light" ? "#eef1f5" : "#12151b");
  }, [theme]);

  return { theme, setTheme };
}

function DownloadButton({ compact = false, onClick }: { compact?: boolean; onClick?: () => void }) {
  return (
    <a
      className={compact ? "button button-primary button-compact" : "button button-primary"}
      href={DOWNLOAD_PATH}
      download
      onClick={onClick}
    >
      <ArrowDownToLine aria-hidden="true" strokeWidth={1.75} />
      免费下载 Windows 版
    </a>
  );
}

function Header() {
  const [open, setOpen] = useState(false);
  const { theme, setTheme } = useTheme();
  const close = () => setOpen(false);

  return (
    <header className="site-header">
      <nav className="nav-shell" aria-label="主导航">
        <a className="brand" href="#top" onClick={close}>
          <img src={iconUrl} alt="" width={28} height={28} />
          <span>EnvBox</span>
        </a>
        <div id="mobile-navigation" className={open ? "nav-links is-open" : "nav-links"}>
          <a href="#product" onClick={close}>产品界面</a>
          <a href="#safety" onClick={close}>安全机制</a>
          <a href="#capabilities" onClick={close}>核心能力</a>
          <a href="#compatibility" onClick={close}>系统兼容</a>
          <div className="mobile-download">
            <DownloadButton compact onClick={close} />
          </div>
        </div>
        <div className="nav-actions">
          <button
            className="icon-button"
            type="button"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            aria-label={theme === "dark" ? "切换到浅色主题" : "切换到深色主题"}
          >
            {theme === "dark" ? <Sun aria-hidden="true" strokeWidth={1.75} /> : <Moon aria-hidden="true" strokeWidth={1.75} />}
          </button>
          <div className="desktop-download">
            <DownloadButton compact />
          </div>
          <button
            className="icon-button menu-button"
            type="button"
            aria-expanded={open}
            aria-controls="mobile-navigation"
            aria-label={open ? "关闭导航菜单" : "打开导航菜单"}
            onClick={() => setOpen((value) => !value)}
          >
            {open ? <X aria-hidden="true" strokeWidth={1.75} /> : <Menu aria-hidden="true" strokeWidth={1.75} />}
          </button>
        </div>
      </nav>
    </header>
  );
}

function Caliber() {
  const [focus, setFocus] = useState<{ ring: "system" | "user"; index: number }>({
    ring: "system",
    index: 0,
  });
  const activePath = focus.ring === "system" ? SYSTEM_PATH[focus.index] : USER_PATH[focus.index];

  return (
    <figure className="caliber" aria-label="EnvBox PATH 机芯示意，示例数据已脱敏">
      <div className="caliber-plate">
        <span className="caliber-chapter" aria-hidden="true" />
        {SYSTEM_PATH.map((path, index) => {
          const angle = (360 / SYSTEM_PATH.length) * index - 90;
          return (
            <span key={path} className="tick-arm" style={{ "--angle": `${angle}deg` } as CSSProperties}>
              <button
                type="button"
                className={focus.ring === "system" && focus.index === index ? "caliber-tick is-active" : "caliber-tick"}
                aria-pressed={focus.ring === "system" && focus.index === index}
                aria-label={`系统 PATH ${path}`}
                onClick={() => setFocus({ ring: "system", index })}
                onFocus={() => setFocus({ ring: "system", index })}
              />
            </span>
          );
        })}
        {USER_PATH.map((path, index) => {
          const angle = (360 / USER_PATH.length) * index - 50;
          return (
            <span key={path} className="tick-arm tick-arm-inner" style={{ "--angle": `${angle}deg` } as CSSProperties}>
              <button
                type="button"
                className={
                  focus.ring === "user" && focus.index === index ? "caliber-tick is-active" : "caliber-tick"
                }
                aria-pressed={focus.ring === "user" && focus.index === index}
                aria-label={`用户 PATH ${path}`}
                onClick={() => setFocus({ ring: "user", index })}
                onFocus={() => setFocus({ ring: "user", index })}
              />
            </span>
          );
        })}
        <div className="caliber-core">
          {JEWELS.map((jewel) => (
            <span key={jewel.name} className="jewel">
              <strong>{jewel.version}</strong>
              <small>{jewel.name}</small>
            </span>
          ))}
        </div>
        <span className="caliber-seconds" aria-hidden="true" />
        <div className="caliber-crown" aria-hidden="true">
          <Camera strokeWidth={1.6} />
          快照
        </div>
      </div>
      <figcaption>
        <span>{focus.ring === "system" ? "系统 PATH" : "用户 PATH"}</span>
        <code>{activePath}</code>
      </figcaption>
    </figure>
  );
}

function ProductFrame() {
  const [scope, setScope] = useState<"系统变量" | "用户变量">("系统变量");
  const rows =
    scope === "系统变量"
      ? [
          ["JAVA_HOME", "C:\\Program Files\\Eclipse Adoptium\\jdk-21"],
          ["GOROOT", "C:\\Program Files\\Go"],
          ["DOTNET_ROOT", "C:\\Program Files\\dotnet"],
        ]
      : [
          ["NODE_HOME", "C:\\Tools\\node-v22"],
          ["MAVEN_HOME", "C:\\Tools\\apache-maven"],
          ["CARGO_HOME", "%USERPROFILE%\\.cargo"],
        ];

  return (
    <figure className="product-case">
      <div className="window-bar" aria-hidden="true">
        <span />
        <span />
        <span />
        <strong>EnvBox</strong>
      </div>
      <div className="preview-app" aria-label="EnvBox 脱敏交互预览">
        <aside className="preview-sidebar">
          <div className="preview-logo">
            <img src={iconUrl} alt="" width={20} height={20} />
            <span>EnvBox</span>
          </div>
          <span className="preview-nav-active">
            <LayoutDashboard aria-hidden="true" strokeWidth={1.75} />
            概览
          </span>
          <span>
            <Route aria-hidden="true" strokeWidth={1.75} />
            PATH 管理
          </span>
          <span>
            <Boxes aria-hidden="true" strokeWidth={1.75} />
            SDK 中心
          </span>
          <span>
            <Camera aria-hidden="true" strokeWidth={1.75} />
            快照回滚
          </span>
          <span>
            <Settings aria-hidden="true" strokeWidth={1.75} />
            设置
          </span>
        </aside>
        <div className="preview-main">
          <div className="preview-toolbar">
            <div>
              <strong>环境概览</strong>
              <small>所有示例数据均已脱敏</small>
            </div>
            <span className="preview-search">
              <Search aria-hidden="true" strokeWidth={1.75} />
              搜索变量
            </span>
          </div>
          <div className="preview-body">
            <div className="preview-summary">
              <div>
                <small>环境变量</small>
                <strong>48</strong>
              </div>
              <div>
                <small>PATH 条目</small>
                <strong>27</strong>
              </div>
              <div>
                <small>已识别 SDK</small>
                <strong>12</strong>
              </div>
            </div>
            <div className="preview-panel">
              <div className="preview-tabs" role="tablist" aria-label="变量作用域">
                {(["系统变量", "用户变量"] as const).map((item) => (
                  <button
                    type="button"
                    role="tab"
                    aria-selected={scope === item}
                    className={scope === item ? "is-active" : ""}
                    onClick={() => setScope(item)}
                    key={item}
                  >
                    {item}
                  </button>
                ))}
              </div>
              <div className="preview-rows" aria-live="polite">
                {rows.map(([name, value]) => (
                  <div className="preview-row" key={name}>
                    <span>{name}</span>
                    <code>{value}</code>
                    <small>{scope === "系统变量" ? "系统" : "用户"}</small>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </figure>
  );
}

const SNAPSHOT_ROWS = [
  { name: "PATH", kind: "移除", value: "C:\\Tools\\legacy\\bin" },
  { name: "JAVA_HOME", kind: "更新", value: "C:\\Program Files\\Java\\jdk-21" },
  { name: "NODE_HOME", kind: "新增", value: "C:\\Tools\\node-v22" },
];

function SnapshotDiffPreview({ activeRow }: { activeRow?: number }) {
  return (
    <figure className="snapshot-preview" aria-label="EnvBox 修改前快照与差异预览">
      <figcaption className="snapshot-preview-head">
        <span className="snapshot-preview-icon">
          <Camera aria-hidden="true" strokeWidth={1.6} />
        </span>
        <span>
          <strong>修改前快照</strong>
          <small>确认 3 项环境变更</small>
        </span>
        <span className="snapshot-preview-state">可恢复</span>
      </figcaption>
      <div className="snapshot-diff-list">
        {SNAPSHOT_ROWS.map((row, index) => (
          <div
            className={activeRow === index ? "snapshot-diff-row is-live" : "snapshot-diff-row"}
            key={row.name}
          >
            <span>{row.name}</span>
            <div>
              <small>{row.kind}</small>
              <code>{row.value}</code>
            </div>
          </div>
        ))}
      </div>
      <div className={activeRow === 2 ? "snapshot-preview-foot is-live" : "snapshot-preview-foot"}>
        <ShieldCheck aria-hidden="true" strokeWidth={1.6} />
        写入失败时自动回滚
      </div>
    </figure>
  );
}

function Hero() {
  return (
    <section className="hero section-shell" id="top">
      <div className="hero-copy">
        <h1>
          复杂环境，
          <span className="hero-accent">重新有序。</span>
        </h1>
        <p className="hero-lead">统一查看变量、整理 PATH、切换 SDK，并在每次修改前留下可恢复快照。</p>
        <div className="hero-actions">
          <DownloadButton />
          <a className="button button-secondary" href="#safety">
            了解安全机制
            <ArrowRight aria-hidden="true" strokeWidth={1.75} />
          </a>
        </div>
      </div>
      <div className="hero-visual">
        <Caliber />
      </div>
    </section>
  );
}

function ProofRail() {
  const items = [
    ["Windows 10/11", "明确兼容"],
    ["本地优先", "无需账户"],
    ["自动化测试", "随版本验证"],
    ["修改之前", "强制快照"],
  ];
  return (
    <section className="proof-rail" aria-label="产品事实">
      <div className="section-shell proof-grid">
        {items.map(([value, label]) => (
          <div key={value} className="proof-item">
            <strong>{value}</strong>
            <span>{label}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function ProductStory() {
  return (
    <section className="product-story section-shell" id="product">
      <div className="story-copy">
        <h2>从散落各处，到一块校准过的表盘。</h2>
        <p>
          环境变量、PATH、SDK 与项目版本声明不再各自为政。EnvBox
          把它们组织成可以理解、可以检查、可以恢复的完整上下文。
        </p>
      </div>
      <ProductFrame />
    </section>
  );
}

function Safety() {
  const sectionRef = useRef<HTMLElement>(null);
  const [live, setLive] = useState(false);
  const [beat, setBeat] = useState(0);
  const steps = [
    {
      title: "先留下完整快照",
      text: "用户与系统注册表必须全部读取成功，否则写操作不会开始。",
    },
    {
      title: "再看清具体变化",
      text: "新增、修改与删除逐项预览，敏感变量默认遮蔽。",
    },
    {
      title: "失败就回到原状态",
      text: "恢复与导入出现错误时自动回滚，并明确报告是否完整。",
    },
  ];

  useEffect(() => {
    const node = sectionRef.current;
    if (!node) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (reduce.matches) return;
    const observer = new IntersectionObserver(
      ([entry]) => setLive(Boolean(entry?.isIntersecting)),
      { threshold: 0.32 }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!live) return;
    const timer = window.setInterval(() => {
      setBeat((value) => (value + 1) % 3);
    }, 1700);
    return () => window.clearInterval(timer);
  }, [live]);

  return (
    <section className="safety" id="safety" ref={sectionRef}>
      <div className="section-shell safety-layout">
        <div className="safety-copy">
          <div className="safety-intro">
            <h2>安全是执行顺序，不是提醒。</h2>
            <p>快照、差异预览与自动回滚被写进每次高风险修改的必经路径，而不是藏在确认框的附注里。</p>
          </div>
          <ol className="safety-steps">
            {steps.map(({ title, text }, index) => (
              <li key={title} className={live && beat === index ? "is-current" : undefined}>
                <h3>{title}</h3>
                <p>{text}</p>
              </li>
            ))}
          </ol>
        </div>
        <div className="safety-well">
          <SnapshotDiffPreview activeRow={live ? beat : undefined} />
        </div>
      </div>
    </section>
  );
}

function Capabilities() {
  return (
    <section className="capabilities section-shell" id="capabilities">
      <div className="section-heading">
        <h2>一个工具，覆盖开发环境的完整生命周期。</h2>
        <p>从发现问题到安全修改，再到版本切换与残留清理，每项能力都围绕真实的 Windows 工作流展开。</p>
      </div>
      <div className="capability-grid">
        <article className="capability capability-wide">
          <div className="capability-copy">
            <CircleGauge aria-hidden="true" strokeWidth={1.6} />
            <h3>PATH 终于可以逐条理解</h3>
            <p>区分失效、重复、未解析和暂时离线的网络路径。清理动作只处理真正安全的条目。</p>
          </div>
          <div className="path-stack" aria-hidden="true">
            <span>
              <Check strokeWidth={2} /> %JAVA_HOME%\bin
            </span>
            <span>
              <Check strokeWidth={2} /> C:\Program Files\Git\cmd
            </span>
            <span className="path-muted">C:\Tools\legacy\bin</span>
          </div>
        </article>
        <article className="capability capability-metal">
          <Boxes aria-hidden="true" strokeWidth={1.6} />
          <h3>多版本 SDK，一键切换</h3>
          <p>识别 JDK、Python、Node.js、Go、Rust、.NET 等常用工具，并尊重 nvm、Conda、rustup 等外部管理器。</p>
        </article>
        <article className="capability capability-image">
          <SnapshotDiffPreview />
        </article>
        <article className="capability capability-plain">
          <MonitorCheck aria-hidden="true" strokeWidth={1.6} />
          <h3>项目版本，立即对照本机</h3>
          <p>读取 `.nvmrc`、`.python-version`、`.tool-versions` 与 `global.json`，标出当前、已安装和缺失状态。</p>
        </article>
        <article className="capability capability-plain">
          <ScanSearch aria-hidden="true" strokeWidth={1.6} />
          <h3>环境体检给出可行动结果</h3>
          <p>集中发现变量冲突、异常快照、离线网络路径和未完成安装，不用逐处排查。</p>
        </article>
      </div>
    </section>
  );
}

function Compatibility() {
  return (
    <section className="compatibility section-shell" id="compatibility">
      <div className="compatibility-panel">
        <div>
          <h2>为 Windows 10/11 构建。</h2>
          <p>需要 Microsoft Edge WebView2 Runtime。SDK 自动安装使用 winget 或 Scoop，核心环境管理能力不依赖在线账户。</p>
        </div>
        <ul className="compat-list" aria-label="系统要求">
          <li>
            <Check aria-hidden="true" strokeWidth={2} /> Windows 10 及更高版本
          </li>
          <li>
            <Check aria-hidden="true" strokeWidth={2} /> WebView2 Runtime
          </li>
          <li>
            <Check aria-hidden="true" strokeWidth={2} /> winget 或 Scoop 可选
          </li>
          <li>
            <Check aria-hidden="true" strokeWidth={2} /> 本地数据与快照
          </li>
        </ul>
      </div>
    </section>
  );
}

function FinalCta() {
  return (
    <section className="final-cta section-shell">
      <img src={iconUrl} alt="" width={48} height={48} />
      <div>
        <h2>把环境配置的主动权拿回来。</h2>
        <p>免费下载 EnvBox，开始整理你的 Windows 开发环境。</p>
      </div>
      <DownloadButton />
    </section>
  );
}

function Footer() {
  return (
    <footer className="footer">
      <div className="section-shell footer-inner">
        <div className="brand footer-brand">
          <img src={iconUrl} alt="" width={28} height={28} />
          <span>EnvBox</span>
        </div>
        <p>统一识别、管理与切换 Windows 开发环境。</p>
        <div className="footer-links">
          <a href="#safety">安全机制</a>
          <a href="#compatibility">系统要求</a>
          <a href="mailto:xu814667@gmail.com">联系作者</a>
        </div>
        <p className="copyright">© 2026 Nagnip</p>
      </div>
    </footer>
  );
}

function App() {
  useEffect(() => {
    document.documentElement.style.setProperty("--photo-movement", `url(${movementUrl})`);
    document.documentElement.style.setProperty("--photo-geneva", `url(${genevaUrl})`);
  }, []);

  useEffect(() => {
    const scrollToHash = () => {
      const id = decodeURIComponent(window.location.hash.slice(1));
      if (!id) return;
      const target = document.getElementById(id);
      if (!target) return;
      const previousBehavior = document.documentElement.style.scrollBehavior;
      document.documentElement.style.scrollBehavior = "auto";
      target.scrollIntoView({ block: "start" });
      document.documentElement.style.scrollBehavior = previousBehavior;
    };
    const timeout = window.setTimeout(scrollToHash, 120);
    window.addEventListener("hashchange", scrollToHash);
    return () => {
      window.clearTimeout(timeout);
      window.removeEventListener("hashchange", scrollToHash);
    };
  }, []);

  return (
    <>
      <div className="grain" aria-hidden="true" />
      <Header />
      <main id="main">
        <Hero />
        <ProofRail />
        <ProductStory />
        <Safety />
        <Capabilities />
        <Compatibility />
        <FinalCta />
      </main>
      <Footer />
    </>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
