# EnvBox 官网

官网是与 Tauri 桌面应用隔离的 React/Vite 站点，使用项目根目录已有依赖，不需要再次安装一套前端包。

## 本地开发

```powershell
cd website
npm run dev
```

打开 `http://127.0.0.1:4174`。

## 验证与构建

```powershell
npm run typecheck
npm run build
npm run preview
```

构建结果位于 `website/release/`。本地若要把 `EnvBox.exe` 放进预览下载目录，再运行 `npm run copy-download`。线上下载指向 GitHub Releases。

## 视觉方向

官网视觉按瑞士机械机芯理解：铑灰色盘面、日内瓦纹金属摄影、发蓝钢强调色。PATH 在首屏被做成可点选的机芯刻度，快照是上弦柄。深色是夜灯下的机芯，浅色是冷色珐琅表盘。不使用紫色光球、渐变字或三张同款功能卡。

## 设计边界

- 支持 Windows 10/11。
- 不虚构客户、下载量、媒体背书或用户证言。
- 产品预览只使用写死的脱敏示例数据，不读取访问者或开发机器的环境变量。
- 明暗主题均通过语义化 OKLCH 色彩变量实现。
- 动效尊重 `prefers-reduced-motion`。
