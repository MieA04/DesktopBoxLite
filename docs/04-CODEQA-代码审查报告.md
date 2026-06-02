# 代码审查报告

## 总览
- **审查文件数**：26 个（9 Rust 源文件 + 6 配置/构建文件 + 8 TS 源文件 + 2 前端配置 + 1 CSS）
- **严重问题**：6 个（必须修复）
- **建议改进**：15 个（推荐）
- **审查结论**：不通过

---

## 严重问题（必须修复）

### [FE] 拖拽缩放破坏窗口居中定位
- **文件**：`src/components/ResizeHandle.ts:70-92`
- **问题**：从 N、S、W、E、NW、NE、SW、SE 方向拖拽缩放时，代码会设置 `this.target.style.top` 和 `this.target.style.left` 为像素值，覆盖了 CSS 中 `top: 50%; left: 50%; transform: translate(-50%, -50%)` 的居中定位。这导致窗口在第一次从 N/W 方向缩放后偏离中心，后续缩放会持续累积位置偏移。
- **建议**：对于 N/S/W/E 方向缩放，不应覆盖 `top`/`left` 百分比定位。如需要从上方/左侧缩放，可改为改变 `transform-origin` 或调整 `margin`/`translate` 值。

### [RUST] 缺少快捷键注销功能且退出时未清理快捷键
- **文件**：`src-tauri/src/lib.rs:157-161`、`src-tauri/src/hotkey.rs`
- **问题**：`on_window_event` 的 `CloseRequested` 处理中只恢复了桌面图标（`show_desktop_icons`），没有调用 `hotkey::unregister_hotkeys()`。而且 `hotkey.rs` 中根本没有实现 `unregister_hotkeys` 函数。Tauri 2 的全局快捷键在进程退出时可能残留。
- **建议**：在 `hotkey.rs` 中实现 `unregister_hotkeys` 函数（调用 `app.global_shortcut().unregister_all()`），在退出时调用。

### [RUST] executor.rs 使用 `.spawn()` 替代 `.output()` 导致执行结果丢失
- **文件**：`src-tauri/src/executor.rs:57-63`
- **问题**：`run_shell_command` 函数使用 `Command::spawn()` 而非 `Command::output()`，导致：
  1. 子进程的执行结果（成功/失败）无法获取
  2. `execute_command` 返回 `Ok(())` 即使内部命令执行失败（如文件不存在）
  3. 用户点击损坏的快捷方式时得不到任何反馈
- **建议**：使用 `.output()` 替换 `.spawn()`，检查退出码并返回错误。如果不想等待（如启动 GUI 程序），至少应捕获 `.spawn()` 本身的错误。

### [RUST/CFG] env_logger 列为依赖但从未使用
- **文件**：`src-tauri/Cargo.toml:26`
- **问题**：`env_logger = "0.11"` 在 Cargo.toml 中声明为依赖，但所有 Rust 源文件中没有任何 `use env_logger` 导入。日志系统使用自定义 `FileLogger` 实现，`env_logger` 完全未被引用，徒增编译时间和二进制体积。
- **建议**：从 `Cargo.toml` 中移除 `env_logger` 依赖。

### [FE] 前端存在两处死代码
- **文件**：`src/utils/icons.ts:50-60` 和 `src/utils/icons.ts:10`
- **问题**：
  1. `stripExtension` 函数定义为 `export` 但没有任何文件导入使用它
  2. `ICON_HEIGHT` 常量定义后从未被任何代码引用
  TypeScript 配置已启用 `noUnusedLocals`，但因为它们是 `export` 的，编译器认为"可能被外部使用"而不会报错。
- **建议**：移除未使用的 `stripExtension` 函数和 `ICON_HEIGHT` 常量。如果需要保留供未来使用，请添加 `// TODO` 注释说明预期用途。

### [FE] App.ts 构造函数中 setupIconRefresh 在配置加载前执行
- **文件**：`src/components/App.ts:34-43`
- **问题**：构造函数中 `loadConfig()` 是异步的（`then` 回调），但 `setupIconRefresh()` 在构造函数末尾同步调用。此时 `this.config === null`，`icon_refresh_interval_ms` 永远使用默认值 `500`，忽略用户配置的自定义刷新间隔。
- **建议**：将 `setupIconRefresh()` 的调用移到 `loadConfig().then()` 回调内部，确保配置加载完成后再设置刷新间隔。

