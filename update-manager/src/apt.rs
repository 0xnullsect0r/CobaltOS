//! apt integration for cobalt-update.

use anyhow::Result;
use tracing::info;

/// Check for available apt upgrades. Returns list of upgradable package names.
pub async fn check_upgrades() -> Result<Vec<String>> {
    info!("Checking for apt upgrades");

    let output = tokio::process::Command::new("apt-get")
        .args(["--simulate", "upgrade"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("Inst ") {
            if let Some(pkg) = line.split_whitespace().nth(1) {
                packages.push(pkg.to_string());
            }
        }
    }

    info!("{} apt package(s) upgradable", packages.len());
    Ok(packages)
}

/// Apply all pending apt upgrades.
pub async fn apply_upgrades() -> Result<()> {
    info!("Applying apt upgrades");

    let status = tokio::process::Command::new("apt-get")
        .args(["-y", "upgrade"])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status()
        .await?;

    if status.success() {
        info!("apt upgrades applied successfully");
    } else {
        anyhow::bail!("apt-get upgrade failed with status: {status}");
    }
    Ok(())
}
