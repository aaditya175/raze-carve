#!/bin/bash
set -e

case "$1" in
    tui|sforge)
        shift
        exec sforge "$@"
        ;;
    gui|server|web)
        shift
        exec secureforge-desktop --server --host 0.0.0.0 --port 7878 "$@"
        ;;
    cli)
        shift
        exec sih149 "$@"
        ;;
    sih149|secureforge-desktop)
        exec "$@"
        ;;
    bash|sh)
        exec "$@"
        ;;
    *)
        exec sih149 "$@"
        ;;
esac
