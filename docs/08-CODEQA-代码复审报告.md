# 代码复审报告

## 总览
- **审查文件数**：20（Rust 源文件 7 个 + 配置 3 个 + 前端 TypeScript 源文件 8 个 + CSS 1 个 + Rust 入口 1 个）
- **编译状态**：通过（`cargo check` 通过, `tsc --noEmit` + `vite build` 通过）
- **严重问题**：1
- **建议改进**：4
- **总体结论**：有条件通过（修复严重问题后即可）

---

## 严重问题

### S1. 自定义命令快捷键缺少事件类型过滤（双次执行）

- **文件**：`d:\program\DesktopBoxLite\src-tauri\src\lib.rs` 第 136–151 行
- **问题**：全局快捷键的自定义命令注册循环中，`on_shortcut` 回调接收 `_event` 参数但完全忽略。与 `toggle_window` 的快捷不同，这里没有对 `ShortcutState::Pressed` 进行过滤，导致**每次按键触发两次命令执行**（Pressed + Released 各一次）。
- **影响**：自定义命令（如 `calc.exe`、`notepad.exe` 甚至是文件操作）会被执行两次。这与修复 P0 问题 3（快捷键反复横跳）的修改方案相矛盾——同一个文件中 toggle 快捷键过滤了事件，自定义命令却未过滤。
- **根因**：`for cmd in &config.hotkeys.custom_commands` 循环内的 handler 没有像 toggle handler 一样添加 `if event.state != ShortcutState::Pressed { return; }` 判断。
- **修复建议**：在自定义命令的 handler 中添加事件过滤：
  ```rust
  move |_app, _shortcut, event| {
      if event.state != ShortcutState::Pressed {
          return;
      }
      // ... 执行命令
  }
  ```
  注意需要将 `_event` 参数名改为 `event`，因为现在使用了它。

---

## 建议改进

### I1. 图标刷新间隔过于激进

- **文件**：`d:\program\DesktopBoxLite\src-tauri\src\config.rs` 第 74 行
- **问题**：`icon_refresh_interval_ms` 默认值为 `500`（每 500 毫秒轮询一次桌面图标）。这会导致每秒执行 2 次 `std::fs::read_dir` + `.lnk` 解析 + 前端完整重渲染。
- **影响**：在图标较多或桌面路径为网络驱动器时存在性能风险。
- **修复建议**：将默认值改为 `5000`（5 秒）或 `10000`（10 秒）。500ms 的轮询对于桌面图标变更场景（用户新增/删除/重命名文件）几乎没有实际增益，却能显著降低 CPU 开销。

### I2. 日志级别在 release 下仍为 Debug

- **文件**：`d:\program\DesktopBoxLite\src-tauri\src\logging.rs` 第 96 行
- **问题**：`log::set_max_level(LevelFilter::Debug)` 无条件设置，release 构建下仍然输出大量 Debug 日志。这会增加 I/O 负担和日志文件大小。
- **修复建议**：根据构建配置动态选择级别：
  ```rust
  #[cfg(debug_assertions)]
  log::set_max_level(LevelFilter::Debug);
  #[cfg(not(debug_assertions))]
  log::set_max_level(LevelFilter::Info);
  ```

### I3. 图标图像源使用文件路径而非解析后的图标路径

- **文件**：`d:\program\DesktopBoxLite\src\components\IconItem.ts` 第 12 行
- **文件**：`d:\program\DesktopBoxLite\src-tauri\src\icons.rs` 第 109 行
- **问题**：`IconItem.ts` 使用 `getIconImageUrl(icon.path)` 作为 `<img>` 的 `src`，将文件路径直接作为图像源。而对于 `.lnk` 等非图像文件，这会导致图片加载失败（`onerror` 回调隐藏图像）。同时，后端 `scan_directory` 中 `icon_path` 始终设置为 `String::new()` 空字符串。
- **影响**：快捷方式图标不会显示图像，仅有文字标签。
- **修复建议**：实现图标提取逻辑（如使用 `windows-sys` 的 `ExtractIcon` API 或调用 Shell32 的 `SHGetFileInfoW`）来填充 `DesktopIcon.icon_path` 字段，并将前端图像源改为优先使用 `icon_path`。

### I4. 动画期间快速切换可能导致状态竞争

