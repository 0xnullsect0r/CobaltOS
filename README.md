# CobaltOS

> A faster, freer Linux for your Chromebook.

[![CI](https://github.com/0xnullsect0r/CobaltOS/actions/workflows/ci.yml/badge.svg)](https://github.com/0xnullsect0r/CobaltOS/actions/workflows/ci.yml)

CobaltOS is a lightweight, user-friendly, Debian-based Linux distribution designed specifically for Chromebooks. It replaces ChromeOS entirely using [MrChromebox's UEFI Full ROM firmware](https://mrchromebox.tech), giving you a full, privacy-respecting Linux desktop on hardware you already own.

📖 **[Installation Guide](https://0xnullsect0r.github.io/CobaltOS/install/)** · 💻 **[Supported Devices](https://0xnullsect0r.github.io/CobaltOS/devices/)** · 🌐 **[Website](https://0xnullsect0r.github.io/CobaltOS/)**

---

## Highlights

- **Debian Stable base** — rock solid, well-supported, huge package ecosystem
- **COSMIC Desktop** — a modern, Wayland-native desktop built in Rust by System76
- **Chromebook-native** — keyboard, audio, touchpad, and HiDPI scaling work out of the box
- **Rust-powered tooling** — every component we can replace, we have
- **Fast** — boots in under 10 seconds on eMMC; idles under 600 MB RAM
- **User-friendly installer** — GUI + TUI installer with automatic hardware detection
- **Live session** — try CobaltOS before installing
- **zram swap** — zstd-compressed RAM swap, no partition needed

---

## Version Roadmap

| Version | Status | Description |
|---------|--------|-------------|
| v0.1.0 | ✅ Released | Foundation — repo scaffolding, 5 Rust crates, GitHub Pages website |
| v0.2.0 | ✅ Released | Apps — full iced GUI installer, ratatui TUI, cobalt-welcome, cobalt-oobe |
| v0.3.0 | ✅ Released | Install flow — debootstrap pipeline, GitHub Actions CI, systemd-boot, Plymouth |
| v0.4.0 | ✅ Released | Config — UCM audio profiles, live session autostart, COSMIC theme, hardware probe |
| v0.5.0 | ✅ Released | Polish — 60+ board database, password wiring, complete zshrc, package list |
| v1.0.0 | ✅ Released | First stable release — all components complete and integrated |
| v1.1.0 | 🚀 Released | Touchpad config, btrfs filesystem option, smart update daemon |
| v1.2.0 | 🚀 Released | Welcome wizard (locale + browser + app install), OOBE expansion (timezone + privacy + account), systemd update timer |

---

## Repository Structure

```
CobaltOS/
├── .github/workflows/      # CI (cargo check/test/clippy) + GitHub Pages deploy
├── build/
│   ├── build-iso.sh        # ISO build entry point (live-build + cargo)
│   ├── hooks/              # Chroot setup hook (services, Plymouth, keyd, zram)
│   └── package-lists/      # Debian package list for the live image
├── installer/              # cobalt-installer (Rust + iced GUI + ratatui TUI)
├── hardware-probe/         # cobalt-hardware-probe (board detection + config apply)
├── update-manager/         # cobalt-update (apt + flatpak update manager)
├── welcome/                # cobalt-welcome (first-boot welcome wizard)
├── oobe/                   # cobalt-oobe (out-of-box experience)
├── config/
│   ├── applications/       # .desktop launcher files
│   ├── audio/ucm2/         # UCM2 audio profiles (eve, atlas, nami, generic-hda)
│   ├── boot/               # systemd-boot loader.conf + entry template
│   ├── cosmic/             # COSMIC dark/light theme (cobalt palette)
│   ├── keyd/               # Chromebook keyboard remapping configs
│   ├── live/               # Live session autostart configuration
│   ├── plymouth/cobalt/    # Plymouth boot splash theme + asset generator
│   ├── systemd/            # cobalt-hardware-probe, cobalt-update, cobalt-welcome services
│   └── zram/               # zram-generator.conf (zstd, RAM/2)
├── config/zsh/             # Default .zshrc + starship.toml
└── docs/                   # GitHub Pages website source
```

---

## Getting Started

### For Users

Visit the **[CobaltOS Installation Guide](https://0xnullsect0r.github.io/CobaltOS/install/)** for step-by-step instructions, or check **[Supported Devices](https://0xnullsect0r.github.io/CobaltOS/devices/)** to see if your Chromebook is compatible.

### For Developers

#### Prerequisites

- Debian/Ubuntu host system
- Rust toolchain (`rustup` — https://rustup.rs)
- `live-build`, `debootstrap`, `xorriso`

```bash
# Install build dependencies
sudo apt install live-build debootstrap xorriso mtools squashfs-tools \
                 libwayland-dev libxkbcommon-dev pkg-config
```

#### Building the Rust Components

```bash
# Check all components compile
cargo check --workspace

# Build all components (release)
cargo build --release --workspace

# Run tests
cargo test --workspace

# Run a specific component
cargo run -p cobalt-hardware-probe
```

#### Building the ISO

```bash
cd build/
sudo ./build-iso.sh
```

The resulting ISO will be at `build/output/cobaltos-<version>-amd64.iso`.

---

## Custom Components

| Component | Description | Tech |
|---|---|---|
| `cobalt-installer` | Guided installer with GUI (iced) and TUI (ratatui) modes, real debootstrap pipeline | Rust + iced + ratatui + tokio |
| `cobalt-hardware-probe` | First-boot hardware detection — applies keyd, UCM audio, HiDPI scaling, power profile | Rust + tokio + zbus |
| `cobalt-update` | Update manager daemon — apt + flatpak, desktop notifications | Rust + tokio |
| `cobalt-welcome` | First-boot welcome wizard (4-page iced GUI) | Rust + iced |
| `cobalt-oobe` | Out-of-box experience, runs before desktop on first login | Rust + iced |

---

## Hardware Support

CobaltOS supports all Chromebooks compatible with [MrChromebox's UEFI Full ROM firmware](https://docs.mrchromebox.tech/docs/supported-devices.html).

The `cobalt-hardware-probe` service recognizes **60+ board names** across all supported platforms:

**Intel platforms:** Sandy Bridge, Ivy Bridge, Haswell, Broadwell, Bay Trail, Braswell, Apollo Lake, Gemini Lake, Kaby Lake, Coffee Lake, Whiskey Lake, Comet Lake, Ice Lake, Jasper Lake, Tiger Lake, Alder Lake, Raptor Lake

**AMD platforms:** Stoney Ridge, Picasso/Dali, Cezanne/Barcelo, Mendocino, Phoenix

---

## System Requirements

| Component | Minimum |
|---|---|
| Architecture | x86_64 |
| RAM | 2 GB (4 GB recommended) |
| Storage | 16 GB |
| Firmware | MrChromebox UEFI Full ROM |
| USB Drive | 8 GB (for installation) |

---

## Performance Targets

| Metric | Target |
|---|---|
| Boot to desktop | < 10 seconds (eMMC) |
| Idle RAM | < 600 MB |
| Idle CPU | < 2% |
| ISO size | < 2 GB |

---

## Contributing

Contributions are welcome! Please open an issue before submitting large PRs.

Areas where help is especially appreciated:
- UCM audio profiles for specific Chromebook boards
- `keyd` keyboard mapping configs for specific models
- Testing on specific Chromebook hardware
- UI/UX improvements to the installer

---

## License

CobaltOS is licensed under the [GNU General Public License v3.0](LICENSE).

Component licenses:
- COSMIC Desktop: [GPL-3.0](https://github.com/pop-os/cosmic)
- MrChromebox Firmware Utility: [GPL-3.0](https://github.com/MrChromebox/scripts)
- All bundled Rust tools retain their respective upstream licenses.

---

## Acknowledgements

- [MrChromebox](https://mrchromebox.tech) — for the UEFI firmware that makes this possible
- [System76](https://system76.com) — for the COSMIC desktop environment
- The Debian project, the Linux kernel contributors, and the open-source community