---

## 建议改进

### [RUST] DesktopIcon 和 IconInfo 结构体完全重复
- **文件**：`src-tauri/src/icons.rs:5-15` vs `src-tauri/src/lib.rs:12-18`
- **问题**：`icons::DesktopIcon` 和 `lib.rs::IconInfo` 具有完全相同的字段（name, path, icon_path, is_shortcut）。`get_icons` 命令无意义地做了逐字段映射转换。增加了维护成本——修改一个结构体时必须同步修改另一个。
- **建议**：删除 `lib.rs` 中的 `IconInfo`，Tauri 命令直接返回 `Vec<icons::DesktopIcon>`。

### [RUST/FE] 前后端重复实现了 strip_extension 逻辑
- **文件**：`src-tauri/src/icons.rs:118-128` 和 `src/utils/icons.ts:50-60`
- **问题**：Rust 后端的 `strip_extension` 和 TypeScript 前端的 `stripExtension` 功能完全一致。后端已做名称处理（`icons.rs:90`），前端的 `stripExtension` 虽是死代码但定义了重复逻辑。违背"单一职责"和"约定大于配置"原则。
- **建议**：删除前端 `stripExtension` 函数。名称处理全部由后端完成，前端只渲染已处理好的数据。

### [RUST] get_icons 的 open_file 实现偏离技术设计
- **文件**：`src-tauri/src/lib.rs:39-41` vs `docs/02-TD-技术设计.md`
- **问题**：技术设计规定 `open_file` 应使用 `opener::open` 直接打开。实际实现代理到 `executor::execute_command`（走 `cmd /c` 包装）。这导致 `.lnk` 文件打开行为可能不一致，且 URL 和文件走不同的路径（`is_url` 判断）。
- **建议**：遵循技术设计，`open_file` 直接调用 `opener::open(&path)` 而非通过 `execute_command`。保留 `execute_command` 仅用于自定义快捷键执行的系统指令。

### [RUST] 使用 env 变量直接读取路径，缺少 dirs crate
- **文件**：`src-tauri/src/config.rs:106-108`、`src-tauri/src/icons.rs:28-30`
- **问题**：技术设计指定使用 `dirs = "5"` crate 获取标准目录，实际代码使用 `std::env::var("APPDATA")` 和 `std::env::var("USERPROFILE")` 直接读取环境变量。这：
  1. 在环境变量缺失时回退到 `PathBuf::from(".")`（当前目录），导致配置/图标丢失
  2. 不利于跨平台（Linux/macOS 上没有这些变量）
- **建议**：添加 `dirs = "5"` 依赖，使用 `dirs::config_dir()` 和 `dirs::desktop_dir()`。

### [RUST/LIB] 缺少 setup 中快捷键事件绑定的文档注释
- **文件**：`src-tauri/src/lib.rs:109-146`
- **问题**：`tauri-plugin-global-shortcut` 的 `on_shortcut` API 注册了大量闭包。这些闭包使用了大量 `clone()`（`command.clone()`, `keys_str.clone()`, `keys_for_closure.clone()`），但没有任何注释解释为什么需要 clone。这降低了可读性。
- **建议**：添加注释解释闭包需要 `'static` 生命周期因此需要 move + clone。考虑将快捷键回调提取为单独的命名函数。

### [RUST] tray.rs 的 "reload" 事件前端无监听器
- **文件**：`src-tauri/src/tray.rs:40`
- **问题**：`"reload"` 菜单项触发了 `app.emit("config-reloaded", &config)`，但前端没有任何代码监听此事件。因此重载配置功能对用户完全是静默的——前端不会更新窗口大小/样式。
- **建议**：在前端 `App.ts` 的 `init` 或构造函数中添加 `listen("config-reloaded", ...)`，更新窗口尺寸和 CSS 路径。

