# EnvBox

EnvBox 是一款面向 Windows 的开发环境管理桌面工具，用于查看和修改环境变量、整理 PATH、扫描和切换 SDK，并在高风险操作前创建可恢复快照。

## 主要能力

- 管理用户级和系统级环境变量
- 检测 PATH 中的重复项、失效项和 SDK 条目
- 扫描并切换 JDK、Python、Node.js、Go、Rust、.NET 等开发工具
- 通过 winget 或 Scoop 安装受支持的 SDK
- 创建、恢复和清理环境快照
- 导入、导出与环境健康检查
- 简体中文、繁体中文、英语、日语及深浅主题

## 安全原则

- 写操作必须先成功创建快照，否则取消操作
- 未提权时不修改系统级变量
- 未找到可信卸载器时不递归删除外部 SDK 目录
- 导入文件先检查尺寸、格式、版本和变量内容
- 疑似令牌、密码和密钥的变量值在列表中默认遮蔽

环境快照和导出文件可能包含敏感变量，请只保存在受信任的位置，不要提交到版本库或发送给他人。

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
```

自动化测试包含前端工具测试、Rust 单元测试和 Windows 只读冒烟测试。涉及注册表写入、提权、安装与卸载的功能仍应在一次性 Windows 虚拟机中按 [TEST_CHECKLIST.md](./TEST_CHECKLIST.md) 完成人工回归。

## 设计资料

- [DESIGN.md](./DESIGN.md)：产品目标、架构与实现说明
- [TEST_CHECKLIST.md](./TEST_CHECKLIST.md)：功能验收和回归清单
- [SECURITY.md](./SECURITY.md)：安全边界与问题报告说明
