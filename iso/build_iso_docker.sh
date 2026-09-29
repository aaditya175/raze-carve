#!/usr/bin/env bash
# SecureForge Universal Live ISO Containerized Builder
# Builds a hybrid UEFI + Legacy BIOS bootable Arch Linux ISO using privileged Docker
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/iso/output"
WORK_DIR="$PROJECT_ROOT/iso/work"

mkdir -p "$OUTPUT_DIR" "$WORK_DIR"
chmod +x "$SCRIPT_DIR/inner_build.sh"

echo "============================================================"
echo "   🔒 SECUREFORGE UNIVERSAL LIVE ISO CONTAINER BUILDER      "
echo "============================================================"
echo "[*] Project root : $PROJECT_ROOT"
echo "[*] Output ISO   : $OUTPUT_DIR"
echo "[*] Workspace    : $WORK_DIR"
echo "[*] Launching privileged Arch Linux build container..."

docker run --rm --privileged \
    -v "$PROJECT_ROOT:/workspace" \
    archlinux:latest bash /workspace/iso/inner_build.sh

echo "============================================================"
echo "       🎉 SECUREFORGE LIVE ISO BUILD FINISHED!             "
echo "============================================================"
ls -lh "$OUTPUT_DIR"/*.iso
