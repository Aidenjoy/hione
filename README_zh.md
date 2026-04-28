# hione — 全能 AI 工具集

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: macOS/Windows/Linux](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](#)

[English](README.md)

hione 是全能 AI 工具桌面应用 + CLI 系统。Tauri 桌面应用提供图形化界面，用于管理智能体、MCP 服务器、技能和全局配置。hi 命令则是强大的 CLI 工具，用于在终端一键启动多窗口 AI 协作。

## 支持的工具

| 工具 | 安装方式 | 卸载方式 |
|------|---------|---------|
| [Claude Code](https://github.com/anthropics/claude-code) | `npm install -g @anthropic-ai/claude-code` | `npm uninstall -g @anthropic-ai/claude-code` |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | `npm install -g @google/gemini-cli` | `npm uninstall -g @google/gemini-cli` |
| [OpenCode](https://github.com/anomalyco/opencode) | `npm install -g opencode-ai` | `npm uninstall -g opencode-ai` |
| [Codex CLI](https://github.com/openai/codex) | `npm install -g @openai/codex` | `npm uninstall -g @openai/codex` |
| [Qwen Code](https://github.com/QwenLM/qwen-code) | `npm install -g @qwen-code/qwen-code` | `npm uninstall -g @qwen-code/qwen-code` |
| 任意自定义工具 | 直接传入可执行文件名 |

## 工作方式

**hione** 支持两种运行模式：

1. **tmux 模式** (CLI 默认)：`hi start` 会将当前 tmux 会话分割成多个窗格（pane），每个 AI 代理占据一个，并自动启动。
2. **桌面模式** (Tauri)：每个代理运行在独立的系统原生窗口中，内嵌 XTerm.js 终端。此模式提供更丰富的任务管理和配置界面。

## 核心特性

- **多窗口并行**：`hi start claude,opencode,gemini` 一键启动多个 AI 终端
- **任务派发**：`hi push <target> "<task>"` 把任务路由给指定代理
- **结构化结果检测**：读取各工具原生历史存储（JSONL / SQLite），无需脆弱的文本匹配
- **卡死自动拉取**：未知工具 30 秒无响应时自动抓取终端内容兜底
- **tmux 布局**：智能分屏（偶数均分，奇数右侧优先）
- **SQLite 持久化**：所有消息、快照、任务队列落盘可追溯
- **Auto / Resume 模式**：`-a` 全自动跳过确认，`-r` 恢复上次会话上下文

## 前置依赖

### 运行时（必需）

**tmux** — 终端布局模式的运行时依赖：

```bash
# macOS
brew install tmux

# Ubuntu / Debian
sudo apt install tmux

# Fedora
sudo dnf install tmux
```

**Windows** — tmux 不能在原生 Windows 环境下运行，有两种方式：

- **WSL2（推荐）**：安装 [WSL2](https://learn.microsoft.com/zh-cn/windows/wsl/install)，在 WSL 终端内执行 `sudo apt install tmux`，然后在 WSL 环境中使用 `hi`。
- **Git Bash / Scoop**：支持有限，不建议用于多窗口协作功能。

**AI 工具** — 安装你需要的代理（见上表）。

### 普通用户 — 无需安装 Rust

从 [Releases 页面](https://github.com/your-org/hi/releases) 下载对应平台的预编译安装包。

### 从源码构建

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust + Cargo | 1.80+ | 编译所有核心组件 |
| Node.js | 20+ | 前端构建（仅 Tauri 桌面版需要）|

```bash
# 安装 Rust (macOS / Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node.js（通过 fnm）
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 20 && fnm use 20
```

Windows：

```powershell
# 安装 Rust
winget install Rustlang.Rustup

# 安装 Node.js
winget install OpenJS.NodeJS.LTS

# 安装后重启 PowerShell，然后验证：
rustc --version
cargo --version
node --version
```

## 安装

### 方式一 — 预编译安装包

前往 [Releases](https://github.com/your-org/hi/releases) 下载：

| 平台 | 文件 |
|------|------|
| macOS | `hi_macos.dmg` 或 `hi.app.zip` |
| Windows | `hi_windows_setup.exe` |
| Linux | `hi_linux.AppImage` 或 `.deb` |

### 方式二 — 从源码构建 (CLI + 桌面版)

#### 使用安装脚本（推荐）

```bash
git clone https://github.com/your-org/hi.git
cd hione

# macOS / Linux
bash scripts/install.sh --with-desktop
export PATH="$HOME/.local/bin:$PATH"

# Windows (PowerShell)
pwsh scripts/install.ps1 -WithDesktop
```

`--with-desktop` 标志会触发 Tauri 构建流程，并将桌面版应用与 `hi` CLI 一并安装。

#### 手动构建

```bash
git clone https://github.com/your-org/hi.git
cd hione

# 1. 安装 tauri-cli（cargo tauri build 必需）
cargo install tauri-cli --locked

# 2. 安装前端依赖（仅首次需要）
cd crates/hi-tauri && npm install && cd ../..

# 3. 构建桌面应用（beforeBuildCommand 自动构建 CLI 和前端）
cargo tauri build
```

Windows PowerShell:

```powershell
git clone https://github.com/your-org/hi.git
cd hione

# 1. 安装 tauri-cli
cargo install tauri-cli --locked

# 2. 安装前端依赖
cd crates\hi-tauri; npm install; cd ..\..

# 3. 构建桌面应用
cargo tauri build
```

## 桌面端应用使用

安装完成后，你可以从应用程序文件夹 (macOS)、开始菜单 (Windows) 启动 **hione**，或通过命令行启动：

```bash
# 启动桌面端 UI
hi-tauri
```

在桌面端应用中，你可以：
- **配置代理**：为每个 AI 工具设置路径、启动标志等。
- **会话启动器**：选择工具和项目目录，一键开启多窗口协作会话。
- **任务面板**：实时监控所有活跃代理的任务队列和执行结果。
- **技能管理**：管理自定义技能仓库和 MCP 服务器。

## 快速开始 (CLI)

**前提条件**：CLI 终端模式需要在 tmux session 中运行。如果未安装桌面应用 (`hi-tauri`)，请先进入 tmux：

```bash
# 创建 tmux session（终端模式必需）
tmux new -s hi

# 然后在 tmux 中启动三个 AI 代理窗口
hi start claude,opencode,gemini

# 全自动模式（跳过所有权限确认）
hi start -a claude,opencode,gemini

# 恢复上次会话上下文
hi start -r claude,opencode,gemini

# 派发任务
hi push opencode "实现 JWT 登录接口"

# 查看代理在线状态
hi check opencode

# 拉取当前终端内容
hi pull opencode
```

## CLI 命令参考

| 命令 | 简写 | 说明 |
|------|------|------|
| `hi start [-a] [-r] <tools>` | `hi s` | 启动会话 |
| `hi push <target> "<task>"` | `hi p` | 派发任务 |
| `hi pull <target> [-t N]` | `hi pl` | 拉取窗口内容（默认超时 5s）|
| `hi check <target> [-t N]` | `hi ck` | 检查代理在线状态（默认超时 3s）|
| `hi result <id> "<content>"` | `hi r` | 回复任务结果 |
| `hi esc <id>` | `hi e` | 取消任务 |

### start 启动标志

| 标志 | 说明 |
|------|------|
| `-a, --auto` | 跳过目标工具的所有权限确认提示 |
| `-r, --resume` | 恢复最近一次会话上下文 |

各工具实际使用的 flags：

| 工具 | `--auto` flag | `--resume` flag |
|------|---------------|-----------------|
| claude | `--dangerously-skip-permissions` | `--continue` |
| gemini | `--yolo` | `--resume latest` |
| opencode | 无 | `--continue` |
| qwen | `--yolo` | `--continue` |
| codex | `--full-auto` | `codex resume --last` |

## 多 Agent 协作

### 自定义工具

在项目目录的 `.hione/tools.toml` 中配置第三方工具的 auto/resume flags：

```toml
[tools.ccg]
auto_flags   = ["--dangerously-skip-permissions"]
resume_flags = ["--continue"]
```

未配置的未知工具会原样透传命令，不附加任何 flag。

### 任务协议

通过 `hi push` 派发的任务，代理完成后需在回复末尾写：

```
Task DONE: <uuid>
```

Hi 会自动从各工具的结构化历史存储（JSONL / SQLite）检测完成状态。

## 架构

```text
┌──────────────────────────────────────────────────────┐
│  hi push opencode "实现登录接口"                       │
└──────────────────────┬───────────────────────────────┘
                       │ Unix Socket (.hione/hi.sock)
                       ▼
              ┌────────────────┐         每 10s 轮询
              │   hi-monitor   │ ──────────────────────┐
              │  （守护进程）   │                       │
              └────────┬───────┘                       ▼
                       │ 任务队列                ┌──────────────┐
                       │                         │   hi-tauri   │
                       ▼                         │  + XTerm.js  │
              ┌────────────────┐                 │  + PTY       │
              │ SQLite (.hione)│◄────快照写入────┤              │
              │ messages/      │                 └──────┬───────┘
              │ snapshots/     │                        │
              │ task_queue     │                        ▼
              └────────────────┘               ┌─────────────────┐
                                               │  claude / gemini │
                                               │  opencode / ...  │
                                               └─────────────────┘
```

## 贡献

欢迎提交 Issue 和 PR。请先阅读 [CLAUDE.md](CLAUDE.md) 了解协作规范。

## 许可证

基于 [MIT License](LICENSE) 开源。
