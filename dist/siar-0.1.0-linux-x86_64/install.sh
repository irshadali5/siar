#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${1:-$HOME/.local}"

echo "Installing SIAR to ${PREFIX}..."

mkdir -p "${PREFIX}/bin"
mkdir -p "${PREFIX}/share/applications"
mkdir -p "${PREFIX}/share/pixmaps"

install -m 755 "${SCRIPT_DIR}/bin/siar" "${PREFIX}/bin/siar"
install -m 755 "${SCRIPT_DIR}/bin/siar-desktop" "${PREFIX}/bin/siar-desktop"
install -m 755 "${SCRIPT_DIR}/bin/siar-emergency-node" "${PREFIX}/bin/siar-emergency-node"

install -m 644 "${SCRIPT_DIR}/share/applications/siar.desktop" "${PREFIX}/share/applications/siar.desktop"
install -m 644 "${SCRIPT_DIR}/share/pixmaps/siar.png" "${PREFIX}/share/pixmaps/siar.png"

for sz in 16 32 64 128 256 512; do
    mkdir -p "${PREFIX}/share/icons/hicolor/${sz}x${sz}/apps"
    install -m 644 "${SCRIPT_DIR}/share/icons/hicolor/${sz}x${sz}/apps/siar.png" "${PREFIX}/share/icons/hicolor/${sz}x${sz}/apps/siar.png"
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${PREFIX}/share/icons/hicolor" 2>/dev/null || true
fi

echo "SIAR installed successfully!"
echo "Binaries available in: ${PREFIX}/bin"
