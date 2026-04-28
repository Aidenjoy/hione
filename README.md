# hione — All In One AI Tools

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: macOS/Windows/Linux](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](#)

[中文文档](README_zh.md)

hione is an all-in-one AI tools desktop app + CLI system. The Tauri desktop app provides a graphical interface for managing agents, MCP servers, skills, and settings. The hi command is the CLI tool for launching multi-window AI collaboration in your terminal.

## Supported Tools

| Tool | Install | Uninstall |
|------|---------|---------|
| [Claude Code](https://github.com/anthropics/claude-code) | `npm install -g @anthropic-ai/claude-code` | `npm uninstall -g @anthropic-ai/claude-code` |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | `npm install -g @google/gemini-cli` | `npm uninstall -g @google/gemini-cli` |
| [OpenCode](https://github.com/anomalyco/opencode) | `npm install -g opencode-ai` | `npm uninstall -g opencode-ai` |
| [Codex CLI](https://github.com/openai/codex) | `npm install -g @openai/codex` | `npm uninstall -g @openai/codex` |
| [Qwen Code](https://github.com/QwenLM/qwen-code) | `npm install -g @qwen-code/qwen-code` | `npm uninstall -g @qwen-code/qwen-code` |
| Any custom tool | Pass the binary name directly | — |

## How It Works

**hione** operates in two modes:

1. **tmux mode** (CLI default): `hi start` splits your current tmux session into panes — one per AI agent — and launches each tool automatically.
2. **Desktop mode** (Tauri): Each agent runs in a separate native window with an embedded XTerm.js terminal. This mode provides a richer UI for managing tasks and configurations.

## Features

- **Multi-Window Parallelism**: `hi start claude,opencode,gemini` launches multiple AI terminals at once
- **Task Dispatching**: `hi push <target> "<task>"` routes tasks to specific agents
- **Structured Result Detection**: Reads each tool's native history storage — no fragile text scraping
- **Auto-Pull on Stuck**: Falls back to terminal snapshot capture for unknown tools after 30s
- **tmux Layout**: Splits your session into a smart grid (even = equal halves, odd = right-weighted)
- **SQLite Persistence**: All messages, snapshots, and task queues are stored for traceability
- **Auto & Resume Modes**: `-a` skips all permission prompts; `-r` restores last session context

## Prerequisites

### Runtime (always required)

**tmux** — used for the terminal layout when Tauri desktop is not available:

```bash
# macOS
brew install tmux

# Ubuntu / Debian
sudo apt install tmux

# Fedora
sudo dnf install tmux
```

**Windows** — tmux does not run natively on Windows. Use one of:

- **WSL2 (recommended)**: Install [WSL2](https://learn.microsoft.com/en-us/windows/wsl/install), then inside the WSL terminal run `sudo apt install tmux` and use `hi` from there.
- **Git Bash / Scoop**: Limited support, not recommended for multi-window features.

**AI tools** — install whichever agents you want to use (see table above).

### For end users — no Rust needed

Download a pre-built release from the [Releases page](https://github.com/your-org/hi/releases) and run the installer.

### For building from source

| Dependency | Version | Purpose |
|------------|---------|---------|
| Rust + Cargo | 1.80+ | Compile all crates |
| Node.js | 20+ | Frontend build (Tauri desktop only) |

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via fnm)
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 20 && fnm use 20
```

## Installation

### Option A — Pre-built Binary

Go to [Releases](https://github.com/your-org/hi/releases) and download:

| Platform | File |
|----------|------|
| macOS | `hi_macos.dmg` or `hi.app.zip` |
| Windows | `hi_windows_setup.exe` |
| Linux | `hi_linux.AppImage` or `.deb` |

### Option B — Build from source (CLI + Desktop)

To build both the CLI and the Desktop application:

```bash
git clone https://github.com/your-org/hi.git
cd hione

# macOS / Linux
bash scripts/install.sh --with-desktop
export PATH="$HOME/.local/bin:$PATH"

# Windows (PowerShell)
pwsh scripts/install.ps1 -WithDesktop
```

The `--with-desktop` flag will build the Tauri application and install it alongside the `hi` CLI.

## Desktop Application Usage

Once installed, you can launch the **hione** desktop app from your Applications folder (macOS), Start Menu (Windows), or via the command line:

```bash
# Launch the desktop UI
hi-tauri
```

In the desktop app, you can:
- **Configure Agents**: Set up paths and flags for each AI tool.
- **Session Launcher**: Select tools and project directories to start a new collaborative session.
- **Task Panel**: Monitor task queues and results across all active agents.
- **Skill Manager**: Manage custom skills and MCP servers.

## Quick Start (CLI)

**Prerequisite**: CLI terminal mode requires running inside a tmux session. If you haven't installed the desktop app (`hi-tauri`), first enter tmux:

```bash
# Create a tmux session (required for terminal mode)
tmux new -s hi

# Then start three AI agents in separate windows
hi start claude,opencode,gemini

# Auto mode — skip all permission prompts
hi start -a claude,opencode,gemini

# Resume last session context
hi start -r claude,opencode,gemini

# Dispatch a task
hi push opencode "implement JWT auth"

# Check agent status
hi check opencode

# Pull current terminal content
hi pull opencode
```

## CLI Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `hi start [-a] [-r] <tools>` | `hi s` | Start a session |
| `hi push <target> "<task>"` | `hi p` | Dispatch a task |
| `hi pull <target> [-t N]` | `hi pl` | Pull window content (default timeout: 5s) |
| `hi check <target> [-t N]` | `hi ck` | Check if agent is online (default timeout: 3s) |
| `hi result <id> "<content>"` | `hi r` | Return a task result |
| `hi esc <id>` | `hi e` | Cancel a task |

### Start flags

| Flag | Description |
|------|-------------|
| `-a, --auto` | Skip all permission confirmations in the launched tool |
| `-r, --resume` | Resume the most recent session context |

Per-tool flags used internally:

| Tool | `--auto` flag | `--resume` flag |
|------|---------------|-----------------|
| claude | `--dangerously-skip-permissions` | `--continue` |
| gemini | `--yolo` | `--resume latest` |
| opencode | _(none)_ | `--continue` |
| qwen | `--yolo` | `--continue` |
| codex | `--full-auto` | `codex resume --last` |

## Multi-Agent Collaboration

### Custom Tools

Any tool not listed above can be configured in `.hione/tools.toml`:

```toml
[tools.ccg]
auto_flags   = ["--dangerously-skip-permissions"]
resume_flags = ["--continue"]
```

Unknown tools without config are passed through unchanged.

### Task Protocol

When an agent completes a task pushed via `hi push`, it should end its reply with:

```
Task DONE: <uuid>
```

Hi detects completion automatically via each tool's structured history storage (JSONL / SQLite). The `Task DONE` line is required only when Claude coordinates other agents inside a session.

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│  hi push opencode "Implement login API"              │
└──────────────────────┬───────────────────────────────┘
                       │ Unix Socket (.hione/hi.sock)
                       ▼
              ┌────────────────┐         poll every 10s
              │   hi-monitor   │ ──────────────────────┐
              │  (daemon)      │                       │
              └────────┬───────┘                       ▼
                       │ Task Queue              ┌──────────────┐
                       │                         │   hi-tauri   │
                       ▼                         │  + XTerm.js  │
              ┌────────────────┐                 │  + PTY       │
              │ SQLite (.hione)│◄────Snapshots───┤              │
              │ messages/      │                 └──────┬───────┘
              │ snapshots/     │                        │
              │ task_queue     │                        ▼
              └────────────────┘               ┌─────────────────┐
                                               │  claude / gemini │
                                               │  opencode / ...  │
                                               └─────────────────┘
```

## Contributing

Please read [CLAUDE.md](CLAUDE.md) for contribution guidelines before submitting a PR.

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
