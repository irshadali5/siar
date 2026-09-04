#!/usr/bin/env bash
# ==============================================================================
# SIAR - Universal Cross-Distribution Nix Installer Entrypoint
# ==============================================================================
# This script forwards execution to the comprehensive installer script located in
# scripts/install-nix.sh with all supplied arguments.
#
# Usage:
#   ./install-nix.sh [OPTIONS]
#
# See './install-nix.sh --help' for full options and features.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALLER_PATH="${SCRIPT_DIR}/scripts/install-nix.sh"

if [[ ! -f "$INSTALLER_PATH" ]]; then
    echo "Error: Installer script not found at ${INSTALLER_PATH}" >&2
    exit 1
fi

chmod +x "$INSTALLER_PATH" 2>/dev/null || true
exec "$INSTALLER_PATH" "$@"
