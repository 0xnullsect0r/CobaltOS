//! Safe rollback support for cobalt-update.
//!
//! Before each upgrade:
//!   1. Captures dpkg package-selection state to a file.
//!   2. If the root filesystem is btrfs, creates a read-only snapshot.
//!
//! If an upgrade fails, `rollback()` restores the dpkg state and, on btrfs,
//! prints the snapshot path so the user can boot from it via systemd-boot.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, warn};

const STATE_DIR: &str = "/var/lib/cobalt-update";
const DPKG_STATE_FILE: &str = "/var/lib/cobalt-update/pre-upgrade-dpkg-state.txt";
const SNAPSHOT_BASE: &str = "/.snapshots";

// ── Snapshot creation ────────────────────────────────────────────────────────

/// Create a pre-upgrade safety snapshot.
///
/// Always saves a dpkg selection dump. Additionally creates a btrfs snapshot
/// when the root filesystem is detected as btrfs.
pub async fn create_snapshot() -> Result<()> {
    std::fs::create_dir_all(STATE_DIR)
        .with_context(|| format!("create {STATE_DIR}"))?;

    save_dpkg_state().await?;

    if is_btrfs_root().await {
        match create_btrfs_snapshot().await {
            Ok(path) => info!("btrfs snapshot created at {path}"),
            Err(e) => warn!("btrfs snapshot skipped: {e:#}"),
        }
    }

    Ok(())
}

/// Restore system state from the most recent pre-upgrade snapshot.
///
/// Restores dpkg selections and attempts `apt-get dselect-upgrade` to
/// downgrade/remove packages added during the failed upgrade.
pub async fn rollback() -> Result<()> {
    info!("Starting cobalt-update rollback");

    let state_file = PathBuf::from(DPKG_STATE_FILE);
    if !state_file.exists() {
        anyhow::bail!("No rollback snapshot found at {DPKG_STATE_FILE}. Nothing to restore.");
    }

    restore_dpkg_state(&state_file).await?;

    // Hint about btrfs snapshots if any exist
    if Path::new(SNAPSHOT_BASE).exists() {
        let entries = std::fs::read_dir(SNAPSHOT_BASE)?;
        let snapshots: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("pre-update-")
            })
            .collect();
        if !snapshots.is_empty() {
            info!(
                "btrfs snapshots available in {SNAPSHOT_BASE} — \
                 you can boot from one via systemd-boot if needed."
            );
        }
    }

    info!("Rollback complete.");
    Ok(())
}

// ── dpkg state helpers ───────────────────────────────────────────────────────

async fn save_dpkg_state() -> Result<()> {
    info!("Saving dpkg package state to {DPKG_STATE_FILE}");

    let out = Command::new("dpkg")
        .arg("--get-selections")
        .output()
        .await
        .context("dpkg --get-selections")?;

    if !out.status.success() {
        anyhow::bail!("dpkg --get-selections failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    std::fs::write(DPKG_STATE_FILE, &out.stdout)
        .context("write dpkg state file")?;

    let pkg_count = String::from_utf8_lossy(&out.stdout).lines().count();
    info!("{pkg_count} packages captured in dpkg snapshot");
    Ok(())
}

async fn restore_dpkg_state(state_file: &Path) -> Result<()> {
    info!("Restoring dpkg selections from {}", state_file.display());

    // Feed the saved selections back to dpkg
    let selections = std::fs::read(state_file).context("read dpkg state file")?;

    let mut child = Command::new("dpkg")
        .arg("--set-selections")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("dpkg --set-selections")?;

    if let Some(stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let mut stdin = tokio::io::BufWriter::new(stdin);
        stdin.write_all(&selections).await?;
        stdin.flush().await?;
    }
    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("dpkg --set-selections failed");
    }

    // Run dselect-upgrade to enforce the restored state
    info!("Running apt-get dselect-upgrade to apply restored state");
    let status = Command::new("apt-get")
        .args(["-y", "--fix-broken", "dselect-upgrade"])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status()
        .await
        .context("apt-get dselect-upgrade")?;

    if !status.success() {
        warn!("dselect-upgrade did not exit cleanly — some packages may need manual attention");
    }

    info!("dpkg state restored.");
    Ok(())
}

// ── btrfs helpers ────────────────────────────────────────────────────────────

/// Returns true if the root filesystem is btrfs.
async fn is_btrfs_root() -> bool {
    let out = Command::new("findmnt")
        .args(["--noheadings", "--output", "FSTYPE", "/"])
        .output()
        .await;

    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "btrfs",
        Err(_) => false,
    }
}

/// Create a read-only btrfs snapshot of the @ (root) subvolume.
async fn create_btrfs_snapshot() -> Result<String> {
    std::fs::create_dir_all(SNAPSHOT_BASE)
        .with_context(|| format!("create {SNAPSHOT_BASE}"))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let snapshot_path = format!("{SNAPSHOT_BASE}/pre-update-{ts}");

    let status = Command::new("btrfs")
        .args(["subvolume", "snapshot", "-r", "/", &snapshot_path])
        .status()
        .await
        .context("btrfs subvolume snapshot")?;

    if !status.success() {
        anyhow::bail!("btrfs snapshot command failed");
    }

    // Limit to 3 pre-upgrade snapshots to avoid filling the disk
    prune_old_snapshots(3).await;

    Ok(snapshot_path)
}

/// Delete oldest pre-upgrade snapshots, keeping only `keep` most recent.
async fn prune_old_snapshots(keep: usize) {
    let base = Path::new(SNAPSHOT_BASE);
    if let Ok(entries) = std::fs::read_dir(base) {
        let mut snapshots: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("pre-update-")
            })
            .map(|e| e.path())
            .collect();

        // Sort by name (timestamp suffix) ascending → oldest first
        snapshots.sort();

        if snapshots.len() > keep {
            for old in &snapshots[..snapshots.len() - keep] {
                info!("Pruning old snapshot: {}", old.display());
                let _ = Command::new("btrfs")
                    .args(["subvolume", "delete", old.to_str().unwrap_or("")])
                    .status()
                    .await;
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_base_path_is_correct() {
        assert_eq!(SNAPSHOT_BASE, "/.snapshots");
    }

    #[test]
    fn dpkg_state_file_is_in_state_dir() {
        assert!(DPKG_STATE_FILE.starts_with(STATE_DIR));
    }

    #[tokio::test]
    async fn is_btrfs_root_returns_bool() {
        // Just check it doesn't panic; result depends on the host system
        let _ = is_btrfs_root().await;
    }
}
