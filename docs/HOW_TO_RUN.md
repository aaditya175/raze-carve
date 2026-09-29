# SecureForge — How to Run & Deployment Guide

> **Universal Platform Support:** Linux ✅ Full Native | Windows ✅ Universal Docker + Web GUI | macOS ✅ Universal Docker + Web GUI

---

## 🚀 Quickest Path: Universal Docker (Zero Local Dependencies)

SecureForge provides a multi-stage Docker setup supporting the full React Web GUI, the Ratatui interactive TUI, and the automated CLI on **any operating system** (Windows with Docker Desktop, Linux, or macOS).

```bash
# Clone the repository
git clone https://github.com/Normie69K/SecureForge.git
cd SecureForge

# 1. Full Web GUI Mode (Recommended for Windows & macOS)
# Runs the axum REST API server and serves the compiled React web station
docker compose up gui
# -> Open your browser at http://localhost:7878

# 2. Interactive Terminal TUI Mode (Best for Live ISOs, SSH, or Minimal Linux)
docker compose run --rm tui

# 3. Forensic CLI Automation
docker compose run --rm cli info
docker compose run --rm cli test
docker compose run --rm cli recover --source /data/evidence.img --output /data/recovered
```

---

## 🐧 Linux — Native Compilation & Execution

### 1. System Prerequisites

```bash
# Debian / Ubuntu / Pop!_OS
sudo apt update && sudo apt install -y \
    curl git build-essential pkg-config libssl-dev libsqlite3-dev \
    python3 python3-pip python3-venv \
    nodejs npm \
    libmagic1 libmagic-dev \
    nvme-cli hdparm smartmontools \
    webkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora / RHEL
sudo dnf install -y curl git gcc openssl-devel sqlite-devel \
    python3 python3-pip nodejs npm file-devel \
    nvme-cli hdparm smartmontools \
    webkit2gtk4.1-devel gtk3-devel
```

### 2. Build All Workspace Binaries

```bash
# Build CLI (`sih149`), TUI (`sforge`), and Desktop Server (`secureforge-desktop`)
cargo build --release -p sih149-cli -p sih149-tui -p secureforge-desktop

# Build React Web UI
npm --prefix src-ui install
npm --prefix src-ui run build
```

---

## ⌨️ Running the TUI (`sforge`)

The Terminal User Interface (`sforge`) is completely decoupled from the desktop GUI and runs in any modern 256-color terminal or SSH session.

```bash
# Launch TUI (requires root for direct block device access)
sudo ./target/release/sforge
```

### TUI Keyboard Reference (F1–F7)

| Key | View | Controls & Workflow |
|:---|:---|:---|
| <kbd>F1</kbd> | **Dashboard** | Mission control overview with storage device health, system metrics, and live audit event log (`[R]` to refresh) |
| <kbd>F2</kbd> | **Drive Manager** | Hardware browser with SMART status and auto-hotplug polling. Use `<↑/↓>` to navigate, `<Enter>` to select for sanitization, and `<E>` to view entropy |
| <kbd>F3</kbd> | **Entropy Heatmap** | Shannon randomness visualizer (`0.0–8.0 b/B`). Press `<E>` or `<Enter>` to trigger background asynchronous entropy scanning |
| <kbd>F4</kbd> | **Sanitize Wizard** | Multi-pass wiping wizard. `<↑/↓>` selects algorithm (Zero, DoD 3-pass, DoD 7-pass, Gutmann 35-pass), `<V>` toggles verification, `<E>` toggles expert mode, `<Enter>` initiates confirmation |
| <kbd>F5</kbd> | **Forensic Carver** | Signature & structure carver. `<Tab>` cycles between Source, Output, and Min Confidence fields. Type custom paths, and press `<Enter>` to carve and extract files |
| <kbd>F6</kbd> | **Self-Test Engine** | Integrated in-situ diagnostic runner. Press `<Enter>` or `<Space>` to run all 19 subsystem assertions; use `<↑/↓>` (or `<k/j>`) to inspect assertion details |
| <kbd>F7</kbd> | **Help & Docs** | Comprehensive on-screen keyboard guide, entropy scale reference, and compliance standards |
| <kbd>Q</kbd> | **Quit** | Clean exit and terminal state restoration |

---

## 💻 CLI Commands (`sih149`)

```bash
# 1. Run Subsystem Self-Tests (19 in-situ diagnostic checks)
./target/release/sih149 test
./target/release/sih149 test --json

# 2. Block Device Enumeration & SMART Data
sudo ./target/release/sih149 info
sudo ./target/release/sih149 info --device /dev/sdb

# 3. Forensic File Carving (with sector scanning & boundary extraction)
sudo ./target/release/sih149 recover \
    --source /dev/sdb \
    --output ./recovered \
    --types jpg,png,pdf,zip,sqlite \
    --min-confidence 60

# 4. Secure Drive Sanitization (zero-allocation chunk buffer)
sudo ./target/release/sih149 wipe \
    --device /dev/sdb \
    --method dod3 \
    --verify

# 5. Cryptographic Audit Report Generation
python3 pipeline/report_gen.py \
    --input ./audit.json \
    --template erasure \
    --output ./certificate.pdf \
    --investigator "Forensic Examiner" \
    --case-id "CASE-2026-001"
```

---

## 🌐 Web GUI / Remote Server Mode

You can run the web server on a Linux forensic station and access it from any browser on your network (Windows, Mac, tablet, or phone):

```bash
./target/release/secureforge-desktop --server --host 0.0.0.0 --port 7878

# Open http://<server-ip>:7878
```

The Web GUI includes:
- Command Center with real-time drive detection
- Sanitization Studio with visual wipe progress
- Forensic File Carver with interactive hex inspection
- System Diagnostics & Self-Test tab with live assertion execution

---

## 🧪 Running the Complete Automated Test Suite

```bash
# Run all workspace unit and integration tests (117 tests)
cargo test --workspace

# Run core engine tests only (67 tests)
cargo test -p sih149-core --lib

# Run TUI integration tests (21 tests)
cargo test -p sih149-tui
```
