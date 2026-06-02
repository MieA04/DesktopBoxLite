# DesktopBox Lite 功能需求文档

**文档版本**：v1.0  
**创建日期**：2026-06-02  
**项目名称**：DesktopBox Lite  
**状态**：✅ 定稿

---

## 一、项目概述

DesktopBox Lite 是一个轻量级桌面图标收纳工具，采用全屏透明无边框窗口作为画板容器，以网格形式展示桌面图标。支持通过全局快捷键控制窗口显示/隐藏，并允许用户自定义快捷键执行系统指令或脚本。窗口尺寸可拖拽调整，图标自动响应式换行，并提供搜索过滤功能。

**核心设计原则**：
- 极简轻量：内存占用 < 100MB，无复杂后台进程
- 高度可配置：所有行为通过 JSON 配置文件控制
- 视觉自定义：支持外部 CSS 样式文件

---

## 二、术语表

| 术语 | 说明 |
|------|------|
| **收纳盒窗口** | 全屏透明无边框主窗口，承载图标网格，永远置于顶层 |
| **全局快捷键** | 系统级别的键盘监听，即使 DesktopBox 窗口未获焦点也能响应 |
| **config.json** | 主配置文件，定义快捷键映射、样式路径、行为参数等 |
| **CSS 样式文件** | 用户自定义收纳盒外观的样式表，支持调整图标大小、间距、背景等 |
| **公共桌面** | `C:\Users\Public\Desktop`，存放对所有用户可见的快捷方式 |
| **用户桌面** | `%USERPROFILE%\Desktop`，存放当前用户的桌面图标 |

---

## 三、功能需求总览

### 需求优先级定义

| 优先级 | 标签 | 说明 |
|--------|------|------|
| **P0** | 核心 | 必须实现，缺失则产品无法交付 |
| **P1** | 重要 | 强烈建议实现 |
| **P2** | 增强 | 可后续迭代 |

### 需求全景图

```
┌─────────────────────────────────────────────────┐
│              DesktopBox Lite                     │
├───────────────────┬─────────────────────────────┤
│   图标收纳盒核心   │       配置与扩展系统         │
│                   │                             │
│ • 全屏透明顶层窗口│ • config.json 配置           │
│ • 窗口拖拽调整大小│ • 快捷键→系统指令映射         │
│ • 双桌面图标合并  │ • 快捷键→脚本/程序执行        │
│ • 响应式网格布局  │ • 自定义 CSS 样式加载         │
│ • 单击运行程序    │ • 快捷键控制收纳盒显隐        │
│ • 名称模糊搜索    │ • 开发日志输出                │
│ • 隐藏后缀名      │                             │
└───────────────────┴─────────────────────────────┘
```

---

## 四、详细功能需求

### 4.1 图标收纳盒核心功能

#### REQ-BOX-001：全屏透明顶层窗口

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-001 |
| **标题** | 全屏透明顶层窗口 |
| **优先级** | P0 |
| **描述** | 创建一个全屏、透明、无边框的窗口，并永远置于所有窗口的最顶层 |
| **实现方式** | Tauri `window.always_on_top = true` + `transparent = true` + `decorations = false` + 全屏 |
| **验收标准** | 1. 窗口覆盖整个主显示器 2. 窗口完全透明（桌面壁纸可见） 3. 窗口始终在最前，不被其他应用遮挡 |

---

#### REQ-BOX-002：窗口可拖拽调整大小

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-002 |
| **标题** | 窗口可拖拽调整大小 |
| **优先级** | P0 |
| **描述** | 用户可通过拖拽窗口边缘或右下角手柄来改变收纳盒窗口的尺寸 |
| **实现方式** | 前端监听鼠标事件，动态修改窗口 `width`/`height`，并触发布局重新计算 |
| **验收标准** | 1. 窗口四边及右下角支持拖拽缩放 2. 缩放过程中图标网格实时重排 3. 缩放后的窗口尺寸持久化保存 |

---

#### REQ-BOX-003：合并公共桌面与用户桌面图标

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-003 |
| **标题** | 合并公共桌面与用户桌面图标 |
| **优先级** | P0 |
| **描述** | 同时读取用户桌面（`%USERPROFILE%\Desktop`）和公共桌面（`C:\Users\Public\Desktop`）中的图标，合并展示 |
| **验收标准** | 1. 显示两个目录下的所有有效图标 2. 重复图标（同名、同目标）去重 3. 图标来源不需要在界面上区分 |

