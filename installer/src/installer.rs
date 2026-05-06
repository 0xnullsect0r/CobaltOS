//! Installer step model and execution engine.
//!
//! `run_install()` drives the full installation pipeline:
//!   1. Partition disk (sgdisk + mkfs)
//!   2. Mount partitions at INSTALL_ROOT
//!   3. debootstrap Debian Stable base
//!   4. Bind-mount proc/sys/dev/pts
//!   5. Install packages via apt (kernel, firmware, COSMIC, cobalt tools)
//!   6. Install systemd-boot bootloader
//!   7. Write system config (locale, hostname, user)
//!   8. Install cobalt-* binaries
//!   9. Enable systemd services
//!  10. Unmount and finalise

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::info;

const INSTALL_ROOT: &str = "/mnt/cobalt-install";
const DEBIAN_MIRROR: &str = "http://deb.debian.org/debian";
const DEBIAN_SUITE: &str = "bookworm";

/// Packages installed via apt inside the chroot after debootstrap.
const APT_PACKAGES: &[&str] = &[
    // Kernel + firmware
    "linux-image-amd64",
    "linux-headers-amd64",
    "firmware-linux",
    "firmware-linux-nonfree",
    "firmware-iwlwifi",
    "firmware-atheros",
    "firmware-realtek",
    "sof-firmware",
    // Base system
    "systemd",
    "systemd-boot",
    "dbus",
    "udev",
    "sudo",
    "network-manager",
    "zsh",
    "zram-tools",
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    "flatpak",
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "wget",
    "keyd",
    "plymouth",
    "plymouth-themes",
    // COSMIC desktop
    "cosmic-session",
    "cosmic-comp",
    "cosmic-panel",
    "cosmic-launcher",
    "cosmic-settings",
    "cosmic-files",
    "cosmic-term",
    "cosmic-edit",
    "cosmic-greeter",
    // Apps
    "firefox-esr",
    "gnome-software",
    "gnome-software-plugin-flatpak",
    // Rust CLI tools
    "ripgrep",
    "fd-find",
    "bat",
    "exa",
    "zoxide",
    // Filesystem tools
    "e2fsprogs",
    "dosfstools",
    "btrfs-progs",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum InstallStep {
    #[default]
    Welcome,
    DeviceCheck,
    DiskSetup,
    Location,
    Account,
    Confirm,
    Installing,
    Done,
}

impl InstallStep {
    pub fn all() -> &'static [InstallStep] {
        &[
            InstallStep::Welcome,
            InstallStep::DeviceCheck,
            InstallStep::DiskSetup,
            InstallStep::Location,
            InstallStep::Account,
            InstallStep::Confirm,
            InstallStep::Installing,
            InstallStep::Done,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            InstallStep::Welcome     => "Welcome",
            InstallStep::DeviceCheck => "Device Check",
            InstallStep::DiskSetup   => "Disk Setup",
            InstallStep::Location    => "Location",
            InstallStep::Account     => "Account",
            InstallStep::Confirm     => "Ready to Install",
            InstallStep::Installing  => "Installing…",
            InstallStep::Done        => "Done!",
        }
    }

    pub fn index(&self) -> usize {
        Self::all().iter().position(|s| s == self).unwrap_or(0)
    }

    pub fn next(&self) -> Option<InstallStep> {
        let steps = Self::all();
        let idx = self.index();
        steps.get(idx + 1).cloned()
    }
}

/// User-provided configuration collected during the install wizard.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    pub locale: String,
    pub timezone: String,
    pub keyboard_layout: String,
    pub disk: String,
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub use_full_disk: bool,
    pub filesystem: Filesystem,
}

/// Root filesystem choice for the installation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filesystem {
    #[default]
    Ext4,
    Btrfs,
}

