#!/usr/bin/env bash
set -euo pipefail

# SIAR Architecture Wiki - Live Server Script
# Serves the full sys-arch documentation book locally and across LAN / VPN.

PORT="${1:-3000}"
HOST="0.0.0.0"

# Navigate to project root
cd "$(dirname "$0")/.."

if ! command -v mdbook &> /dev/null; then
    echo "[-] Error: 'mdbook' is not installed."
    echo "    To install mdbook, run:"
    echo "    cargo install mdbook"
    echo "    or download from https://github.com/rust-lang/mdBook/releases"
    exit 1
fi

echo "============================================================"
echo " 📚 Starting SIAR System Architecture Interactive Wiki"
echo "============================================================"
echo ""
echo " Local Access:        http://localhost:${PORT}"

# Detect LAN IP for mobile/tablet reading
if command -v hostname &> /dev/null; then
    for ip in $(hostname -I 2>/dev/null || true); do
        if [[ "$ip" != "127."* ]]; then
            echo " Mobile/LAN Access:   http://${ip}:${PORT}"
        fi
    done
fi

echo ""
echo " Shortcuts in Wiki:"
echo "   - Search:           Press 's' or '/'"
echo "   - Switch Theme:     Press 't'"
echo "   - Prev/Next Page:   Press '←' or '→'"
echo ""
echo " Press Ctrl+C to stop the server."
echo "============================================================"
echo ""

exec mdbook serve --hostname "${HOST}" --port "${PORT}"
