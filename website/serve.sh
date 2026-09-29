#!/usr/bin/env bash
# Launch local preview server for SecureForge Enterprise Website
PORT="${1:-8080}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SERVE_DIR="$SCRIPT_DIR"
if [[ -d "$SCRIPT_DIR/dist" ]]; then
    SERVE_DIR="$SCRIPT_DIR/dist"
fi

echo "============================================================"
echo "   🌐 SECUREFORGE ENTERPRISE WEB PORTAL PREVIEW SERVER      "
echo "============================================================"
echo "[*] Serving directory: $SERVE_DIR"
echo "[*] Access URL       : http://localhost:$PORT"
echo "[*] Press Ctrl+C to terminate server."
echo "============================================================"

python3 -m http.server "$PORT" --directory "$SERVE_DIR"
