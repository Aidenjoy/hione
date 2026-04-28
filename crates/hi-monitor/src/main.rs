use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

use hi_monitor::{detector, server};

#[derive(Parser)]
#[command(name = "hi-monitor")]
struct Args {
    #[arg(long)]
    hione_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 日志写入 .hione/logs/monitor.log
    let logs_dir = args.hione_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;
    let log_file = logs_dir.join("monitor.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?),
        )
        .with_writer(file)
        .init();

    tracing::info!(
        "hi-monitor starting, hione_dir={}",
        args.hione_dir.display()
    );

    // 初始化 DB
    let pool = hi_core::db::init_db(&args.hione_dir).await?;

    // 加载 session
    let session_json = std::fs::read_to_string(args.hione_dir.join("session.json"))?;
    let session: hi_core::session::SessionInfo = serde_json::from_str(&session_json)?;

    // 构建共享状态
    let state = server::MonitorState::new(session, pool, args.hione_dir.clone());

    // 启动 session 重载任务（每 30 秒）
    let state_for_reload = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = state_for_reload.reload_session().await {
                tracing::warn!("Failed to reload session: {e}");
            }
        }
    });

    // 启动 IPC 服务器 + 快照检测器（并发运行）
    tokio::try_join!(
        server::run(state.clone()),
        detector::run(state.clone()),
    )?;

    Ok(())
}
