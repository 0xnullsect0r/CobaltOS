//! Power profile configuration optimised for Chromebook battery hardware.

use anyhow::Result;
use tracing::info;

use crate::dmi::Board;

/// Apply the optimal power profile for this board.
pub async fn apply_power_profile(board: &Board) -> Result<()> {
    info!("Applying power profile for board: {}", board.name);

    // Detect if power-profiles-daemon is available
    let has_ppd = which("powerprofilesctl").is_some();
    if has_ppd {
        set_ppd_profile("balanced").await;
    }

    // Enable powertop auto-tune via systemd service if available
    enable_powertop().await;

    // Enable Intel or AMD panel self-refresh
    apply_psr(board).await?;

    Ok(())
}

async fn set_ppd_profile(profile: &str) {
    let r = tokio::process::Command::new("powerprofilesctl")
        .args(["set", profile])
        .status()
        .await;
    match r {
        Ok(s) if s.success() => info!("Power profile set to '{profile}'"),
        Ok(s) => tracing::warn!("powerprofilesctl exited {s}"),
        Err(e) => tracing::warn!("powerprofilesctl not available: {e}"),
    }
}

async fn enable_powertop() {
    let r = tokio::process::Command::new("systemctl")
        .args(["enable", "--now", "powertop"])
        .status()
        .await;
    if matches!(r, Ok(s) if s.success()) {
        info!("powertop auto-tune enabled");
    }
}

/// Enable Panel Self-Refresh (PSR) to reduce power draw.
async fn apply_psr(board: &Board) -> Result<()> {
    // Most Intel and AMD Chromebooks support PSR.
    // Write the kernel parameter via sysfs if available.
    let psr_path = "/sys/module/i915/parameters/enable_psr";
    if std::path::Path::new(psr_path).exists() {
        std::fs::write(psr_path, "1")?;
        info!("Intel PSR enabled for board: {}", board.name);
    }
    Ok(())
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(cmd))
            .find(|p| p.is_file())
    })
}
