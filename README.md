# EnvBox

EnvBox 是一款面向 Windows 的开发环境管理桌面工具，用于查看和修改环境变量、整理 PATH、扫描和切换 SDK，并在高风险操作前创建可恢复快照。

## 主要能力

- 管理用户级和系统级环境变量
- 检测 PATH 中的重复项、失效项和 SDK 条目
- 并发扫描并切换 JDK、Python、Node.js、Go、Rust、.NET 等开发工具，单个版本探测超时后自动终止进程树
- 通过 winget 或 Scoop 安装受支持的 SDK
- 安装/卸载任务实时显示受限日志，并支持取消可跟踪的子进程任务
- 创建、恢复和清理带格式版本的环境快照
- 恢复前预览变量新增、修改和删除差异，可只恢复选中项，失败时自动回滚
- 导入前预览并选择具体变量；导出与环境健康检查
- 只读识别项目中的 `.nvmrc`、`.python-version`、`.tool-versions`、`global.json` 和构建 Wrapper，并对照本机已安装/当前版本
- 简体中文、繁体中文、英语、日语及深浅主题

## 安全原则

- 写操作必须先成功创建快照，否则取消操作
- 快照必须完整读取用户与系统注册表；任一读取失败都不会生成不完整快照
- 未提权时不修改系统级变量
- EnvBox 记录自身发起的安装及包标识；未确认归属或可信卸载器时不递归删除外部 SDK 目录
- 识别 nvm、fnm、Volta、Conda、rustup、Jabba 管理的版本，不与其争用 PATH
- 导入文件先检查尺寸、格式、版本和变量内容
- 疑似令牌、密码和密钥的变量值在列表中默认遮蔽

环境快照和导出文件可能包含敏感变量。EnvBox 会收紧应用数据目录的 Windows ACL，但按产品设计**不加密快照内容**；请只在受信任的 Windows 账户和磁盘上使用，不要把文件提交到版本库或发送给他人。

## 开发

需要 Node.js、Rust 和 Windows WebView2。

```powershell
npm install
npm run dev
```

浏览器预览使用模拟数据，所有系统写操作都会被拒绝。运行桌面应用：

```powershell
npm run tauri dev
```

## 验证

```powershell
npm run build
npm run test:all
npm run check
```

自动化测试包含前端工具测试、Rust 单元测试和 Windows 只读冒烟测试。涉及注册表写入、提权、安装与卸载的功能仍应在一次性 Windows 虚拟机中按 [TEST_CHECKLIST.md](./TEST_CHECKLIST.md) 完成人工回归。

`npm run check` 还会检查四处版本号一致性、Rust 格式和严格 Clippy。仓库包含 Windows GitHub Actions 工作流，推送到 GitHub 后会自动执行同类检查及 Tauri 桌面构建。

## 设计资料

- [DESIGN.md](./DESIGN.md)：产品目标、架构与实现说明
- [TEST_CHECKLIST.md](./TEST_CHECKLIST.md)：功能验收和回归清单
- [SECURITY.md](./SECURITY.md)：安全边界与问题报告说明
