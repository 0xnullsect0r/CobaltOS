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

    info!("cobalt-oobe v{}", env!("CARGO_PKG_VERSION"));

    // OOBE runs before the desktop session on first login.
    // It presents a ChromeOS-inspired setup wizard and writes a
    // completion marker so it doesn't run again on subsequent logins.
    let marker = dirs_next();
    if marker.exists() {
        info!("OOBE already completed, exiting");
        return Ok(());
    }

    app::run()?;
    std::fs::write(&marker, "")?;
    Ok(())
}

fn dirs_next() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("cobalt-oobe-done")
}
