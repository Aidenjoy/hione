# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**hione** is an all-in-one AI tools desktop app + CLI system. The Tauri desktop app provides a graphical interface for managing agents, MCP servers, skills, and settings. The **`hi`** CLI launches multi-window AI collaboration in terminal (tmux) or desktop mode.

## Commands

### Rust (workspace root)
```bash
cargo build --workspace          # Build all crates
cargo build -p hi-cli --release  # Release CLI binary
cargo build -p hi-tauri --release # Release desktop app
cargo test --workspace           # All tests
cargo test -p hi-core            # hi-core tests only
cargo test -p hi-cli             # CLI command tests
cargo test -p hi-monitor         # Monitor tests
cargo test -p hi-tauri --lib     # Tauri backend tests
cargo check --workspace          # Type check
cargo clippy --workspace         # Lint
cargo fmt --check                # Format check
bash scripts/coverage.sh         # Coverage report (HTML + lcov)
bash scripts/install.sh          # Install CLI binaries (macOS/Linux)
bash scripts/install.sh --with-desktop  # Install CLI + desktop app
bash scripts/uninstall.sh        # Uninstall binaries
pwsh scripts/install.ps1         # Install binaries (Windows)
pwsh scripts/install.ps1 -WithDesktop   # Install CLI + desktop (Windows)
pwsh scripts/uninstall.ps1       # Uninstall binaries (Windows)
```

### Frontend (crates/hi-tauri/)
```bash
npm install
npm run dev    # Vite dev server
npm run build  # tsc + vite production build
```

### Tauri Desktop App
```bash
cargo tauri dev     # Dev mode (from crates/hi-tauri/)
cargo tauri build   # Full desktop app build
```

## Architecture

Four-crate Rust workspace:

```
hi-core (shared library)
  ├── hi-cli (user-facing binary: `hi`)
  ├── hi-monitor (background daemon)
  └── hi-tauri (Tauri app + React frontend)
```

### hi-core
Shared protocol types and infrastructure:
- `message.rs` — `Message`, `MessageType`, `TaskStatus` structs
- `ipc.rs` — IPC frame codec (4-byte length-prefixed JSON over Unix sockets)
- `protocol.rs` — Task/result envelope formatting and result extraction
- `db.rs` — SQLite CRUD operations for messages
- `session.rs` — `SessionInfo`/`WindowInfo` for `.hione/session.json`
- `history.rs` — Read AI tool native history storage (JSONL/SQLite)
- `error.rs` — Error types

**MessageType variants:** `Task`, `Result`, `Cancel`, `Check`, `CheckAck`, `Pull`, `Snapshot`, `SnapshotData`, `SessionReady`

### hi-cli
CLI commands routed via Clap:
- `start` — Launch session (tmux or desktop mode, with `-a` auto, `-r` resume, `-T` terminal, `-D` desktop, `--monitor-only`)
- `push` — Dispatch task to target agent
- `pull` — Pull terminal content/result from agent
- `check` — Check if agent window exists
- `result` — Submit task result manually
- `esc` — Cancel a task by ID

Each command sends a `Message` over Unix socket to hi-monitor at `.hione/hi.sock`.

### hi-monitor
Background daemon spawned by `hi start`. Responsibilities:
- IPC server on `.hione/hi.sock`
- Per-window task queues (`TaskQueueMap`)
- Task dispatch to tmux panes via `tmux send-keys`
- Snapshot polling (uses tool native history storage, falls back to `tmux capture-pane`)
- Result extraction via `protocol::extract_result`
- Auto-pull cooldown (60s between pulls)
- `SessionReady` message handling for pane_id updates

### hi-tauri
Tauri desktop application with two parts:

**Backend (Rust):**
- `commands/` — Tauri IPC handlers for frontend
  - `setup.rs` — Check/install dependencies (tmux, node, rust, hi, hi-monitor)
  - `tools.rs` — List/install/uninstall/update AI tools
  - `agent.rs` — CRUD for agent configurations
  - `mcp.rs` — MCP server management
  - `skill.rs` — Skill/repo management
  - `session.rs` — Session launch/connect/disconnect
  - `task.rs` — Task push/cancel/check
  - `custom_tools.rs` — Read/write `.hione/tools.toml`
  - `settings.rs` — App settings (language, theme, binary paths)
- `services/` — Business logic implementations
  - `session.rs` — Terminal launching (iTerm2/Terminal on macOS, wt/cmd on Windows, gnome-terminal/etc on Linux)
  - `agent_config.rs` — Agent CRUD + connection testing
  - `mcp_config.rs` — MCP server CRUD + tool sync
  - `skill_manager.rs` — Skill CRUD + sync
  - `tool_manager.rs` — Tool install/uninstall logic
  - `ipc_client.rs` — Connect to hi-monitor socket
  - `notify.rs` — Desktop notifications
- `db/schema.rs` — SQLite schema for agents, mcp_servers, skills, skill_repos, recent_sessions

**Frontend (React + TypeScript):**
- React Router + TanStack Query + Zustand (state management)
- Tailwind CSS + Radix UI components
- Pages: Setup, Launcher, TaskPanel, ToolManager, AgentConfig, Mcp, Skills, CustomTools, Help, About, Settings
- Components: Layout, Sidebar, StatusBar, AgentIcon, StatusIcon
- i18n support (English/Chinese)

