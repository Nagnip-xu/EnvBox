# EnvBox

**统一识别、管理与切换 Windows 开发环境的可视化桌面工具。**

当前版本 **v1.0.0**（正式版）  
作者 [Nagnip](mailto:xu814667@gmail.com) · 适用平台 **Windows 10 / 11**

EnvBox 把散落在用户变量、系统变量和 PATH 中的开发配置收进一个界面：查看与修改环境变量、体检并整理 PATH、扫描/切换/安装 SDK，并在每次高风险写入前强制留下可恢复快照。

---

## 官网页面

**有。** 仓库内已包含独立的产品官网，源码在 [`website/`](./website/)，与桌面应用隔离。

官网负责展示产品能力、安全机制与系统兼容性，并提供 Windows 版下载（`EnvBox.exe`）。它是单独的 React + Vite 站点，复用项目根目录的前端依赖，不另装一套包。

| 说明 | 内容 |
|------|------|
| 源码目录 | `website/` |
| 本地预览 | `http://127.0.0.1:4174` |
| 构建产物 | `website/release/` |
| 公网地址 | [https://nagnip-xu.github.io/EnvBox/](https://nagnip-xu.github.io/EnvBox/) |
| Windows 下载 | GitHub Releases 中的 `EnvBox.exe` |

本地运行官网：

```powershell
npm install          # 在项目根目录，只需一次
cd website
npm run dev
```

构建官网（发布前请先把最新 `EnvBox.exe` 放到项目根目录）：

```powershell
cd website
npm run typecheck
npm run build
npm run preview
```

官网设计边界见 [`website/README.md`](./website/README.md) 与 [`PRODUCT.md`](./PRODUCT.md)：不虚构客户、下载量或证言；产品预览使用脱敏示例数据，不读取访问者机器的环境变量。

---

## 它解决什么问题

本机同时装着 JDK 8/11/17/21、Python 3.x、Node、Go、Rust、.NET 时，常见情况是：

- 环境变量分在「用户 / 系统」两处，PATH 又长又难读
- 切换版本要改 `JAVA_HOME` / `GOROOT`，还要在 PATH 里手动挪顺序
- PATH 里残留失效路径、重复项、已卸载软件的死目录
- 改完「不生效」，其实是已开终端没有刷新
- 改错之后没有备份，很难回滚

EnvBox 的定位是：**把这些配置变成可以看清、可以检查、可以安全修改、可以回滚的工作台。** 它是本地工具，不需要账户，数据留在本机。

---

## 主要能力

### 概览：环境变量

- 分组展示系统变量、用户变量、进程变量（进程级只读，用于对比）
- 按变量名或值搜索
- 新建、编辑、删除用户变量；系统变量写入需管理员权限
- 同名变量同时存在于系统级和用户级时高亮冲突（用户级会覆盖）
- 疑似令牌、密码、密钥的变量值在列表中默认遮蔽
- 导入前预览差异并可勾选具体变量；导出前会提示文件可能含敏感值
- 导入文件会检查尺寸（最大 5 MiB）、格式、数据版本和变量内容

### PATH 管理

把 PATH 从分号字符串拆成可排序的条目列表：

- 显示完整路径、展开后的真实路径、来源（系统 / 用户）
- 状态区分：可用、目录缺失、未解析变量、暂时离线的网络路径
- 标记重复项，并为已知 SDK 目录打标签（如 JDK、Node、Python）
- 拖拽排序、单条删除、一键清理**安全可删**的无效项、去重
- 未解析变量和离线 UNC 路径**不会**被一键清理误删
- 底部显示 PATH 总长度，便于留意 Windows 注册表长度上限
- 搜索时隐藏拖拽手柄，避免误改顺序

### SDK 中心

并发扫描本机安装，单个版本探测约 5 秒超时后终止该进程树，其余扫描不受影响。

**可识别并切换：** JDK、Python、Node.js、Go、Rust、.NET、Ruby、PHP、Git、Maven、Gradle、Deno、Bun。

**可通过 winget / Scoop 安装的目录包括：**

| 类型 | 示例发行版 |
|------|------------|
| JDK | Temurin、Oracle、Microsoft OpenJDK、Amazon Corretto、Azul Zulu |
| Node.js | 官方 LTS / Current、nvm-windows |
| Python | 官方 CPython 3.10–3.13、uv |
| Go / Rust / .NET | 官方 Go、rustup、.NET SDK 6/8/9/10 |
| 其它 | RubyInstaller、PHP、Git、Maven、Gradle、Deno、Bun |

其它能力：

- 首次进入显示扫描进度；再次进入先展示缓存，再后台刷新
- 一键设为当前版本（写入对应 `*_HOME` 并调整 PATH 顺序）
- 安装 / 卸载任务流式显示受限日志，可取消 EnvBox 能跟踪到 PID 的子进程
- 识别 nvm、fnm、Volta、Conda、rustup、Jabba 管理的版本，**不与其争用 PATH**，提示使用原生管理器切换
- 用选定版本打开临时终端：只注入当前会话，不改全局配置
- EnvBox 记录自身发起的安装及包标识；无法确认归属时不递归删除外部 SDK 目录

### 快照与回滚

- **任何写操作前必须先成功创建快照**，否则操作取消
- 快照必须完整读取用户与系统注册表；任一读取失败都不会生成不完整文件
- 恢复前预览新增 / 修改 / 删除差异，可只恢复勾选项
- 恢复或导入失败时自动回滚，并报告回滚是否完整
- 选择项由 Rust 后端重新校验，从前端不能提交任意替代值
- 支持手动创建、删除、按天数清理（默认不自动删除）
- 变更历史（审计日志）与快照联动
- 快照存于 `%AppData%\EnvBox\snapshots\`，采用带 `schemaVersion` 的明文 JSON

### 项目版本对照（只读）

在设置中选择项目目录后，识别：

- `.nvmrc`、`.node-version`、`.python-version`、`.java-version`、`.ruby-version`
- `.tool-versions`、`global.json`
- Gradle Wrapper、Maven Wrapper

对照本机已安装 / 当前版本，标出「当前」「已安装但未启用」「缺失」「Wrapper」，**不修改项目文件**。缺失项可跳转到 SDK 中心。

### 环境体检

一键生成本机健康报告：变量冲突、无效 / 重复 PATH、未解析路径、离线网络路径、异常快照、未完成的安装记录，并给出可行动说明。

### 界面与体验

- 五个页面：概览、PATH 管理、SDK 中心、快照回滚、设置
- 全局命令面板（`Ctrl+K`）、搜索聚焦（`Ctrl+F`）
- 简体中文、繁体中文、英语、日语
- 浅色 / 深色 / 跟随系统
- 关闭窗口隐藏到系统托盘，左键托盘或右键「显示窗口」可恢复；右键「退出 EnvBox」才真正退出
- 下载镜像源可在设置中选择官方源、清华 TUNA、华为云、rsproxy（默认）

---

## 安全原则

EnvBox 能修改注册表、PATH 和 SDK 配置，应按高权限本地工具对待。

- 写操作必须先成功创建快照
- 未提权时不修改系统级变量
- 普通用户模式只写入用户级注册表；系统变量恢复与写入需要管理员权限
- `%AppData%\EnvBox` 首次创建快照时会收紧 Windows ACL，仅当前用户、SYSTEM 和本机 Administrators 拥有完全控制；加固失败则不创建快照
- 快照与导出文件**不加密**（按产品设计如此），可能包含令牌和密码；只在受信任的 Windows 账户和磁盘上使用，不要提交到版本库或发给他人
- 损坏、未来 schema、文件名与内部 ID 不一致的快照会被拒绝；旧版无 schema 快照仍按 v1 兼容读取
- 不要导入来源不明的环境备份

完整边界与问题报告方式见 [`SECURITY.md`](./SECURITY.md)。

---

## 技术架构

```
┌─────────────────────────────────────────────┐
│              前端 (WebView2)                 │
│     React 18 + TypeScript + Tailwind CSS    │
│  概览 · PATH · SDK 中心 · 快照 · 设置        │
└────────────────────┬────────────────────────┘
                     │  Tauri IPC / event
┌────────────────────┴────────────────────────┐
│              后端 (Rust / Tauri 2)           │
│  注册表读写 · PATH 体检 · SDK 扫描            │
│  winget/Scoop 编排 · 快照 · 托盘 · 提权       │
└───────┬──────────────────────────┬──────────┘
        │                          │
  Windows 注册表              winget / Scoop
  HKCU\Environment            安装 / 卸载子进程
  HKLM\...\Environment
```

修改环境变量后会广播 `WM_SETTINGCHANGE`（参数 `"Environment"`）。**已经打开的终端不会自动刷新**，需要重新打开终端才能看到新值。

浏览器里运行 `npm run dev` 时使用模拟数据，**所有系统写操作都会被拒绝**。真实读写只发生在桌面应用（`npm run tauri dev` 或打包后的 `EnvBox.exe`）中。

### 目录结构

```
EnvBox/
├─ src/                    # 桌面应用前端
│  ├─ pages/               # Dashboard, PathManager, SdkCenter, Snapshots, Settings
│  ├─ components/          # 侧栏、顶栏、命令面板、对话框、Toast
│  ├─ store/               # Zustand 状态与 SDK 扫描缓存
│  ├─ lib/                 # Tauri invoke 封装；浏览器预览走 mock
│  └─ i18n.ts / theme.ts
├─ src-tauri/              # Rust 后端
│  ├─ src/
│  │  ├─ lib.rs            # Tauri 命令入口；写操作统一先快照
│  │  ├─ env_registry.rs   # 环境变量注册表读写 + 广播
│  │  ├─ path_manager.rs   # PATH 解析、体检、清理、去重
│  │  ├─ sdk_scanner.rs    # 多语言探测器与切换
│  │  ├─ installer.rs      # winget/Scoop 编排、进度、清单、卸载
│  │  ├─ snapshot.rs       # 快照、Diff、选择性恢复、审计
│  │  ├─ misc.rs           # 体检、导入导出、项目检查、临时终端
│  │  ├─ tray.rs / win.rs  # 托盘、提权、Windows API
│  │  └─ models.rs
│  └─ tauri.conf.json
├─ website/                # 产品官网（独立 Vite 站点）
├─ scripts/                # 版本一致性检查、图标处理、冒烟脚本
├─ .github/workflows/ci.yml
├─ DESIGN.md               # 产品目标、架构与实现说明
├─ PRODUCT.md              # 官网定位与转化叙事
├─ SECURITY.md
└─ TEST_CHECKLIST.md       # 功能验收与回归清单
```

### 技术栈

| 层 | 选型 |
|----|------|
| 桌面框架 | Tauri 2.x |
| 后端 | Rust（`winreg` / `winapi`） |
| 前端 | React 18、TypeScript、Vite 6、Tailwind CSS、Zustand、lucide-react |
| 安装引擎 | winget（优先）、Scoop |
| 官网 | 独立 Vite 站点，端口 `4174`，构建输出 `website/release/` |
| 测试 | Vitest（前端）、Cargo 单元测试、Windows 只读 Smoke |

---

## 系统要求

- Windows 10 或更高版本
- [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
- 修改系统变量、恢复系统级快照、部分机器级安装需要管理员权限（UAC）
- SDK 自动安装需要已安装 **winget** 或 **Scoop**（核心的变量 / PATH / 快照能力不依赖它们）
- 开发构建另需 Node.js（CI 使用 22）和 Rust 稳定版

数据目录：`%AppData%\EnvBox\`（快照、安装清单 `installs.json`、审计日志）。

---

## 开发

在项目根目录：

```powershell
npm install
```

### 浏览器预览（只读演示数据）

```powershell
npm run dev
```

前端默认在 `http://localhost:1420`。界面可用，但写操作会提示「请在桌面应用中进行」。

### 桌面应用

```powershell
npm run tauri dev
```

### 发布构建

```powershell
npm run tauri build
```

产物位于 `src-tauri/target/release/`（可执行文件名以打包配置为准）。把生成的 `EnvBox.exe` 放到仓库根目录后，再构建官网即可带上下载文件。

图标更新：替换根目录图标源文件后运行 `scripts/process-icon.ps1`，再执行 `npm run tauri icon src-tauri/icon-source.png`。

版本号必须在以下四处保持一致（`npm run check:version` 会检查）：

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src/version.ts`

---

## 验证

```powershell
npm run build
npm run test:all
npm run check
```

| 命令 | 作用 |
|------|------|
| `npm test` | 前端 Vitest（i18n、主题、缓存等） |
| `npm run test:rust` | Rust 单元测试（PATH、SDK、选择性操作、格式兼容、清单与输入校验） |
| `npm run test:smoke` | Windows 只读冒烟测试（不写注册表） |
| `npm run test:all` | 以上全部 |
| `npm run check` | 版本一致性 + 前端构建 + 全套测试 + `cargo fmt --check` + 严格 Clippy |

涉及注册表写入、提权、安装与卸载的功能，仍应在一次性 Windows 虚拟机中按 [`TEST_CHECKLIST.md`](./TEST_CHECKLIST.md) 做人工回归。

推送到 GitHub 的 `main` 或发起 Pull Request 时，[Windows CI](./.github/workflows/ci.yml) 会自动跑同类检查，并执行 Tauri debug 构建（`--no-bundle`）。

---

## 设计资料

- [`DESIGN.md`](./DESIGN.md) — 产品目标、架构、功能设计与实现要点
- [`PRODUCT.md`](./PRODUCT.md) — 官网定位、转化路径与品牌原则
- [`website/README.md`](./website/README.md) — 官网本地开发与构建
- [`TEST_CHECKLIST.md`](./TEST_CHECKLIST.md) — 功能验收和回归清单
- [`SECURITY.md`](./SECURITY.md) — 安全边界与问题报告说明

---

## 当前范围与后续方向

v1 聚焦 **Windows 单机**。不做云端账号同步、不做 macOS / Linux（架构预留）。配置可通过导出文件手动共享。

设计书中列出的后续方向包括：更多工具链、项目级自动切版本、团队模板、CLI，以及官网在线托管与版本目录在线更新。

---

## 联系

作者 **Nagnip** · [xu814667@gmail.com](mailto:xu814667@gmail.com)

报告问题时请勿附带真实环境导出、完整快照、令牌或个人路径；提供脱敏后的复现步骤、EnvBox 版本、Windows 版本和相关错误信息即可。
