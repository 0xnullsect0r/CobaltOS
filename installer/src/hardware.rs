//! Hardware detection for the installer.
//!
//! Checks for MrChromebox UEFI firmware and enumerates available disks.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub board_name: String,
    pub has_uefi_firmware: bool,
    pub disks: Vec<DiskInfo>,
    pub ram_mb: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub size_gb: f64,
    pub model: String,
    pub removable: bool,
}

pub async fn probe() -> Result<HardwareInfo> {
    let mut warnings = Vec::new();

    // Read board/firmware info from DMI
    let bios_version = read_file("/sys/class/dmi/id/bios_version");
    let board_name = parse_board_name(&bios_version);
    let has_uefi_firmware = bios_version.to_lowercase().contains("mrchromebox")
        || bios_version.to_lowercase().contains("coreboot");

    if !has_uefi_firmware {
        warnings.push(
            "MrChromebox UEFI firmware not detected. \
             Please flash UEFI firmware before installing. \
             See the installation guide for instructions.".to_string()
        );
    }

    // Read RAM from /proc/meminfo
    let ram_mb = read_ram_mb();
    if ram_mb < 2048 {
        warnings.push(format!("Low RAM detected ({ram_mb} MB). CobaltOS recommends at least 2 GB."));
    }

    // Enumerate block devices
    let disks = enumerate_disks().await;
    if disks.is_empty() {
        warn!("No suitable installation disks found");
        warnings.push("No installation disks found. If your NVMe drive is missing, run: sudo modprobe nvme".to_string());
    }

    Ok(HardwareInfo { board_name, has_uefi_firmware, disks, ram_mb, warnings })
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_board_name(bios_version: &str) -> String {
    if let Some(rest) = bios_version.strip_prefix("Google_") {
        return rest.split('.').next().unwrap_or("").to_uppercase();
    }
    String::new()
}

fn read_ram_mb() -> u64 {
    let meminfo = read_file("/proc/meminfo");
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            let kb: u64 = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            return kb / 1024;
        }
    }
    0
}

async fn enumerate_disks() -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    let sys_block = std::path::Path::new("/sys/block");

    let Ok(entries) = std::fs::read_dir(sys_block) else {
        return disks;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip loop, ram, sr, zram devices
        if name.starts_with("loop") || name.starts_with("ram")
            || name.starts_with("sr") || name.starts_with("zram") {
            continue;
        }

        let base = entry.path();
        let removable: bool = std::fs::read_to_string(base.join("removable"))
            .unwrap_or_default().trim() == "1";
        let size_bytes: u64 = std::fs::read_to_string(base.join("size"))
            .unwrap_or_default().trim().parse().unwrap_or(0);
        let size_gb = (size_bytes * 512) as f64 / 1_073_741_824.0;
        let model = std::fs::read_to_string(base.join("device/model"))
            .unwrap_or_default().trim().to_string();

        if size_gb < 8.0 { continue; }

        disks.push(DiskInfo {
            path: format!("/dev/{name}"),
            size_gb,
            model,
            removable,
        });
    }
    disks
}
