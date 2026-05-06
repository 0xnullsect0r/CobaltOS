//! DMI/ACPI board detection for Chromebook hardware identification.
//!
//! Reads `/sys/class/dmi/id/` entries to determine the board name
//! (e.g. "BOBBA", "EVE", "ZORK") which drives all downstream
//! hardware configuration decisions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const DMI_PATH: &str = "/sys/class/dmi/id";

/// Represents a detected Chromebook board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// ChromeOS board name in uppercase (e.g. "BOBBA", "EVE").
    pub name: String,
    /// Full product name from DMI (e.g. "Chromebook 311").
    pub product_name: String,
    /// Vendor/manufacturer string from DMI.
    pub vendor: String,
    /// BIOS version string — used to verify MrChromebox UEFI is present.
    pub bios_version: String,
}

impl Board {
    /// Returns `true` if the BIOS version indicates MrChromebox UEFI Full ROM.
    pub fn has_uefi_full_rom(&self) -> bool {
        let v = self.bios_version.to_lowercase();
        v.contains("mrchromebox") || v.contains("coreboot") || v.contains("uefi")
    }

    /// Returns `true` if this looks like a Chromebook (not a generic PC).
    pub fn is_chromebook(&self) -> bool {
        let product = self.product_name.to_lowercase();
        let vendor = self.vendor.to_lowercase();
        product.contains("chromebook")
            || product.contains("chromebox")
            || product.contains("chromebase")
            || vendor.contains("google")
            || !self.name.is_empty()
    }
}

/// Read a single DMI sysfs attribute, stripping whitespace.
fn read_dmi(attr: &str) -> Result<String> {
    let path = format!("{DMI_PATH}/{attr}");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read DMI attribute: {path}"))?;
    Ok(raw.trim().to_string())
}

/// Detect the Chromebook board from DMI/ACPI data.
///
/// Falls back to empty strings on individual read failures so that
/// partial detection still works on unusual setups.
pub async fn detect_board() -> Result<Board> {
    // The ChromeOS board name is stored in the BIOS version string or
    // board_name field, often formatted as "Google_BOARDNAME" or just
    // "BOARDNAME". We normalise to uppercase.
    let bios_version = read_dmi("bios_version").unwrap_or_default();
    let board_name_raw = read_dmi("board_name").unwrap_or_default();
    let product_name = read_dmi("product_name").unwrap_or_default();
    let vendor = read_dmi("sys_vendor").unwrap_or_default();

    debug!("DMI bios_version: {bios_version}");
    debug!("DMI board_name:   {board_name_raw}");
    debug!("DMI product_name: {product_name}");
    debug!("DMI vendor:       {vendor}");

    // Parse the board name. ChromeOS firmware encodes it as:
    //   "Google_BOARDNAME.version" or just "BOARDNAME"
    let name = parse_board_name(&bios_version, &board_name_raw);

    if name.is_empty() {
        warn!("Could not determine ChromeOS board name from DMI data");
    }

    Ok(Board {
        name,
        product_name,
        vendor,
        bios_version,
    })
}

/// Extract the board name from DMI strings.
///
/// ChromeOS boards appear in `bios_version` as e.g.:
///   "Google_Bobba.12672.430.0" → "BOBBA"
/// or in `board_name` directly as "BOBBA".
fn parse_board_name(bios_version: &str, board_name: &str) -> String {
    // Try bios_version first: "Google_Boardname.x.y.z"
    if let Some(after_google) = bios_version.strip_prefix("Google_") {
        let candidate: String = after_google
            .split('.')
            .next()
            .unwrap_or("")
            .to_uppercase();
        if !candidate.is_empty() {
            return candidate;
        }
    }

    // Fall back to board_name DMI field
    let candidate = board_name.trim().to_uppercase();
    if !candidate.is_empty() {
        return candidate;
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_google_bios_version() {
        assert_eq!(parse_board_name("Google_Bobba.12672.430.0", ""), "BOBBA");
        assert_eq!(parse_board_name("Google_Eve.9584.413.0", ""), "EVE");
        assert_eq!(parse_board_name("Google_Zork.13434.682.0", ""), "ZORK");
    }

    #[test]
    fn parse_board_name_fallback() {
        assert_eq!(parse_board_name("", "ATLAS"), "ATLAS");
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parse_board_name("", ""), "");
    }

    #[test]
    fn has_uefi_full_rom_detection() {
        let board = Board {
            name: "EVE".into(),
            product_name: "Pixelbook".into(),
            vendor: "Google".into(),
            bios_version: "MrChromebox-4.21.0".into(),
        };
        assert!(board.has_uefi_full_rom());
    }

    #[test]
    fn stock_firmware_not_detected_as_uefi() {
        let board = Board {
            name: "EVE".into(),
            product_name: "Pixelbook".into(),
            vendor: "Google".into(),
            bios_version: "Google_Eve.9584.413.0".into(),
        };
        assert!(!board.has_uefi_full_rom());
    }
}
