#!/usr/bin/env bash
# CobaltOS ISO build script
# Usage: sudo ./build-iso.sh [--clean]
#
# Requires: live-build, debootstrap, xorriso, mtools, squashfs-tools
# Must be run as root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$SCRIPT_DIR/output"
BUILD_DIR="$SCRIPT_DIR/.build"
VERSION="$(cat "$REPO_ROOT/VERSION" 2>/dev/null || echo "0.1.0")"
ISO_NAME="cobaltos-${VERSION}-amd64.iso"

# --- Colours ---
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

info()    { echo -e "${CYAN}[INFO]${NC}  $*"; }
success() { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
die()     { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

# --- Root check ---
[[ $EUID -eq 0 ]] || die "This script must be run as root: sudo $0"

# --- Dependency check ---
for cmd in lb debootstrap xorriso mtools cargo; do
    command -v "$cmd" &>/dev/null || die "Missing dependency: $cmd"
done

# --- Clean build ---
if [[ "${1:-}" == "--clean" ]]; then
    info "Cleaning previous build..."
    rm -rf "$BUILD_DIR"
fi

mkdir -p "$OUTPUT_DIR" "$BUILD_DIR"
cd "$BUILD_DIR"

info "Building CobaltOS v${VERSION} ISO..."
info "Output: ${OUTPUT_DIR}/${ISO_NAME}"

# --- Build Rust binaries ---
BINARIES_DIR="$BUILD_DIR/config/includes.chroot/usr/local/bin"
mkdir -p "$BINARIES_DIR"

info "Building cobalt-* Rust binaries (release)..."
pushd "$REPO_ROOT" > /dev/null
cargo build --release --workspace 2>&1 | tee -a "$OUTPUT_DIR/build.log"
popd > /dev/null

for bin in cobalt-installer cobalt-welcome cobalt-oobe cobalt-update cobalt-hardware-probe; do
    src="$REPO_ROOT/target/release/$bin"
    if [[ -f "$src" ]]; then
        cp "$src" "$BINARIES_DIR/$bin"
        success "Copied $bin"
    else
        warn "Binary not found: $src (skipping)"
    fi
done

# --- Configure live-build ---
lb config \
    --architecture amd64 \
    --distribution bookworm \
    --archive-areas "main contrib non-free non-free-firmware" \
    --bootloader grub-efi \
    --binary-images iso-hybrid \
    --iso-volume "CobaltOS ${VERSION}" \
    --iso-application "CobaltOS" \
    --iso-publisher "CobaltOS Project" \
    --memtest none \
    --debian-installer none \
    --firmware-binary true \
    --firmware-chroot true \
    --bootappend-live "boot=live components quiet splash" \
    --linux-packages "linux-image-amd64 linux-headers-amd64" \
    --apt-recommends false

# Copy package lists
cp "$SCRIPT_DIR/package-lists/"*.list.chroot config/package-lists/ 2>/dev/null || true

# Copy hooks
cp "$SCRIPT_DIR/hooks/"*.chroot config/hooks/normal/ 2>/dev/null || true

# Copy config files into chroot
mkdir -p config/includes.chroot/etc/cobaltos
cp -r "$REPO_ROOT/config/." config/includes.chroot/etc/cobaltos/

# Install Plymouth theme
PLYMOUTH_DST="config/includes.chroot/usr/share/plymouth/themes/cobalt"
mkdir -p "$PLYMOUTH_DST"
cp "$REPO_ROOT/config/plymouth/cobalt/cobalt.plymouth" "$PLYMOUTH_DST/"
cp "$REPO_ROOT/config/plymouth/cobalt/cobalt.script"   "$PLYMOUTH_DST/"

# Install systemd service units
SYSTEMD_DST="config/includes.chroot/usr/lib/systemd/system"
mkdir -p "$SYSTEMD_DST"
cp "$REPO_ROOT/config/systemd/"*.service "$SYSTEMD_DST/"

# Enable services via preset
PRESET_DST="config/includes.chroot/usr/lib/systemd/system-preset"
mkdir -p "$PRESET_DST"
cat > "$PRESET_DST/80-cobaltos.preset" <<'EOF'
enable cobalt-hardware-probe.service
enable cobalt-update.service
EOF

# --- Build ---
info "Running lb build (this takes 15–30 minutes)..."
lb build 2>&1 | tee "$OUTPUT_DIR/build.log"

# --- Move output ---
if [[ -f "live-image-amd64.hybrid.iso" ]]; then
    mv "live-image-amd64.hybrid.iso" "${OUTPUT_DIR}/${ISO_NAME}"
    success "ISO built: ${OUTPUT_DIR}/${ISO_NAME}"

    # Generate SHA256 checksum
    sha256sum "${OUTPUT_DIR}/${ISO_NAME}" > "${OUTPUT_DIR}/${ISO_NAME}.sha256"
    success "Checksum: ${OUTPUT_DIR}/${ISO_NAME}.sha256"
else
    die "Build failed — live-image-amd64.hybrid.iso not found. Check ${OUTPUT_DIR}/build.log"
fi

info "Done! Flash with:"
echo "  sudo dd if=${OUTPUT_DIR}/${ISO_NAME} of=/dev/sdX bs=4M status=progress conv=fdatasync"
