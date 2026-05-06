//! Partition management for the installer.
//! All functions require root. Called during actual installation (v1.0.0 flow).
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use std::process::Command;
use tracing::info;

/// Wipe, partition, and format `disk` for a CobaltOS installation.
/// Requires root. `disk` is a block device path like `/dev/sda` or `/dev/mmcblk0`.
pub async fn partition_disk(disk: &str) -> Result<()> {
    info!("Partitioning disk: {disk}");

    if !std::path::Path::new(disk).exists() {
        bail!("Disk {disk} not found");
    }

    run_cmd("sgdisk", &["-Z", disk])?;
    run_cmd("sgdisk", &["-n", "1:0:+512M", "-t", "1:ef00", "-c", "1:EFI", disk])?;
    run_cmd("sgdisk", &["-n", "2:0:0", "-t", "2:8300", "-c", "2:root", disk])?;
    run_cmd("partprobe", &[disk])?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let suffix = if disk.contains("nvme") || disk.contains("mmcblk") { "p" } else { "" };
    let esp = format!("{disk}{suffix}1");
    let root = format!("{disk}{suffix}2");

    run_cmd("mkfs.fat", &["-F32", "-n", "EFI", &esp])?;
    run_cmd("mkfs.ext4", &["-L", "CobaltOS", "-F", &root])?;

    info!("Partitioning complete: ESP={esp}, root={root}");
    Ok(())
}

/// Mount root and ESP into `mountpoint` for installation.
pub fn mount_partitions(disk: &str, mountpoint: &str) -> Result<()> {
    let suffix = if disk.contains("nvme") || disk.contains("mmcblk") { "p" } else { "" };
    let esp = format!("{disk}{suffix}1");
    let root = format!("{disk}{suffix}2");

    std::fs::create_dir_all(mountpoint)?;
    run_cmd("mount", &[&root, mountpoint])?;

    let esp_dir = format!("{mountpoint}/boot/efi");
    std::fs::create_dir_all(&esp_dir)?;
    run_cmd("mount", &[&esp, &esp_dir])?;

    Ok(())
}

/// Unmount everything under `mountpoint`.
pub fn unmount_partitions(mountpoint: &str) -> Result<()> {
    run_cmd("umount", &["-R", mountpoint])?;
    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run {program}"))?;
    if !status.success() {
        bail!("{program} exited with status {status}");
    }
    Ok(())
}
