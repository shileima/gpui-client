# GPUI AI 聊天客户端

基于 [Zed GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 的 AI 聊天桌面客户端，复用 Automan / Electron 的聊天架构：

- **本地 AIGC**：通过 `cclocal` provider 连接本地 Claude Code 服务（与 Automan `main-desktop` 一致）
- **远端 AIChat**：通过 Module Federation 加载 [aichat-dynamic-module](https://github.com) 的 `PureDialog` / `chatExt`
- **GPUI 壳层**：原生窗口 + WebView 嵌入 React 聊天 UI + Rust Bridge 模拟 `electronAPI`

## 架构

```
GPUI Window (Rust)
  └── WebView (wry)
        └── React Chat Shell (web/)
              ├── aichat/dialog  → 欢迎页 PureDialog
              └── aichat/chatExt → 会话详情 AichatModule
  └── HTTP Bridge (:38472/__gpui_bridge/*)
        └── AIGC Runtime (检测/代理本地 NestJS 服务)
```

## 环境要求

- Rust stable 1.89+
- Node.js 22+ / pnpm
- macOS：Xcode + Metal Toolchain
- 可选：Automan/Electron 本地 AIGC 模块，或已运行的 AIGC 服务（`~/.automan/.aigc-port`）

## 快速开始

### 1. 构建 Web 聊天 UI

```bash
cd web
pnpm install
pnpm dev    # 开发：http://127.0.0.1:5173
# 或
pnpm build  # 生产：产物输出到 web/dist，由 Rust 内置服务托管
```

### 2. 运行 GPUI 客户端

```bash
cargo run
```

开发模式默认加载 `http://127.0.0.1:5173`（需先 `pnpm dev`）。  
生产模式在 `web/dist` 构建完成后自动加载内置 `http://127.0.0.1:38472`。

## 环境变量

| 变量 | 说明 | 默认 |
|------|------|------|
| `GPUI_CHAT_WEB_URL` | 聊天页 URL | 自动检测 |
| `GPUI_CHAT_WEB_PORT` | Bridge/静态服务端口 | `38472` |
| `GPUI_AICHAT_REMOTE_URL` | aichat remoteEntry | `https://aichat.sankuai.com/remoteEntry.js` |
| `GPUI_AIGC_MODULE_PATH` | AIGC main.js 路径 | 自动检测 waimai-qa-aie-fe |
| `GPUI_USER_MIS` | 用户 MIS | `gpui-user` |

## 功能

- 本地 / 远端双模式 Tab 切换
- PureDialog 欢迎屏 + 首条消息创建会话
- AichatModule 完整会话（流式 WS、工具调用等由 aichat 模块提供）
- `electronAPI` Bridge：`aigcServiceGetStatus`、`aigcSessionCreateV2` 等
- 快捷键：`Cmd+R` 重载、`Cmd+Q` 退出

## 与 Automan 的差异

当前版本聚焦核心聊天链路。以下能力可在后续迭代：

- ChatEnv 侧栏（文件树 / Git Review）
- 历史会话列表 / Agent 列表
- 技能商店 / 录制回放
- 完整 Electron preload API 覆盖

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Cmd+R | 重新加载聊天页 |
| Cmd+Shift+I | 开发者工具（WebView） |
| Cmd+Q | 退出 |
