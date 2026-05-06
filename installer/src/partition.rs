//! Partition management for the installer.
//! All functions require root. Called during actual installation (v1.0.0 flow).
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use std::process::Command;
use tracing::info;

use crate::installer::Filesystem;

/// Wipe, partition, and format `disk` for a CobaltOS installation.
/// Requires root. `disk` is a block device path like `/dev/sda` or `/dev/mmcblk0`.
/// The root partition is formatted as `filesystem` (ext4 or btrfs with zstd).
pub async fn partition_disk(disk: &str, filesystem: &Filesystem) -> Result<()> {
    info!("Partitioning disk: {disk} (filesystem: {filesystem:?})");

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

    match filesystem {
        Filesystem::Ext4 => {
            run_cmd("mkfs.ext4", &["-L", "CobaltOS", "-F", &root])?;
        }
        Filesystem::Btrfs => {
            run_cmd("mkfs.btrfs", &["-L", "CobaltOS", "-f", &root])?;
            // Mount temporarily to create subvolumes
            run_cmd("mount", &[&root, "/mnt"])?;
            run_cmd("btrfs", &["subvolume", "create", "/mnt/@"])?;
            run_cmd("btrfs", &["subvolume", "create", "/mnt/@home"])?;
            run_cmd("btrfs", &["subvolume", "create", "/mnt/@snapshots"])?;
            run_cmd("umount", &["/mnt"])?;
            info!("btrfs subvolumes created: @, @home, @snapshots");
        }
    }

    info!("Partitioning complete: ESP={esp}, root={root}");
    Ok(())
}

/// Mount root and ESP into `mountpoint` for installation.
pub fn mount_partitions(disk: &str, mountpoint: &str) -> Result<()> {
    let suffix = if disk.contains("nvme") || disk.contains("mmcblk") { "p" } else { "" };
    let esp = format!("{disk}{suffix}1");
    let root = format!("{disk}{suffix}2");

    std::fs::create_dir_all(mountpoint)?;

    // Detect whether the root partition is btrfs and use subvolumes if so
    let fstype = detect_fstype(&root).unwrap_or_default();
    if fstype == "btrfs" {
        run_cmd(
            "mount",
            &["-o", "subvol=@,compress=zstd:3,noatime", &root, mountpoint],
        )?;
        let home_dir = format!("{mountpoint}/home");
        std::fs::create_dir_all(&home_dir)?;
        run_cmd(
            "mount",
            &["-o", "subvol=@home,compress=zstd:3,noatime", &root, &home_dir],
        )?;
    } else {
        run_cmd("mount", &[&root, mountpoint])?;
    }

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

fn detect_fstype(dev: &str) -> Option<String> {
    let out = Command::new("blkid")
        .args(["-s", "TYPE", "-o", "value", dev])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
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
