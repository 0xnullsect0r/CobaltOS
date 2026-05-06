//! Update manager daemon logic.

use anyhow::Result;
use tracing::info;

use crate::{apt, flatpak, notify};

/// Check for updates and notify if any are available.
pub async fn check_updates() -> Result<()> {
    info!("Checking for updates");

    let apt_updates = apt::check_upgrades().await.unwrap_or_default();
    let flatpak_updates = flatpak::check_upgrades().await.unwrap_or_default();

    let total = apt_updates.len() + flatpak_updates.len();

    if total > 0 {
        let body = format!(
            "{} system package(s) and {} app(s) ready to update.",
            apt_updates.len(),
            flatpak_updates.len()
        );
        notify::send("Updates available", &body).await?;
        info!("{}", body);
    } else {
        info!("System is up to date");
    }

    Ok(())
}

/// Apply all pending updates.
pub async fn apply_updates() -> Result<()> {
    info!("Applying all updates");

    apt::apply_upgrades().await?;
    flatpak::apply_upgrades().await?;

    notify::send("Update complete", "Your system is up to date.").await?;
    info!("All updates applied successfully");
    Ok(())
}

/// Run as a background daemon, checking for updates on a schedule.
pub async fn run_daemon() -> Result<()> {
    info!("cobalt-update daemon started (checking every 6 hours)");

    loop {
        check_updates().await.unwrap_or_else(|e| {
            tracing::warn!("Update check failed: {e}");
        });
        // Sleep 6 hours between checks
        tokio::time::sleep(tokio::time::Duration::from_secs(6 * 3600)).await;
    }
}
