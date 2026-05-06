//! Display scaling configuration for HiDPI Chromebook screens.

use anyhow::Result;
use tracing::info;

use crate::dmi::Board;

/// Boards known to have HiDPI displays (≥200 DPI) that need 2× scaling.
const HIDPI_BOARDS: &[&str] = &[
    "LINK",    // Chromebook Pixel 2013 — 239 DPI
    "SAMUS",   // Chromebook Pixel 2015 — 239 DPI
    "EVE",     // Pixelbook — 235 DPI
    "ATLAS",   // Pixelbook Go — 166 DPI (1.5× may be better)
    "NOCTURNE", // Pixel Slate — 293 DPI
];

/// Boards with displays that work well at 1.5× fractional scaling.
const HIDPI_FRACTIONAL_BOARDS: &[&str] = &["ATLAS"];

/// Apply the correct display scaling for the detected board.
pub async fn apply_scaling(board: &Board) -> Result<()> {
    let scale = detect_scale(board);
    info!(
        "Applying display scale {}× for board: {}",
        scale, board.name
    );

    // Write COSMIC display config
    write_cosmic_scale(scale).await?;

    // Also set Xorg DPI for X11 fallback applications
    write_xresources_dpi(scale).await?;

    Ok(())
}

fn detect_scale(board: &Board) -> f32 {
    if HIDPI_FRACTIONAL_BOARDS.contains(&board.name.as_str()) {
        1.5
    } else if HIDPI_BOARDS.contains(&board.name.as_str()) {
        2.0
    } else {
        1.0
    }
}

async fn write_cosmic_scale(scale: f32) -> Result<()> {
    let config_dir = "/etc/cobaltos/cosmic";
    std::fs::create_dir_all(config_dir)?;
    let content = format!("display_scale = {scale:.1}\n");
    std::fs::write(format!("{config_dir}/display.toml"), content)?;
    Ok(())
}

async fn write_xresources_dpi(scale: f32) -> Result<()> {
    let base_dpi = 96.0;
    let dpi = (base_dpi * scale) as u32;
    let xresources = format!("Xft.dpi: {dpi}\n");
    let path = "/etc/X11/Xresources/cobaltos-dpi";
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, xresources)?;
    Ok(())
}
