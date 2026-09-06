# GPUI 任务客户端

基于 [Zed GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 构建的完整功能桌面客户端。

## 功能

- 任务增删改查（添加、完成切换、删除、清除已完成）
- 侧边栏筛选（全部 / 进行中 / 已完成）
- 可滚动任务列表
- 文本输入框（光标、选区、Backspace/Delete）
- 原生 macOS 菜单栏
- 键盘快捷键

## 环境要求

- Rust stable 1.89+
- macOS：Xcode + Command Line Tools + Metal Toolchain

```bash
xcode-select --install
xcodebuild -downloadComponent MetalToolchain
```

## 运行

```bash
cd gpui-client
cargo run
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Enter | 添加任务 |
| Cmd+N | 聚焦输入框 |
| Cmd+Backspace | 清除已完成 |
| Cmd+Q | 退出 |
