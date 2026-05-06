//! System configuration applied to the installed root.
//! All functions expect to be called with the target rootfs mounted at `root`.
#![allow(dead_code)]

use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

pub struct SystemConfig {
    pub locale: String,
    pub timezone: String,
    pub keyboard: String,
    pub hostname: String,
    pub username: String,
    pub password: String,
}

/// Write all system configuration into the installed root at `root`.
pub async fn apply(cfg: &SystemConfig, root: &str) -> Result<()> {
    write_hostname(cfg, root)?;
    write_locale(cfg, root)?;
    write_timezone(cfg, root)?;
    write_keyboard(cfg, root)?;
    create_user(cfg, root)?;
    info!("System configuration applied to {root}");
    Ok(())
}

fn write_hostname(cfg: &SystemConfig, root: &str) -> Result<()> {
    let path = format!("{root}/etc/hostname");
    std::fs::write(&path, format!("{}\n", cfg.hostname))
        .with_context(|| format!("write {path}"))?;

    let hosts = format!(
        "127.0.0.1\tlocalhost\n127.0.1.1\t{}\n::1\t\tlocalhost ip6-localhost\n",
        cfg.hostname
    );
    std::fs::write(format!("{root}/etc/hosts"), hosts)?;
    Ok(())
}

fn write_locale(cfg: &SystemConfig, root: &str) -> Result<()> {
    let locale_gen = format!("{}.UTF-8 UTF-8\n", cfg.locale);
    std::fs::write(format!("{root}/etc/locale.gen"), &locale_gen)?;
    std::fs::write(format!("{root}/etc/locale.conf"), format!("LANG={}.UTF-8\n", cfg.locale))?;

    chroot_run(root, "locale-gen", &[])?;
    Ok(())
}

fn write_timezone(cfg: &SystemConfig, root: &str) -> Result<()> {
    // Remove existing symlink if present
    let tz_link = format!("{root}/etc/localtime");
    let _ = std::fs::remove_file(&tz_link);

    std::os::unix::fs::symlink(
        format!("/usr/share/zoneinfo/{}", cfg.timezone),
        &tz_link,
    )?;
    std::fs::write(format!("{root}/etc/timezone"), format!("{}\n", cfg.timezone))?;
    Ok(())
}

fn write_keyboard(cfg: &SystemConfig, root: &str) -> Result<()> {
    let content = format!(
        "XKBMODEL=\"pc105\"\nXKBLAYOUT=\"{}\"\nXKBVARIANT=\"\"\nXKBOPTIONS=\"\"\nBACKSPACE=\"guess\"\n",
        cfg.keyboard
    );
    std::fs::write(format!("{root}/etc/default/keyboard"), content)?;
    Ok(())
}

fn create_user(cfg: &SystemConfig, root: &str) -> Result<()> {
    // Create user with home dir and add to sudo/wheel group
    chroot_run(
        root,
        "useradd",
        &["-m", "-G", "sudo,audio,video,plugdev,netdev", "-s", "/bin/zsh", &cfg.username],
    )?;

    // Set password via chpasswd
    let chpasswd_input = format!("{}:{}", cfg.username, cfg.password);
    let mut child = std::process::Command::new("chroot")
        .args([root, "chpasswd"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawn chroot chpasswd")?;
    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin.write_all(chpasswd_input.as_bytes())?;
    }
    child.wait().context("chpasswd wait")?;

    info!("Created user '{}'", cfg.username);
    Ok(())
}

fn chroot_run(root: &str, program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new("chroot")
        .arg(root)
        .arg(program)
        .args(args)
        .status()
        .with_context(|| format!("chroot {root} {program}"))?;
    if !status.success() {
        anyhow::bail!("chroot {program} failed with status {status}");
    }
    Ok(())
}