---

#### REQ-BOX-004：响应式网格布局

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-004 |
| **标题** | 响应式网格布局 |
| **优先级** | P0 |
| **描述** | 图标以固定大小排列成网格。每行最多显示 10 个图标。当窗口宽度不足以容纳 10 个图标时，自动将超出部分折到下一行；宽度增大时自动提升每行数量（不超过 10）。窗口宽度最小限制为刚好容纳 1 个图标（即最小宽度 = 单个图标宽度 + 内边距）。 |
| **实现方式** | CSS Grid 布局，`grid-template-columns: repeat(auto-fill, minmax(图标宽度, 图标宽度))`，配合 `max-width` 限制 |
| **验收标准** | 1. 每行最多 10 个图标 2. 窗口缩窄 → 图标自动折行 3. 窗口最窄时仍能显示 1 个图标（不出现空白溢出） 4. 图标大小固定（通过 CSS 指定宽高） |

---

#### REQ-BOX-005：图标大小固定

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-005 |
| **标题** | 图标大小固定 |
| **优先级** | P0 |
| **描述** | 每个图标的图片尺寸和名称区域大小固定，不随窗口缩放改变 |
| **实现方式** | CSS 固定 `width`/`height`，图标图片固定尺寸 |
| **验收标准** | 1. 所有图标图片尺寸一致 2. 名称区域高度一致 3. 窗口缩放时图标尺寸不变 |

---

#### REQ-BOX-006：单击运行程序

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-006 |
| **标题** | 单击图标运行程序 |
| **优先级** | P0 |
| **描述** | 单击图标即可打开对应的文件、文件夹或执行快捷方式 |
| **实现方式** | Tauri command → `opener` crate 或 `ShellExecuteW` |
| **验收标准** | 1. 单击文件 → 系统默认程序打开 2. 单击文件夹 → 资源管理器打开 3. 单击快捷方式 → 执行对应目标 4. 避免双击延迟，单击即时响应 |

---

#### REQ-BOX-007：图标名称去除后缀

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-007 |
| **标题** | 图标名称去除后缀 |
| **优先级** | P0 |
| **描述** | 显示的图标名称应移除常见的文件扩展名，如 `.lnk`、`.exe`、`.url` 等 |
| **验收标准** | 1. `快捷方式.lnk` → 显示为“快捷方式” 2. `calc.exe` → 显示为“calc” 3. `website.url` → 显示为“website” 4. 普通文件保留主文件名，文件夹名称不变 |

---

#### REQ-BOX-008：搜索过滤

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-008 |
| **标题** | 图标名称模糊搜索 |
| **优先级** | P0 |
| **描述** | 在收纳盒窗口顶部提供一个搜索输入框，用户输入关键词后实时过滤图标列表（不区分大小写，模糊匹配图标名称） |
| **验收标准** | 1. 搜索框固定在窗口顶部 2. 输入即过滤，无延迟 3. 清空搜索框恢复全部图标 4. 过滤后网格自动重新排列 |

---

#### REQ-BOX-009：快捷键控制显示/隐藏

| 字段 | 内容 |
|------|------|
| **ID** | REQ-BOX-009 |
| **标题** | 全局快捷键控制收纳盒显示/隐藏 |
| **优先级** | P0 |
| **描述** | 通过全局快捷键（默认 `Ctrl+Shift+D`）切换收纳盒窗口的显示和隐藏状态 |
| **实现方式** | 全局键盘监听（如 `global_hotkey` crate），触发时调用 `window.hide()` / `window.show()` |
| **验收标准** | 1. 快捷键全局生效（焦点在其他应用时也可用） 2. 首次按下隐藏窗口，再次按下显示窗口 3. 快捷键可通过配置文件自定义 |

---

### 4.2 配置系统功能

#### REQ-CFG-001：config.json 配置文件

| 字段 | 内容 |
|------|------|
| **ID** | REQ-CFG-001 |
| **标题** | config.json 配置文件 |
| **优先级** | P0 |
| **描述** | 使用 JSON 格式存储所有用户配置，位于应用目录下（如 `%APPDATA%/DesktopBoxLite/config.json`） |
| **验收标准** | 1. 首次启动时自动生成默认配置文件 2. 配置文件格式错误时回退到默认配置并记录日志 3. 支持热重载（可选，P2） |

**默认配置文件结构**：

