//! Touchpad configuration via libinput for Chromebook hardware.
//!
//! Chromebook touchpads generally support multi-finger gestures and
//! require tap-to-click and natural scroll to be enabled. Some boards
//! also need specific acceleration or palm-rejection settings.

use anyhow::Result;
use tracing::{info, warn};

use crate::dmi::Board;

const LIBINPUT_CONFIG_DIR: &str = "/etc/libinput";
const XORG_TOUCHPAD_DIR: &str = "/etc/X11/xorg.conf.d";
const COBALT_TOUCHPAD_BOARDS_DIR: &str = "/etc/cobaltos/libinput/boards";

/// Default libinput touchpad config applied to all Chromebook boards.
/// Written to /etc/libinput/local-overrides.quirks
const BASE_QUIRKS: &str = r#"# CobaltOS libinput touchpad quirks
# Applied by cobalt-hardware-probe on first boot.

[Chromebook Touchpad]
MatchUdevType=touchpad
ModelChromebook=1
AttrEventCode=-BTN_TOOL_MOUSE;-BTN_TOOL_LENS;
AttrEventCode=+BTN_TOOL_QUINTTAP;
"#;

/// Xorg InputClass config for touchpad — used as X11 fallback.
const XORG_TOUCHPAD_CONF: &str = r#"# CobaltOS Xorg touchpad configuration
# Written by cobalt-hardware-probe.
Section "InputClass"
    Identifier      "CobaltOS Chromebook Touchpad"
    MatchIsTouchpad "on"
    MatchDevicePath "/dev/input/event*"
    Driver          "libinput"
    Option          "Tapping"              "on"
    Option          "TappingDrag"          "on"
    Option          "TappingDragLock"      "off"
    Option          "NaturalScrolling"     "true"
    Option          "DisableWhileTyping"   "on"
    Option          "MiddleEmulation"      "on"
    Option          "AccelProfile"         "adaptive"
    Option          "AccelSpeed"           "0.3"
EndSection
"#;

/// Per-board touchpad sensitivity overrides.
struct BoardTouchpadConfig {
    accel_speed: f32,
    palm_detection: bool,
    clickfinger_count: u8,
}

impl BoardTouchpadConfig {
    fn for_board(board: &Board) -> Self {
        match board.name.as_str() {
            // Pixelbook / Pixelbook Go — large glass trackpad, very sensitive
            "EVE" | "ATLAS" => Self { accel_speed: 0.2, palm_detection: true, clickfinger_count: 2 },
            // Pixel Slate — tablet with detachable keyboard
            "NOCTURNE" => Self { accel_speed: 0.15, palm_detection: true, clickfinger_count: 2 },
            // HP/Lenovo 360/C13 — average-size touchpads
            "ELEMI" | "LILLIPUP" | "STARMIE" => Self { accel_speed: 0.3, palm_detection: true, clickfinger_count: 2 },
            // Older boards with smaller, less precise touchpads
            "WOLF" | "LEON" | "MCCLOUD" | "MONROE" => Self { accel_speed: 0.4, palm_detection: false, clickfinger_count: 1 },
            // Default for everything else
            _ => Self { accel_speed: 0.3, palm_detection: true, clickfinger_count: 2 },
        }
    }
}

/// Apply touchpad configuration for the detected board.
pub async fn apply_touchpad_config(board: &Board) -> Result<()> {
    info!("Applying touchpad configuration for board: {}", board.name);

    // Write libinput quirks
    std::fs::create_dir_all(LIBINPUT_CONFIG_DIR)?;
    std::fs::write(
        format!("{LIBINPUT_CONFIG_DIR}/local-overrides.quirks"),
        BASE_QUIRKS,
    )?;
    info!("libinput quirks written to {LIBINPUT_CONFIG_DIR}/local-overrides.quirks");

    // Write Xorg fallback config
    std::fs::create_dir_all(XORG_TOUCHPAD_DIR)?;
    std::fs::write(
        format!("{XORG_TOUCHPAD_DIR}/40-cobaltos-touchpad.conf"),
        XORG_TOUCHPAD_CONF,
    )?;

    // Check for a board-specific override file
    let board_config_path = format!(
        "{COBALT_TOUCHPAD_BOARDS_DIR}/{}.conf",
        board.name.to_lowercase()
    );
    if std::path::Path::new(&board_config_path).exists() {
        apply_board_quirks_file(&board_config_path, board).await?;
    } else {
        apply_generated_config(board).await?;
    }

    // Write COSMIC/Wayland compositor touchpad settings
    write_cosmic_touchpad_config(board).await?;

    info!("Touchpad configuration complete for board: {}", board.name);
    Ok(())
}

