#!/usr/bin/env bash
# SecureForge Forensic Station — Background Service Manager
# Manages the continuous background API & Web engine without terminal overhead.

set -e

SERVICE_NAME="secureforge.service"
BIN_PATH="/home/karan/Projects/SIH_149/target/release/secureforge-desktop"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
SERVICE_FILE="${SYSTEMD_USER_DIR}/${SERVICE_NAME}"
TOKEN_FILE="${HOME}/.config/secureforge/api_token"

show_help() {
    echo "SecureForge Service Manager"
    echo "Usage: $0 {start|stop|restart|status|logs|enable|disable}"
    echo ""
    echo "Commands:"
    echo "  start    - Start continuous background server"
    echo "  stop     - Stop background server"
    echo "  restart  - Restart background server"
    echo "  status   - Check daemon status, port, and PID"
    echo "  logs     - Follow real-time daemon logs"
    echo "  enable   - Enable auto-start on user login"
    echo "  disable  - Disable auto-start on user login"
}

ensure_service_file() {
    mkdir -p "${SYSTEMD_USER_DIR}"
    if [ ! -f "${SERVICE_FILE}" ]; then
        cat << EOF > "${SERVICE_FILE}"
[Unit]
Description=SecureForge Forensic Station Web & API Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/karan/Projects/SIH_149
ExecStart=${BIN_PATH} --server --port 7878
Restart=always
RestartSec=3
Environment=PATH=${HOME}/.cargo/bin:/usr/local/bin:/usr/bin

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload
    fi
}

case "$1" in
    start)
        ensure_service_file
        systemctl --user start "${SERVICE_NAME}"
        echo "[✔] SecureForge background service started."
        systemctl --user status "${SERVICE_NAME}" --no-pager | grep -E "(Active:|Main PID:)"
        ;;
    stop)
        systemctl --user stop "${SERVICE_NAME}"
        echo "[✔] SecureForge background service stopped."
        ;;
    restart)
        ensure_service_file
        systemctl --user restart "${SERVICE_NAME}"
        echo "[✔] SecureForge background service restarted."
        systemctl --user status "${SERVICE_NAME}" --no-pager | grep -E "(Active:|Main PID:)"
        ;;
    status)
        echo "=== SecureForge Background Service Status ==="
        systemctl --user status "${SERVICE_NAME}" --no-pager || true
        echo ""
        if [ -f "${TOKEN_FILE}" ]; then
            echo "Persistent API Token: $(cat "${TOKEN_FILE}")"
        fi
        echo "Web Console URL     : http://127.0.0.1:7878"
        ;;
    logs)
        journalctl --user -u "${SERVICE_NAME}" -f
        ;;
    enable)
        ensure_service_file
        systemctl --user enable "${SERVICE_NAME}"
        echo "[✔] SecureForge enabled to run continuously on user login."
        ;;
    disable)
        systemctl --user disable "${SERVICE_NAME}"
        echo "[✔] SecureForge auto-start disabled."
        ;;
    *)
        show_help
        exit 1
        ;;
esac