```json
{
  "hotkeys": {
    "toggle_window": "Ctrl+Shift+D",
    "custom_commands": [
      {
        "keys": "Ctrl+Alt+C",
        "command": "calc.exe",
        "description": "打开计算器"
      },
      {
        "keys": "Ctrl+Alt+N",
        "command": "notepad.exe",
        "description": "打开记事本"
      },
      {
        "keys": "Ctrl+Alt+T",
        "command": "cmd.exe /c echo Hello > C:\\test.txt",
        "description": "执行系统指令"
      },
      {
        "keys": "Ctrl+Alt+R",
        "command": "C:\\scripts\\my_script.bat",
        "description": "执行脚本文件"
      }
    ]
  },
  "appearance": {
    "css_path": "C:\\Users\\YourName\\DesktopBoxLite\\style.css"
  },
  "behavior": {
    "auto_start": false,
    "icon_refresh_interval_ms": 500,
    "window_width": 800,
    "window_height": 600
  }
}
```

---

#### REQ-CFG-002：快捷键自定义映射

| 字段 | 内容 |
|------|------|
| **ID** | REQ-CFG-002 |
| **标题** | 快捷键自定义映射 |
| **优先级** | P0 |
| **描述** | 用户可在配置文件中自定义快捷键与动作的映射关系 |
| **实现方式** | 解析 `config.json` 中的 `hotkeys` 节，动态注册全局快捷键 |
| **验收标准** | 1. `toggle_window` 可修改为其他组合键 2. `custom_commands` 数组支持配置多个快捷键 3. 快捷键格式支持 `Ctrl`, `Alt`, `Shift`, `Win` + `A-Z` / `0-9` / `F1-F12` 4. 快捷键冲突时给出日志警告 |

---

#### REQ-CFG-003：快捷键执行系统指令或脚本

| 字段 | 内容 |
|------|------|
| **ID** | REQ-CFG-003 |
| **标题** | 快捷键执行系统指令或脚本 |
| **优先级** | P0 |
| **描述** | 按下配置的快捷键时，执行对应的 Windows 系统指令或打开指定路径的可执行文件/脚本 |
| **实现方式** | Rust 后端使用 `std::process::Command` 执行，不显示窗口（`CREATE_NO_WINDOW` 标志） |
| **验收标准** | 1. 支持 `.exe` 可执行文件 2. 支持 `.bat`, `.cmd`, `.ps1` 脚本 3. 支持带参数的系统指令 4. 执行失败时记录日志，不弹出错误弹窗 |

---

#### REQ-CFG-004：自定义 CSS 样式文件

| 字段 | 内容 |
|------|------|
| **ID** | REQ-CFG-004 |
| **标题** | 自定义 CSS 样式文件 |
| **优先级** | P0 |
| **描述** | 用户可通过配置文件指定外部 CSS 文件路径，用于自定义图标收纳盒的视觉样式 |
| **验收标准** | 1. 启动时加载指定路径的 CSS 文件 2. CSS 文件不存在时使用内置默认样式 3. 支持热重载（可选，P2） |

---

### 4.3 系统集成与开发支持

#### REQ-SYS-001：隐藏/恢复系统桌面图标

| 字段 | 内容 |
|------|------|
| **ID** | REQ-SYS-001 |
| **标题** | 隐藏/恢复系统桌面图标 |
| **优先级** | P0 |
| **描述** | 软件启动时自动隐藏 Windows 原生桌面图标，退出时自动恢复 |
| **实现方式** | `FindWindow(L"Progman", NULL)` → 遍历找到 `SysListView32` → `ShowWindow(SW_HIDE)` |
| **验收标准** | 1. 启动后桌面图标消失 2. 退出后桌面图标恢复 3. 隐藏/恢复过程无闪烁 |

---

#### REQ-SYS-002：开机自启动

| 字段 | 内容 |
|------|------|
| **ID** | REQ-SYS-002 |
| **标题** | 开机自启动 |
| **优先级** | P1 |
| **描述** | 支持配置是否开机自动启动 DesktopBox Lite |
| **实现方式** | 写入 Windows 注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| **验收标准** | `config.json` 中 `auto_start: true` 时生效 |

---

#### REQ-SYS-003：系统托盘图标

| 字段 | 内容 |
|------|------|
| **ID** | REQ-SYS-003 |
| **标题** | 系统托盘图标 |
| **优先级** | P1 |
| **描述** | 在系统托盘中显示图标，右键菜单支持退出、显示/隐藏、重载配置 |
| **验收标准** | 1. 托盘图标常驻 2. 右键菜单包含：显示/隐藏、重载配置、退出 3. 左键单击切换显示/隐藏 |

