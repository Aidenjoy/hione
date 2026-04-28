use clap::{Parser, Subcommand};
use hi_cli::commands;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "hi", about = "Hi multi-window AI collaboration CLI")]
struct Cli {
    #[arg(short = 'v', long = "version")]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动多窗口会话：hi start [-a] [-r] [-T] [-D] claude,opencode,gemini
    #[command(alias = "s")]
    Start {
        /// 全自动模式，跳过所有权限确认
        #[arg(short = 'a', long = "auto")]
        auto_mode: bool,
        /// 恢复上一次会话上下文
        #[arg(short = 'r', long = "resume")]
        resume_mode: bool,
        /// 强制使用终端 (tmux) 模式
        #[arg(short = 'T', long = "terminal")]
        terminal_mode: bool,
        /// 强制使用桌面 (Tauri) 模式
        #[arg(short = 'D', long = "desktop")]
        desktop_mode: bool,
        /// 仅启动 monitor 守护进程 (内部使用)
        #[arg(long = "monitor-only", hide = true)]
        monitor_only: bool,
        /// 逗号分隔的工具列表，如 claude,opencode,gemini
        tools: String,
    },
    /// 派发任务给目标窗口：hi push opencode "implement auth"
    #[command(alias = "p")]
    Push {
        target: String,
        content: String,
    },
    /// 强制拉取目标窗口当前内容：hi pull opencode
    #[command(alias = "pl")]
    Pull {
        target: String,
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
    /// 子窗口回复任务结果：hi result <task_id> "result content"
    #[command(name = "result", alias = "r")]
    ResultCmd {
        task_id: String,
        content: String,
    },
    /// 取消任务：hi esc <task_id>
    #[command(alias = "e")]
    Esc {
        task_id: String,
    },
    /// 检查目标窗口是否在线：hi check opencode
    #[command(alias = "ck")]
    Check {
        target: String,
        #[arg(short, long, default_value = "3")]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if cli.version {
        println!("hi {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match cli.command {
        Some(Commands::Start { auto_mode, resume_mode, terminal_mode, desktop_mode, monitor_only, tools }) => {
            let names: Vec<String> = tools
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            commands::start::run(auto_mode, resume_mode, terminal_mode, desktop_mode, monitor_only, names).await
        }
        Some(Commands::Push { target, content }) => commands::push::run(target, content).await,
        Some(Commands::Pull { target, timeout }) => commands::pull::run(target, timeout).await,
        Some(Commands::ResultCmd { task_id, content }) => {
            commands::result::run(task_id, content).await
        }
        Some(Commands::Esc { task_id }) => commands::esc::run(task_id).await,
        Some(Commands::Check { target, timeout }) => commands::check::run(target, timeout).await,
        None => {
            // No subcommand provided, show help
            println!("Use 'hi --help' for usage information.");
            Ok(())
        }
    }
}
