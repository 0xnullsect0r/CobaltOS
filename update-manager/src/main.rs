mod apt;
mod flatpak;
mod notify;
mod rollback;
mod daemon;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("cobalt-update v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("check")    => daemon::check_updates().await,
        Some("apply")    => daemon::apply_updates().await,
        Some("rollback") => rollback::rollback().await,
        Some("daemon")   => daemon::run_daemon().await,
        _                => daemon::run_daemon().await,
    }
}
