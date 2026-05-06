#!/usr/bin/env bash
# Generate placeholder Plymouth PNG assets for CobaltOS boot splash.
# Requires ImageMagick (convert). Install with: sudo apt install imagemagick
#
# Usage: ./generate-assets.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR"

if ! command -v convert &>/dev/null; then
    echo "ImageMagick 'convert' not found. Install with: sudo apt install imagemagick"
    exit 1
fi

echo "Generating CobaltOS Plymouth theme assets..."

# logo.png — cobalt blue C-shaped logo mark placeholder
convert \
    -size 180x180 xc:"#111318" \
    -fill "#0047AB" \
    -stroke "#0047AB" \
    -strokewidth 16 \
    -draw "arc 30,30 150,150 45,315" \
    -antialias \
    "$OUT_DIR/logo.png"
echo "  logo.png"

# wordmark.png — "CobaltOS" text in Inter Bold
convert \
    -size 260x40 xc:"#111318" \
    -fill "#0047AB" \
    -font "Inter-Bold" \
    -pointsize 26 \
    -gravity Center \
    -annotate 0 "CobaltOS" \
    "$OUT_DIR/wordmark.png" 2>/dev/null || \
convert \
    -size 260x40 xc:"#111318" \
    -fill "#0047AB" \
    -font "DejaVu-Sans-Bold" \
    -pointsize 26 \
    -gravity Center \
    -annotate 0 "CobaltOS" \
    "$OUT_DIR/wordmark.png"
echo "  wordmark.png"

# bar.png — 1×4 cobalt blue fill pixel (scaled by script)
convert -size 1x4 xc:"#0047AB" "$OUT_DIR/bar.png"
echo "  bar.png"

# bar-bg.png — 1×4 dark track pixel
convert -size 1x4 xc:"#1e2230" "$OUT_DIR/bar-bg.png"
echo "  bar-bg.png"

echo "Done! Assets written to $OUT_DIR"
