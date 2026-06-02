# DesktopBox Lite

桌面图标收纳盒 — 基于 Tauri 2 的 Windows 桌面图标管理工具。

将杂乱的桌面图标收纳到一个可拖拽、可搜索、支持高清图标的悬浮窗口中，释放桌面空间。

## 功能特性

- **图标收纳** — 自动扫描桌面和公共桌面目录，将所有图标集中展示
- **高清渲染** — 通过 `IShellItemImageFactory` COM API 提取 256×256 高清图标
- **常用应用栏** — 记录点击次数，自动将前 10 个最常用应用置顶显示
- **实时搜索** — 按名称快速筛选图标（50ms 防抖）
- **智能刷新** — 基于文件指纹 + mtime 的增量变更检测，500ms 轮询，后台异步提取
- **磁盘缓存** — 图标缓存到 `<exe_dir>/cache/`，支持陈旧条目自动清理
- **可拖拽窗口** — 顶部 12px 拖拽手柄，支持四角/四边缩放
- **全局快捷键** — `Ctrl+Shift+D` 切换显示/隐藏，支持自定义命令快捷键
- **系统托盘** — 托盘菜单：显示/隐藏、重载配置、退出
- **自定义 CSS** — 支持通过配置文件加载外部样式表

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | [Tauri 2](https://v2.tauri.app/) |
| 后端语言 | Rust (edition 2021) |
| 前端语言 | TypeScript |
| 构建工具 | Vite 5 |
| 图标提取 | `IShellItemImageFactory` COM API (Windows) |
| 图标缓存 | JSON 文件 + mtime 校验 |
| 图片编码 | `image` crate → PNG → base64 |

## 项目结构

```
DesktopBoxLite/
├── src/                          # 前端 (TypeScript)
│   ├── main.ts                   # 入口：DOM 构建 + App 实例化
│   ├── components/
│   │   ├── App.ts                # 主控制器：配置、轮询、动画
│   │   ├── IconGrid.ts           # 图标网格：分栏渲染（常用/所有）
│   │   ├── IconItem.ts           # 单个图标 DOM 元素
│   │   ├── ResizeHandle.ts       # 窗口缩放手柄
│   │   └── SearchBar.ts          # 搜索栏（50ms 防抖）
│   ├── utils/
│   │   ├── icons.ts              # 工具函数：获取、过滤、打开
│   │   ├── poll_manager.ts       # 变更检测轮询引擎
│   │   └── types.ts              # TypeScript 类型定义
│   └── styles/
│       └── default.css           # 默认样式（毛玻璃主题）
├── src-tauri/                    # 后端 (Rust)
│   ├── src/
│   │   ├── main.rs               # Rust 入口
│   │   ├── lib.rs                # Tauri 命令注册、应用生命周期
│   │   ├── config.rs             # 配置读写（config.json）
│   │   ├── icons.rs              # 图标提取、元数据扫描、指纹计算
│   │   ├── icon_cache.rs         # 磁盘缓存（JSON + mtime 校验）
│   │   ├── icon_state.rs         # 托管状态 + 后台异步提取队列
│   │   ├── tray.rs               # 系统托盘图标与菜单
│   │   ├── hotkey.rs             # 全局快捷键注册/注销
│   │   ├── executor.rs           # 系统命令执行器
│   │   ├── desktop.rs            # 开机自启管理
│   │   └── logging.rs            # 日志初始化
│   ├── icons/                    # 应用图标
│   ├── tauri.conf.json           # Tauri 配置
│   └── Cargo.toml                # Rust 依赖
├── docs/                         # 项目文档（需求、设计、审查）
├── package.json
├── tsconfig.json
├── vite.config.ts
└── .gitignore
```

## 快速开始

### 前置依赖

- [Rust](https://www.rust-lang.org/) (edition 2021)
- [Node.js](https://nodejs.org/) (LTS)
- Windows 10+ (项目基于 Windows API 构建)

### 开发

```bash
# 安装前端依赖
npm install

# 启动开发模式（热重载）
npm run tauri dev
```

### 构建

```bash
# 生产构建
npm run tauri build
```

构建产物位于 `src-tauri/target/release/DesktopBox Lite.exe`。

## 配置

配置文件 `config.json` 位于可执行文件同目录，示例如下：

```json
{
  "hotkeys": {
    "toggle_window": "Ctrl+Shift+D",
    "custom_commands": [
      { "keys": "Ctrl+Alt+C", "command": "calc.exe", "description": "打开计算器" },
      { "keys": "Ctrl+Alt+N", "command": "notepad.exe", "description": "打开记事本" },
      { "keys": "Ctrl+Shift+F", "command": "wt.exe", "description": "打开 Windows Terminal" }
    ]
  },
  "appearance": {
    "css_path": null
  },
  "behavior": {
    "auto_start": false,
    "icon_refresh_interval_ms": 500,
    "window_width": 800,
    "window_height": 600
  }
}
```

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+D` | 切换显示/隐藏窗口 |
| `Ctrl+Alt+C` | 打开计算器 |
| `Ctrl+Alt+N` | 打开记事本 |
| `Ctrl+Shift+F` | 打开 Windows Terminal |

## 架构说明

### 图标变更检测流程

```
轮询 (500ms)
  → check_icons_changed()       # 轻量指纹对比（仅元数据，~5ms）
    → changed? 否 → 继续轮询
    → changed? 是 → refresh_icons()
      → 更新指纹 + 取消旧任务 + 启动后台线程
        → 遍历桌面文件
          → 检查磁盘缓存（mtime 匹配？）
            → 命中 → 复用缓存图标 + 点击次数
            → 未命中 → IShellItemImageFactory 提取 256x256 PNG
                      → 写入缓存（含点击次数）
        → 发送 "icons-ready" 事件
          → 前端接收 → 更新网格 → 分栏渲染
```

### 点击计数流程

```
用户点击图标
  → increment_click_count() (fire-and-forget)
  → 缓存 JSON click_count++
  → 前端本地状态更新 + 即时重渲染
  → 下次全量刷新时持久化数据同步
```

## 性能

- **初始扫描**：纯元数据扫描约 5ms（100+ 文件）
- **图标提取**：后台线程，每图标约 10-50ms（256×256 PNG 编码）
- **变更检测**：指纹哈希对比约 5ms
- **缓存命中**：mtime 匹配时直接返回 base64 数据，无 IO 瓶颈
- **前端渲染**：`IntersectionObserver` 式按需加载，不卡主线程

## 许可证

MIT