---

#### REQ-SYS-004：开发日志输出

| 字段 | 内容 |
|------|------|
| **ID** | REQ-SYS-004 |
| **标题** | 开发日志输出 |
| **优先级** | P0 |
| **描述** | 开发阶段必须将运行日志输出到 `logs` 目录，便于调试和问题追踪 |
| **实现方式** | 使用 `log` crate + `env_logger` 或 `simple_logger`，写入文件 |
| **验收标准** | 1. 日志目录 `./logs/` 自动创建 2. 日志文件按日期滚动（如 `2026-06-02.log`） 3. 记录信息至少包括：启动/退出、配置加载、快捷键注册、执行指令、错误信息 4. 日志级别可配置（Info/Debug/Error） |

---

## 五、非功能需求

### 5.1 性能指标

| ID | 指标 | 目标值 | 说明 |
|----|------|--------|------|
| NFR-PERF-001 | 内存占用（空闲） | < 80 MB | 无用户交互时 |
| NFR-PERF-002 | 内存占用（图标满载） | < 120 MB | 桌面有 200+ 图标时 |
| NFR-PERF-003 | 图标刷新延迟 | < 500 ms | 桌面文件变化到界面更新 |
| NFR-PERF-004 | 快捷键响应时间 | < 100 ms | 按下到动作执行 |
| NFR-PERF-005 | 窗口显隐切换 | 即时 | 无动画，保证响应速度 |
| NFR-PERF-006 | 搜索过滤响应 | < 50 ms | 输入时界面更新 |

### 5.2 兼容性

| ID | 要求 | 说明 |
|----|------|------|
| NFR-COMP-001 | 支持 Windows 10/11 | 主力目标平台 |
| NFR-COMP-002 | 与 Wallpaper Engine 兼容 | 透明窗口不干扰壁纸渲染 |
| NFR-COMP-003 | 多显示器支持 | 仅主显示器显示 |

### 5.3 可用性

| ID | 要求 | 说明 |
|----|------|------|
| NFR-UX-001 | 单击启动，无延迟 | 单击即时响应 |
| NFR-UX-002 | 窗口拖拽缩放流畅 | 缩放时实时重排，不掉帧 |
| NFR-UX-003 | 优雅退出 | 退出时恢复桌面图标 |
| NFR-UX-004 | 配置热重载（推荐） | 修改配置文件后无需重启应用 |

---

## 六、用例分析

### UC-001：日常桌面管理

```
角色：普通用户
前置条件：DesktopBox Lite 已启动
主流程：
  1. 用户在收纳盒中看到所有桌面图标（合并用户和公共桌面）
  2. 单击某个应用图标 → 程序启动
  3. 在桌面新建文件 → 收纳盒在 500ms 内自动显示新文件
后置条件：无
```

### UC-002：窗口缩放与自适应

```
角色：用户
前置条件：收纳盒窗口已显示
主流程：
  1. 用户拖拽窗口右下角，缩小窗口宽度
  2. 图标网格自动折行，每行图标数从 10 逐渐减少
  3. 窗口宽度缩到最窄（刚好显示 1 个图标）时无法继续缩小
  4. 用户放大窗口，图标每行数量增加，最多回到 10 个
后置条件：窗口尺寸持久化保存
```

### UC-003：搜索过滤图标

```
角色：用户
前置条件：桌面上有大量图标
主流程：
  1. 用户在搜索框输入“chrome”
  2. 图标列表实时过滤，只显示名称包含“chrome”的图标
  3. 清空搜索框，全部图标恢复显示
后置条件：无
```

### UC-004：快捷键显隐

```
角色：用户
前置条件：需要临时查看原生桌面
主流程：
  1. 用户按下 Ctrl+Shift+D
  2. DesktopBox Lite 窗口隐藏，原生桌面图标显示
  3. 用户再次按下 Ctrl+Shift+D
  4. DesktopBox Lite 窗口显示，原生桌面图标再次隐藏
后置条件：显隐状态持续到下次切换
```

### UC-005：自定义快捷键执行指令

```
角色：高级用户
前置条件：已配置自定义快捷键
主流程：
  1. 用户在 config.json 中配置 Ctrl+Alt+C → calc.exe
  2. 重载配置（或重启应用）
  3. 用户按下 Ctrl+Alt+C → 计算器打开
后置条件：无
```

### UC-006：开发调试查看日志