### [RUST] Tauri 命令 `execute_custom_command` 和 `set_auto_start` 注册但从未调用
- **文件**：`src-tauri/src/lib.rs:69-77`
- **问题**：这两个命令在 `invoke_handler` 中注册，但前端没有任何代码调用它们。`execute_custom_command` 尤其可疑——自定义快捷键在 Rust 端直接调用 `executor::execute_command`，不走 Tauri 命令。这是死代码。
- **建议**：如果前端确实需要这些功能，添加前端调用代码。否则从 `invoke_handler` 中移除。

### [CFG] capabilities/default.json 存在未使用的权限声明
- **文件**：`src-tauri/capabilities/default.json`
- **问题**：`core:window:allow-set-position` 和 `core:window:allow-center` 声明了但前端代码从未使用。这违反了最小权限原则。
- **建议**：移除未使用的权限。仅保留实际需要的权限。

### [FE] IconGrid 渲染未使用 DocumentFragment
- **文件**：`src/components/IconGrid.ts:16-24`
- **问题**：每次 `render()` 循环中对每个图标单独 `appendChild`，会导致多次 DOM 回流。技术设计明确要求使用 `document.createDocumentFragment` 进行批量操作以减小性能开销。
- **建议**：改用 `DocumentFragment` 一次性追加所有图标节点。

### [FE] App.ts 使用非空断言 `!` 存在潜在空指针风险
- **文件**：`src/components/App.ts:16-19`
- **问题**：连续 4 次使用 `document.getElementById("app")!` 和 `querySelector<HTMLElement>(...)!` 的非空断言。如果 DOM 结构不完整或元素缺失，会在运行时 crash。而 `main.ts:7-10` 已经展示了正确的防御式写法。
- **建议**：使用 `if (!element) return;` 模式替代 `!` 断言。或至少将断言集中在构造函数开头检查一次。

### [FE] openFilePath 静默吞噬错误，用户无反馈
- **文件**：`src/utils/icons.ts:34-40`
- **问题**：`openFilePath` 函数 catch 所有异常后仅 `console.error`，调用方（`IconItem.ts:30`）只 `await` 但不检查结果。用户点击损坏/缺失的快捷方式时不会有任何视觉反馈。
- **建议**：考虑使用 Tauri dialog 插件或添加 toast-like 提示通知用户打开失败。至少应在前端组件级别添加错误状态显示。

### [FE] 文件名使用 PascalCase 而非 camelCase，与技术设计规范不一致
- **文件**：`src/components/App.ts`, `IconGrid.ts`, `IconItem.ts`, `SearchBar.ts`, `ResizeHandle.ts`
- **问题**：技术设计明确规定文件/目录名使用 camelCase（如 `iconGrid.ts`、`searchBar.ts`），但实际文件全部使用 PascalCase（`IconGrid.ts`、`SearchBar.ts`）。
- **建议**：统一重命名为 camelCase，或更新技术设计文档以匹配实际命名。

### [FE] 注释普遍解释"是什么"而非"为什么"
- **文件**：多个文件（全项目范围）
- **问题**：大量注释描述代码"正在做什么"（如 `// Loads configuration from the backend`），这违背 AGENTS.md 中"注释应解释'为什么'而非'是什么'"的原则。代码本身已能表达功能，注释应解释设计决策和上下文。
- **建议**：移除 trivial 的功能描述注释，对关键设计决策添加"为什么"注释（如"为什么这里用 50ms 防抖"、"为什么 N 方向缩放不做 top 调整"）。

### [CSS] CSS 中硬编码的布局尺寸与 JS 常量不共享
- **文件**：`src/styles/default.css:78`、`src/utils/icons.ts:8-10`
- **问题**：CSS 中的 `grid-template-columns: repeat(auto-fill, minmax(80px, 80px))`、`max-width: calc(10 * 80px + 9 * 16px)` 等值与 `utils/icons.ts` 中的 `ICON_WIDTH = 80`, `MAX_ICONS_PER_ROW = 10` 重复。修改一个就必须同步修改另一个，容易不同步。
- **建议**：考虑通过 CSS 变量（`--icon-width: 80px`）传递值，或通过 JS 动态设置 CSS 变量实现单点维护。