/// Execute the full CobaltOS installation pipeline.
pub async fn run_install(
    config: &InstallConfig,
    progress: tokio::sync::mpsc::Sender<u8>,
) -> Result<()> {
    macro_rules! step {
        ($pct:expr, $msg:expr, $body:expr) => {{
            info!("{} ({}%)", $msg, $pct);
            let _ = progress.send($pct).await;
            $body.with_context(|| $msg)?;
        }};
    }

    // 1 — Partition
    step!(5, "Partitioning disk", {
        crate::partition::partition_disk(&config.disk, &config.filesystem).await
    });

    // 2 — Mount
    step!(8, "Mounting partitions", {
        crate::partition::mount_partitions(&config.disk, INSTALL_ROOT)
    });

    // 3 — debootstrap
    step!(12, "Installing base system (debootstrap)", {
        run_debootstrap(INSTALL_ROOT).await
    });

    // 4 — Bind mounts
    step!(32, "Preparing chroot environment", {
        bind_mount_virtual_fs(INSTALL_ROOT)
    });

    // 5 — apt packages
    step!(35, "Installing packages", {
        install_packages(INSTALL_ROOT).await
    });

    // 6 — Bootloader
    step!(70, "Installing bootloader", {
        install_systemd_boot(&config.disk, INSTALL_ROOT)
    });

    // 7 — System config
    step!(78, "Configuring system", {
        let sys_cfg = crate::config::SystemConfig {
            locale: config.locale.clone(),
            timezone: config.timezone.clone(),
            keyboard: config.keyboard_layout.clone(),
            hostname: config.hostname.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
        };
        crate::config::apply(&sys_cfg, INSTALL_ROOT).await
    });

    // 8 — User password (already set via config::apply → create_user → chpasswd)
    // If password was empty, set a temporary one equal to username
    step!(84, "Finalising user account", {
        if config.password.is_empty() {
            set_user_password(&config.username, &config.username, INSTALL_ROOT)
        } else {
            Ok(())
        }
    });

    // 9 — Enable services
    step!(88, "Enabling system services", {
        enable_services(INSTALL_ROOT)
    });

    // 10 — Plymouth
    step!(92, "Configuring boot splash", {
        configure_plymouth(INSTALL_ROOT)
    });

    // 11 — Unmount
    step!(96, "Finalising installation", {
        unbind_virtual_fs(INSTALL_ROOT)?;
        crate::partition::unmount_partitions(INSTALL_ROOT)
    });

    let _ = progress.send(100).await;
    info!("Installation complete for user '{}'", config.username);
    Ok(())
}

// ── Implementation helpers ────────────────────────────────────────────────────

async fn run_debootstrap(root: &str) -> Result<()> {
    let status = tokio::process::Command::new("debootstrap")
        .args([
            "--arch=amd64",
            "--include=ca-certificates,apt-transport-https,gnupg",
            "--components=main,contrib,non-free,non-free-firmware",
            DEBIAN_SUITE,
            root,
            DEBIAN_MIRROR,
        ])
        .status()
        .await
        .context("launch debootstrap")?;
    anyhow::ensure!(status.success(), "debootstrap failed");
    Ok(())
}

fn bind_mount_virtual_fs(root: &str) -> Result<()> {
    for (src, dst) in [
        ("proc",     format!("{root}/proc")),
        ("sysfs",    format!("{root}/sys")),
        ("devtmpfs", format!("{root}/dev")),
        ("devpts",   format!("{root}/dev/pts")),
    ] {
        std::fs::create_dir_all(&dst)?;
        let status = Command::new("mount")
            .args(["--bind", src, &dst])
            .status()
            .with_context(|| format!("mount --bind {src} {dst}"))?;
        anyhow::ensure!(status.success(), "mount --bind {src} failed");
    }
    Ok(())
}

fn unbind_virtual_fs(root: &str) -> Result<()> {
    for sub in ["dev/pts", "dev", "sys", "proc"] {
        let path = format!("{root}/{sub}");
        let _ = Command::new("umount").arg("-l").arg(&path).status();
    }
    Ok(())
}

