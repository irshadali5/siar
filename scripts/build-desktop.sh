#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo build -p siar-desktop --release

PREFIX="${PREFIX:-/usr/local}"
case "$PREFIX" in
  /*) ;;
  *) echo "PREFIX must be an absolute path" >&2; exit 2 ;;
esac

install -Dm0755 target/release/siar "$PREFIX/bin/siar"
install -Dm0644 scripts/siar.desktop "$PREFIX/share/applications/siar.desktop"
if [[ "$PREFIX" != "/usr/local" ]]; then
  sed -i "s|^Exec=.*|Exec=$PREFIX/bin/siar|" "$PREFIX/share/applications/siar.desktop"
fi
for size in 16 32 48 64 128 256 512 1024; do
  install -Dm0644 "assets/icons/hicolor/${size}x${size}/apps/siar.png" \
    "$PREFIX/share/icons/hicolor/${size}x${size}/apps/siar.png"
done

echo "Installed Linux app under $PREFIX"
