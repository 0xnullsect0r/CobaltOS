mod ui;
mod tui;
mod installer;
mod hardware;
mod partition;
mod config;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("cobalt-installer v{}", env!("CARGO_PKG_VERSION"));

    // Detect whether we have a display server available.
    // If DISPLAY or WAYLAND_DISPLAY is set, launch the GUI installer.
    // Otherwise, fall back to the TUI installer.
    let has_display = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("DISPLAY").is_ok();

    if has_display {
        info!("Launching GUI installer (iced)");
        ui::run()
    } else {
        info!("Launching TUI installer (ratatui)");
        tui::run().await
    }
}
