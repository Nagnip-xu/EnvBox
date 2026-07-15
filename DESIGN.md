# EnvBox 设计书

> 一款统一识别、管理与切换开发环境变量的可视化桌面工具
>
> 版本：**v1.0.0 正式版**　作者：**Nagnip**（xu814667@gmail.com）  
> 适用平台：Windows 10/11（架构预留 macOS / Linux）

---

## 1. 项目背景与目标

### 1.1 痛点
开发者本机通常同时存在多套语言运行时与多个版本：JDK 8/11/17/21、Python 3.9/3.11/3.12、Node 16/18/20、Go 1.20/1.22 等。随之而来的问题：

- 环境变量散落在「用户变量 / 系统变量」两处，`PATH` 冗长且难以阅读；
- 想切换某语言版本，要手动改 `JAVA_HOME`、`GOROOT`，还要在 `PATH` 里挪来挪去；
- `PATH` 里存在无效路径、重复项、失效的软件残留，没人清理；
- 改完系统变量常常「不生效」，需要重开终端甚至重启；
- 一旦改错，没有备份，难以回滚。

### 1.2 目标
做一款**可视化、统一管理**本机环境变量的桌面工具 **EnvBox**，核心价值：

1. **一处看全**：自动识别系统级 / 用户级 / 当前进程级的全部环境变量。
2. **PATH 可视化**：把 `PATH` 拆成一行行卡片，检测无效 / 重复 / 冲突项。
3. **多版本 SDK 一键切换**：自动扫描本机的 JDK/Python/Node/Go 等安装，点一下即可切换当前版本。
4. **一键下载安装**：从内置目录选择版本，后台调用包管理器/官方源自动下载安装并完成环境变量配置（用户一次授权即可）。
5. **一键卸载清理**：删除某个版本或整套环境，并**自动清理它遗留的环境变量与 PATH 条目**（这是市面工具普遍缺失的能力）。
6. **安全可回滚**：任何修改前自动快照，支持一键恢复。
7. **好看好用**：简约、现代、深浅色主题，符合现代审美。

### 1.3 非目标（v1 不做）
- 不做团队协作 / 云端账号同步（先做好单机；配置可通过导出文件手动共享）。
- 不做除 JDK/Python/Node/Go 之外语言的下载安装（v1 先覆盖这四类主流；框架可扩展）。
- 不做 macOS / Linux（架构预留，v2 再做）。

---

## 2. 技术选型

| 层 | 选型 | 理由 |
|---|---|---|
| 桌面框架 | **Tauri 2.x** | 安装包小（~5MB）、内存占用低、启动快；Rust 后端直接、安全地操作注册表与进程 |
| 后端语言 | **Rust** | 稳定操作 Windows 注册表、进程、文件；类型安全 |
| 前端框架 | **React 18 + TypeScript** | 生态成熟，组件化 |
| UI 组件 | **shadcn/ui + Radix** | 简约现代，可定制，无重样式包袱 |
| 样式 | **Tailwind CSS** | 快速构建一致的现代界面 |
| 状态管理 | **Zustand** | 轻量，够用 |
| 图标 | **lucide-react** | 线性图标，风格统一 |
| 构建 | **Vite** | 前端快速构建 |
| 下载/安装引擎 | **winget + Scoop（+ 直连官方源兜底）** | 复用系统级成熟包管理器，安全、可命令化、可读进度；避免自己维护下载逻辑 |
| 网络请求 | **reqwest（Rust）** | 直连官方源下载压缩包、校验哈希时使用 |

> 备选方案：若团队更熟悉前端全栈，可用 **Electron + Node.js**（用 `regedit` / `child_process` 操作注册表），代价是体积和内存更大。本设计书以 Tauri 方案为主线。

### 2.1 下载安装的实现策略（重要）
EnvBox 不自己造下载器，而是做「**编排器（orchestrator）**」，按优先级选择后端引擎：

