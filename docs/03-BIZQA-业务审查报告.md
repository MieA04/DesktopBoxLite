# 业务审查报告

**审查角色**：BIZ-QA (Business Reviewer)  
**审查日期**：2026-06-02  
**审查范围**：DesktopBox Lite 全量需求实现  

---

## 总览

| 指标 | 数值 |
|------|------|
| 需求总数 | 17 (P0: 15, P1: 2) |
| 通过 | 12 项 |
| 未通过 | 1 项 |
| 部分实现 | 4 项 |
| 通过率 | 70.6% |

---

## 逐项审查结果

---

### [通过] REQ-BOX-001：全屏透明顶层窗口

- **实现位置**：`src-tauri/tauri.conf.json` 第 16-19 行
- **验证内容**：
  - `fullscreen: true` — 全屏覆盖主显示器
  - `transparent: true` — 窗口完全透明，壁纸可见
  - `decorations: false` — 无边框窗口
  - `alwaysOnTop: true` — 始终置于最顶层
  - `skipTaskbar: true` — 不在任务栏显示
- **审查结论**：四项核心标志全部正确设置，满足验收标准。
- **改进建议**：无。

---

### [通过] REQ-BOX-002：窗口拖拽调整大小

- **实现位置**：
  - 逻辑实现：`src/components/ResizeHandle.ts` 全部代码
  - 样式定义：`src/styles/default.css` 第 124-194 行（`.resize-handle-*`）
  - 尺寸持久化：`src/components/App.ts` 第 79-89 行 `handleResize()`
  - 后端写入：`src-tauri/src/config.rs` 第 161-166 行 `save_window_size()`
- **验证内容**：
  - 8 方向手柄（N, S, W, E, NW, NE, SW, SE）全部实现
  - `mousedown` → `mousemove` → `mouseup` 事件链完整
  - 缩放完成后通过 `invoke("save_window_size")` 持久化到 config.json
  - 最小宽度 120px / 最小高度 150px 约束
- **审查结论**：8 方向拖拽、实时重排、尺寸持久化全部完整实现。
- **改进建议**：无。

---

### [通过] REQ-BOX-003：合并公共桌面与用户桌面图标

- **实现位置**：`src-tauri/src/icons.rs` 第 36-61 行 `scan_desktop_icons()`
- **验证内容**：
  - 同时扫描用户桌面（`%USERPROFILE%\Desktop`）和公共桌面（`C:\Users\Public\Desktop`）
  - 使用 `HashSet` 去重，`(name_normalized, target)` 作为唯一键
  - 用户桌面优先级高于公共桌面（先扫描用户桌面）
  - 按名称排序输出
- **审查结论**：双桌面扫描、合并、去重全部实现。
- **改进建议**：无。

---

### [部分实现] REQ-BOX-004：响应式网格布局

- **实现位置**：
  - CSS Grid：`src/styles/default.css` 第 73-82 行
  - 最大列数常量：`src/utils/icons.ts` 第 5 行 `MAX_ICONS_PER_ROW = 10`
- **验证内容**：
  - CSS 使用 `grid-template-columns: repeat(auto-fill, minmax(80px, 80px))` 实现自动折行
  - `gap: 16px` 网格间距
  - `min-width: 120px` 最小窗口宽度（容纳 1 个图标 + 内边距）
- **发现问题**：
  - 需求规定"每行最多显示 10 个图标"，但 CSS 使用 `auto-fill` 机制会根据容器宽度自动填充尽可能多的列。在 4K 显示器全屏场景下（窗口可被拖拽到很大），`auto-fill` 会产生远超 10 列的布局。
  - `MAX_ICONS_PER_ROW = 10` 虽然已定义为常量并通过 `style.setProperty("--max-columns", ...)` 注入到 DOM，但 CSS 中未使用 `--max-columns` 变量来限制最大列数。
  - 需要将 CSS `grid-template-columns` 改为类似 `repeat(auto-fill, minmax(80px, 1fr)); max-width: calc(80px * 10 + 16px * 9 + ...)` 或使用 `repeat(10, 80px)` + 水平滚动的方式。
- **审查结论**：自动折行和最小宽度已实现，但未限制每行最大 10 列。
- **改进建议**：在 CSS 中添加 `max-width` 约束，或使用 `repeat(auto-fill, minmax(80px, 1fr))` 配合 `grid-template-columns: repeat(10, 80px)` 的双重策略，在 `IconGrid.render()` 中根据实际可用宽度动态设置列数。

---

### [通过] REQ-BOX-005：图标大小固定

- **实现位置**：`src/styles/default.css` 第 85-122 行
- **验证内容**：
  - `.icon-item { width: 80px; }` — 图标项固定宽度
  - `.icon-image { width: 48px; height: 48px; }` — 图标图片固定尺寸
  - `.icon-label { font-size: 11px; max-width: 76px; }` — 名称区域固定
  - 所有尺寸均为绝对值（px），不随窗口缩放变化
