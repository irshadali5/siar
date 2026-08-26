#!/usr/bin/env bash
set -euo pipefail

# SIAR Architecture Wiki - Rootless Podman Management Script

CONTAINER_NAME="siar-arch-wiki"
IMAGE_NAME="localhost/siar-arch-wiki:latest"
PORT="${PORT:-3000}"

cd "$(dirname "$0")/.."

if ! command -v podman &> /dev/null; then
    echo "[-] Error: 'podman' is not installed."
    exit 1
fi

action="${1:-start}"

case "${action}" in
    build)
        echo "==> Building Podman image '${IMAGE_NAME}' using Containerfile.wiki..."
        podman build -t "${IMAGE_NAME}" -f Containerfile.wiki .
        echo "[+] Build complete!"
        ;;

    start)
        # Check if container is already running
        if podman ps --format '{{.Names}}' | grep -wq "${CONTAINER_NAME}"; then
            echo "[!] Container '${CONTAINER_NAME}' is already running at http://localhost:${PORT}"
            exit 0
        fi

        # Check if image exists, build if not
        if ! podman image exists "${IMAGE_NAME}"; then
            echo "==> Image '${IMAGE_NAME}' not found. Building first..."
            podman build -t "${IMAGE_NAME}" -f Containerfile.wiki .
        fi

        # Remove stopped container if exists
        podman rm -f "${CONTAINER_NAME}" 2>/dev/null || true

        echo "==> Starting rootless Podman container '${CONTAINER_NAME}' on port ${PORT}..."
        podman run -d \
            --name "${CONTAINER_NAME}" \
            --restart unless-stopped \
            -p "${PORT}:8080" \
            "${IMAGE_NAME}"

        echo ""
        echo "============================================================"
        echo " 🦭 SIAR Wiki is now running via Podman!"
        echo "============================================================"
        echo " Local Access:        http://localhost:${PORT}"

        if command -v hostname &> /dev/null; then
            for ip in $(hostname -I 2>/dev/null || true); do
                if [[ "$ip" != "127."* ]]; then
                    echo " Mobile/LAN Access:   http://${ip}:${PORT}"
                fi
            done
        fi
        echo "============================================================"
        ;;

    stop)
        echo "==> Stopping container '${CONTAINER_NAME}'..."
        podman stop "${CONTAINER_NAME}" || true
        podman rm "${CONTAINER_NAME}" || true
        echo "[+] Container stopped and removed."
        ;;

    logs)
        podman logs -f "${CONTAINER_NAME}"
        ;;

    status)
        podman ps -a --filter "name=${CONTAINER_NAME}"
        ;;

    *)
        echo "Usage: $0 {start|stop|build|logs|status}"
        exit 1
        ;;
esac