1. **winget**（首选）：Win10/11 自带，命令化强。例：`winget install EclipseAdoptium.Temurin.17.JDK`。机器级安装触发一次 UAC。
2. **Scoop**（用户级免管理员）：适合不想弹 UAC 的场景，安装到用户目录，天然多版本共存。例：`scoop install openjdk17`。
3. **直连官方源兜底**：当上面都不可用时，从内置目录里维护的官方下载地址拉取压缩包（Adoptium / python.org / nodejs.org / go.dev），**校验 SHA256**，解压到受管目录 `%LocalAppData%\EnvBox\runtimes\`。

安装流程统一为：**选版本 → 用户授权 → 后台执行命令并实时回传进度 → 自动写入/配置对应环境变量 → 广播刷新 → 完成**。整个过程可在界面看到日志与进度条。

---

## 3. 系统架构

```
┌───────────────────────────────────────────────┐
│                   前端 (WebView)                │
│  React + TS + Tailwind + shadcn/ui             │
│  ┌────────┬────────┬────────┬────────┬────────┐│
│  │概览面板│PATH管理│SDK 中心│快照回滚│  设置  ││
│  │        │        │装/切/删│        │        ││
│  └────────┴────────┴────────┴────────┴────────┘│
└───────────────┬───────────────────────────────┘
                │  Tauri IPC (invoke / event + 进度流)
┌───────────────┴───────────────────────────────┐
│                后端 (Rust Core)                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ 变量读写 │ │ SDK扫描器│ │ 快照管理 │        │
│  │ (注册表) │ │ (探测器) │ │(JSON备份)│        │
│  └──────────┘ └──────────┘ └──────────┘        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ 安装编排 │ │ 卸载清理 │ │ 版本目录 │        │
│  │winget/   │ │(删目录+  │ │(可装版本 │        │
│  │scoop/直连│ │ 清变量)  │ │ 元数据)  │        │
│  └──────────┘ └──────────┘ └──────────┘        │
│  ┌────────────────────────────────────────┐    │
│  │ 变更广播 WM_SETTINGCHANGE / 权限提升    │    │
│  └────────────────────────────────────────┘    │
└──────┬───────────────────────────┬────────────┘
       │                           │
