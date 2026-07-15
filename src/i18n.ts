export type Lang = "zh" | "zhHant" | "en" | "ja";

export const LANGS: { id: Lang; label: string }[] = [
  { id: "zh", label: "简体中文" },
  { id: "zhHant", label: "繁體中文" },
  { id: "en", label: "English" },
  { id: "ja", label: "日本語" },
];

type Entry = Partial<Record<Lang, string>>;
type Params = Record<string, string | number>;

const dict: Record<string, Entry> = {
  // ── 侧边栏 / 导航 ─────────────────────────────
  "app.name": { zh: "EnvBox", zhHant: "EnvBox", en: "EnvBox", ja: "EnvBox" },
  "app.edition": { zh: "正式版", zhHant: "正式版", en: "Stable", ja: "安定版" },
  "nav.dashboard": { zh: "概览", zhHant: "總覽", en: "Overview", ja: "概要" },
  "nav.path": { zh: "PATH 管理", zhHant: "PATH 管理", en: "PATH Manager", ja: "PATH 管理" },
  "nav.sdk": { zh: "SDK 中心", zhHant: "SDK 中心", en: "SDK Center", ja: "SDK センター" },
  "nav.snapshots": { zh: "快照回滚", zhHant: "快照還原", en: "Snapshots", ja: "スナップショット" },
  "nav.settings": { zh: "设置", zhHant: "設定", en: "Settings", ja: "設定" },

  // ── 顶栏 ─────────────────────────────────────
  "topbar.search": {
    zh: "搜索变量名或值...",
    zhHant: "搜尋變數名稱或值...",
    en: "Search variable name or value...",
    ja: "変数名または値を検索...",
  },
  "topbar.refresh": { zh: "刷新", zhHant: "重新整理", en: "Refresh", ja: "更新" },
  "topbar.import": { zh: "导入", zhHant: "匯入", en: "Import", ja: "インポート" },
  "topbar.export": { zh: "导出", zhHant: "匯出", en: "Export", ja: "エクスポート" },
  "topbar.newVar": { zh: "新建变量", zhHant: "新增變數", en: "New Variable", ja: "変数を追加" },
  "topbar.browserPreview": {
    zh: "浏览器预览 · 演示数据",
    zhHant: "瀏覽器預覽 · 示範資料",
    en: "Browser preview · demo data",
    ja: "ブラウザプレビュー · デモデータ",
  },
  "topbar.exportConfirmTitle": {
    zh: "导出环境变量",
    zhHant: "匯出環境變數",
    en: "Export environment variables",
    ja: "環境変数をエクスポート",
  },
  "topbar.exportConfirmMessage": {
    zh: "导出文件可能包含令牌、密码和其他敏感值。请仅保存到受信任的位置，并避免提交到版本库。",
    zhHant: "匯出檔案可能包含權杖、密碼和其他敏感值。請僅儲存到受信任的位置，並避免提交到版本庫。",
    en: "The export may contain tokens, passwords, and other sensitive values. Save it only to a trusted location and do not commit it to version control.",
    ja: "エクスポートにはトークン、パスワードなどの機密値が含まれる場合があります。信頼できる場所にのみ保存し、バージョン管理へ登録しないでください。",
  },
  "topbar.importConfirmTitle": {
    zh: "确认导入环境变量",
    zhHant: "確認匯入環境變數",
    en: "Confirm environment import",
    ja: "環境変数のインポートを確認",
  },
  "topbar.importConfirmMessage": {
    zh: "文件包含用户变量 {user} 个、系统变量 {system} 个，其中疑似敏感变量 {sensitive} 个。导入前将强制创建安全快照。",
    zhHant: "檔案包含使用者變數 {user} 個、系統變數 {system} 個，其中疑似敏感變數 {sensitive} 個。匯入前將強制建立安全快照。",
    en: "The file contains {user} user variables and {system} system variables, including {sensitive} potentially sensitive variables. A safety snapshot will be required before import.",
    ja: "ファイルにはユーザー変数 {user} 件、システム変数 {system} 件、機密の可能性がある変数 {sensitive} 件が含まれます。インポート前に安全スナップショットを作成します。",
  },

  // ── 通用 ─────────────────────────────────────
  "common.cancel": { zh: "取消", zhHant: "取消", en: "Cancel", ja: "キャンセル" },
  "common.confirm": { zh: "确定", zhHant: "確定", en: "Confirm", ja: "確定" },
  "common.close": { zh: "关闭", zhHant: "關閉", en: "Close", ja: "閉じる" },
  "common.save": { zh: "保存", zhHant: "儲存", en: "Save", ja: "保存" },
  "common.delete": { zh: "删除", zhHant: "刪除", en: "Delete", ja: "削除" },
  "common.edit": { zh: "编辑", zhHant: "編輯", en: "Edit", ja: "編集" },
  "common.run": { zh: "执行", zhHant: "執行", en: "Run", ja: "実行" },
  "common.browse": { zh: "浏览...", zhHant: "瀏覽...", en: "Browse...", ja: "参照..." },
  "common.reset": { zh: "重置", zhHant: "重設", en: "Reset", ja: "リセット" },
  "common.revealSecret": {
    zh: "显示敏感值",
    zhHant: "顯示敏感值",
    en: "Reveal sensitive value",
    ja: "機密値を表示",
  },
  "common.hideSecret": {
    zh: "隐藏敏感值",
    zhHant: "隱藏敏感值",
    en: "Hide sensitive value",
    ja: "機密値を隠す",
  },

  // ── 作用域 ───────────────────────────────────
  "scope.system": { zh: "系统变量", zhHant: "系統變數", en: "System Variables", ja: "システム変数" },
  "scope.user": { zh: "用户变量", zhHant: "使用者變數", en: "User Variables", ja: "ユーザー変数" },
  "scope.process": {
    zh: "进程变量（只读）",
    zhHant: "行程變數（唯讀）",
    en: "Process Variables (read-only)",
    ja: "プロセス変数（読み取り専用）",
  },
  "scope.system.short": { zh: "系统", zhHant: "系統", en: "System", ja: "システム" },
  "scope.user.short": { zh: "用户", zhHant: "使用者", en: "User", ja: "ユーザー" },

  // ── 概览页 ───────────────────────────────────
  "dash.noMatch": { zh: "无匹配变量", zhHant: "無相符變數", en: "No matching variables", ja: "一致する変数なし" },
  "dash.newTitle": { zh: "新建变量", zhHant: "新增變數", en: "New Variable", ja: "変数を追加" },
  "dash.editTitle": { zh: "编辑 {name}", zhHant: "編輯 {name}", en: "Edit {name}", ja: "{name} を編集" },
  "dash.scopeLabel": { zh: "作用域", zhHant: "作用域", en: "Scope", ja: "スコープ" },
  "dash.scope.user": { zh: "用户变量", zhHant: "使用者變數", en: "User", ja: "ユーザー変数" },
  "dash.scope.system": {
    zh: "系统变量（需管理员）",
    zhHant: "系統變數（需管理員）",
    en: "System (admin required)",
    ja: "システム変数（管理者必要）",
  },
  "dash.nameLabel": { zh: "变量名", zhHant: "變數名稱", en: "Name", ja: "変数名" },
  "dash.namePlaceholder": { zh: "例如 JAVA_HOME", zhHant: "例如 JAVA_HOME", en: "e.g. JAVA_HOME", ja: "例：JAVA_HOME" },
  "dash.valueLabel": { zh: "值", zhHant: "值", en: "Value", ja: "値" },
  "dash.valuePlaceholder": {
    zh: "变量值，可包含 %OTHER_VAR%",
    zhHant: "變數值，可包含 %OTHER_VAR%",
    en: "Value, may contain %OTHER_VAR%",
    ja: "値（%OTHER_VAR% を含められます）",
  },
  "dash.conflict": { zh: "冲突", zhHant: "衝突", en: "Conflict", ja: "競合" },
  "dash.conflictTip": {
    zh: "该变量在系统级与用户级都存在，用户级会覆盖",
    zhHant: "此變數在系統層級與使用者層級都存在，使用者層級會覆蓋",
    en: "Exists in both system and user scope; user scope takes precedence",
    ja: "システムとユーザーの両方に存在し、ユーザー側が優先されます",
  },
  "dash.hasVar": { zh: "含 %VAR%", zhHant: "含 %VAR%", en: "has %VAR%", ja: "%VAR% あり" },
  "dash.delTitle": { zh: "删除变量", zhHant: "刪除變數", en: "Delete Variable", ja: "変数を削除" },
  "dash.delMsg": {
    zh: "确定删除{scope}变量 {name} 吗？",
    zhHant: "確定刪除{scope}變數 {name} 嗎？",
    en: "Delete the {scope} variable {name}?",
    ja: "{scope}変数 {name} を削除しますか？",
  },

  // ── PATH 管理页 ──────────────────────────────
  "path.stat.total": { zh: "条目总数", zhHant: "項目總數", en: "Total Entries", ja: "項目総数" },
  "path.stat.invalid": { zh: "无效路径", zhHant: "無效路徑", en: "Invalid Paths", ja: "無効パス" },
  "path.stat.dup": { zh: "重复项", zhHant: "重複項", en: "Duplicates", ja: "重複" },
  "path.stat.length": { zh: "PATH 长度", zhHant: "PATH 長度", en: "PATH Length", ja: "PATH 長" },
  "path.unit.chars": { zh: " 字符", zhHant: " 字元", en: " chars", ja: " 文字" },
  "path.clean": { zh: "一键清理无效项", zhHant: "一鍵清理無效項", en: "Clean Invalid", ja: "無効項目を一括整理" },
  "path.dedupe": { zh: "去重", zhHant: "去重", en: "Deduplicate", ja: "重複を削除" },
  "path.reorderTip": {
    zh: "提示：拖动左侧手柄可在同一作用域内调整 PATH 顺序（越靠前优先级越高）。",
    zhHant: "提示：拖曳左側控制點可在同一作用域內調整 PATH 順序（越靠前優先度越高）。",
    en: "Tip: drag the handle to reorder PATH within the same scope (earlier = higher priority).",
    ja: "ヒント：左のハンドルをドラッグして同一スコープ内で PATH の順序を変更できます（前ほど優先）。",
  },
  "path.noMatch": { zh: "无匹配条目", zhHant: "無相符項目", en: "No matching entries", ja: "一致する項目なし" },
  "path.dupBadge": { zh: "重复", zhHant: "重複", en: "Duplicate", ja: "重複" },
  "path.moveUp": { zh: "上移", zhHant: "上移", en: "Move up", ja: "上へ" },
  "path.moveDown": { zh: "下移", zhHant: "下移", en: "Move down", ja: "下へ" },
  "path.cleanTitle": { zh: "清理无效项", zhHant: "清理無效項", en: "Clean Invalid Entries", ja: "無効項目を整理" },
  "path.cleanMsg": {
    zh: "确定从用户级与系统级 PATH 中移除所有指向不存在目录的条目吗？当前无效路径 {n} 项。",
    zhHant: "確定從使用者級與系統級 PATH 中移除所有指向不存在目錄的項目嗎？目前無效路徑 {n} 項。",
    en: "Remove all entries pointing to non-existent folders from user and system PATH? Currently {n} invalid.",
    ja: "存在しないフォルダを指す項目をユーザーとシステムの PATH から削除しますか？現在 {n} 件が無効です。",
  },
  "path.dedupeTitle": { zh: "去重", zhHant: "去重", en: "Deduplicate", ja: "重複を削除" },
  "path.dedupeMsg": {
    zh: "确定移除用户级与系统级 PATH 中的重复条目（保留首次出现）吗？当前重复项 {n} 项。",
    zhHant: "確定移除使用者級與系統級 PATH 中的重複項目（保留首次出現）嗎？目前重複項 {n} 項。",
    en: "Remove duplicate entries from user and system PATH (keep first occurrence)? Currently {n} duplicates.",
    ja: "ユーザーとシステムの PATH から重複項目を削除しますか（最初の出現を保持）？現在 {n} 件の重複です。",
  },
  "path.delTitle": { zh: "删除 PATH 条目", zhHant: "刪除 PATH 項目", en: "Delete PATH Entry", ja: "PATH 項目を削除" },
  "path.delMsg": {
    zh: "确定从 {scope} PATH 中删除该条目吗？\n{raw}",
    zhHant: "確定從 {scope} PATH 中刪除此項目嗎？\n{raw}",
    en: "Delete this entry from the {scope} PATH?\n{raw}",
    ja: "{scope} PATH からこの項目を削除しますか？\n{raw}",
  },
  "path.toast.done": {
    zh: "{label}：处理了 {n} 项",
    zhHant: "{label}：處理了 {n} 項",
    en: "{label}: {n} entries processed",
    ja: "{label}：{n} 件を処理しました",
  },
  "path.toast.none": {
    zh: "{label}：没有可处理的项",
    zhHant: "{label}：沒有可處理的項",
    en: "{label}: nothing to process",
    ja: "{label}：処理対象がありません",
  },
  "path.toast.partFail": {
    zh: "部分操作失败（{n} 处）",
    zhHant: "部分操作失敗（{n} 處）",
    en: "Some operations failed ({n})",
    ja: "一部の操作が失敗しました（{n} 件）",
  },
  "path.toast.deleted": { zh: "已删除该条目", zhHant: "已刪除此項目", en: "Entry deleted", ja: "項目を削除しました" },
  "path.reorderFail": { zh: "调整顺序失败", zhHant: "調整順序失敗", en: "Reorder failed", ja: "並べ替えに失敗" },
  "path.delFail": { zh: "删除失败", zhHant: "刪除失敗", en: "Delete failed", ja: "削除に失敗" },
  "path.sysPermErr": {
    zh: "修改系统级 PATH 需要管理员权限。",
    zhHant: "修改系統級 PATH 需要管理員權限。",
    en: "Editing the system PATH requires administrator privileges.",
    ja: "システム PATH の変更には管理者権限が必要です。",
  },
  "path.sysPermErrShort": {
    zh: "系统级修改需要管理员权限。",
    zhHant: "系統級修改需要管理員權限。",
    en: "System-level changes require administrator privileges.",
    ja: "システム単位の変更には管理者権限が必要です。",
  },

  // ── SDK 中心 ─────────────────────────────────
  "sdk.scanning": {
    zh: "正在扫描本机已安装的 SDK...",
    zhHant: "正在掃描本機已安裝的 SDK...",
    en: "Scanning installed SDKs...",
    ja: "インストール済み SDK をスキャン中...",
  },
  "sdk.refreshing": {
    zh: "正在刷新 SDK 列表...",
    zhHant: "正在重新整理 SDK 清單...",
    en: "Refreshing SDK list...",
    ja: "SDK リストを更新中...",
  },
  "sdk.envVar": { zh: "环境变量：{var}", zhHant: "環境變數：{var}", en: "Env var: {var}", ja: "環境変数：{var}" },
  "sdk.installNew": { zh: "下载安装新版本", zhHant: "下載安裝新版本", en: "Install New Version", ja: "新バージョンを追加" },
  "sdk.current": { zh: "当前", zhHant: "目前", en: "Current", ja: "現在" },
  "sdk.tempTerminal": {
    zh: "用此版本打开临时终端",
    zhHant: "以此版本開啟臨時終端",
    en: "Open a temporary terminal with this version",
    ja: "このバージョンで一時ターミナルを開く",
  },
  "sdk.setCurrent": { zh: "设为当前", zhHant: "設為目前", en: "Set as Current", ja: "現在に設定" },
  "sdk.switchTitle": { zh: "切换版本", zhHant: "切換版本", en: "Switch Version", ja: "バージョン切替" },
  "sdk.switchMsg": {
    zh: "是否切换到 {target}？",
    zhHant: "是否切換到 {target}？",
    en: "Switch to {target}?",
    ja: "{target} に切り替えますか？",
  },
  "sdk.uninstall": { zh: "卸载", zhHant: "解除安裝", en: "Uninstall", ja: "アンインストール" },
  "sdk.more": { zh: "更多可安装", zhHant: "更多可安裝", en: "More to Install", ja: "その他インストール可能" },
  "sdk.moreDesc": {
    zh: "未检测到以下工具，可一键下载安装官方版本",
    zhHant: "未偵測到以下工具，可一鍵下載安裝官方版本",
    en: "Not detected below; install the official version in one click",
    ja: "以下は未検出です。ワンクリックで公式版をインストールできます",
  },
  "sdk.uninstallTitle": { zh: "卸载版本", zhHant: "解除安裝版本", en: "Uninstall Version", ja: "バージョンをアンインストール" },
  "sdk.uninstallMsg": {
    zh: "确定卸载 {version} 吗？\n目录：{home}\n\n将删除该版本并清理它遗留的 PATH 条目与相关环境变量。",
    zhHant: "確定解除安裝 {version} 嗎？\n目錄：{home}\n\n將刪除該版本並清理其遺留的 PATH 項目與相關環境變數。",
    en: "Uninstall {version}?\nFolder: {home}\n\nThe version will be removed along with its leftover PATH entries and related env vars.",
    ja: "{version} をアンインストールしますか？\nフォルダ：{home}\n\nこのバージョンと残った PATH 項目・関連環境変数を削除します。",
  },
  "sdk.installTitle": { zh: "下载安装 {label}", zhHant: "下載安裝 {label}", en: "Install {label}", ja: "{label} をインストール" },
  "sdk.noEngine": {
    zh: "未检测到 winget 或 scoop，无法自动安装。请先安装其一。",
    zhHant: "未偵測到 winget 或 scoop，無法自動安裝。請先安裝其一。",
    en: "Neither winget nor scoop found; automatic install is unavailable. Install one first.",
    ja: "winget も scoop も見つからず、自動インストールできません。まずどちらかを導入してください。",
  },
  "sdk.installHelp.lead": { zh: "安装方式说明：", zhHant: "安裝方式說明：", en: "How install works: ", ja: "インストール方法： " },
  "sdk.installHelp.body": {
    zh: "EnvBox 会调用系统上的包管理器帮你下载安装。",
    zhHant: "EnvBox 會呼叫系統上的套件管理器協助你下載安裝。",
    en: "EnvBox uses your system package manager to download and install.",
    ja: "EnvBox はシステムのパッケージマネージャーを使ってダウンロード・インストールします。",
  },
  "sdk.installHelp.winget": {
    zh: "是 Windows 自带的官方工具（推荐，机器级安装会弹一次管理员授权）；",
    zhHant: "是 Windows 內建的官方工具（推薦，機器級安裝會彈一次管理員授權）；",
    en: " is the official Windows tool (recommended; machine-level install prompts UAC once);",
    ja: " は Windows 標準の公式ツール（推奨。マシン単位のインストールで UAC が一度表示）；",
  },
  "sdk.installHelp.scoop": {
    zh: "是第三方工具（免管理员，需你已安装）。直接点「安装」即可，会自动选择可用方式。",
    zhHant: "是第三方工具（免管理員，需你已安裝）。直接點「安裝」即可，會自動選擇可用方式。",
    en: " is a third-party tool (no admin, must be pre-installed). Just click Install; the best available method is chosen automatically.",
    ja: " はサードパーティ製（管理者不要、要事前導入）。「インストール」を押すと利用可能な方法を自動選択します。",
  },
  "sdk.willUse": { zh: "将使用 {engine} 安装", zhHant: "將使用 {engine} 安裝", en: "Will install via {engine}", ja: "{engine} でインストールします" },
  "sdk.noEngineShort": {
    zh: "未检测到可用的 winget/scoop",
    zhHant: "未偵測到可用的 winget/scoop",
    en: "No available winget/scoop",
    ja: "利用可能な winget/scoop なし",
  },
  "sdk.install": { zh: "安装", zhHant: "安裝", en: "Install", ja: "インストール" },
  "sdk.waiting": { zh: "等待输出...", zhHant: "等待輸出...", en: "Waiting for output...", ja: "出力待ち..." },

  // 安装/卸载阶段
  "phase.downloading": { zh: "下载中...", zhHant: "下載中...", en: "Downloading...", ja: "ダウンロード中..." },
  "phase.installing": { zh: "安装中...", zhHant: "安裝中...", en: "Installing...", ja: "インストール中..." },
  "phase.configuring": { zh: "配置环境变量...", zhHant: "設定環境變數...", en: "Configuring env vars...", ja: "環境変数を設定中..." },
  "phase.cleaning": { zh: "清理中...", zhHant: "清理中...", en: "Cleaning up...", ja: "クリーンアップ中..." },
  "phase.done": { zh: "完成 ✅", zhHant: "完成 ✅", en: "Done ✅", ja: "完了 ✅" },
  "phase.error": { zh: "出错 ❌", zhHant: "出錯 ❌", en: "Error ❌", ja: "エラー ❌" },
  "phase.uninstall.preparing": { zh: "准备中...", zhHant: "準備中...", en: "Preparing...", ja: "準備中..." },
  "phase.uninstall.running": { zh: "卸载中...", zhHant: "解除安裝中...", en: "Uninstalling...", ja: "アンインストール中..." },
  "phase.uninstall.cleaning": {
    zh: "清理残留与环境变量...",
    zhHant: "清理殘留與環境變數...",
    en: "Cleaning leftovers and env vars...",
    ja: "残留物と環境変数を整理中...",
  },
  "sdk.uacNote": {
    zh: "若调用官方卸载程序，可能会弹出一次管理员授权(UAC)，请在弹窗中点「是」。",
    zhHant: "若呼叫官方解除安裝程式，可能會彈出一次管理員授權(UAC)，請在彈窗中點「是」。",
    en: "The official uninstaller may prompt UAC once; click Yes in the dialog.",
    ja: "公式アンインストーラーを呼ぶ際に UAC が一度表示される場合があります。ダイアログで「はい」を押してください。",
  },
  "sdk.log.uninstallStart": {
    zh: "开始卸载 {version}",
    zhHant: "開始解除安裝 {version}",
    en: "Uninstalling {version}",
    ja: "{version} のアンインストールを開始",
  },
  "sdk.log.dir": { zh: "目录：{home}", zhHant: "目錄：{home}", en: "Folder: {home}", ja: "フォルダ：{home}" },
  "sdk.log.startFail": { zh: "启动失败：{err}", zhHant: "啟動失敗：{err}", en: "Start failed: {err}", ja: "開始に失敗：{err}" },
  "toast.uninstallStartFail": {
    zh: "无法启动卸载：{err}",
    zhHant: "無法啟動解除安裝：{err}",
    en: "Cannot start uninstall: {err}",
    ja: "アンインストールを開始できません：{err}",
  },
  "distro.Temurin": {
    zh: "Eclipse Temurin（Adoptium）：最主流的免费开源 OpenJDK，个人与商用均免费，推荐首选。",
    zhHant: "Eclipse Temurin（Adoptium）：最主流的免費開源 OpenJDK，個人與商用均免費，推薦首選。",
    en: "Eclipse Temurin (Adoptium): the most popular free, open-source OpenJDK, free for personal and commercial use. Recommended.",
    ja: "Eclipse Temurin（Adoptium）：最も普及した無料オープンソースの OpenJDK。個人・商用ともに無料で、おすすめ。",
  },
  "distro.Oracle": {
    zh: "Oracle JDK：官方原厂构建。个人开发/测试可免费使用，但商用生产环境需购买 Oracle 订阅授权。",
    zhHant: "Oracle JDK：官方原廠建置。個人開發/測試可免費使用，但商用生產環境需購買 Oracle 訂閱授權。",
    en: "Oracle JDK: the official build. Free for personal dev/testing, but commercial production requires an Oracle subscription.",
    ja: "Oracle JDK：公式ビルド。個人の開発/テストは無料ですが、商用の本番環境には Oracle サブスクリプションが必要です。",
  },
  "distro.Microsoft": {
    zh: "Microsoft Build of OpenJDK：微软维护的免费 OpenJDK，适合 Windows / Azure 环境。",
    zhHant: "Microsoft Build of OpenJDK：微軟維護的免費 OpenJDK，適合 Windows / Azure 環境。",
    en: "Microsoft Build of OpenJDK: a free OpenJDK maintained by Microsoft, ideal for Windows / Azure.",
    ja: "Microsoft Build of OpenJDK：Microsoft が保守する無料の OpenJDK。Windows / Azure 環境に最適。",
  },
  "distro.Corretto": {
    zh: "Amazon Corretto：亚马逊维护的免费 OpenJDK，提供长期安全更新，适合 AWS 场景。",
    zhHant: "Amazon Corretto：亞馬遜維護的免費 OpenJDK，提供長期安全更新，適合 AWS 場景。",
    en: "Amazon Corretto: a free OpenJDK by Amazon with long-term security updates, ideal for AWS.",
    ja: "Amazon Corretto：Amazon が保守する無料の OpenJDK。長期セキュリティ更新を提供し、AWS 環境に最適。",
  },
  "distro.Zulu": {
    zh: "Azul Zulu（社区版）：免费开源 OpenJDK，平台覆盖广。",
    zhHant: "Azul Zulu（社群版）：免費開源 OpenJDK，平台覆蓋廣。",
    en: "Azul Zulu (Community): a free, open-source OpenJDK with broad platform coverage.",
    ja: "Azul Zulu（コミュニティ版）：無料オープンソースの OpenJDK。幅広いプラットフォームに対応。",
  },

  // ── 快照页 ───────────────────────────────────
  "snap.create": { zh: "手动创建快照", zhHant: "手動建立快照", en: "Create Snapshot", ja: "スナップショット作成" },
  "snap.autoTip": {
    zh: "每次写操作前都会自动创建快照，改错了可一键恢复。",
    zhHant: "每次寫入操作前都會自動建立快照，改錯了可一鍵還原。",
    en: "A snapshot is auto-created before every write, so mistakes can be reverted in one click.",
    ja: "書き込み操作の前に自動でスナップショットを作成。失敗してもワンクリックで復元できます。",
  },
  "snap.autoClean": { zh: "自动清理", zhHant: "自動清理", en: "Auto-clean", ja: "自動整理" },
  "snap.retention.never": { zh: "永不删除", zhHant: "永不刪除", en: "Never delete", ja: "削除しない" },
  "snap.retention.days": { zh: "保留 {n} 天", zhHant: "保留 {n} 天", en: "Keep {n} days", ja: "{n} 日保持" },
  "snap.tab.snaps": { zh: "快照（{n}）", zhHant: "快照（{n}）", en: "Snapshots ({n})", ja: "スナップショット（{n}）" },
  "snap.tab.audit": { zh: "变更历史（{n}）", zhHant: "變更歷史（{n}）", en: "History ({n})", ja: "変更履歴（{n}）" },
  "snap.empty": { zh: "暂无快照", zhHant: "暫無快照", en: "No snapshots yet", ja: "スナップショットなし" },
  "snap.restore": { zh: "恢复", zhHant: "還原", en: "Restore", ja: "復元" },
  "snap.delTip": { zh: "删除该快照", zhHant: "刪除此快照", en: "Delete this snapshot", ja: "このスナップショットを削除" },
  "snap.auditEmpty": { zh: "暂无记录", zhHant: "暫無記錄", en: "No records yet", ja: "記録なし" },
  "snap.createTitle": { zh: "创建快照", zhHant: "建立快照", en: "Create Snapshot", ja: "スナップショット作成" },
  "snap.nameLabel": { zh: "快照名称", zhHant: "快照名稱", en: "Snapshot Name", ja: "スナップショット名" },
  "snap.namePlaceholder": {
    zh: "给这个快照起个名字",
    zhHant: "為此快照命名",
    en: "Name this snapshot",
    ja: "このスナップショットに名前を付ける",
  },
  "snap.nameDesc": {
    zh: "将保存当前用户变量的完整状态，之后可随时一键恢复。",
    zhHant: "將保存目前使用者變數的完整狀態，之後可隨時一鍵還原。",
    en: "Saves the full current user-variable state; restore anytime in one click.",
    ja: "現在のユーザー変数の完全な状態を保存し、いつでもワンクリックで復元できます。",
  },
  "snap.createBtn": { zh: "创建", zhHant: "建立", en: "Create", ja: "作成" },
  "snap.defaultName": { zh: "手动快照 {stamp}", zhHant: "手動快照 {stamp}", en: "Manual snapshot {stamp}", ja: "手動スナップショット {stamp}" },
  "snap.restoreTitle": { zh: "恢复快照", zhHant: "還原快照", en: "Restore Snapshot", ja: "スナップショットを復元" },
  "snap.restoreMsg": {
    zh: "确定恢复到「{desc}」({time}) 吗？\n当前的用户变量将被覆盖为该快照的状态。",
    zhHant: "確定還原到「{desc}」({time}) 嗎？\n目前的使用者變數將被覆蓋為該快照的狀態。",
    en: "Restore to “{desc}” ({time})?\nCurrent user variables will be overwritten with this snapshot.",
    ja: "「{desc}」({time}) に復元しますか？\n現在のユーザー変数はこのスナップショットで上書きされます。",
  },
  "snap.delTitle": { zh: "删除快照", zhHant: "刪除快照", en: "Delete Snapshot", ja: "スナップショットを削除" },
  "snap.delMsg": {
    zh: "确定删除快照「{desc}」({time}) 吗？此操作不可撤销。",
    zhHant: "確定刪除快照「{desc}」({time}) 嗎？此操作無法復原。",
    en: "Delete snapshot “{desc}” ({time})? This cannot be undone.",
    ja: "スナップショット「{desc}」({time}) を削除しますか？この操作は取り消せません。",
  },

  // ── 确认框 / 错误边界 ────────────────────────
  "confirm.snapNote": {
    zh: "操作前已自动创建快照，可在「快照回滚」中恢复。",
    zhHant: "操作前已自動建立快照，可在「快照還原」中還原。",
    en: "A snapshot was auto-created before this action; restore it from Snapshots.",
    ja: "操作前に自動でスナップショットを作成しました。「スナップショット」から復元できます。",
  },
  "err.title": { zh: "界面出现了异常", zhHant: "介面發生異常", en: "Something went wrong", ja: "画面でエラーが発生しました" },
  "err.desc": {
    zh: "发生了一个未预期的错误，你的环境变量数据不受影响。可尝试重新加载界面。",
    zhHant: "發生了一個未預期的錯誤，你的環境變數資料不受影響。可嘗試重新載入介面。",
    en: "An unexpected error occurred. Your environment data is safe. Try reloading the UI.",
    ja: "予期しないエラーが発生しました。環境変数データは影響を受けません。画面の再読み込みをお試しください。",
  },
  "err.reload": { zh: "重新加载", zhHant: "重新載入", en: "Reload", ja: "再読み込み" },

  // ── 设置页 ───────────────────────────────────
  "settings.perm.title.admin": {
    zh: "权限：管理员（可修改系统变量）",
    zhHant: "權限：管理員（可修改系統變數）",
    en: "Privilege: Administrator (can edit system variables)",
    ja: "権限：管理者（システム変数を変更可能）",
  },
  "settings.perm.title.user": {
    zh: "权限：普通用户（仅可改用户变量）",
    zhHant: "權限：一般使用者（僅可改使用者變數）",
    en: "Privilege: Standard user (user variables only)",
    ja: "権限：標準ユーザー（ユーザー変数のみ）",
  },
  "settings.perm.desc": {
    zh: "修改系统变量或做机器级安装/卸载时需要管理员权限",
    zhHant: "修改系統變數或進行機器層級安裝/解除安裝時需要管理員權限",
    en: "Admin rights are required to edit system variables or do machine-level install/uninstall",
    ja: "システム変数の変更やマシン単位のインストール/アンインストールには管理者権限が必要です",
  },
  "settings.perm.relaunch": {
    zh: "以管理员重启",
    zhHant: "以管理員重新啟動",
    en: "Restart as Admin",
    ja: "管理者で再起動",
  },
  "settings.health.title": { zh: "环境体检", zhHant: "環境體檢", en: "Environment Health", ja: "環境ヘルスチェック" },
  "settings.health.desc": {
    zh: "扫描无效/重复/冲突/PATH 长度并给出建议",
    zhHant: "掃描無效/重複/衝突/PATH 長度並給出建議",
    en: "Scan for invalid/duplicate/conflicting entries and PATH length",
    ja: "無効/重複/競合/PATH 長を検査して提案します",
  },
  "settings.health.run": { zh: "立即体检", zhHant: "立即體檢", en: "Run Check", ja: "今すぐ検査" },
  "settings.health.totalVars": { zh: "变量总数", zhHant: "變數總數", en: "Total Vars", ja: "変数総数" },
  "settings.health.invalid": { zh: "无效路径", zhHant: "無效路徑", en: "Invalid Paths", ja: "無効パス" },
  "settings.health.dup": { zh: "重复项", zhHant: "重複項", en: "Duplicates", ja: "重複" },
  "settings.health.conflict": { zh: "冲突", zhHant: "衝突", en: "Conflicts", ja: "競合" },

  "settings.lang.title": { zh: "界面语言", zhHant: "介面語言", en: "Language", ja: "表示言語" },
  "settings.lang.desc": {
    zh: "切换后立即预览界面语言，保存后永久生效。",
    zhHant: "切換後立即預覽介面語言，儲存後永久生效。",
    en: "Preview instantly on switch; the choice is saved automatically.",
    ja: "切り替えると即座にプレビュー、選択は自動保存されます。",
  },
  "settings.theme.title": { zh: "外观主题", zhHant: "外觀主題", en: "Appearance", ja: "外観テーマ" },
  "settings.theme.desc": {
    zh: "选择应用的外观主题，立即生效。",
    zhHant: "選擇應用的外觀主題，立即生效。",
    en: "Choose the app appearance; applies immediately.",
    ja: "アプリの外観を選択、すぐに反映されます。",
  },
  "settings.theme.light": { zh: "浅色", zhHant: "淺色", en: "Light", ja: "ライト" },
  "settings.theme.dark": { zh: "深色", zhHant: "深色", en: "Dark", ja: "ダーク" },
  "settings.theme.system": { zh: "跟随系统", zhHant: "跟隨系統", en: "System", ja: "システムに従う" },

  "settings.installPath.title": {
    zh: "默认安装路径",
    zhHant: "預設安裝路徑",
    en: "Default Install Path",
    ja: "既定のインストール先",
  },
  "settings.installPath.desc": {
    zh: "下载安装 SDK 时的目标目录（支持 winget 的包会安装到此处；留空则用各自默认位置）。",
    zhHant: "下載安裝 SDK 時的目標目錄（支援 winget 的套件會安裝到此處；留空則用各自預設位置）。",
    en: "Target folder for SDK installs (used by winget packages that support it; empty = each tool's default).",
    ja: "SDK インストール先フォルダ（対応する winget パッケージで使用。空欄は各既定の場所）。",
  },
  "settings.installPath.placeholder": {
    zh: "例如 D:\\DevTools（留空使用默认）",
    zhHant: "例如 D:\\DevTools（留空使用預設）",
    en: "e.g. D:\\DevTools (empty for default)",
    ja: "例：D:\\DevTools（空欄で既定）",
  },
  "settings.mirror.title": { zh: "下载镜像源", zhHant: "下載鏡像源", en: "Download Mirror", ja: "ダウンロードミラー" },
  "settings.mirror.desc": {
    zh: "下载安装 SDK 时的镜像，国内建议使用镜像加速",
    zhHant: "下載安裝 SDK 時的鏡像，中國大陸建議使用鏡像加速",
    en: "Mirror used when downloading SDKs",
    ja: "SDK ダウンロード時に使用するミラー",
  },
  "settings.mirror.official": { zh: "官方源", zhHant: "官方源", en: "Official", ja: "公式" },
  "settings.shortcuts.title": { zh: "快捷键", zhHant: "快捷鍵", en: "Keyboard Shortcuts", ja: "ショートカット" },
  "settings.shortcuts.desc": {
    zh: "常用操作可通过键盘快速触发",
    zhHant: "常用操作可透過鍵盤快速觸發",
    en: "Trigger common actions quickly from the keyboard",
    ja: "よく使う操作をキーボードで素早く実行できます",
  },
  "settings.shortcuts.palette": {
    zh: "打开命令面板（快速跳转页面与操作）",
    zhHant: "開啟命令面板（快速跳轉頁面與操作）",
    en: "Open command palette (jump to pages & actions)",
    ja: "コマンドパレットを開く（ページ・操作へ移動）",
  },
  "settings.shortcuts.search": {
    zh: "聚焦搜索框（在概览 / PATH 页搜索变量）",
    zhHant: "聚焦搜尋框（在總覽 / PATH 頁搜尋變數）",
    en: "Focus the search box (search vars on Overview / PATH)",
    ja: "検索ボックスにフォーカス（概要 / PATH で変数を検索）",
  },
  "settings.shortcuts.close": {
    zh: "关闭弹窗 / 命令面板",
    zhHant: "關閉彈窗 / 命令面板",
    en: "Close dialogs / command palette",
    ja: "ダイアログ / コマンドパレットを閉じる",
  },
  "settings.about.title": { zh: "关于", zhHant: "關於", en: "About", ja: "情報" },
  "settings.about.desc": {
    zh: "EnvBox v{version} · Tauri + React · 统一管理开发环境变量",
    zhHant: "EnvBox v{version} · Tauri + React · 統一管理開發環境變數",
    en: "EnvBox v{version} · Tauri + React · Unified dev environment manager",
    ja: "EnvBox v{version} · Tauri + React · 開発環境変数の統合管理",
  },
  "settings.about.author": { zh: "作者", zhHant: "作者", en: "Author", ja: "作者" },
  "settings.about.contact": { zh: "联系", zhHant: "聯絡", en: "Contact", ja: "連絡" },

  // ── 命令面板 ─────────────────────────────────
  "palette.placeholder": {
    zh: "输入命令或页面名称...",
    zhHant: "輸入命令或頁面名稱...",
    en: "Type a command or page name...",
    ja: "コマンドまたはページ名を入力...",
  },
  "palette.empty": { zh: "无匹配项", zhHant: "無相符項", en: "No matches", ja: "一致なし" },
  "palette.group.nav": { zh: "前往", zhHant: "前往", en: "Go to", ja: "移動" },
  "palette.group.action": { zh: "操作", zhHant: "操作", en: "Actions", ja: "操作" },
  "palette.action.newVar": { zh: "新建变量", zhHant: "新增變數", en: "New variable", ja: "変数を追加" },
  "palette.action.refresh": { zh: "刷新数据", zhHant: "重新整理資料", en: "Refresh data", ja: "データを更新" },
  "palette.action.theme": { zh: "切换深浅主题", zhHant: "切換深淺主題", en: "Toggle light/dark theme", ja: "ライト/ダークを切替" },
  "palette.hint": {
    zh: "↑↓ 选择 · Enter 确认 · Esc 关闭",
    zhHant: "↑↓ 選擇 · Enter 確認 · Esc 關閉",
    en: "↑↓ navigate · Enter select · Esc close",
    ja: "↑↓ 選択 · Enter 決定 · Esc 閉じる",
  },

  // ── Toast 提示 ───────────────────────────────
  "toast.saved": { zh: "已保存", zhHant: "已儲存", en: "Saved", ja: "保存しました" },
  "toast.nameEmpty": { zh: "变量名不能为空", zhHant: "變數名稱不可為空", en: "Variable name cannot be empty", ja: "変数名は空にできません" },
  "toast.varSaved": {
    zh: "已保存 {name}，请重开终端生效",
    zhHant: "已儲存 {name}，請重開終端機生效",
    en: "Saved {name}; reopen your terminal to apply",
    ja: "{name} を保存しました。反映にはターミナルを開き直してください",
  },
  "toast.varDeleted": { zh: "已删除 {name}", zhHant: "已刪除 {name}", en: "Deleted {name}", ja: "{name} を削除しました" },
  "toast.saveFail": { zh: "保存失败：{err}", zhHant: "儲存失敗：{err}", en: "Save failed: {err}", ja: "保存に失敗：{err}" },
  "toast.deleteFail": { zh: "删除失败：{err}", zhHant: "刪除失敗：{err}", en: "Delete failed: {err}", ja: "削除に失敗：{err}" },
  "toast.exported": { zh: "已导出环境变量", zhHant: "已匯出環境變數", en: "Environment variables exported", ja: "環境変数をエクスポートしました" },
  "toast.exportFail": { zh: "导出失败：{err}", zhHant: "匯出失敗：{err}", en: "Export failed: {err}", ja: "エクスポート失敗：{err}" },
  "toast.imported": { zh: "已导入 {n} 个变量", zhHant: "已匯入 {n} 個變數", en: "Imported {n} variables", ja: "{n} 個の変数をインポートしました" },
  "toast.importFail": { zh: "导入失败：{err}", zhHant: "匯入失敗：{err}", en: "Import failed: {err}", ja: "インポート失敗：{err}" },
  "toast.refreshing": { zh: "正在刷新数据…", zhHant: "正在重新整理資料…", en: "Refreshing data…", ja: "データを更新中…" },
  "toast.relaunching": {
    zh: "正在以管理员身份重启…",
    zhHant: "正在以管理員身份重新啟動…",
    en: "Restarting as administrator…",
    ja: "管理者として再起動中…",
  },
  "toast.relaunchFail": { zh: "提权失败：{err}", zhHant: "提權失敗：{err}", en: "Elevation failed: {err}", ja: "権限昇格に失敗：{err}" },
  "toast.switched": {
    zh: "已切换到 {version}，请重开终端生效",
    zhHant: "已切換到 {version}，請重開終端機生效",
    en: "Switched to {version}; reopen your terminal to apply",
    ja: "{version} に切り替えました。反映にはターミナルを開き直してください",
  },
  "toast.switchNote": { zh: "注意：{warn}", zhHant: "注意：{warn}", en: "Note: {warn}", ja: "注意：{warn}" },
  "toast.switchFail": { zh: "切换失败：{err}", zhHant: "切換失敗：{err}", en: "Switch failed: {err}", ja: "切り替えに失敗：{err}" },
  "toast.terminalOpened": { zh: "已打开临时终端", zhHant: "已開啟臨時終端機", en: "Temporary terminal opened", ja: "一時ターミナルを開きました" },
  "toast.terminalFail": { zh: "打开失败：{err}", zhHant: "開啟失敗：{err}", en: "Open failed: {err}", ja: "オープン失敗：{err}" },
  "toast.installed": { zh: "{target} 安装完成", zhHant: "{target} 安裝完成", en: "{target} installed", ja: "{target} のインストール完了" },
  "toast.installFail": { zh: "安装失败：{target}", zhHant: "安裝失敗：{target}", en: "Install failed: {target}", ja: "インストール失敗：{target}" },
  "toast.installStartFail": { zh: "无法启动安装：{err}", zhHant: "無法啟動安裝：{err}", en: "Cannot start install: {err}", ja: "インストールを開始できません：{err}" },
  "toast.startFail": { zh: "启动失败：{err}", zhHant: "啟動失敗：{err}", en: "Start failed: {err}", ja: "開始に失敗：{err}" },
  "toast.uninstalled": { zh: "{target} 已卸载", zhHant: "{target} 已解除安裝", en: "{target} uninstalled", ja: "{target} をアンインストールしました" },
  "toast.uninstallFail": { zh: "卸载失败：{err}", zhHant: "解除安裝失敗：{err}", en: "Uninstall failed: {err}", ja: "アンインストール失敗：{err}" },
  "toast.snapCreated": { zh: "已创建快照", zhHant: "已建立快照", en: "Snapshot created", ja: "スナップショットを作成しました" },
  "toast.snapCreateFail": { zh: "创建失败：{err}", zhHant: "建立失敗：{err}", en: "Create failed: {err}", ja: "作成に失敗：{err}" },
  "toast.snapRestored": {
    zh: "已恢复到该快照，请重开终端",
    zhHant: "已還原到該快照，請重開終端機",
    en: "Restored to snapshot; reopen your terminal",
    ja: "スナップショットに復元しました。ターミナルを開き直してください",
  },
  "toast.snapRestoreFail": { zh: "恢复失败：{err}", zhHant: "還原失敗：{err}", en: "Restore failed: {err}", ja: "復元に失敗：{err}" },
  "toast.snapDeleted": { zh: "已删除该快照", zhHant: "已刪除此快照", en: "Snapshot deleted", ja: "スナップショットを削除しました" },
  "toast.snapDeleteFail": { zh: "删除失败：{err}", zhHant: "刪除失敗：{err}", en: "Delete failed: {err}", ja: "削除に失敗：{err}" },
  "toast.snapPruned": {
    zh: "已自动清理 {n} 个过期快照",
    zhHant: "已自動清理 {n} 個過期快照",
    en: "Auto-cleaned {n} expired snapshots",
    ja: "期限切れのスナップショット {n} 件を自動整理しました",
  },
};

export function translate(
  lang: Lang,
  key: string,
  paramsOrFallback?: Params | string,
  fallback?: string
): string {
  let params: Params | undefined;
  let fb: string | undefined;
  if (typeof paramsOrFallback === "string") {
    fb = paramsOrFallback;
  } else {
    params = paramsOrFallback;
    fb = fallback;
  }

  const e = dict[key];
  let s = e ? e[lang] ?? e.zh ?? fb ?? key : fb ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.split(`{${k}}`).join(String(v));
    }
  }
  return s;
}

export function loadLang(): Lang {
  const v = localStorage.getItem("envbox.lang");
  return v === "zh" || v === "zhHant" || v === "en" || v === "ja" ? v : "zh";
}