- **审查结论**：图标尺寸完全固定，满足验收标准。
- **改进建议**：无。

---

### [通过] REQ-BOX-006：单击运行程序

- **实现位置**：
  - 前端单击事件：`src/components/IconItem.ts` 第 29-32 行
  - 前端调用：`src/utils/icons.ts` 第 34-40 行 `openFilePath()`
  - 后端处理：`src-tauri/src/lib.rs` 第 38-41 行 `open_file` 命令
  - 实际执行：`src-tauri/src/executor.rs` 全部代码
- **验证内容**：
  - 通过 `cmd /c` 执行路径打开文件/程序
  - 使用 `CREATE_NO_WINDOW` 标志避免控制台窗口
  - 执行失败时仅记录日志，不弹出错误弹窗
- **审查结论**：单击即时响应，错误不外显，满足验收标准。
- **改进建议**：当前使用 `executor::execute_command()`（即 `cmd /c`）而非 `opener::open()` 来打开 `.lnk` 快捷方式。虽然 `cmd /c` 能处理大多数情况，但 `opener` crate 对 `.lnk` 的解析更可靠。建议在 `open_file` 中改用 `opener::open(&path)` 直接打开文件/快捷方式。

---

### [通过] REQ-BOX-007：图标名称去除后缀

- **实现位置**：
  - 后端：`src-tauri/src/icons.rs` 第 118-128 行 `strip_extension()`
  - 前端（冗余保护）：`src/utils/icons.ts` 第 50-60 行 `stripExtension()`
- **验证内容**：
  - 后端去除的扩展名：`.lnk`, `.exe`, `.url`, `.txt`, `.doc`, `.docx`, `.pdf`
  - 前端去除的扩展名：`.lnk`, `.exe`, `.url`, `.txt`, `.doc`, `.docx`, `.pdf`
  - 使用 `rfind('.')` 处理点号位置
- **审查结论**：后缀去除完整实现，前后端双重保障。
- **改进建议**：前端 `stripExtension()` 未在任何代码中被调用（后端已在 Rust 层完成此处理）。可考虑移除或保留作为文档用途。

---

### [通过] REQ-BOX-008：搜索过滤

- **实现位置**：
  - 搜索栏组件：`src/components/SearchBar.ts` 全部代码
  - 过滤逻辑：`src/utils/icons.ts` 第 24-31 行 `filterIcons()`
  - 搜索回调：`src/components/App.ts` 第 73-76 行 `handleSearch()`
- **验证内容**：
  - 搜索框固定在窗口顶部（`.search-bar` 在 `.resizable-window` 顶部）
  - 使用 `input` 事件实现实时过滤（50ms 防抖）
  - `filterIcons()` 执行不区分大小写的子串匹配
  - 清空输入框恢复全部图标
  - 过滤后调用 `iconGrid.render()` 触发网格重排
- **审查结论**：实时过滤、大小写不敏感、清空恢复、网格重排全部实现。
- **改进建议**：无。

---

### [通过] REQ-BOX-009：快捷键控制显示/隐藏

- **实现位置**：
  - 快捷键注册：`src-tauri/src/hotkey.rs` 第 90-103 行 `register_shortcut()`
  - 注册与回调：`src-tauri/src/lib.rs` 第 100-119 行
  - 默认快捷键：`Ctrl+Shift+D`（定义于 `config.rs` `HotkeyConfig::default()`）
- **验证内容**：
  - 使用 `tauri_plugin_global_shortcut` 实现全局快捷键
  - 快捷键在 `setup` 阶段注册，全局生效
  - 回调根据窗口可见性切换 `hide()` / `show()` 
  - 默认键 `Ctrl+Shift+D` 可通过 config.json 自定义
- **审查结论**：全局快捷键显隐完整实现。
- **改进建议**：无。

---

### [通过] REQ-CFG-001：config.json 配置文件

- **实现位置**：`src-tauri/src/config.rs` 全部代码
- **验证内容**：
  - 配置文件路径：`%APPDATA%/DesktopBoxLite/config.json`
  - 首次启动自动生成默认配置文件（`load_config()` 中检查文件是否存在）
  - JSON 解析失败时回退到 `Config::default()` 并记录 error 日志
  - 默认配置内容与需求文档定义一致（toggle_window + 4 个自定义快捷键）
- **审查结论**：配置加载、自动生成、错误回退全部实现。
- **改进建议**：无。

---

### [通过] REQ-CFG-002：快捷键自定义映射

