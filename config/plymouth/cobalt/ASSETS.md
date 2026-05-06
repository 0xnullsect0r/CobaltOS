# CobaltOS Plymouth theme assets
#
# The following PNG files are required in this directory:
#   logo.png       — CobaltOS logo, ~200×60px, transparent background
#   wordmark.png   — "CobaltOS" text in Inter Bold, cobalt blue (#0047AB)
#   bar.png        — 1×4px cobalt blue (#0047AB) fill (scaled by script)
#   bar-bg.png     — 1×4px dark track (#1e2230) fill (scaled by script)
#
# To generate placeholder assets for testing (requires ImageMagick):
#
#   convert -size 200x60 xc:"#111318" \
#       -fill "#0047AB" -font Inter-Bold -pointsize 28 \
#       -gravity Center -annotate 0 "CobaltOS" logo.png
#
#   convert -size 200x24 xc:"#111318" \
#       -fill "#0047AB" -font Inter-Bold -pointsize 16 \
#       -gravity Center -annotate 0 "CobaltOS" wordmark.png
#
#   convert -size 1x4 xc:"#0047AB" bar.png
#   convert -size 1x4 xc:"#1e2230" bar-bg.png
#
# Final assets should be committed to this directory.
