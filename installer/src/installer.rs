//! Installer step model and execution engine.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum InstallStep {
    #[default]
    Welcome,
    DeviceCheck,
    DiskSetup,
    Location,
    Account,
    Confirm,
    Installing,
    Done,
}

impl InstallStep {
    pub fn all() -> &'static [InstallStep] {
        &[
            InstallStep::Welcome,
            InstallStep::DeviceCheck,
            InstallStep::DiskSetup,
            InstallStep::Location,
            InstallStep::Account,
            InstallStep::Confirm,
            InstallStep::Installing,
            InstallStep::Done,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            InstallStep::Welcome    => "Welcome",
            InstallStep::DeviceCheck => "Device Check",
            InstallStep::DiskSetup  => "Disk Setup",
            InstallStep::Location   => "Location",
            InstallStep::Account    => "Account",
            InstallStep::Confirm    => "Ready to Install",
            InstallStep::Installing => "Installing…",
            InstallStep::Done       => "Done!",
        }
    }

    pub fn index(&self) -> usize {
        Self::all().iter().position(|s| s == self).unwrap_or(0)
    }

    pub fn next(&self) -> Option<InstallStep> {
        let steps = Self::all();
        let idx = self.index();
        steps.get(idx + 1).cloned()
    }
}

/// User-provided configuration collected during the install wizard.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    pub locale: String,
    pub timezone: String,
    pub keyboard_layout: String,
    pub disk: String,
    pub username: String,
    pub hostname: String,
    pub use_full_disk: bool,
}

/// Execute the actual installation using the provided config.
/// In a real implementation this drives debootstrap, partition formatting,
/// bootloader installation, etc.
pub async fn run_install(config: &InstallConfig, progress: tokio::sync::mpsc::Sender<u8>) -> Result<()> {
    use tracing::info;

    let steps: &[(&str, u8)] = &[
        ("Partitioning disk", 5),
        ("Formatting partitions", 10),
        ("Installing base system", 30),
        ("Installing kernel", 50),
        ("Configuring system", 65),
        ("Installing bootloader", 75),
        ("Creating user account", 85),
        ("Applying hardware config", 92),
        ("Finalizing", 98),
        ("Complete", 100),
    ];

    for (label, pct) in steps {
        info!("Install step: {} ({}%)", label, pct);
        let _ = progress.send(*pct).await;
        // Simulate work — real impl calls debootstrap/apt/etc here
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    info!("Installation complete for user '{}'", config.username);
    Ok(())
}
