//! Partition management for the installer.

use anyhow::Result;
use tracing::info;

/// Partition and format a disk for CobaltOS.
///
/// Layout:
///   - Partition 1: 512 MB, FAT32, EFI System Partition (ESP)
///   - Partition 2: Remaining space, ext4, root filesystem
pub async fn partition_disk(disk: &str) -> Result<()> {
    info!("Partitioning disk: {disk}");

    // In a real implementation this drives sgdisk/parted:
    //   sgdisk -Z {disk}
    //   sgdisk -n 1:0:+512M -t 1:ef00 -c 1:"EFI" {disk}
    //   sgdisk -n 2:0:0     -t 2:8300 -c 2:"root" {disk}
    //   mkfs.fat -F32 {disk}1
    //   mkfs.ext4 -L "CobaltOS" {disk}2

    tracing::warn!("partition_disk is a stub — no actual partitioning performed");
    Ok(())
}
