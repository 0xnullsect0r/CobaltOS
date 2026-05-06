# Audio UCM profiles directory
#
# Board-specific UCM profiles are stored here as:
#   <BOARDNAME>.conf
#
# cobalt-hardware-probe reads these on first boot and installs them to
# /usr/share/alsa/ucm2/<boardname>/
#
# To add support for a new board:
#   1. Find or create the UCM files for your board
#      (check https://github.com/alsa-project/alsa-ucm-conf or
#       https://chromium.googlesource.com/chromiumos/third_party/adhd/)
#   2. Create a directory named after your board (uppercase)
#   3. Add it to this index with a brief description
#
# Known boards with confirmed audio support:
#   EVE        — Pixelbook (2017), Intel SST
#   ATLAS      — Pixelbook Go, Intel SST
#   BOBBA      — Acer CB311, Intel SOF
#   HELIOS     — Asus Flip CX5, Intel SOF
#   ZORK       — Acer Spin 514 AMD, AMD ACP
#   VILBOZ     — HP CB14 AMD, AMD ACP
#   VOEMA      — Acer Spin 514 TGL, Intel SOF
#
# Pull requests for new board UCM profiles are very welcome!
