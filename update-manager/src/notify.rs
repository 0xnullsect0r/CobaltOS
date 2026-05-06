//! Desktop notification support for cobalt-update.

use anyhow::Result;
use tracing::info;

/// Send a desktop notification via libnotify / D-Bus.
pub async fn send(summary: &str, body: &str) -> Result<()> {
    info!("Sending notification: {summary}");

    tokio::process::Command::new("notify-send")
        .args([
            "--app-name=CobaltOS Update",
            "--icon=software-update-available",
            summary,
            body,
        ])
        .status()
        .await?;

    Ok(())
}