```
角色：开发者
前置条件：程序运行中发生错误或需要追踪行为
主流程：
  1. 打开程序所在目录的 logs 文件夹
  2. 查看当天的日志文件（如 2026-06-02.log）
  3. 根据日志信息定位问题
后置条件：日志持续追加，不丢失历史
```

---

## 七、需求优先级矩阵

| 需求 ID | 名称 | 优先级 |
|---------|------|--------|
| REQ-BOX-001 | 全屏透明顶层窗口 | P0 |
| REQ-BOX-002 | 窗口可拖拽调整大小 | P0 |
| REQ-BOX-003 | 合并公共桌面与用户桌面图标 | P0 |
| REQ-BOX-004 | 响应式网格布局 | P0 |
| REQ-BOX-005 | 图标大小固定 | P0 |
| REQ-BOX-006 | 单击运行程序 | P0 |
| REQ-BOX-007 | 图标名称去除后缀 | P0 |
| REQ-BOX-008 | 搜索过滤 | P0 |
| REQ-BOX-009 | 快捷键控制显隐 | P0 |
| REQ-CFG-001 | config.json 配置 | P0 |
| REQ-CFG-002 | 快捷键自定义映射 | P0 |
| REQ-CFG-003 | 快捷键执行指令/脚本 | P0 |
| REQ-CFG-004 | 自定义 CSS 样式 | P0 |
| REQ-SYS-001 | 隐藏/恢复桌面图标 | P0 |
| REQ-SYS-002 | 开机自启动 | P1 |
| REQ-SYS-003 | 系统托盘图标 | P1 |
| REQ-SYS-004 | 开发日志输出 | P0 |

### 优先级统计

| 优先级 | 数量 | 占比 |
|--------|------|------|
| P0（核心） | 15 | 88% |
| P1（重要） | 2 | 12% |
| **合计** | **17** | **100%** |

---

## 八、附录：默认 CSS 样式（内置）

```css
/* DesktopBox Lite 默认样式 */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    user-select: none;
}

body {
    background-color: transparent;
    overflow: hidden;
    font-family: 'Segoe UI', '微软雅黑', sans-serif;
}

/* 主容器：全屏透明窗口 */
.app-container {
    width: 100vw;
    height: 100vh;
    background-color: transparent;
    position: relative;
}

/* 可拖拽调整大小的窗口框 */
.resizable-window {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background-color: rgba(20, 20, 30, 0.7);
    backdrop-filter: blur(20px);
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
    border: 1px solid rgba(255,255,255,0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    resize: both;
    min-width: 120px;  /* 最小宽度容纳1个图标+内边距 */
    min-height: 150px;
}

/* 搜索栏区域 */
.search-bar {
    padding: 12px 16px;
    background-color: rgba(0,0,0,0.3);
    border-bottom: 1px solid rgba(255,255,255,0.1);
}

.search-bar input {
    width: 100%;
    padding: 8px 12px;
    background-color: rgba(255,255,255,0.15);
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    outline: none;
}

.search-bar input::placeholder {
    color: rgba(255,255,255,0.6);
}

/* 图标网格容器 */
.icon-grid {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 80px));
    gap: 16px;
    align-content: flex-start;
    justify-content: center;
}

/* 单个图标项 */
.icon-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    cursor: pointer;
    transition: transform 0.1s ease;
    width: 80px;
}

.icon-item:hover {
    transform: scale(1.05);
    background-color: rgba(255,255,255,0.1);
    border-radius: 8px;
}

.icon-image {
    width: 48px;
    height: 48px;
    object-fit: contain;
    margin-bottom: 6px;
}

.icon-label {
    font-size: 11px;
    color: white;
    text-shadow: 1px 1px 0 rgba(0,0,0,0.5);
    max-width: 76px;
    word-break: break-word;
    line-height: 1.3;
}

/* 滚动条样式 */
.icon-grid::-webkit-scrollbar {
    width: 6px;
}

.icon-grid::-webkit-scrollbar-track {
    background: rgba(0,0,0,0.2);
    border-radius: 3px;
}

.icon-grid::-webkit-scrollbar-thumb {
    background: rgba(255,255,255,0.4);
    border-radius: 3px;
}
```

---

**文档状态**：✅ 定稿  
**需求总数**：17 项（P0: 15, P1: 2）  
**预计开发周期**：1-2 周（1 名开发者）  
**技术栈建议**：Tauri 2 + TypeScript + 原生 HTML/CSS