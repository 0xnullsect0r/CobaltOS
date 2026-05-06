//! Update manager daemon logic.

use anyhow::Result;
use std::collections::HashSet;
use tracing::info;

use crate::{apt, flatpak, notify, rollback};

const STATE_DIR: &str = "/var/lib/cobalt-update";
const STATE_FILE: &str = "/var/lib/cobalt-update/last-notified.json";

/// Check for updates and notify only if the set of available updates has changed.
pub async fn check_updates() -> Result<()> {
    info!("Checking for updates");

    let apt_updates = apt::check_upgrades().await.unwrap_or_default();
    let flatpak_updates = flatpak::check_upgrades().await.unwrap_or_default();

    let mut current: HashSet<String> = apt_updates.iter().cloned().collect();
    current.extend(flatpak_updates.iter().cloned());

    if current.is_empty() {
        info!("System is up to date");
        // Clear state so next cycle notifies again when updates appear
        let _ = save_notified_set(&HashSet::new());
        return Ok(());
    }

    let last_notified = load_notified_set().unwrap_or_default();

    // Only notify if there are new packages we haven't seen before
    let new_packages: HashSet<&String> = current.difference(&last_notified).collect();
    if !new_packages.is_empty() {
        let body = format!(
            "{} system package(s) and {} app(s) ready to update.",
            apt_updates.len(),
            flatpak_updates.len()
        );
        notify::send("Updates available", &body).await?;
        info!("{}", body);
        save_notified_set(&current)?;
    } else {
        info!(
            "{} update(s) pending (already notified, skipping)",
            current.len()
        );
    }

    Ok(())
}

/// Apply all pending updates with a pre-upgrade snapshot for safe rollback.
/// On failure, rolls back via dpkg state; on btrfs systems a subvolume snapshot
/// is also available.
pub async fn apply_updates() -> Result<()> {
    info!("Creating pre-upgrade snapshot");
    rollback::create_snapshot().await.unwrap_or_else(|e| {
        tracing::warn!("Pre-upgrade snapshot failed (continuing anyway): {e}");
    });

    info!("Applying all updates");
    let result = async {
        apt::apply_upgrades().await?;
        flatpak::apply_upgrades().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    match result {
        Ok(()) => {
            // Clear state — next check will show fresh update set
            let _ = save_notified_set(&HashSet::new());
            notify::send("Update complete", "Your system is up to date.").await?;
            info!("All updates applied successfully");
        }
        Err(e) => {
            tracing::error!("Update failed: {e}. Attempting rollback…");
            notify::send(
                "Update failed",
                "An error occurred during updates. CobaltOS is restoring the previous state.",
            )
            .await
            .ok();
            rollback::rollback().await.unwrap_or_else(|re| {
                tracing::error!("Rollback also failed: {re}");
            });
            return Err(e);
        }
    }

    Ok(())
}

/// Run as a background daemon, checking for updates every 6 hours.
/// Uses an interval timer to avoid drift from slow network/apt operations.
pub async fn run_daemon() -> Result<()> {
    info!("cobalt-update daemon started (checking every 6 hours)");

    // Check immediately on start
    check_updates().await.unwrap_or_else(|e| {
        tracing::warn!("Initial update check failed: {e}");
    });

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 3600));
    interval.tick().await; // consume the immediate first tick

    loop {
        interval.tick().await;
        check_updates().await.unwrap_or_else(|e| {
            tracing::warn!("Update check failed: {e}");
        });
    }
}

// ── State persistence ────────────────────────────────────────────────────────

fn load_notified_set() -> Result<HashSet<String>> {
    if !std::path::Path::new(STATE_FILE).exists() {
        return Ok(HashSet::new());
    }
    let content = std::fs::read_to_string(STATE_FILE)?;
    let packages: Vec<String> = serde_json::from_str(&content)?;
    Ok(packages.into_iter().collect())
}

fn save_notified_set(packages: &HashSet<String>) -> Result<()> {
    std::fs::create_dir_all(STATE_DIR)?;
    let mut sorted: Vec<&String> = packages.iter().collect();
    sorted.sort();
    let content = serde_json::to_string_pretty(&sorted)?;
    std::fs::write(STATE_FILE, content)?;
    Ok(())
}