### [RUST] logging.rs 文件句柄克隆开销
- **文件**：`src-tauri/src/logging.rs:27-53`
- **问题**：`get_or_open_file()` 每次写日志时都调用 `file.try_clone()` 创建一个新的文件句柄。高频日志记录下（DEBUG 级别）会产生大量不必要的句柄克隆操作。
- **建议**：将 `Mutex<File>` 直接存储在 `LogFile` 中，写日志时直接持锁写入，避免每次克隆。

---

## 规范遵守情况

### Rust 规范：❌（3 项违规）
- **命名规范**：通过。snake_case 函数、PascalCase 类型、SCREAMING_SNAKE_CASE 常量均正确
- **错误处理**：通过。使用 `Result<T, String>` 模式，无 `unwrap()`/`expect()`（日志初始化除外）
- **注释规范**：⚠️ 部分通过。公有函数有 `///` 注释，但多为"是什么"而非"为什么"
- **模块导入顺序**：通过。std -> 外部 crate -> 内部模块
- **未使用依赖**：❌ `env_logger = "0.11"` 在 Cargo.toml 中但从未使用
- **跨平台处理**：⚠️ 桌面处理有 `#[cfg]` 条件编译，但路径读取依赖 Windows 特定环境变量

### TypeScript 规范：❌（2 项违规）
- **严格模式**：通过。`strict: true`、`noUnusedLocals: true`、`noUnusedParameters: true`
- **命名规范**：通过。camelCase 函数、PascalCase 类/接口
- **文件命名规范**：❌ 文件使用 PascalCase（`IconGrid.ts`）而非约定声明的 camelCase（`iconGrid.ts`）
- **死代码**：❌ 存在 `stripExtension` 和 `ICON_HEIGHT` 未使用导出
- **非空断言**：⚠️ App.ts 使用 4 处 `!` 断言存在隐患

### CSS 规范：✅ 通过
- 类名使用 kebab-case
- 无 `!important`
- 无 ID 选择器
- 合理的分层和样式组织

### 配置文件规范：❌（2 项违规）
- **tauri.conf.json**：`$schema` 指向非官方源（`nicegui` 项目），应使用 `https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-utils/schema.json`
- **capabilities/default.json**：存在 2 个未使用的权限（`allow-set-position`、`allow-center`）
- **Cargo.toml**：存在 1 个未使用的依赖（`env_logger`）
- **package.json**：与 TD 设计一致，无问题

---

## 总结

### 关键发现

| 类别 | 数量 | 说明 |
|------|------|------|
| 功能 Bug | 2 | 窗口缩放偏移、图标刷新间隔配置不生效 |
| 行为缺陷 | 2 | 退出未清理快捷键、命令执行错误静默 |
| 死代码 | 4 | stripExtension、ICON_HEIGHT、execute_custom_command、env_logger |
| 架构偏离 | 2 | IconInfo 与 DesktopIcon 重复、open_file 实现偏离设计 |
| 安全/防御性 | 3 | 非空断言、错误静默、命令执行结果丢失 |
| 规范违背 | 4 | 文件命名、未用权限、未用依赖、注释风格 |

### 风险等级评估

- **高风险**：窗口缩放偏移（严重影响用户体验，每次缩放都会累积偏移）
- **中风险**：open_file 执行错误被 .spawn() 吞没（用户无法知道操作是否成功）
- **中风险**：快捷键退出未清理（可能造成快捷键残留）
- **低风险**：死代码和未用依赖（增加维护负担和编译时间）

### 审查结论：不通过

需要修复所有 6 个严重问题后方可重新审查。建议修复顺序：
1. 修复 ResizeHandle 窗口缩放偏移（最影响用户体验）
2. 修复 executor.rs 使用 .spawn() 导致结果丢失
3. 添加快捷键退出清理
4. 修复 setupIconRefresh 在配置加载前执行
5. 移除未使用的 env_logger 依赖
6. 移除前端死代码

重新审查前须通过编译测试（`cargo build` + `npx tsc --noEmit`）。
