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

    /// Returns a human-friendly device name, falling back to the DMI product name.
    pub fn friendly_name(&self) -> &str {
        match board_friendly_name(&self.name) {
            "" => &self.product_name,
            s  => s,
        }
    }

    /// Returns the platform/generation string for this board (e.g. "Kaby Lake").
    pub fn platform(&self) -> &str {
        board_platform(&self.name)
    }
}

/// Look up a friendly device name by board code.
fn board_friendly_name(board: &str) -> &'static str {
    match board {
        // ── Google-branded ──────────────────────────────────────────────────
        "LINK"     => "Chromebook Pixel (2013)",
        "SAMUS"    => "Chromebook Pixel (2015)",
        "EVE"      => "Google Pixelbook",
        "ATLAS"    => "Google Pixelbook Go",
        "NOCTURNE" => "Google Pixel Slate",
        "DRALLION" => "Google Pixelbook Go (2019)",

        // ── Acer ────────────────────────────────────────────────────────────
        "PEPPY"    => "Acer Chromebook C720",
        "PAINE"    => "Acer Chromebook 11 C740",
        "YUNA"     => "Acer Chromebook 15 CB5-571",
        "BANON"    => "Acer Chromebook R11",
        "FIZZ"     => "Acer Chromebox CXI3",
        "AKALI"    => "Acer Chromebook 13 (CB713-1W)",
        "DRATINI"  => "Acer Chromebook 311",
        "KOHAKU"   => "Acer Chromebook Spin 713 (2020)",
        "LILLIPUP" => "Acer Chromebook 314",
        "CEZANNE"  => "Acer Chromebook Spin 514",
        "GARG"     => "Acer Chromebook 715",

        // ── ASUS ────────────────────────────────────────────────────────────
        "GANDOF"   => "ASUS Chromebook C300",
        "GNAWTY"   => "ASUS Chromebook C201",
        "GUADO"    => "ASUS Chromebox CN60",
        "HANA"     => "ASUS Chromebook C423",
        "DOOD"     => "ASUS Chromebook CX9 (CX9400)",

        // ── Dell ────────────────────────────────────────────────────────────
        "CANDY"    => "Dell Chromebook 11 (3120)",
        "SWANKY"   => "Dell Chromebook 11 (3180)",
        "VAYNE"    => "Dell Chromebook 13 (7310)",
        "ARCADA"   => "Dell Inspiron Chromebook 14 (7460)",
        "SARIEN"   => "Dell Latitude 5300 2-in-1 Chromebook",

        // ── HP ──────────────────────────────────────────────────────────────
        "FALCO"    => "HP Chromebook 14",
        "TIDUS"    => "HP Chromebook 14 G4",
        "CAVE"     => "HP Chromebook 13 G1",
        "OCTOPUS"  => "HP Chromebook x360 11 G2",
        "MEEP"     => "HP Chromebook x360 11 G3",
        "KEFKA"    => "HP Chromebook 14A G5",
        "CELES"    => "HP Chromebook x360 14 G1",
        "DRAGONAIR"=> "HP Pro c645 Chromebook",
        "BERKNIP"  => "HP Pro c640 Chromebook",
        "JINLON"   => "HP Elite c1030 Chromebook",

        // ── Lenovo ──────────────────────────────────────────────────────────
        "GLIMMER"  => "Lenovo ThinkPad X131e",
        "LARS"     => "Lenovo ThinkPad 11e/Yoga 11e (Gen 4)",
        "TERRA"    => "Lenovo Chromebook S330",
        "CAREENA"  => "Lenovo IdeaPad Flex 3",
        "MORPHIUS"  => "Lenovo IdeaPad Flex 5i Chromebook",
        "BOTEN"    => "Lenovo IdeaPad Gaming Chromebook 16",

        // ── Samsung ─────────────────────────────────────────────────────────
        "LUMPY"    => "Samsung Chromebook (Series 5 550)",
        "STUMPY"   => "Samsung Chromebox (Series 3)",
        "SNOW"     => "Samsung Chromebook 2 11",
        "NYAN_BIG" => "NVIDIA Tegra K1 Chromebook",

        // ── Toshiba ─────────────────────────────────────────────────────────
        "LEON"     => "Toshiba Chromebook 2",
        "TERRA"    => "Toshiba Chromebook 2 (2015)",

        // ── Common AMD platforms ─────────────────────────────────────────────
        "ZORK"     => "Various AMD Ryzen 3000 Chromebooks",
        "GUYBRUSH"  => "Various AMD Ryzen 5000 Chromebooks",
        "SKYRIM"   => "Various AMD Ryzen 6000 Chromebooks",

        _ => "",
    }
}

