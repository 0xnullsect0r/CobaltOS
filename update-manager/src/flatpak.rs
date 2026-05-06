//! Flatpak integration for cobalt-update.

use anyhow::Result;
use tracing::info;

pub async fn check_upgrades() -> Result<Vec<String>> {
    info!("Checking for Flatpak updates");

    let output = tokio::process::Command::new("flatpak")
        .args(["remote-ls", "--updates"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect();

    info!("{} Flatpak app(s) have updates", packages.len());
    Ok(packages)
}

pub async fn apply_upgrades() -> Result<()> {
    info!("Applying Flatpak updates");

    let status = tokio::process::Command::new("flatpak")
        .args(["update", "-y"])
        .status()
        .await?;

    if status.success() {
        info!("Flatpak updates applied");
    } else {
        anyhow::bail!("flatpak update failed with status: {status}");
    }
    Ok(())
}
