//! Chromebook keyboard remapping via keyd.
//!
//! Applies per-board keyboard config to remap the Search key to Super,
//! top-row function keys, and other Chromebook-specific keys.

use anyhow::Result;
use tracing::{info, warn};

use crate::dmi::Board;

const KEYD_CONFIG_DIR: &str = "/etc/keyd";
const KEYD_CONFIG_FILE: &str = "/etc/keyd/default.conf";

/// Board-specific configs are installed to this directory by the live-build hook.
const COBALT_KEYD_BOARDS_DIR: &str = "/etc/cobaltos/keyd/boards";

/// Base keyd config applied to all Chromebook boards.
const BASE_CONFIG: &str = r#"[ids]
*

[main]
# Remap Search/Capslock to Super (Meta/Windows key)
capslock = leftmeta
search = leftmeta

# Top-row: ChromeOS media/action keys by default
f1 = back
f2 = forward
f3 = refresh
f4 = f11
f5 = f5
f6 = brightnessdown
f7 = brightnessup
f8 = mute
f9 = volumedown
f10 = volumeup

[meta]
# Hold Search (Super) + top-row = standard F-keys
f1 = f1
f2 = f2
f3 = f3
f4 = f4
f5 = f5
f6 = f6
f7 = f7
f8 = f8
f9 = f9
f10 = f10
"#;

/// Apply keyboard remapping for the detected board.
pub async fn apply_remapping(board: &Board) -> Result<()> {
    info!("Applying keyboard remapping for board: {}", board.name);

    std::fs::create_dir_all(KEYD_CONFIG_DIR)?;

    let config = load_board_config(board);
    std::fs::write(KEYD_CONFIG_FILE, &config)?;

    reload_keyd().await;

    info!("Keyboard remapping applied: {KEYD_CONFIG_FILE}");
    Ok(())
}

/// Load a board-specific keyd config if one exists in the cobalt config dir,
/// otherwise fall back to generated config.
fn load_board_config(board: &Board) -> String {
    // Try exact board name first (e.g. EVE.conf), then lowercase
    let candidates = [
        format!("{COBALT_KEYD_BOARDS_DIR}/{}.conf", board.name),
        format!("{COBALT_KEYD_BOARDS_DIR}/{}.conf", board.name.to_lowercase()),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            info!("Using board-specific keyd config: {path}");
            return content;
        }
    }

    // Fall back to generated config with per-board tweaks
    generate_board_config(board)
}

/// Generate a keyd config for boards without a dedicated config file.
fn generate_board_config(board: &Board) -> String {
    match board.name.as_str() {
        // Boards with a dedicated assistant/launcher key
        "HELIOS" | "VOXEL" | "VOLTA" | "DROBIT" => format!(
            "{BASE_CONFIG}\n# {}: assistant key → Super+S\nassistant = meta-s\n",
            board.name
        ),
        // Boards with extended top row (13 keys)
        "SAMUS" | "LINK" => format!(
            "{BASE_CONFIG}\n# {}: extended top row\nf11 = f11\nf12 = f12\n",
            board.name
        ),
        _ => BASE_CONFIG.to_string(),
    }
}

/// Attempt to reload keyd. Logs a warning if keyd is not installed or not running.
async fn reload_keyd() {
    let status = tokio::process::Command::new("systemctl")
        .args(["reload-or-restart", "keyd"])
        .status()
        .await;

    match status {
        Ok(s) if s.success() => info!("keyd reloaded successfully"),
        Ok(s) => warn!("keyd reload exited with status: {s}"),
        Err(e) => warn!("Could not reload keyd (is it installed?): {e}"),
    }
}
