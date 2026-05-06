#!/usr/bin/env bash
# Autostart cobalt-installer in the live session.
# Placed at /etc/xdg/autostart/cobalt-installer-live.desktop
# Only runs when ~/.cobalt-live-session exists (set by live hook).

set -euo pipefail

LIVE_MARKER="/run/cobalt-live-session"

if [[ -f "$LIVE_MARKER" ]]; then
    # Small delay to let the desktop fully load
    sleep 3
    exec cobalt-installer
fi