┌──────┴────────┐        ┌─────────┴──────────┐
│ Windows 注册表 │        │ 外部引擎 / 网络     │
│ HKCU\Environment       │ winget / scoop      │
│ HKLM\...\Environment   │ 官方下载源(reqwest) │
└───────────────┘        └────────────────────┘
```

### 3.1 Windows 环境变量的真相（实现要点）

- **用户变量**：注册表 `HKEY_CURRENT_USER\Environment`
- **系统变量**：注册表 `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`（写入需**管理员权限**）
- **`PATH` 的实际值** = 系统 PATH + 用户 PATH 拼接。
- 含 `%VAR%` 的变量应存为 `REG_EXPAND_SZ`，纯值存 `REG_SZ`。
- 修改后需广播 `WM_SETTINGCHANGE`（参数 `"Environment"`）通知 Explorer 等进程刷新；**已开启的终端不会刷新**，需提示用户重开终端。
- **进程级变量**：仅当前 EnvBox 进程可见，通过 `std::env::vars()` 读取，只读展示，用于对比。

---

## 4. 功能设计

### 4.1 概览面板（Dashboard）
- 三栏/分组展示：**系统变量**、**用户变量**、**进程变量（只读）**。
- 顶部搜索框：按变量名/值实时过滤。
- 每个变量卡片：变量名、值预览（超长折叠）、来源标签、编辑/删除按钮。
- 右上角：`+ 新建变量`、`导入`、`导出`、`刷新`。
- 冲突提示：同名变量在系统级和用户级都存在时高亮，说明「用户级会覆盖」。

### 4.2 PATH 可视化管理器（核心亮点）
把 `PATH` 从一坨分号字符串变成**可排序的卡片列表**：

- 每一行一个路径条目，显示：完整路径、来源（系统/用户）、状态图标。
- **自动体检**：
  - 🔴 无效路径（目录不存在）
  - 🟡 重复项
  - 🔵 指向已知 SDK（自动打标签，如「JDK 17」「Node 18」）
- 支持：拖拽排序、启用/禁用（软删除）、单条删除、新增、去重、一键清理无效项。
- 底部显示 `PATH` 总长度（Windows 对单条注册表值有长度上限，超长告警）。

### 4.3 SDK 多版本管理与切换（核心亮点）
- **自动扫描**本机常见安装位置，识别已安装的版本：
  - JDK：`%ProgramFiles%\Java`、`%ProgramFiles%\Eclipse Adoptium`、Scoop/Chocolatey 目录、`JAVA_HOME` 等；执行 `java -version` 校验。
  - Python：`%LocalAppData%\Programs\Python`、`C:\PythonXX`、Microsoft Store、conda envs；`python --version`。
  - Node：`nvm` 目录、`%ProgramFiles%\nodejs`、Scoop；`node -v`。
  - Go：`%ProgramFiles%\Go`、`GOROOT`；`go version`。
- 每种语言一张卡片，列出所有检测到的版本，**高亮当前生效版本**。
- **一键切换**：例如切 JDK →
  1. 设 `JAVA_HOME = <所选版本目录>`；
  2. 保证 `PATH` 中含 `%JAVA_HOME%\bin` 且排在其它 JDK 之前；
  3. 广播刷新并提示「重开终端生效」。
- 支持手动添加扫描不到的自定义版本路径。

### 4.4 快照与回滚（安全网）
- **任何写操作前自动创建快照**（时间戳 + 操作描述）。
- 快照内容：系统变量 + 用户变量完整 JSON，并记录 schema 与应用版本；两个作用域必须都完整读取成功。
- 支持：查看快照列表、Diff 对比、选择部分变量恢复、一键完整恢复、手动打快照；恢复失败时自动回滚。
- 快照存于 `%AppData%\EnvBox\snapshots\`。
- 快照不加密，但应用数据目录会收紧 Windows ACL；损坏、未来 schema 和 ID 不一致的文件会被拒绝。

### 4.5 导入 / 导出
- 导出：全部或选定变量 → `.json` / `.env` / `.reg`。
- 导入：从文件导入，预览新增/修改差异并选择具体变量后再应用；系统级选择仍需提权。

### 4.6 一键下载安装（核心亮点）
把「下载→安装→配环境变量」压缩成一次点击。

- **内置版本目录**：进入某语言（如 JDK），列出可安装版本（发行版 + 版本号，如 Temurin 8/11/17/21、GraalVM 等），标注是否 LTS、大小、来源引擎。
- **选择安装引擎**：默认自动选择（优先 winget，其次 scoop，最后直连）；高级用户可手动指定。
- **一次授权**：若需管理员（winget 机器级 / choco），仅弹一次 UAC；scoop / 直连解压到用户目录则无需提权。
- **实时进度**：后台并发读取 stdout/stderr，通过 Tauri event 把进度流式回传到界面；日志有行数/字节上限，可跟踪的子进程任务支持取消并终止进程树。
- **装完即配**：安装成功后自动：
  1. 登记该版本到 SDK 列表；
  2. 询问「是否设为当前版本」，是则顺带执行 4.3 的切换逻辑（设 `*_HOME` + 调 PATH + 广播）。
- **镜像加速**：针对国内网络慢，设置里可为直连下载配置镜像源（如清华 TUNA、华为云等），显著提速。
- **断点/失败处理**：下载失败可重试；安装失败保留日志并提示原因，不残留半成品。

### 4.7 一键卸载与清理（核心亮点）
删除得干净，是本工具区别于手动删文件夹的关键。

- **卸载单个版本**：
  1. 优先根据原子持久化的安装归属清单，使用原始包 ID 调用对应引擎卸载（`winget uninstall` / `scoop uninstall`）；
  2. **联动清理**：扫描并移除指向该版本的 PATH 条目、以及仅为它服务的环境变量（如该 JDK 是当前 `JAVA_HOME` 指向对象时，提示改指其它版本或清空）；
  3. 更新 SDK 列表，广播刷新。
- **卸载整套环境**：一键移除某语言的**所有**已管理版本及相关变量（二次确认 + 快照）。
- **残留体检**：提供「清理助手」，扫描 PATH / 变量中指向**已不存在目录**的 SDK 残留，一键清除（很多人卸载软件后 PATH 里留着死路径，这里统一收拾）。
- **安全约束**：卸载前强制快照；对非 EnvBox 安装的软件，卸载走系统卸载程序而非直接删目录，避免破坏。

### 4.8 权限处理
- 启动时以普通权限运行（只读全部、可改用户变量、可用 scoop 装到用户目录）。
- 当用户尝试修改**系统变量**、或用 winget/choco 做机器级安装/卸载时，弹窗请求 UAC 提权（Tauri 侧重启为管理员，或调用带 `runas` 的 helper 进程执行该批操作）。
- 尽量**批量合并**需要提权的操作，减少反复弹 UAC。

### 4.9 补充功能（v1 一并纳入）
- **临时终端启动器**：用某个选定版本「打开一个新终端」，仅对该会话注入环境变量，不改全局——适合临时试用某版本而不影响系统。
- **环境体检报告**：一键生成本机环境健康报告（无效 PATH、重复项、冲突变量、多版本混乱、缺失 `*_HOME` 等），给出修复建议。
- **全局命令面板（Ctrl+K）**：快速搜索变量、跳转功能、执行「切换 JDK 17」等操作，键盘流。
- **变更历史 / 操作审计**：记录每一次写操作（时间、内容、来自哪个功能），可追溯，与快照联动。
- **系统托盘常驻**：最小化到托盘，托盘菜单可快速切换常用 SDK 版本。
- **配置模板/预设**：把「一整套环境配置」存成预设（如「Java 后端开发」= JDK17 + Maven + 相关变量），一键套用或分享。
- **首次启动向导**：引导做一次环境体检 + 自动建立初始快照，给用户安全感。

---

## 5. 界面设计（UI/UX）

### 5.1 设计语言
- **风格**：简约、留白充足、卡片式、圆角（`rounded-xl`）、柔和阴影。
- **主题**：深色为主，支持浅色一键切换；跟随系统。
- **主色**：中性灰底 + 一个强调色（建议 Indigo/蓝紫），状态色语义化（红=错误、黄=警告、绿=正常）。
- **字体**：界面用系统 UI 字体；路径/值等用等宽字体（如 `JetBrains Mono`）。
- **布局**：左侧竖向导航栏 + 右侧主内容区。

### 5.2 布局草图

```
┌────────────┬──────────────────────────────────────────┐
│  EnvBox    │  [ 🔍 搜索变量... ]        [导入][导出][+]│
│            │──────────────────────────────────────────│
│ ▸ 概览      │  当前视图内容区                            │
│ ▸ PATH     │  ┌────────────────────────────────────┐  │
│ ▸ SDK 中心  │  │ JAVA_HOME   C:\...\jdk-17   [系统]  │  │
│ ▸ 快照      │  │ (装/切/卸一体，含下载进度)          │  │
│ ▸ 设置      │  └────────────────────────────────────┘  │
│            │                                           │
│  ◐ 主题     │  底部状态栏：PATH 长度 / 未保存更改提示    │
└────────────┴──────────────────────────────────────────┘
```

### 5.3 关键交互
- 所有**破坏性操作**（删除、切换、清理）都有确认，并说明「已自动创建快照」。
- 修改后统一在顶部出现「有 N 处未应用的更改 [应用][放弃]」的可撤销条（也可设计为即时生效 + 快照回滚）。
- 应用系统变量成功后，Toast 提示「已生效，请重新打开终端」。

---

## 6. 数据结构（示例）

```ts
type EnvScope = 'system' | 'user' | 'process';

