# CobaltOS

> A faster, freer Linux for your Chromebook.

CobaltOS is a lightweight, user-friendly, Debian-based Linux distribution designed specifically for Chromebooks. It replaces ChromeOS entirely using [MrChromebox's UEFI Full ROM firmware](https://mrchromebox.tech), giving you a full, privacy-respecting Linux desktop on hardware you already own.

---

## Highlights

- **Debian Stable base** — rock solid, well-supported, huge package ecosystem
- **COSMIC Desktop** — a modern, Wayland-native desktop built in Rust by System76
- **Chromebook-native** — keyboard, audio, touchpad, and scaling work out of the box
- **Rust-powered tooling** — every component we can replace, we have
- **Fast** — boots in under 10 seconds on eMMC; idles under 600 MB RAM
- **User-friendly installer** — GUI + TUI installer with automatic hardware detection

---

## Repository Structure

```
cobalt-os/
├── build/                  # ISO build scripts (live-build)
├── installer/              # cobalt-installer (Rust + iced/ratatui)
├── hardware-probe/         # cobalt-hardware-probe (Rust daemon)
├── update-manager/         # cobalt-update (Rust)
├── welcome/                # cobalt-welcome (Rust + iced)
├── oobe/                   # cobalt-oobe (Rust + iced)
├── config/
│   ├── zsh/                # Default .zshrc, starship.toml, plugin setup
│   ├── cosmic/             # COSMIC theme files and default settings
│   ├── keyd/               # Chromebook keyboard remapping configs per model
│   └── audio/              # UCM profiles per Chromebook board name
├── packages/               # Custom .deb packages
├── docs/                   # GitHub Pages site
└── README.md
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
sudo apt install live-build debootstrap xorriso mtools squashfs-tools
```

#### Building the Rust Components

```bash
# Build all components at once (workspace build)
cargo build --workspace

# Build and run a specific component
cargo run -p cobalt-hardware-probe
```

#### Building the ISO

```bash
cd build/
sudo ./build-iso.sh
```

The resulting ISO will be at `build/output/cobaltos-amd64.iso`.

---

## Custom Components

| Component | Description | Tech |
|---|---|---|
| `cobalt-installer` | Guided installer with GUI and TUI modes | Rust + iced + ratatui |
| `cobalt-hardware-probe` | First-boot hardware detection daemon | Rust + tokio + zbus |
| `cobalt-update` | Update manager with GUI tray notifications | Rust + tokio |
| `cobalt-welcome` | First-boot welcome wizard | Rust + iced |
| `cobalt-oobe` | Out-of-box experience, runs before desktop | Rust + iced |

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

## Hardware Support

CobaltOS supports all Chromebooks compatible with [MrChromebox's UEFI Full ROM firmware](https://docs.mrchromebox.tech/docs/supported-devices.html), including:

**Intel platforms:** Sandy Bridge, Ivy Bridge, Haswell, Broadwell, Bay Trail, Braswell, Apollo Lake, Gemini Lake, Kaby Lake, Coffee Lake, Whiskey Lake, Comet Lake, Ice Lake, Jasper Lake, Tiger Lake, Alder Lake, Raptor Lake

**AMD platforms:** Stoney Ridge, Picasso/Dali, Cezanne/Barcelo, Mendocino, Phoenix

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
