#!/usr/bin/env bash
set -euo pipefail

PREFIX="${1:-$HOME/.local}"

echo "Uninstalling SIAR from ${PREFIX}..."
rm -f "${PREFIX}/bin/siar" "${PREFIX}/bin/siar-desktop" "${PREFIX}/bin/siar-emergency-node"
rm -f "${PREFIX}/share/applications/siar.desktop"
rm -f "${PREFIX}/share/pixmaps/siar.png"

for sz in 16 32 64 128 256 512; do
    rm -f "${PREFIX}/share/icons/hicolor/${sz}x${sz}/apps/siar.png"
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true
fi

echo "SIAR uninstalled successfully."