/// Look up the SoC platform for a board code.
pub fn board_platform(board: &str) -> &'static str {
    match board {
        // Intel Sandy Bridge
        "LUMPY" | "STUMPY" | "ALEX" | "ZGB" => "Intel Sandy Bridge",
        // Intel Ivy Bridge
        "PARROT" | "STOUT" | "BUTTERFLY" => "Intel Ivy Bridge",
        // Intel Haswell
        "FALCO" | "PEPPY" | "LINK" | "WOLF" | "LEON" | "GANDOF" => "Intel Haswell",
        // Intel Broadwell
        "SAMUS" | "PAINE" | "YUNA" | "GUADO" | "RIKKU" | "TIDUS" | "AURON" => "Intel Broadwell",
        // Intel Bay Trail
        "GNAWTY" | "SWANKY" | "CANDY" | "BANJO" | "CLAPPER" | "ENGUARDE" | "GLIMMER"
        | "KIP" | "QUAWKS" | "ORCO" | "SQUAWKS" | "SUMO" | "WINKY" => "Intel Bay Trail",
        // Intel Braswell
        "BANON" | "CELES" | "CYAN" | "EDGAR" | "KEFKA" | "REKS" | "RELM" | "SETZER"
        | "STRAGO" | "TERRA" => "Intel Braswell",
        // Intel Apollo Lake
        "CORAL" | "ROBO360" | "NASHER" | "BLUE" | "BRUCE" | "CAVE" | "ASTRONAUT"
        | "BABYMEGA" | "BABYTIGER" | "BLACKTIP" | "BLACKTIP360" | "ELECTRO"
        | "EPAULETTE" | "LAVA" | "NASHER360" | "RABBID" | "SAND" | "SANTA"
        | "WHITETIP" | "ROBO" => "Intel Apollo Lake",
        // Intel Gemini Lake
        "OCTOPUS" | "MEEP" | "FLEEX" | "DROID" | "AMPTON" | "SPARKY" | "SPARKY360"
        | "BOBBA" | "BOBBA360" | "BLUEBIRD" | "CASTA" | "DORP" | "GARFOUR"
        | "LASER14" | "LICK" | "MIMROCK" | "NOSPIKE" | "ORBATRIX" | "PHASER"
        | "PHASER360" | "RIPTO" | "SUMO" | "VORTICON" => "Intel Gemini Lake",
        // Intel Kaby Lake
        "EVE" | "LARS" | "SORAKA" | "SNAPPY" | "FIZZ" | "KARMA" | "KENCH" | "TEEMO"
        | "SENTRY" | "REEF" | "SAND" | "PYRO" | "ELECTRO" | "ROBO360" => "Intel Kaby Lake",
        // Intel Coffee Lake / Whiskey Lake / Comet Lake (Nami/Nautilus/Rammus)
        "VAYNE" | "AKALI" | "BARD" | "EKKO" | "SONA" | "SYNDRA" | "PANTHEON"
        | "NAMI" | "NAUTILUS" | "RAMMUS" | "ARCADA" | "SARIEN" | "HELIOS" | "VOXEL"
        | "VOLTA" | "JINLON" => "Intel Coffee Lake / Whiskey Lake",
        // Intel Ice Lake
        "DRATINI" | "DRAGONAIR" | "DITTO" => "Intel Ice Lake",
        // Intel Tiger Lake
        "VOLTEER" | "DELBIN" | "DROBIT" | "ELDRID" | "ELEMI" | "LILLIPUP" | "MALEFOR"
        | "NIGHTFURY" | "TERRADOR" | "TRONDO" | "VOXEL" | "WADDLEDOO" => "Intel Tiger Lake",
        // Intel Alder Lake
        "BRYA" | "BANSHEE" | "CONSTITUTION" | "CROTA" | "FELWINTER" | "GIMBLE"
        | "KANO" | "OSIRIS" | "PRIMUS" | "TANIKS" | "TAEKO" | "TARANZA"
        | "ZAVALA" => "Intel Alder Lake",
        // Intel Jasper Lake
        "DEDEDE" | "BLIPPER" | "BOOKEM" | "CRET" | "DRAWPER" | "GOOEY" | "GUEY"
        | "KRACKO" | "KRACKO360" | "MAGISTER" | "METAKNIGHT" | "SASUKE" | "STORO"
        | "WALREIN" => "Intel Jasper Lake",
        // AMD Stoney Ridge
        "GRUNT" | "BARLA" | "CAREENA" | "KASUMI" | "KASUMI360" | "LIARA" | "TREEYA"
        | "TREEYA360" => "AMD Stoney Ridge",
        // AMD Picasso / Dali
        "ZORK" | "BERKNIP" | "BERKNIP360" | "DIRINBOZ" | "DALBOZ" | "EZKINIL"
        | "VILBOZ" | "MORPHIUS" | "WOOMAX" | "GUMBOZ" => "AMD Picasso / Dali",
        // AMD Cezanne / Barcelo
        "GUYBRUSH" | "DEWATT" | "NIPPERKIN" | "STARMIE" | "STARYU" | "VILBOZ360" => "AMD Cezanne / Barcelo",
        // AMD Mendocino
        "SKYRIM" | "FROSTFLOW" | "CRYSTALDRIFT" | "MARKARTH" | "WHITERUN"
        | "ZENITH" => "AMD Mendocino",
        // AMD Phoenix
        "MYST" | "ULDREN" | "OMNIGUL" | "TANIKS360" => "AMD Phoenix",
        _ => "Unknown Platform",
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

    #[test]
    fn friendly_name_known_board() {
        let board = Board {
            name: "EVE".into(),
            product_name: "Generic".into(),
            vendor: "Google".into(),
            bios_version: "".into(),
        };
        assert_eq!(board.friendly_name(), "Google Pixelbook");
    }

    #[test]
    fn friendly_name_unknown_falls_back_to_product() {
        let board = Board {
            name: "UNKNOWNBOARD".into(),
            product_name: "My Chromebook".into(),
            vendor: "Google".into(),
            bios_version: "".into(),
        };
        assert_eq!(board.friendly_name(), "My Chromebook");
    }

    #[test]
    fn platform_known_board() {
        assert_eq!(board_platform("EVE"), "Intel Kaby Lake");
        assert_eq!(board_platform("ZORK"), "AMD Picasso / Dali");
        assert_eq!(board_platform("SAMUS"), "Intel Broadwell");
    }
}