- **实现位置**：
  - 配置定义：`src-tauri/src/config.rs` 第 5-47 行
  - 快捷键解析：`src-tauri/src/hotkey.rs` 第 10-32 行 `parse_hotkey()`
  - 快捷键注册：`src-tauri/src/lib.rs` 第 122-139 行
- **验证内容**：
  - 支持修饰键：Ctrl, Alt, Shift, Win
  - 支持普通键：A-Z, 0-9, F1-F12
  - `toggle_window` 可通过 config.json 自定义
  - `custom_commands` 数组支持多个快捷键
  - 注册失败时输出 warn 日志，不影响其他快捷键
- **审查结论**：快捷键自定义映射完整实现。
- **改进建议**：无。

---

### [通过] REQ-CFG-003：快捷键执行系统指令或脚本

- **实现位置**：
  - 触发执行：`src-tauri/src/lib.rs` 第 130-134 行（custom_commands 回调）
  - 执行引擎：`src-tauri/src/executor.rs` 全部代码
- **验证内容**：
  - 支持 `.exe` 可执行文件（通过 `cmd /c` 执行）
  - 支持 `.bat`, `.cmd`, `.ps1` 脚本（通过 `cmd /c` 执行）
  - 支持带参数的系统指令
  - 使用 `CREATE_NO_WINDOW` 标志避免弹窗
  - 执行失败时只记录 error 日志，不弹出错误弹窗
- **审查结论**：指令执行、无窗口执行、失败静默日志全部实现。
- **改进建议**：无。

---

### [未通过] REQ-CFG-004：自定义 CSS 样式文件

- **实现位置**：
  - 配置字段：`src-tauri/src/config.rs` 第 51-53 行 `AppearanceConfig.css_path`
- **验证内容**：
  - `config.json` 中 `appearance.css_path` 字段已定义
  - 前端 `appearance` 类型已定义 `css_path: string | null`
- **发现问题**：
  - 代码中完全没有任何逻辑来读取 `css_path` 并加载外部 CSS 文件。
  - `src/components/App.ts` 中的 `loadConfig()` 仅用于设置窗口尺寸，从未检查 `config.appearance.css_path`。
  - 技术设计文档中提到的 `applyExternalCSS(cssPath)` 方法未曾实现。
  - 启动时仅通过 `index.html` 中的 `<link>` 标签加载 `default.css`，用户无法自定义样式。
- **审查结论**：配置字段存在，但加载逻辑完全缺失。
- **改进建议**：在 `App.loadConfig()` 或 `App.init()` 中添加以下处理：
  1. 加载 config 后读取 `config.appearance.css_path`
  2. 如果路径非空且文件存在，创建一个 `<link>` 元素追加到 `<head>`
  3. 如果文件不存在，记录 warn 日志并跳过

---

### [部分实现] REQ-SYS-001：隐藏/恢复系统桌面图标

- **实现位置**：
  - 隐藏：`src-tauri/src/desktop.rs` 第 19-31 行 `hide_desktop_icons()`
  - 恢复：`src-tauri/src/desktop.rs` 第 35-48 行 `show_desktop_icons()`
  - 窗口查找：`src-tauri/src/desktop.rs` 第 56-79 行 `find_syslistview32()`
  - 启动时调用：`src-tauri/src/lib.rs` 第 91-93 行
  - 退出时恢复：`src-tauri/src/lib.rs` 第 151-154 行（`CloseRequested` 事件）
- **发现问题**：
  - 技术设计文档和需求明确指出需要处理 **WorkerW 窗口备选路径**（某些 Windows 10/11 版本中，桌面图标存在于 `WorkerW` → `SHELLDLL_DefView` 的窗口层级下，而非 `Progman` → `SHELLDLL_DefView`）。
  - 当前 `find_syslistview32()` 仅实现了 `Progman` → `SHELLDLL_DefView` → `SysListView32` 这一条路径，缺少 `WorkerW` 备选路径。
  - 在 Windows 10/11 某些版本或开启"平板模式"后，桌面图标可能位于 WorkerW 窗口下，此时隐藏/恢复操作会静默失败（返回 None，调用处仅打印 error 日志）。
- **审查结论**：基础功能已实现，但缺少 WorkerW 备选路径导致兼容性不足。
- **改进建议**：参照技术设计文档第 3.4 节中的伪代码，在 `find_syslistview32()` 中添加 WorkerW 窗口遍历逻辑。具体步骤：
  1. 在 Progman 下查找 SHELLDLL_DefView 失败后
  2. 通过 `FindWindowExW(None, prev, "WorkerW", None)` 循环遍历所有 WorkerW 窗口
  3. 在每个 WorkerW 下查找 SHELLDLL_DefView
  4. 找到后继续查找 SysListView32

---

### [部分实现] REQ-SYS-002：开机自启动

