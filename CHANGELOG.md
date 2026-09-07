# 更新日志

本项目的所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.0.1] - 2026-09-07

首个 AI 聊天版本，核心聊天链路已跑通。

### 新增

- GPUI 原生窗口 + WebView 嵌入 React 聊天 UI
- 本地 AIGC 模式：通过 Bridge 连接 Automan / 本地 Claude Code 服务
- 远端 AIChat 模式：Module Federation 加载 `PureDialog` / `chatExt`
- HTTP Bridge 服务（`:38472/__gpui_bridge/*`），模拟 `electronAPI`
- 本地 / 远端双模式 Tab 切换与会话详情页
- 环境变量配置（端口、AIGC 路径、用户 MIS 等）
- 快捷键：`Cmd+R` 重载、`Cmd+Shift+I` 开发者工具、`Cmd+Q` 退出

### 验证

- Web UI 构建通过（`pnpm build`）
- Rust 客户端构建通过（`cargo build`）
- Bridge 接口可用：`/__gpui_bridge/config`、`/aigc/status`、静态页 `200`
- 本地 AIGC 服务连接正常

[0.0.1]: https://github.com/shileima/gpui-client/releases/tag/0.0.1
