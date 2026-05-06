//! D-Bus service interface for cobalt-hardware-probe.
//!
//! Exposes a D-Bus API so other cobalt components (welcome, installer)
//! can query hardware info and trigger re-detection at runtime.

use anyhow::Result;
use tracing::info;
use zbus::{connection, interface};

use crate::dmi::{detect_board, Board};

pub struct HardwareProbeService {
    board: Board,
}

#[interface(name = "tech.cobaltos.HardwareProbe1")]
impl HardwareProbeService {
    /// Returns the detected board name (e.g. "BOBBA").
    fn board_name(&self) -> &str {
        &self.board.name
    }

    /// Returns the full product name (e.g. "Chromebook 311").
    fn product_name(&self) -> &str {
        &self.board.product_name
    }

    /// Returns true if MrChromebox UEFI Full ROM is detected.
    fn has_uefi_firmware(&self) -> bool {
        self.board.has_uefi_full_rom()
    }

    /// Trigger a re-detection and re-application of all hardware settings.
    async fn reapply(&mut self) -> zbus::fdo::Result<()> {
        let board = detect_board().await.map_err(|e| {
            zbus::fdo::Error::Failed(format!("Board detection failed: {e}"))
        })?;
        self.board = board;
        Ok(())
    }
}

/// Start the D-Bus service. This runs as part of the normal daemon flow.
pub async fn run_service() -> Result<()> {
    let board = detect_board().await?;
    info!("Starting D-Bus service for board: {}", board.name);

    let service = HardwareProbeService { board };
    let _conn = connection::Builder::session()?
        .name("tech.cobaltos.HardwareProbe")?
        .serve_at("/tech/cobaltos/HardwareProbe", service)?
        .build()
        .await?;

    info!("D-Bus service running at tech.cobaltos.HardwareProbe");

    // Keep the service alive
    std::future::pending::<()>().await;
    Ok(())
}