- **实现位置**：
  - 注册表操作：`src-tauri/src/desktop.rs` 第 98-157 行 `set_auto_start()`
  - Tauri 命令注册：`src-tauri/src/lib.rs` 第 74-77 行
- **验证内容**：
  - 写入注册表路径：`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
  - 键名：`DesktopBox Lite`
  - 值：当前可执行文件完整路径
  - 前端可通过 `invoke("set_auto_start", { enabled: true/false })` 调用
- **发现问题**：
  - 虽然 `set_auto_start` 命令存在且可被前端调用，但 **启动时未根据 `config.behavior.auto_start` 配置自动调用**。
  - 需求要求"`config.json` 中 `auto_start: true` 时生效"。但当前代码中，`lib.rs` 的 `setup` 函数加载 config 后，没有检查 `config.behavior.auto_start` 并自动调用 `desktop::set_auto_start(true)`。
  - 用户需要额外逻辑才能实现开机自启；单纯设置 config.json 不会生效。
- **审查结论**：注册表操作能力已实现，但未根据配置自动触发。
- **改进建议**：在 `lib.rs` 的 `setup` 函数中，加载 config 后添加：
  ```rust
  if config.behavior.auto_start {
      if let Err(e) = desktop::set_auto_start(true) {
          log::error!("Failed to enable auto-start: {}", e);
      }
  }
  ```

---

### [通过] REQ-SYS-003：系统托盘图标

- **实现位置**：`src-tauri/src/tray.rs` 全部代码
- **验证内容**：
  - 托盘图标在 `lib.rs` setup 中通过 `tray::build_tray_menu(app.handle())` 创建
  - 右键菜单包含三个菜单项：显示/隐藏（toggle）、重载配置（reload）、退出（quit）
  - 菜单项之间有分隔符
  - `TrayIconBuilder` 设置了 tooltip `"DesktopBox Lite"`
  - 重载配置通过事件 `config-reloaded` 发射
  - 退出前调用 `show_desktop_icons()` 恢复桌面图标
- **审查结论**：系统托盘图标完整实现，菜单功能齐全。
- **改进建议**：可考虑添加左键单击切换显隐功能（当前仅菜单操作），但 P1 需求中未明确要求。

---

### [通过] REQ-SYS-004：开发日志输出

- **实现位置**：`src-tauri/src/logging.rs` 全部代码
- **验证内容**：
  - 日志目录 `./logs/` 在 `init_logging()` 中自动创建
  - 日志文件按日期滚动：`YYYY-MM-DD.log` 格式
  - 自定义 `FileLogger` 实现 `log::Log` trait
  - 日志格式：`[timestamp] [LEVEL] target - message`
  - 日志级别：`LevelFilter::Debug`（开发期间最大化）
  - 日志同时输出到文件和控制台 stderr
  - 记录内容包括启动/退出、配置加载、快捷键注册、执行指令等
- **审查结论**：日志系统完整实现，日期滚动、多级别、多输出目标均满足。
- **改进建议**：可考虑添加日志级别配置支持（通过 `RUST_LOG` 环境变量或其他方式）。

---

## 总结与建议

### 总体评价

DesktopBox Lite 项目整体实现质量较高，17 项需求中 **12 项完全通过**。核心 P0 功能的窗口系统、图标扫描、搜索过滤、快捷键、配置管理等均已正确实现。

### 需要修复的关键问题

| 优先级 | 问题 | 影响 |
|--------|------|------|
| **高** | REQ-CFG-004：自定义 CSS 从未加载 | 用户无法自定义视觉样式 |
| **高** | REQ-SYS-001：缺少 WorkerW 回退路径 | 部分 Windows 版本无法隐藏桌面图标 |
| **中** | REQ-BOX-004：网格未限制每行最大 10 列 | 宽屏下布局不符合设计规范 |
| **中** | REQ-SYS-002：自动启动未根据配置触发 | 配置中 `auto_start` 字段无效 |

### 改进建议汇总

1. **REQ-CFG-004**：在 `App.ts` 中添加 `applyExternalCSS()` 方法，在 `loadConfig` 后读取 `css_path` 并动态加载 `<link>` 元素。
2. **REQ-SYS-001**：在 `desktop.rs` 的 `find_syslistview32()` 中添加 `WorkerW` 窗口遍历备选路径。
3. **REQ-BOX-004**：修改 CSS `grid-template-columns` 或动态计算列数以限制每行最多 10 个图标。
4. **REQ-SYS-002**：在 `lib.rs` setup 中根据 `config.behavior.auto_start` 自动调用 `set_auto_start(true)`。
5. **REQ-BOX-006**（建议）：`open_file` 命令改用 `opener::open()` 替代 `executor::execute_command()`，以获得更可靠的 `.lnk` 快捷方式解析能力。