- **文件**：`d:\program\DesktopBoxLite\src\components\App.ts` 第 53–76 行
- **问题**：当窗口正在执行隐藏动画（`hiding` 类，200ms 内）时，如果用户再次按下快捷键显示窗口，`animate-show` 事件会正确移除 `hiding` 类并添加 `showing` 类。但由于 `animate-hide` 的 `setTimeout` 回调（第 58 行）可能仍在 pending 状态，200ms 超时后仍然会调用 `invoke("finish_hide")` 强制隐藏窗口，导致"刚显示就被隐藏"的状态竞争。
- **修复建议**：添加一个 `isAnimating` 状态标记或使用 Promise 链式调用来确保动画完成前不会调用 `finish_hide`：
  - 在开始隐藏动画时设置标记
  - 收到 show 事件时清除标记并取消 pending 的 `finish_hide` 调用
  - 使用 `clearTimeout` 管理定时器 ID

---

## 规范遵守情况

| 检查项 | 状态 | 说明 |
|--------|------|------|
| **命名规范** | 通过 | Rust 使用 snake_case，TypeScript 使用 camelCase/PascalCase，无违规命名 |
| **注释完整性** | 通过 | 关键函数和模块有文档注释（`///`），变更原因有 `Note:` 注释 |
| **错误处理** | 通过 | Rust 端使用 `Result<(), String>` 模式，前端使用 try/catch + console.error |
| **死代码** | 通过 | `desktop.rs` 已干净移除隐藏/恢复桌面图标代码 |
| **模块职责** | 通过 | 各模块职责清晰（hotkey 注册、executor 执行、icons 扫描、config 管理） |
| **unsafe 使用** | 通过 | `desktop.rs` 中仅有的 `unsafe` 块用于 Windows Registry API，范围最小化 |
| **前端事件处理** | 基本通过 | 动画事件监听正确，但缺少竞争保护（参见 I4） |
| **后端事件过滤** | **不通过** | toggle 快捷正确过滤 Released 事件，但自定义命令未过滤（参见 S1） |

---

## 逐文件审查摘要

### Rust 后端

| 文件 | 行数 | 状态 | 说明 |
|------|------|------|------|
| `lib.rs` | 181 | 有严重问题 | S1（自定义命令缺过滤） |
| `desktop.rs` | 67 | 通过 | 干净移除桌面图标代码，仅保留 `set_auto_start` |
| `hotkey.rs` | 113 | 通过 | 解析 + 注册 + 反注册逻辑完整 |
| `tray.rs` | 55 | 通过 | 移除 `show_desktop_icons`，动画事件发送一致 |
| `executor.rs` | 65 | 通过 | `open_with_system_handler` 使用 `opener` 包 |
| `logging.rs` | 101 | 建议改进 | I2（release Debug 日志） |
| `icons.rs` | 140 | 建议改进 | I3（图标提取未实现） |

### 配置

| 文件 | 状态 | 说明 |
|------|------|------|
| `Cargo.toml` | 通过 | `windows-sys` 依赖精简为 Registry 专用 |
| `tauri.conf.json` | 通过 | 移除 `fullscreen`，`resizable: false` |
| `capabilities/default.json` | 通过 | 权限声明完整（包含 hide/show/event/global-shortcut） |

### 前端

| 文件 | 状态 | 说明 |
|------|------|------|
| `main.ts` | 通过 | DOM 结构正确 |
| `default.css` | 通过 | Flexbox 布局 + @keyframes 动画实现正确 |
| `App.ts` | 建议改进 | I4（动画竞争），`resizeSaveTimer` 声明位置靠后 |
| `ResizeHandle.ts` | 通过 | flexbox 定位适配，8 方向拖拽 |
| `IconGrid.ts` | 通过 | CSS Grid 布局 |
| `SearchBar.ts` | 通过 | 50ms 防抖搜索 |
| `icons.ts` | 通过 | 工具函数正确 |
| `IconItem.ts` | 建议改进 | I3（图标源） |
| `types.ts` | 通过 | 接口定义与后端匹配 |

---

## 与需求/技术方案的符合度

| 需求 | 实现状态 | 说明 |
|------|----------|------|
| REQ-SYS-001 删除（不干预桌面图标） | 已实现 | `desktop.rs` 中已移除所有相关代码 |
| REQ-BOX-001 底部居中浮动面板 | 已实现 | `tauri.conf.json` 移除 fullscreen，CSS flexbox 底部居中 |
| REQ-BOX-009 快捷键稳定性 | 已实现（部分） | toggle 快捷键已过滤，但自定义命令未过滤（S1） |
| REQ-BOX-009 显隐动画 | 已实现 | CSS @keyframes + 前端事件监听 |

---

## 最终建议

1. **必须修复** S1（自定义命令事件过滤），否则自定义快捷键每次触发两次执行，这是和 P0 问题 3 同样的 bug 模式。
2. **建议修复** I1（刷新间隔）和 I2（release 日志级别），在进入正式测试前完成。
3. **可延后处理** I3（图标提取）和 I4（动画竞争）为后续迭代任务。