interface EnvVar {
  name: string;
  value: string;
  scope: EnvScope;
  isExpandable: boolean;   // 是否含 %VAR% (REG_EXPAND_SZ)
  conflictsWith?: EnvScope; // 是否与另一作用域同名
}

interface PathEntry {
  raw: string;
  resolved: string;        // 展开 %VAR% 后的真实路径
  scope: 'system' | 'user';
  exists: boolean;
  duplicate: boolean;
  sdkTag?: string;         // "JDK 17" / "Node 18" ...
  enabled: boolean;
}

type SdkKind = 'jdk' | 'python' | 'node' | 'go';
type InstallEngine = 'winget' | 'scoop' | 'direct';

interface SdkVersion {
  kind: SdkKind;
  version: string;
  home: string;            // 安装目录
  isCurrent: boolean;
  source: 'scan' | 'manual' | 'envbox'; // envbox = 本工具安装，可被本工具卸载
  managedBy?: InstallEngine;            // 由哪个引擎安装（决定如何卸载）
}

// 内置目录里「可安装」的候选版本
interface InstallableVersion {
  kind: SdkKind;
  distro: string;          // 发行版，如 "Temurin" / "GraalVM"
  version: string;
  isLts: boolean;
  engines: InstallEngine[];// 可用的安装方式
  packageId?: string;      // winget/scoop 包 id
  downloadUrl?: string;    // 直连兜底地址
  sha256?: string;         // 直连校验
  sizeMB?: number;
}

