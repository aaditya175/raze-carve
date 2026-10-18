<div align="center">

<img src="website/public/logo.png" alt="RAZE-CARVE" width="420" />

# RAZE-CARVE

**Sanitize. Recover. Certify.**

*Universal Forensic Data Sanitization, Deep Recovery & Cryptographic Certification Platform*  
*Built for Smart India Hackathon (SIH-149) — National Cybersecurity & Cyberforensics Track*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![NIST SP 800-88](https://img.shields.io/badge/NIST_SP_800--88-Rev_1_Compliant-green.svg)](https://csrc.nist.gov/publications/detail/sp/800-88/rev-1/final)
[![DoD 5220.22-M](https://img.shields.io/badge/DoD_5220.22--M-3_%26_7_Pass-brightgreen.svg)]()
[![Tests: 420 Passing](https://img.shields.io/badge/Tests-420_Passing-success.svg)]()
[![Docker Hub](https://img.shields.io/badge/Docker_Hub-lonewolf175%2Fraze--carve-blue.svg?logo=docker)](https://hub.docker.com/r/lonewolf175/raze-carve)
[![Release: v0.2.1](https://img.shields.io/badge/Release-v0.2.1-informational.svg)](https://github.com/aaditya175/raze-carve/releases)

</div>

---

## 🌟 Executive Summary

**RAZE-CARVE** is an integrated, forensic-grade toolkit designed for high-assurance data destruction, deep file carving, cryptographic chain-of-custody verification, and hardware-level diagnostics. It replaces commercial forensic packages costing $3,000–$50,000/year with an open-source, zero-allocation Rust core capable of operating across live crime scenes, field triage setups, forensic laboratories, and air-gapped recovery boots.

### Core Capabilities Matrix

| Subsystem | Forensic Standards & Capabilities |
| :--- | :--- |
| **Media Sanitization** | NIST SP 800-88 Rev 1 (*Clear & Purge*), DoD 5220.22-M (3-Pass & 7-Pass), Gutmann (35-Pass), Zero/FF/CSPRNG overwrite loops |
| **Firmware-Level Erase** | NVMe Cryptographic Erase (scrambles AES media key), NVMe Block Erase, ATA Secure Erase, ATA Enhanced Secure Erase |
| **Forensic File Carver** | Deep chunk-aware boundary reconstruction for JPEG (SOI/EOI), PNG (IHDR/IEND), PDF (trailer/xref), ZIP (EOCD), SQLite (page validation), and TOML signatures |
| **Chain of Custody** | SHA-256 Merkle hash chain in SQLite WAL mode, constant-time timing-oracle mitigation, and RFC 3161 digital timestamping |
| **Shannon Entropy** | High-throughput sector randomness scanner ($0.000$ bits/byte zeroed to $7.999$ bits/byte encrypted) with heatmap visualization |
| **Safety Guardrails** | Automatic boot drive lockdown (`/`, `/boot`, active swap), CoW filesystem warnings (Btrfs/ZFS), symlink traversal defenses |

---

## 🏆 The 5 Winning Forensic Superpowers ("Why RAZE-CARVE Wins")

Judges in cybersecurity and forensics look beyond basic wrappers. RAZE-CARVE implements real forensic engineering that sets it apart:

1. **The SSD Overprovisioning & FTL Blindspot Killer**: Standard software overwrites (`dd`) cannot reach the 10–25% overprovisioned NAND cells, wear-leveling reserves, or retired bad blocks managed by the Flash Translation Layer (FTL). RAZE-CARVE issues controller-level **NVMe Sanitize** and **ATA Enhanced Secure Erase** commands to sanitize the physical silicon directly.
2. **Court-Admissible RFC 3161 Proof of Existence**: An audit certificate is meaningless if an examiner can manipulate the system clock. RAZE-CARVE binds the audit SHA-256 hash to a verifiable **RFC 3161 Time-Stamp Authority (TSA)** token (`.tsr`), proving mathematically that the erasure or recovery occurred at that exact certified second.
3. **Deep Structure-Aware Carver with Extensible Plugins**: Unlike naive carvers that scrape magic bytes and dump arbitrary disk junk, RAZE-CARVE parses internal chunk hierarchies (terminating at genuine EOF/trailers) and supports dynamic file signatures via TOML and Lua scripts without recompiling.
4. **Physics-Based Shannon Entropy Verification**: Eliminates guesswork by measuring information density before and after sanitization. Judges can witness high-entropy encrypted storage ($7.998$ b/B) collapse to flatline zero ($0.000$ b/B) in real-time.
5. **Universal Tri-Modal Architecture in Pure Rust**: Zero garbage collection pauses, zero buffer overflow vulnerabilities, and zero memory leaks. Deploys as a lightweight CLI, a zero-latency terminal TUI, a desktop GUI, or a headless remote web server.

---

## 🏛️ System Architecture

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             RAZE-CARVE CORE (Rust)                              │
│   Wiper Engine  │  Carving Engine  │  Hash Chain Auditing  │  Diagnostic Engine  │
└────────┬───────────────────┬──────────────────────┬──────────────────────┬───────┘
         │                   │                      │                      │
         ▼                   ▼                      ▼                      ▼
    Terminal TUI         Forensic CLI          Web API Server         Desktop GUI
    (sforge)            (sih149)               (Port 7878)            (Tauri v2)
   ┌─────────────┐     ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
   │ Ratatui UI  │     │ Scripting   │       │ Axum REST   │       │ WebKit/GTK3 │
   │ Hotkeys F1-7│     │ JSON output │       │ Bearer Auth │       │ React + TS  │
   │ Air-gapped  │     │ Automation  │       │ Remote Lab  │       │ Interactive │
   └─────────────┘     └─────────────┘       └─────────────┘       └─────────────┘
```

---

## 🚀 Quickest Path: Universal Docker (Zero Local Setup)

Works seamlessly on **Windows (Docker Desktop)**, **macOS**, and **Linux** without installing Rust, Node.js, or C libraries:

```bash
# Pull and run the Web Distribution Portal & Video Showcase (1 command)
docker run -d --name raze-carve -p 5174:80 lonewolf175/raze-carve:latest
```
* **Web Portal URL:** [http://localhost:5174](http://localhost:5174)
* **Direct ISO Download URL:** `http://localhost:5174/releases/raze-carve-live-0.2.1-gui-amd64.iso`

### 💿 Universal Live Bootable ISO (1.35 GB)
The bootable Arch Linux LTS live OS image is available via two zero-friction channels:
1. **GitHub Releases:** [Download raze-carve-live-0.2.1-gui-amd64.iso](https://github.com/aaditya175/raze-carve/releases)
2. **Direct Extraction from Docker Hub (1 command):**
   ```bash
   docker run --rm lonewolf175/raze-carve:latest cat /usr/share/nginx/html/releases/raze-carve-live-0.2.1-gui-amd64.iso > raze-carve-live-0.2.1-gui-amd64.iso
   ```
* **SHA-256 Checksum:** `dbcf2a4981ca7bdfb6dce6cd23a881e1b8431f52d63de089bb1e7ff50540af6e`
* See [docs/LARGE_FILES_AND_RELEASES.md](docs/LARGE_FILES_AND_RELEASES.md) for complete details.

### Clone & Run Forensic Services Locally
```bash
# Clone the repository
git clone https://github.com/aaditya175/raze-carve.git
cd raze-carve

# 1. Launch Web Portal (Port 5174)
docker compose up -d website

# 2. Launch Forensic Web GUI (Port 7878)
docker compose up -d gui

# 3. Interactive Terminal User Interface (TUI)
docker compose run --rm tui

# 4. Forensic Scripting CLI
docker compose run --rm cli info
docker compose run --rm cli test
docker compose run --rm cli recover --source /data/evidence.img --output /data/recovered
```

---

## 🛠️ Complete Manual Installation & Build Guide

Follow these steps to compile and run every component natively from source.

### 1. Install System Prerequisites

#### Debian / Ubuntu / Linux Mint / Pop!_OS
```bash
sudo apt update && sudo apt install -y \
    curl git build-essential pkg-config libssl-dev libsqlite3-dev \
    python3 python3-pip python3-venv \
    nodejs npm \
    libmagic1 libmagic-dev \
    nvme-cli hdparm smartmontools \
    webkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Fedora / RHEL / CentOS Stream
```bash
sudo dnf install -y \
    curl git gcc gcc-c++ openssl-devel sqlite-devel \
    python3 python3-pip nodejs npm file-devel \
    nvme-cli hdparm smartmontools \
    webkit2gtk4.1-devel gtk3-devel libayatana-appindicator-devel librsvg2-devel
```

#### Arch Linux / Manjaro
```bash
sudo pacman -Syu --needed \
    base-devel git curl openssl sqlite \
    python python-pip nodejs npm file \
    nvme-cli hdparm smartmontools \
    webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg
```

### 2. Install Rust Toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc --version  # Requires Rust 1.80+
```

### 3. Compile the Production Binaries
Compile the optimized workspace release targets:
```bash
# Build CLI (sih149), TUI (sforge), and Desktop Server (secureforge-desktop)
cargo build --release -p sih149-cli -p sih149-tui -p secureforge-desktop

# Compile React Web Frontend
npm --prefix src-ui install
npm --prefix src-ui run build
```

The resulting binaries are located in `./target/release/`:
* `target/release/sih149` — Forensic Command-Line Interface (5.4 MB)
* `target/release/sforge` — Ratatui Terminal User Interface (4.5 MB)
* `target/release/secureforge-desktop` — Native Desktop GUI & Headless REST Server (20 MB)

---

## 💻 Manual CLI Guide (`sih149`)

The CLI is engineered for field automation, live incident response, and CI/CD pipelines.

### 0. Live Autonomous Forensic Demo
Execute a self-contained, end-to-end demonstration of evidence disk image synthesis, deep sector carving, bit-exact SHA-256 validation, and court-admissible PDF certificate generation:
```bash
./target/release/sih149 demo

# Output machine-readable JSON for CI/CD gates
./target/release/sih149 demo --json
```

### 1. In-Situ System Diagnostics
Execute all 23 subsystem assertions across wipers, carvers, cryptographic primitives, and database consistency:
```bash
./target/release/sih149 test

# Output machine-readable JSON for automated verification
./target/release/sih149 test --json
```

### 2. Storage Hardware Enumeration
Discover attached block devices, partitions, mountpoints, serial numbers, and bus protocols:
```bash
sudo ./target/release/sih149 info

# Specific device inspection with SMART telemetry
sudo ./target/release/sih149 info --device /dev/sdb --json
```

### 3. Secure File & Directory Shredding
Shred files or entire directory trees with multi-pass in-place overwriting, random inode renaming, and slack space scrubbing:
```bash
# Shred a sensitive document (3 overwrite passes, 8 random renames)
./target/release/sih149 shred --target /path/to/secret.pdf --passes 3 --renames 8 --yes

# Recursively shred an entire folder tree with slack space scrubbing
./target/release/sih149 shred --target /tmp/sensitive_case_data --passes 7 --scrub-slack --yes
```
*Note: RAZE-CARVE detects CoW filesystems (Btrfs/ZFS) and enforces safety guards to prevent false sanitization claims.*

### 4. Forensic File Carving
Extract lost or deleted evidence from physical drives, unallocated slack space, or raw `.dd`/`.img` forensic images:
```bash
# Carve JPEG, PNG, PDF, and ZIP files with minimum 60% structure confidence
sudo ./target/release/sih149 recover \
    --source /dev/sdb \
    --output ./recovered_evidence \
    --types jpg,png,pdf,zip \
    --min-confidence 60
```

### 5. Multi-Pass Block Device Sanitization
Sanitize unmounted external media using international compliance wiping standards:
```bash
# DoD 5220.22-M 3-Pass sanitization with post-wipe sector verification
sudo ./target/release/sih149 wipe \
    --device /dev/sdb \
    --method dod3 \
    --verify

# Available methods:
#   zero        - Single pass 0x00 overwrite (fast)
#   random      - Single pass CSPRNG ChaCha20 random bytes
#   dod3        - DoD 5220.22-M 3-pass (zeros, ones, random)
#   dod7        - DoD 5220.22-M 7-pass multi-pattern sequence
#   gutmann     - Peter Gutmann 35-pass magnetic erasure algorithm
```

### 6. Controller-Level Firmware Erase
Sanitize physical flash memory and overprovisioned sectors on solid-state drives:
```bash
# NVMe Cryptographic Erase (scrambles AES master key)
sudo ./target/release/sih149 firmware-erase \
    --device /dev/nvme1n1 \
    --method nvme-crypto \
    --expert

# ATA Enhanced Secure Erase
sudo ./target/release/sih149 firmware-erase \
    --device /dev/sdc \
    --method ata-enhanced \
    --expert
```

### 7. Audit Case Management
```bash
# List all recorded forensic operations in the audit database
./target/release/sih149 report --list
```

---

## ⌨️ Manual TUI Guide (`sforge`)

The Terminal User Interface is completely decoupled from X11/Wayland and runs in any SSH terminal, headless serial console, or live rescue environment.

```bash
# Launch TUI natively (requires root for direct ioctl block access)
sudo ./target/release/sforge
```

### Keyboard Navigation & Workflow Reference

```text
┌─── F1: Dashboard ─── F2: Drives ─── F3: Entropy ─── F4: Sanitize ─── F5: Carver ─── F6: Self-Test ─── F7: Help ───┐
```

| Key | Tab / Screen | Step-by-Step Workflow |
| :--- | :--- | :--- |
| <kbd>F1</kbd> | **Mission Dashboard** | Real-time overview of drive attachments, live audit logs, and system memory. Press `[R]` to refresh telemetry. |
| <kbd>F2</kbd> | **Drive Manager** | Use `<↑/↓>` to select drives. Press `[R]` to rescan hotplug devices. Press `<Enter>` to stage for sanitization, or `[E]` to jump to entropy visualization. |
| <kbd>F3</kbd> | **Entropy Heatmap** | Shows Shannon randomness across sectors ($0.0–8.0$ b/B). Press `[E]` or `<Enter>` to initiate asynchronous scanning. |
| <kbd>F4</kbd> | **Sanitize Wizard** | Select algorithm using `<↑/↓>`. Press `[V]` to toggle post-wipe verification. Press `[E]` to unlock expert passphrase mode. Press `<Enter>` to initiate confirmation. |
| <kbd>F5</kbd> | **Forensic Carver** | Use `<Tab>` to cycle inputs (Source Path, Output Directory, Min Confidence). Type custom paths and press `<Enter>` to trigger background carving. |
| <kbd>F6</kbd> | **Self-Test Engine** | Press `<Enter>` or `<Space>` to run all 19 diagnostic assertions live. Use `<↑/↓>` or `<k/j>` to inspect assertion status, latency, and messages. |
| <kbd>F7</kbd> | **Documentation** | Built-in reference manual, keyboard map, compliance guidelines, and entropy scale. |
| <kbd>Q</kbd> | **Quit** | Clean exit and terminal state restoration. |

---

## 🖥️ Desktop GUI & Remote Web Server Guide

### Mode A: Native Desktop App (Tauri v2)
Run the native desktop GUI on your local Linux workstation:
```bash
# Run release desktop application
./target/release/secureforge-desktop
```

### Mode B: Remote Forensic Server Mode (Web Browser GUI)
Deploy RAZE-CARVE on a lab server or forensic station and access the full GUI from any browser on your network (Windows, Mac, Linux, tablet):
```bash
# Start HTTP server on all network interfaces with token authentication
./target/release/secureforge-desktop \
    --server \
    --host 0.0.0.0 \
    --port 7878 \
    --api-token "forensic-lab-secret-2026"
```
Open **`http://<SERVER-IP>:7878`** in any web browser and enter your bearer token when prompted.

#### REST API Endpoints

| Method | Endpoint | Authentication | Purpose |
| :--- | :--- | :--- | :--- |
| `GET` | `/health` | Public | Server status and version check |
| `GET` | `/api/drives` | Bearer Token | Enumerates physical disks with SMART metrics |
| `POST` | `/api/wipe` | Bearer Token | Initiates block-level wipe job |
| `POST` | `/api/shred` | Bearer Token | Initiates file/directory shred job |
| `POST` | `/api/scan` | Bearer Token | Launches background carving job |
| `POST` | `/api/entropy` | Bearer Token | Calculates chunked Shannon entropy for a device |
| `POST` | `/api/hex` | Bearer Token | Returns hex and ASCII dump of physical sectors |
| `GET` | `/api/diagnostics` | Bearer Token | Executes live in-situ self-test assertions |

---

## 🐍 Python Forensic Pipeline Guide

The Python pipeline handles advanced post-processing, RFC 3161 timestamping, deep file classification, and PDF generation.

### 1. Set Up Virtual Environment
```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r pipeline/requirements.txt
```

### 2. Generate Cryptographic Audit Certificates (`report_gen.py`)
Renders HTML/PDF certificates with strict context sanitization against template injection:
```bash
python3 pipeline/report_gen.py \
    --input ./audit_data.json \
    --template erasure \
    --output ./certificate.html \
    --case-id "CASE-2026-001" \
    --investigator "Lead Forensic Examiner" \
    --tsa-token ./timestamp.tsr
```
*Templates available: `erasure` (destruction certificate), `recovery` (chain of custody for carved files), `custody` (evidence log).*

### 3. File Classifier (`classify.py`)
Classifies carved files using libmagic, calculates SHA-256 hashes, extracts EXIF metadata, and computes perceptual difference hashes (dHash):
```bash
python3 pipeline/classify.py \
    --scan-dir ./recovered_files \
    --output-json ./classified_evidence.jsonl
```

### 4. RFC 3161 Timestamping Client (`timestamp.py`)
Generates court-admissible cryptographic proof of existence via a certified Time-Stamp Authority:
```bash
# Generate RFC 3161 timestamp token for an audit report
python3 pipeline/timestamp.py \
    --hash $(sha256sum ./certificate.html | cut -d' ' -f1) \
    --tsa-url "http://timestamp.digicert.com" \
    --output ./certificate.tsr

# Verify timestamp token integrity
python3 pipeline/timestamp.py \
    --token ./certificate.tsr \
    --verify
```

---

## 🧪 Automated Testing & Quality Assurance

RAZE-CARVE includes a comprehensive, zero-warning test suite comprising **420 automated tests** across Rust core, integration suites, Python pipeline, and Web UI:

```bash
# 1. Run all workspace unit and integration tests (399 Rust tests)
cargo test --workspace

# 2. Run Python forensic pipeline unit tests (16 tests)
python3 pipeline/test_pipeline.py

# 3. Run Web GUI workstation unit tests (5 tests)
cd src-ui && npm test
```

---

## 🛡️ Fail-Safe Safety Architecture

To prevent accidental data loss during high-stress forensic triage, RAZE-CARVE enforces three independent defense layers:

1. **Boot Device Lockdown**: The core wiper cross-references sysfs partition tables and Linux mountpoints (`/proc/mounts`). Any storage device mounted at `/`, `/boot`, `/home`, or containing active swap space is strictly blocked.
2. **Lexical Path Traversal Normalization**: Input paths are resolved and scrubbed of relative traversal tricks (`../../etc/shadow`) before reaching file operations.
3. **CoW (Copy-on-Write) Filesystem Guard**: RAZE-CARVE detects Btrfs, ZFS, and APFS filesystems. Attempting file-level shredding triggers an immediate warning explaining that physical blocks cannot be modified in-place on CoW media, advising block-level device erasure instead.

---

## 📂 Repository Structure

```text
SecureForge/
├── crates/
│   ├── sih149-core/         # Core sanitization, carving, entropy, diagnostics & DB
│   │   └── src/
│   │       ├── carver/      # Structure scanners, signatures, Lua plugins
│   │       ├── wiper/       # Patterns (zero/ones/random/DoD), verification, shredder
│   │       ├── db/          # SQLite WAL audit database & Merkle hash chain
│   │       └── diagnostics.rs # 19 live in-situ assertion self-tests
│   ├── sih149-cli/          # Command-line interface binary (`sih149`)
│   └── sih149-tui/          # Ratatui Terminal UI binary (`sforge`)
├── src-tauri/               # Tauri v2 desktop & axum HTTP REST server
├── src-ui/                  # React 18 + TypeScript + Tailwind CSS UI
├── pipeline/                # Python forensic workers (classify, report_gen, timestamp)
├── docker/                  # Universal Dockerfile and container entrypoints
├── docs/                    # Architectural documents and HOW_TO_RUN.md
└── tests/                   # End-to-end carver, wiper, and pipeline integration tests
```

---

## 📄 License & Team

Developed for **Smart India Hackathon (SIH-149)**.  
Released under the **[MIT License](LICENSE)**.
