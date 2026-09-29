#!/usr/bin/env bash
# Inner build script executed inside privileged Arch Linux container
set -euo pipefail

OUTPUT_DIR="/workspace/iso/output"
WORK_DIR="/workspace/iso/work"

echo "[1/6] Initializing Arch Linux keyring and package databases..."
pacman-key --init
pacman-key --populate archlinux
pacman -Sy --noconfirm archiso libisoburn squashfs-tools dosfstools mtools rsync zstd

echo "[2/6] Initializing official releng baseline profile..."
rm -rf /tmp/profile
cp -r /usr/share/archiso/configs/releng /tmp/profile

echo "[3/6] Merging SecureForge packages and dependencies..."
cat >> /tmp/profile/packages.x86_64 << "EOF"
nvme-cli
smartmontools
hdparm
parted
gptfdisk
e2fsprogs
dosfstools
ntfs-3g
btrfs-progs
xfsprogs
python
python-pillow
python-jinja
python-pip
openssl
tmux
screen
htop
EOF

echo "[4/6] Configuring profiledef.sh metadata..."
cat > /tmp/profile/profiledef.sh << "EOF"
#!/usr/bin/env bash
# shellcheck disable=SC2034

iso_name="secureforge-live"
iso_label="SECUREFORGE"
iso_publisher="SecureForge Team <https://github.com/secureforge/secureforge>"
iso_application="SecureForge Forensic Sanitization and Recovery OS"
iso_version="0.1.0"
install_dir="arch"
buildmodes=("iso")
bootmodes=("bios.syslinux"
           "uefi.systemd-boot")
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=("-comp" "zstd" "-Xcompression-level" "15")
bootstrap_tarball_compression=("zstd" "-c" "-T0" "--auto-threads=logical" "--long" "-19")
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/.automated_script.sh"]="0:0:755"
  ["/root/.bash_profile"]="0:0:755"
  ["/root/.bashrc"]="0:0:644"
  ["/root/.zlogin"]="0:0:755"
  ["/usr/local/bin/sforge"]="0:0:755"
  ["/usr/local/bin/sih149"]="0:0:755"
  ["/usr/local/bin/secureforge-desktop"]="0:0:755"
)
EOF

echo "[5/6] Injecting SecureForge rootfs overlay..."
# Auto-login root on tty1
mkdir -p /tmp/profile/airootfs/etc/systemd/system/getty@tty1.service.d
cat > /tmp/profile/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf << "EOF"
[Service]
ExecStart=
ExecStart=-/usr/bin/agetty --autologin root --noclear %I $TERM
EOF

# secureforge.service unit
cat > /tmp/profile/airootfs/etc/systemd/system/secureforge.service << "EOF"
[Unit]
Description=SecureForge Forensic Workstation API Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/usr/local/bin/secureforge-desktop --server --port 7878 --no-browser
Restart=always
RestartSec=2
Environment=SECUREFORGE_PORT=7878
Environment=SECUREFORGE_PIPELINE_DIR=/usr/local/share/secureforge/pipeline

[Install]
WantedBy=multi-user.target
EOF

mkdir -p /tmp/profile/airootfs/etc/systemd/system/multi-user.target.wants
ln -sf ../secureforge.service /tmp/profile/airootfs/etc/systemd/system/multi-user.target.wants/secureforge.service

# Root bash profile & banner
cat > /tmp/profile/airootfs/root/.bash_profile << "EOF"
if [[ -f ~/.bashrc ]]; then
    . ~/.bashrc
fi

if [[ "$(tty)" == "/dev/tty1" ]]; then
    clear
    echo -e "\e[1;38;2;56;189;248m┌────────────────────────────────────────────────────────┐\e[0m"
    echo -e "\e[1;38;2;56;189;248m│  🔒 SECUREFORGE FORENSIC WORKSTATION (LIVE BOOT)       │\e[0m"
    echo -e "\e[1;38;2;56;189;248m└────────────────────────────────────────────────────────┘\e[0m"
    echo -e "\e[38;2;148;163;184mStarting SecureForge Terminal Interface...\e[0m\n"
    
    if [[ -x /usr/local/bin/sforge ]]; then
        /usr/local/bin/sforge
    fi

    echo -e "\n\e[1;38;2;52;211;153m[i] SecureForge session closed.\e[0m"
    echo -e "    Type \e[1msforge\e[0m to return to TUI."
    echo -e "    Type \e[1msih149 --help\e[0m for CLI commands."
    echo -e "    Type \e[1msih149 demo\e[0m to run autonomous verification demo."
    echo -e "    Web Workstation active on \e[1;38;2;56;189;248mhttp://localhost:7878\e[0m\n"
fi
EOF

cat > /tmp/profile/airootfs/root/.bashrc << "EOF"
alias ll="ls -la --color=auto"
alias sforge="/usr/local/bin/sforge"
alias tui="/usr/local/bin/sforge"
alias sih149="/usr/local/bin/sih149"
alias cli="/usr/local/bin/sih149"
export PS1="\[\e[1;38;2;239;68;68m\]secureforge-live\[\e[0m\]:\[\e[1;38;2;56;189;248m\]\w\[\e[0m\]# "
export SECUREFORGE_PIPELINE_DIR=/usr/local/share/secureforge/pipeline
EOF

# Ensure zlogin forwards into bash_profile if zsh is used
cat > /tmp/profile/airootfs/root/.zlogin << "EOF"
[[ -f ~/.bash_profile ]] && bash ~/.bash_profile
EOF

cat > /tmp/profile/airootfs/root/.automated_script.sh << "EOF"
#!/usr/bin/env bash
if [[ -f ~/.bash_profile ]]; then
    . ~/.bash_profile
fi
EOF
chmod +x /tmp/profile/airootfs/root/.automated_script.sh

# Stage binaries and pipeline
mkdir -p /tmp/profile/airootfs/usr/local/bin /tmp/profile/airootfs/usr/local/share/secureforge
cp -p /workspace/target/release/sforge /tmp/profile/airootfs/usr/local/bin/
cp -p /workspace/target/release/sih149 /tmp/profile/airootfs/usr/local/bin/
cp -p /workspace/target/release/secureforge-desktop /tmp/profile/airootfs/usr/local/bin/
cp -r /workspace/pipeline /tmp/profile/airootfs/usr/local/share/secureforge/
cp -r /workspace/plugins /tmp/profile/airootfs/usr/local/share/secureforge/
chmod +x /tmp/profile/airootfs/usr/local/bin/*

echo "[6/6] Executing mkarchiso to compile universal hybrid ISO..."
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR" "$OUTPUT_DIR"
mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" /tmp/profile

echo "[✔] Build completed successfully!"