// 安装/卸载任务的实时进度（通过 event 流式推送）
interface JobProgress {
  jobId: string;
  action: 'install' | 'uninstall';
  target: string;          // "Temurin JDK 17"
  phase: 'downloading' | 'installing' | 'configuring' | 'cleaning' | 'cancelled' | 'done' | 'error';
  percent?: number;
  logLine?: string;
}

interface Snapshot {
  id: string;
  createdAt: string;
  description: string;
  system: Record<string, string>;
  user: Record<string, string>;
}
```

### 6.1 后端 Rust 命令接口（Tauri `invoke`）

```rust
list_env_vars(scope) -> Vec<EnvVar>
set_env_var(scope, name, value)         // 系统级触发提权
delete_env_var(scope, name)
get_path_entries() -> Vec<PathEntry>
save_path_entries(scope, entries)
scan_sdks() -> Vec<SdkVersion>
switch_sdk(kind, home)
create_snapshot(description) -> Snapshot
list_snapshots() -> Vec<Snapshot>
restore_snapshot(id)
export_vars(path, format)
import_vars(path)
broadcast_env_change()                  // 发送 WM_SETTINGCHANGE

// —— 下载安装 / 卸载 ——
list_installable(kind) -> Vec<InstallableVersion>   // 读取内置版本目录
install_sdk(kind, version, engine) -> jobId         // 后台任务，进度用 event 推送
uninstall_sdk(kind, home) -> jobId                  // 卸载 + 联动清理变量/PATH
uninstall_all(kind) -> jobId                        // 卸载某语言全部版本
cancel_job(jobId)
scan_orphans() -> Vec<PathEntry>                    // 残留（指向已删目录）体检
clean_orphans(entries)                              // 一键清理残留

