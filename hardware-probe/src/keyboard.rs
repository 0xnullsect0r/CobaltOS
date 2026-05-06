//! Chromebook keyboard remapping via keyd.
//!
//! Applies per-board keyboard config to remap the Search key to Super,
//! top-row function keys, and other Chromebook-specific keys.

use anyhow::Result;
use tracing::{info, warn};

use crate::dmi::Board;

const KEYD_CONFIG_DIR: &str = "/etc/keyd";
const KEYD_CONFIG_FILE: &str = "/etc/keyd/chromebook.conf";

/// Base keyd config applied to all Chromebook boards.
const BASE_CONFIG: &str = r#"
[ids]
*

[main]
# Remap Search/Capslock to Super (Meta/Windows key)
capslock = leftmeta
search = leftmeta

# Top-row function keys — map to F1-F10 when Fn is held,
# or to their ChromeOS actions by default via the modifier key.
# Individual boards may override these.
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
# When Search (now Meta) is held, restore F-key behavior
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

    // Ensure keyd config directory exists
    std::fs::create_dir_all(KEYD_CONFIG_DIR)?;

    // Start with the base config and apply any board-specific overrides
    let config = board_config(board);
    std::fs::write(KEYD_CONFIG_FILE, config)?;

    // Reload keyd if it's running
    reload_keyd().await;

    info!("Keyboard remapping applied: {KEYD_CONFIG_FILE}");
    Ok(())
}

/// Generate the keyd config for this specific board.
/// Most boards use the base config. Add overrides here as needed.
fn board_config(board: &Board) -> String {
    match board.name.as_str() {
        // Boards with an extra dedicated assistant/launcher key
        "HELIOS" | "VOXEL" | "VOLTA" => format!(
            "{BASE_CONFIG}\n# Board {}: assistant key mapped to Super+S\nassistant = meta-s\n",
            board.name
        ),
        // Boards with a different top-row layout (13 keys vs 10)
        "SAMUS" | "LINK" => format!(
            "{BASE_CONFIG}\n# Board {}: extended top row\nf11 = f11\nf12 = f12\n",
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
