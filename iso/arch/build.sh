#!/usr/bin/env bash
# SecureForge Arch Live ISO Automated Builder
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
OUTPUT_DIR="$PROJECT_ROOT/iso/output"
WORK_DIR="/tmp/secureforge-archiso-work"

echo "============================================================"
echo "    🔒 SECUREFORGE ARCH LIVE ISO AUTOMATED BUILDER        "
echo "============================================================"

# 1. Dependency checks
if ! command -v mkarchiso &>/dev/null; then
    echo -e "\n\e[1;38;2;239;68;68m[!] 'mkarchiso' not found on this system.\e[0m"
    echo -e "Please install archiso build tools using pacman:\n"
    echo -e "    \e[1msudo pacman -S --needed archiso squashfs-tools libisoburn\e[0m\n"
    exit 1
fi

# 2. Check root/sudo permissions for mkarchiso
if [[ $EUID -ne 0 ]]; then
    echo -e "\e[1;38;2;251;191;36m[i] mkarchiso requires root privileges to build chroot filesystem.\e[0m"
    echo -e "Re-running with sudo...\n"
    exec sudo "$0" "$@"
fi

# 3. Ensure bootloader templates from releng are merged if not already present
if [[ -d /usr/share/archiso/configs/releng ]]; then
    echo "[*] Synchronizing standard EFI/Syslinux bootloader assets from /usr/share/archiso/configs/releng..."
    for dir in efiboot grub syslinux; do
        if [[ -d "/usr/share/archiso/configs/releng/$dir" && ! -d "$SCRIPT_DIR/$dir" ]]; then
            cp -r "/usr/share/archiso/configs/releng/$dir" "$SCRIPT_DIR/"
        fi
    done
fi

# 4. Clean workdir & output dir
mkdir -p "$OUTPUT_DIR"
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

echo "[*] Staging compiled SecureForge binaries..."
mkdir -p "$SCRIPT_DIR/airootfs/usr/local/bin" "$SCRIPT_DIR/airootfs/usr/local/share/secureforge"
cp -p "$PROJECT_ROOT/target/release/sforge" "$SCRIPT_DIR/airootfs/usr/local/bin/"
cp -p "$PROJECT_ROOT/target/release/sih149" "$SCRIPT_DIR/airootfs/usr/local/bin/"
cp -p "$PROJECT_ROOT/target/release/secureforge-desktop" "$SCRIPT_DIR/airootfs/usr/local/bin/"
cp -r "$PROJECT_ROOT/pipeline" "$SCRIPT_DIR/airootfs/usr/local/share/secureforge/"
cp -r "$PROJECT_ROOT/plugins" "$SCRIPT_DIR/airootfs/usr/local/share/secureforge/"
chmod +x "$SCRIPT_DIR/airootfs/usr/local/bin/"*

echo "[*] Initiating mkarchiso build..."
mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" "$SCRIPT_DIR"

echo -e "\n\e[1;38;2;52;211;153m[✔] SecureForge Live ISO Build Completed Successfully!\e[0m"
echo "Output files in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"/*.iso