// —— 补充功能 ——
open_terminal_with(kind, home)          // 临时终端，仅该会话注入变量
health_check() -> HealthReport          // 环境体检报告
list_audit_log() -> Vec<AuditEntry>     // 变更历史
apply_preset(presetId)                  // 套用配置模板
```

> 事件通道：安装/卸载通过 `emit("job://progress", JobProgress)` 流式推送，前端订阅后实时刷新进度条与日志。

---

## 7. 目录结构（建议）

```
EnvBox/
├─ src-tauri/            # Rust 后端
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ env_registry.rs   # 注册表读写 + 广播
│  │  ├─ path_manager.rs   # PATH 解析/体检/残留清理
│  │  ├─ sdk_scanner.rs    # 各语言探测器
│  │  ├─ installer.rs      # 安装编排 (winget/scoop/直连) + 进度事件
│  │  ├─ uninstaller.rs    # 卸载 + 联动清理变量/PATH
│  │  ├─ catalog.rs        # 内置可安装版本目录 + 镜像源
│  │  ├─ snapshot.rs       # 快照 + 审计日志
│  │  └─ commands.rs       # Tauri 命令入口
│  ├─ catalog.json         # 版本目录数据（可随更新下发）
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ src/                  # 前端 React
│  ├─ pages/  (Dashboard, PathManager, SdkCenter, Snapshots, Settings)
│  ├─ components/
│  ├─ store/             # zustand
│  ├─ lib/               # invoke 封装
│  └─ App.tsx
├─ package.json
└─ DESIGN.md
```

---

## 8. 开发路线图（里程碑）

| 阶段 | 内容 | 产出 |
|---|---|---|
| **M0 脚手架** | Tauri + React + Tailwind + shadcn 初始化，跑通前后端通信 | 空壳应用 |
| **M1 只读概览** | 读取系统/用户/进程变量并展示、搜索 | 能看全部变量 |
| **M2 变量增删改** | 用户变量读写 + 系统变量提权写入 + 变更广播 | 基础可用 |
| **M3 PATH 管理器** | PATH 拆分、体检、拖拽排序、清理 | 核心亮点 1 |
| **M4 SDK 扫描与切换** | JDK/Python/Node/Go 扫描与一键切换 | 核心亮点 2 |
| **M5 快照回滚** | 自动快照、恢复、Diff、审计日志 | 安全网 |
| **M6 一键下载安装** | 版本目录 + winget/scoop/直连编排 + 进度流 + 装完即配 + 镜像加速 | 核心亮点 3 |
| **M7 一键卸载清理** | 卸载版本/整套 + 联动清理变量与 PATH + 残留体检 | 核心亮点 4 |
| **M8 补充能力** | 临时终端、环境体检、命令面板、托盘、配置模板 | 体验增强 |
| **M9 打磨** | 深浅主题、导入导出、动画、打包签名、多语言、系统托盘 | **v1.0.0 已发布** |

---

## 9. 风险与注意事项

1. **管理员权限**：系统变量写入必须提权；设计上尽量把「必须改系统变量」的操作集中，减少反复弹 UAC。
2. **不生效错觉**：务必广播 `WM_SETTINGCHANGE` 并明确提示「已开的终端需重开」，否则用户会以为工具没生效。
3. **PATH 长度上限**：Windows 单条环境变量（注册表 `REG_SZ`）有长度限制，超长要告警，并区分系统/用户两段。
4. **误删风险**：强制自动快照 + 二次确认；系统关键变量（如 `SystemRoot`、`windir`）标记为「危险，不建议修改」。
5. **PATH 顺序敏感**：SDK 切换的本质是「谁在 PATH 里更靠前」，切换逻辑要正确处理排序而不仅是改 `*_HOME`。
6. **跨平台**：macOS/Linux 环境变量在 shell 配置文件（`.zshrc`/`.bashrc`/`.profile`）里，机制完全不同，需另设适配层，v2 再做。
7. **下载安全**：直连下载必须校验 SHA256，只用 HTTPS 官方源/可信镜像；防止供应链污染。版本目录 `catalog.json` 应可校验来源。
8. **卸载破坏性**：只允许直接删「EnvBox 自己安装到受管目录」的版本；对系统里其它软件一律走其自带卸载程序，绝不硬删目录，避免误伤。
9. **引擎依赖**：winget/scoop 可能未安装或版本过旧；需检测可用性并优雅降级（提示安装引擎或改用直连）。
10. **进程占用**：卸载/切换正在被其它进程占用的运行时可能失败；需捕获错误并提示用户关闭相关程序。

---

## 10. 构建与发布

```powershell
# 开发
npm run tauri dev

# 发布构建（生成 release 可执行文件）
npm run tauri build
# 产物：src-tauri/target/release/envbox.exe
```

图标更新流程：替换根目录 `EnBox.png` → 运行 `scripts/process-icon.ps1` → `npm run tauri icon src-tauri/icon-source.png`。

---

## 11. 后续可扩展（v2+）
- 更多语言/工具的下载安装（Rust、.NET、Maven/Gradle、kubectl 等）。
- 项目级 `.env` 管理与一键注入终端（识别 `.envbox` 项目配置，进目录自动切版本）。
- 配置文件云同步 / 团队共享模板。
- 命令行版 `envbox` CLI（脚本化、CI 友好）。
- macOS / Linux 支持。
- 版本目录在线更新（`catalog.json` 定期从服务端拉取，新版本发布即可安装）。
