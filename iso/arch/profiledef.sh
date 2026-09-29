#!/usr/bin/env bash
# SecureForge Arch Live ISO Profile Definition

iso_name="secureforge-live"
iso_label="SECUREFORGE_$(date +%Y%m)"
iso_publisher="SecureForge Team <https://github.com/secureforge/secureforge>"
iso_application="SecureForge Forensic Sanitization & Recovery OS"
iso_version="0.1.0"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito' 'uefi-ia32.systemd-boot.esp' 'uefi-x64.systemd-boot.esp')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '15')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/.bash_profile"]="0:0:755"
  ["/root/.bashrc"]="0:0:644"
  ["/usr/local/bin/sforge"]="0:0:755"
  ["/usr/local/bin/sih149"]="0:0:755"
  ["/usr/local/bin/secureforge-desktop"]="0:0:755"
)
