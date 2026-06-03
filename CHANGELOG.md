# Changelog

## v1.0.1 (2026-06-03)

### Bug Fixes
- **P0-1 窗口动画同步**: 修复 Tauri OS 窗口与内容容器缩放不同步的问题。
  - 使用 `setInterval` 替代 `requestAnimationFrame`，确保零高度窗口也能可靠执行动画
  - 隐藏动画保留最小 1px 高度，避免 webview 进入失效状态
  - 移除 CSS 动画方案，完全使用 Tauri Window API 驱动窗口缩放
- **Tauri 托盘图标**: 修复托盘图标未正确显示 Tauri 框架 logo 的问题

### Features
- **常用应用 / 所有应用分栏**: 新增点击次数追踪与分栏展示。
  - 后端 `increment_click_count` 命令持久化图标点击计数
  - 前端按点击次数排序，前 10 名归入"常用应用"栏
  - 搜索时自动切换为平铺模式
- **显示状态持久化**: 新增 `display.visible` 配置项，记录窗口显隐状态，启动时自动恢复
- **开机自启开关**: 托盘菜单新增"开机自启"选项，支持实时切换
- **直角窗口**: 窗口边角由圆角改为直角，更贴合收纳工具定位

### Performance
- 图标缓存新增 `click_count` 字段（`#[serde(default)]` 兼容旧缓存）
- 构建优化：LTO、panic=abort、opt-level=z、strip
