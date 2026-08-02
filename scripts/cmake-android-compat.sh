#!/usr/bin/env sh
set -eu

case "${1:-}" in
    --build|--install|--open|--help|--version)
        exec /usr/bin/cmake "$@"
        ;;
    *)
        exec /usr/bin/cmake -DCMAKE_POLICY_VERSION_MINIMUM=3.5 "$@"
        ;;
esac