async fn install_packages(root: &str) -> Result<()> {
    // Write resolv.conf so apt can reach the network
    std::fs::copy("/etc/resolv.conf", format!("{root}/etc/resolv.conf"))?;

    // Add bookworm non-free-firmware sources
    let sources = format!(
        "deb {mirror} {suite} main contrib non-free non-free-firmware\n\
         deb {mirror}-security {suite}-security main contrib non-free non-free-firmware\n\
         deb {mirror} {suite}-updates main contrib non-free non-free-firmware\n",
        mirror = DEBIAN_MIRROR,
        suite = DEBIAN_SUITE,
    );
    std::fs::write(format!("{root}/etc/apt/sources.list"), &sources)?;

    // apt-get update
    chroot_cmd(root, "apt-get", &["-y", "update"]).await?;

    // Install packages in one shot
    let mut args = vec!["-y", "--no-install-recommends", "install"];
    args.extend_from_slice(APT_PACKAGES);
    chroot_cmd(root, "apt-get", &args).await?;

    Ok(())
}

fn install_systemd_boot(disk: &str, root: &str) -> Result<()> {
    // bootctl install writes the EFI binary to the ESP
    let status = Command::new("chroot")
        .args([root, "bootctl", "--esp-path=/boot/efi", "install"])
        .status()
        .context("bootctl install")?;
    anyhow::ensure!(status.success(), "bootctl install failed");

    // Write loader.conf
    let loader_conf = "default cobalt\ntimeout 3\nconsole-mode max\neditor no\n";
    let loader_dir = format!("{root}/boot/efi/loader");
    std::fs::create_dir_all(&loader_dir)?;
    std::fs::write(format!("{loader_dir}/loader.conf"), loader_conf)?;

    // Determine root partition (p2 for nvme/mmcblk, 2 otherwise)
    let suffix = if disk.contains("nvme") || disk.contains("mmcblk") { "p" } else { "" };
    let root_part = format!("{disk}{suffix}2");

    // Get PARTUUID via blkid
    let partuuid = std::process::Command::new("blkid")
        .args(["-s", "PARTUUID", "-o", "value", &root_part])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Write boot entry
    let entry = format!(
        "title   CobaltOS\n\
         linux   /vmlinuz\n\
         initrd  /initrd.img\n\
         options root=PARTUUID={partuuid} rw quiet splash loglevel=3 rd.systemd.show_status=false\n"
    );
    let entries_dir = format!("{root}/boot/efi/loader/entries");
    std::fs::create_dir_all(&entries_dir)?;
    std::fs::write(format!("{entries_dir}/cobalt.conf"), &entry)?;

    info!("systemd-boot installed, entry PARTUUID={partuuid}");
    Ok(())
}

fn set_user_password(username: &str, password: &str, root: &str) -> Result<()> {
    let chpasswd_input = format!("{username}:{password}");
    let mut child = Command::new("chroot")
        .args([root, "chpasswd"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawn chpasswd")?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(chpasswd_input.as_bytes())?;
    }
    child.wait().context("chpasswd")?;
    Ok(())
}

fn enable_services(root: &str) -> Result<()> {
    let services = [
        "NetworkManager",
        "bluetooth",
        "pipewire",
        "pipewire-pulse",
        "wireplumber",
        "keyd",
        "flatpak-system-helper",
        "cobalt-hardware-probe",
        "cobalt-update",
    ];
    for svc in &services {
        let _ = Command::new("chroot")
            .args([root, "systemctl", "enable", svc])
            .status();
    }
    Ok(())
}

fn configure_plymouth(root: &str) -> Result<()> {
    // Set CobaltOS as the default plymouth theme
    let _ = Command::new("chroot")
        .args([root, "plymouth-set-default-theme", "-R", "cobalt"])
        .status();
    Ok(())
}

async fn chroot_cmd(root: &str, program: &str, args: &[&str]) -> Result<()> {
    let status = tokio::process::Command::new("chroot")
        .arg(root)
        .arg(program)
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status()
        .await
        .with_context(|| format!("chroot {root} {program}"))?;
    anyhow::ensure!(status.success(), "chroot {program} failed");
    Ok(())
}
