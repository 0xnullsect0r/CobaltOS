//! Audio UCM profile configuration for Chromebook boards.
//!
//! Chromebook audio is routed through complex DSP paths that require
//! board-specific ALSA Use Case Manager (UCM) profiles.

use anyhow::Result;
use tracing::{info, warn};

use crate::dmi::Board;

const UCM_BASE_DIR: &str = "/usr/share/alsa/ucm2";
const COBALT_UCM_DIR: &str = "/usr/share/cobaltos/ucm";

/// Apply the correct UCM profile for the detected board.
pub async fn apply_ucm_profile(board: &Board) -> Result<()> {
    info!("Applying audio UCM profile for board: {}", board.name);

    let profile_path = format!("{COBALT_UCM_DIR}/{}.conf", board.name.to_lowercase());

    if std::path::Path::new(&profile_path).exists() {
        install_ucm_profile(&profile_path, board).await?;
    } else {
        // Try to fall back to a sof-audio generic profile
        warn!(
            "No board-specific UCM profile for {}. Trying sof-audio generic fallback.",
            board.name
        );
        apply_sof_fallback(board).await;
    }

    // Restart PipeWire/PulseAudio to pick up the new profile
    restart_audio_server().await;

    Ok(())
}

async fn install_ucm_profile(profile_path: &str, board: &Board) -> Result<()> {
    let dest = format!("{UCM_BASE_DIR}/{}", board.name.to_lowercase());
    std::fs::create_dir_all(&dest)?;
    // In a real implementation this would symlink/copy the full UCM tree.
    // For now we copy the main conf file.
    std::fs::copy(profile_path, format!("{dest}/{}.conf", board.name.to_lowercase()))?;
    info!("UCM profile installed to {dest}");
    Ok(())
}

async fn apply_sof_fallback(board: &Board) {
    // Attempt to run sof-audio-setup if available
    let result = tokio::process::Command::new("sof-audio-setup")
        .arg(&board.name)
        .status()
        .await;
    match result {
        Ok(s) if s.success() => info!("sof-audio-setup succeeded"),
        Ok(s) => warn!("sof-audio-setup exited {s}"),
        Err(e) => warn!("sof-audio-setup not found: {e}"),
    }
}

/// Invoked by `cobalt-hardware-probe --fix-audio`.
pub async fn fix_audio() -> Result<()> {
    info!("Running audio fix");
    let board = crate::dmi::detect_board().await?;
    apply_ucm_profile(&board).await?;
    Ok(())
}

async fn restart_audio_server() {
    // Try PipeWire first, then PulseAudio
    for service in ["pipewire", "pulseaudio"] {
        let r = tokio::process::Command::new("systemctl")
            .args(["--user", "restart", service])
            .status()
            .await;
        if matches!(r, Ok(s) if s.success()) {
            info!("{service} restarted");
            return;
        }
    }
    warn!("Could not restart audio server");
}
