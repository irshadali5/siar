#!/usr/bin/env bash
set -euo pipefail

# SIAR Architecture Wiki - Static Build Script
# Compiles the sys-arch directory into production-ready static assets.

OUTPUT_DIR="${1:-dist/arch-wiki}"

# Navigate to project root
cd "$(dirname "$0")/.."

if ! command -v mdbook &> /dev/null; then
    echo "[-] Error: 'mdbook' is not installed."
    echo "    To install mdbook, run: cargo install mdbook"
    exit 1
fi

echo "==> Building SIAR System Architecture Wiki..."
mdbook build

echo "==> Exporting static assets to ${OUTPUT_DIR}..."
mkdir -p "${OUTPUT_DIR}"
rm -rf "${OUTPUT_DIR:?}"/*
cp -r book/* "${OUTPUT_DIR}/"

TOTAL_FILES=$(find "${OUTPUT_DIR}" -type f | wc -l)
TOTAL_SIZE=$(du -sh "${OUTPUT_DIR}" | cut -f1)

echo "[+] Build complete!"
echo "    Destination: ${OUTPUT_DIR}"
echo "    Total Files: ${TOTAL_FILES}"
echo "    Total Size:  ${TOTAL_SIZE}"
echo ""
echo "You can now host '${OUTPUT_DIR}' on any static hosting provider (GitHub Pages, Cloudflare Pages, Netlify, Vercel, S3, Nginx)."