## Message Flow

```
hi push opencode "task"
  → Unix socket → hi-monitor
  → task_queue enqueue
  → tmux send-keys to pane
  → AI tool processes task
  → monitor polls history storage / capture-pane
  → extract_result finds "Task DONE: <uuid>"
  → result stored in SQLite
  → envelope delivered to sender pane
```

## Runtime Data

All runtime state lives in `.hione/` (git-ignored):
- `session.json` — Window list, pane IDs, monitor PID
- `hi.db` — SQLite (messages table)
- `hi.sock` — Monitor IPC socket
- `logs/` — monitor.log, session.log
- `CONTEXT.md` — Collaboration context for agents
- `tools.toml` — Custom tool flag configurations

Desktop app stores global config in platform-specific app data dir:
- Agents, MCP servers, Skills, Skill repos, Recent sessions, App settings

## Key Types

**IPC frame:** `[4-byte big-endian u32 length][JSON payload]`

**Message fields:** `id (Uuid)`, `sender`, `receiver`, `timestamp`, `content`, `msg_type (MessageType)`, `status (TaskStatus)`, `parent_id (Option<Uuid>)`

**SessionInfo:** `id`, `windows (Vec<WindowInfo>)`, `work_dir`, `hione_dir`, `socket_path`, `monitor_pid`, `tmux_session_name`

**WindowInfo:** `index`, `name`, `command`, `launch_command`, `auto_mode`, `resume_mode`, `is_main`, `pid`, `tmux_pane_id`

## Supported AI Tools

| Tool | Auto flag | Resume flag |
|------|-----------|-------------|
| claude | `--dangerously-skip-permissions` | `--continue` |
| gemini | `--yolo` | `--resume latest` |
| opencode | (none) | `--continue` |
| qwen | `--yolo` | `--continue` |
| codex | `--full-auto` | `codex resume --last` |

Custom tools can be configured in `.hione/tools.toml` with `auto_flags` and `resume_flags`.

## Docs

- `README.md` — Project intro, architecture diagram, quick start
- `README_zh.md` — Chinese documentation
- `docs/user/installation.md` — Cross-platform installation guide

## Windows Development Notes

**重要：Windows 平台统一使用 PowerShell，不使用 cmd.exe**

- 所有脚本文件使用 `.ps1` 格式（install.ps1, uninstall.ps1）
- 所有进程管理使用 PowerShell 命令（Get-Process, Stop-Process）
- 所有终端窗口启动使用 PowerShell（wt + powershell 或 Start-Process powershell）
- 版本检测、路径查找使用 PowerShell（where.exe, powershell -Command）

**禁止使用 cmd.exe 的场景**：
- ❌ `Command::new("cmd")`
- ❌ `tasklist` 命令
- ❌ `cmd /C start` 启动窗口

**正确做法**：
- ✅ `Command::new("powershell")`
- ✅ `Get-Process -Id <pid>` 检查进程
- ✅ `Start-Process powershell` 启动窗口

## Testing Structure

- **hi-core tests (40 tests):**
  - `tests/protocol_test.rs` — Task/result envelope formatting and extraction
  - `tests/message_test.rs` — Message, MessageType, TaskStatus serialization
  - `tests/ipc_test.rs` — IPC frame codec (send/receive)
  - `tests/db_test.rs` — SQLite CRUD operations
  - `tests/session_test.rs` — SessionInfo/WindowInfo serialization and load_from
  - `tests/error_test.rs` — HiError variants and conversions
  - `tests/history_test.rs` — read_latest_response for supported tools
  - `tests/integration_test.rs` — IPC + DB end-to-end
  - Inline tests in `history.rs` — Private function tests (encode_cwd_for_claude)

- **hi-cli tests (24 tests):**
  - `tests/start_test.rs` — WindowInfo launch_command, multi-window session serialization
  - `tests/push_test.rs` — send_to_monitor IPC client
  - `tests/pull_test.rs` — fetch snapshot content
  - `tests/check_test.rs` — probe agent online status
  - `tests/esc_test.rs` — send_cancel message delivery
  - `tests/result_test.rs` — submit result routing
  - `tests/common_test.rs` — load_session_from file reading
  - Inline tests in `start.rs` — Private function tests (build_launch_command_with_hione)
  - Inline tests in `push.rs` — Private function tests (detect_sender)

- **hi-monitor tests (17 tests):**
  - `tests/task_queue_test.rs` — TaskQueueMap enqueue/pop/cancel
  - `tests/server_integration.rs` — IPC message handling (Task, Result, Cancel, Pull, Check)
  - `tests/detector_test.rs` — auto_return_stuck_content, stuck detection
  - `tests/snapshot_test.rs` — request_snapshot with fallback
  - `tests/tmux_test.rs` — Bracketed paste format logic

- **hi-tauri tests (18 tests):**
  - `tests/types_test.rs` — All type defaults and serde roundtrips
  - `tests/error_test.rs` — AppError variants and conversions

- Coverage: `cargo-llvm-cov` via `scripts/coverage.sh`, output in `target/coverage/`
- Total: **99 tests** across all crates


















<!-- hi-collaboration-start -->
## Hi Multi-Agent Collaboration
@.hione/CONTEXT.md
<!-- hi-collaboration-end -->
