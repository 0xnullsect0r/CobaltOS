mod app;
#[path = "pages/mod.rs"]
mod pages;
mod theme;

use anyhow::Result;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("cobalt-welcome v{}", env!("CARGO_PKG_VERSION"));

    app::run()
}