/// Apply a board-specific libinput quirks file.
async fn apply_board_quirks_file(path: &str, board: &Board) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let dest = format!("{LIBINPUT_CONFIG_DIR}/cobaltos-{}.quirks", board.name.to_lowercase());
    std::fs::write(&dest, content)?;
    info!("Board-specific touchpad quirks installed to {dest}");
    Ok(())
}

/// Generate and apply touchpad config for boards without a dedicated file.
async fn apply_generated_config(board: &Board) -> Result<()> {
    let cfg = BoardTouchpadConfig::for_board(board);

    let quirks = format!(
        r#"# CobaltOS generated touchpad quirks for {}
[{} Touchpad]
MatchUdevType=touchpad
ModelChromebook=1
AttrTouchSizeRange=20:10
AttrPalmSizeThreshold={}
"#,
        board.name,
        board.name,
        if cfg.palm_detection { "20" } else { "0" }
    );

    let dest = format!(
        "{LIBINPUT_CONFIG_DIR}/cobaltos-{}.quirks",
        board.name.to_lowercase()
    );
    std::fs::write(&dest, quirks)?;
    info!("Generated touchpad quirks for {} written to {dest}", board.name);
    Ok(())
}

/// Write COSMIC compositor touchpad settings to /etc/cobaltos/cosmic/touchpad.toml
async fn write_cosmic_touchpad_config(board: &Board) -> Result<()> {
    let cfg = BoardTouchpadConfig::for_board(board);

    let config_dir = "/etc/cobaltos/cosmic";
    std::fs::create_dir_all(config_dir)?;

    let content = format!(
        r#"# CobaltOS COSMIC touchpad settings (generated by cobalt-hardware-probe)
tap_to_click = true
tap_drag = true
natural_scroll = true
disable_while_typing = true
middle_emulation = true
accel_profile = "adaptive"
accel_speed = {:.2}
clickfinger_count = {}
palm_detection = {}
"#,
        cfg.accel_speed,
        cfg.clickfinger_count,
        cfg.palm_detection
    );

    std::fs::write(format!("{config_dir}/touchpad.toml"), content)?;
    info!("COSMIC touchpad config written");
    Ok(())
}

/// Exposed for `cobalt-hardware-probe --fix-touchpad`
pub async fn fix_touchpad() -> Result<()> {
    let board = crate::dmi::detect_board().await?;
    apply_touchpad_config(&board).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dmi::Board;

    fn make_board(name: &str) -> Board {
        Board {
            name: name.to_uppercase(),
            product_name: name.to_string(),
            vendor: String::new(),
            bios_version: String::new(),
        }
    }

    #[test]
    fn pixelbook_gets_low_accel() {
        let board = make_board("EVE");
        let cfg = BoardTouchpadConfig::for_board(&board);
        assert!(cfg.accel_speed < 0.25);
        assert!(cfg.palm_detection);
    }

    #[test]
    fn old_board_disables_palm() {
        let board = make_board("WOLF");
        let cfg = BoardTouchpadConfig::for_board(&board);
        assert!(!cfg.palm_detection);
    }

    #[test]
    fn unknown_board_gets_defaults() {
        let board = make_board("ZZUNKNOWN");
        let cfg = BoardTouchpadConfig::for_board(&board);
        assert!((cfg.accel_speed - 0.3).abs() < 0.01);
        assert!(cfg.palm_detection);
    }
}
