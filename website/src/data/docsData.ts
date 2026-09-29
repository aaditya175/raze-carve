import { DocSection } from '../types';

export const DOCS_DATA: DocSection[] = [
  {
    id: 'intro',
    title: 'Introduction & Architecture',
    category: 'Getting Started',
    content: `RAZE-CARVE is an enterprise forensic media sanitization and hardware recovery engine engineered in Rust. It addresses the critical requirements of data destruction compliance and file recovery across mechanical hard drives, NVMe SSDs, SATA SSDs, SAS arrays, UFS chips, and eMMC devices.

## Introduction

RAZE-CARVE provides defense-grade data sanitization and forensic recovery capabilities in a single unified platform. Built from the ground up in Rust for maximum performance and memory safety, it interfaces directly with storage device controllers, bypassing OS-level caching and filesystem abstractions to ensure every byte is verifiably written or recovered.

The platform supports compliance with NIST SP 800-88 Rev 1, DoD 5220.22-M, BSI-GS, and other international data destruction standards, while simultaneously providing structure-aware file carving and entropy-based verification.

## Architectural Tenets

Core principles that make RAZE-CARVE reliable, efficient, and forensically sound.

### Zero-Trust Verification
Overwriting blocks is never assumed complete. RAZE-CARVE samples post-wipe and computes Shannon Entropy to verify that no residual patterns remain.

### Controller-Direct Invocations
Bypasses the OS filesystem caching and software translation layer to issue native NVMe Sanitize and ATA Secure Erase commands directly to device controllers.

### Defense-in-Depth File Shredding
Employs synchronous I/O, strict symlink protection, and random 8-pass directory entry renaming to prevent TOCTOU attacks and ensure metadata destruction.

### Autonomous Signature Carving
Features a zero-copy, high-performance scanner capable of salvaging orphaned fragments across 24+ common file types, even on corrupt or unpartitioned disks.

## System Architecture

The RAZE-CARVE Core Architecture follows a modular pipeline design with five interconnected processing stages:

1. **Storage Devices** — HDD, SSD, NVMe, USB, SAS, UFS, and eMMC interfaces provide raw block-level I/O using O_DIRECT and O_SYNC kernel flags.
2. **Sanitization Engine** — Implements NIST 800-88, ATA Secure Erase, zero fill, and random pattern overwrite algorithms.
3. **RAZE-CARVE Core Engine** — Central orchestration hub handling I/O control, forensic processing, verification, and audit logging.
4. **Recovery Engine** — Structure-based and structure-aware file carving for deleted and fragmented data recovery.
5. **Verification & Reporting** — SHA-256 hashing, entropy analysis, audit trail generation, and forensic report compilation.

## Verification & Integrity

Every operation in RAZE-CARVE produces a cryptographically verifiable audit trail. Post-wipe entropy sampling ensures that data destruction is mathematically confirmed, not just assumed from write return codes.

\`\`\`bash
# Verify entropy of a sanitized drive
sih149 verify --target /dev/nvme0n1 --entropy-threshold 0.001
\`\`\`

## Next Steps

- Review the [System Requirements](#/docs?section=requirements) to ensure your hardware is compatible.
- Follow the [Installation & Setup](#/docs?section=installation) guide to get RAZE-CARVE running.
- Complete the [Quick Start Tutorial](#/docs?section=quick-start) for a hands-on introduction.`,
  },
  {
    id: 'requirements',
    title: 'System Requirements',
    category: 'Getting Started',
    content: `### Hardware Requirements
- **CPU**: x86_64 architecture with AES-NI support recommended for 1+ GB/s cryptographic generation.
- **RAM**: 2 GB minimum (4 GB+ recommended when running parallel live file carving).
- **Storage Interface**: Native PCIe/M.2 slots for NVMe controller commands, direct SATA 3.0 / SAS HBA controllers, or USB 3.2 Gen 2 enclosures supporting UAS (USB Attached SCSI) with SCSI-to-ATA/NVMe Translation (SAT).

### Operating Environments
- **Bare-Metal Boot (Recommended)**: RAZE-CARVE Universal Live ISO based on Arch Linux LTS kernel 6.x+.
- **Host Linux**: Ubuntu 22.04 LTS+, Debian 12+, Arch Linux, RHEL 9+, Fedora 40+.
- **Container**: Docker Engine 24.0+ or Podman with rootless/privileged pass-through flags.`,
  },
  {
    id: 'installation',
    title: 'Installation & Setup',
    category: 'Getting Started',
    content: `### Option 1: Standalone Binary (Recommended for Linux Workstations)
Download the pre-compiled binary matching your architecture, verify the SHA-256 hash, and mark it executable:

\`\`\`bash
# Download and verify binary
sha256sum sih149
# Expected: ec8e0b7f9dd93b27e0a2fa64c5cdd49e7601923f217cfc9bd8907f0305537b5c

chmod +x sih149
sudo cp sih149 /usr/local/bin/
\`\`\`

### Option 2: Docker Container (Non-Intrusive Testing)
\`\`\`bash
docker run -it --rm --privileged \\
  --network host \\
  -v /dev:/dev \\
  -v /sys:/sys \\
  normie69k/razecarve:latest sih149 test
\`\`\`

### Option 3: Compile from Rust Source
\`\`\`bash
git clone https://github.com/razecarve/razecarve.git
cd razecarve
cargo build --release
# Artifacts located in target/release/sih149 and target/release/sforge
\`\`\``,
  },
  {
    id: 'quick-start',
    title: 'Quick Start Tutorial',
    category: 'Getting Started',
    content: `Run these three commands to verify your setup safely before performing any real drive sanitization:

### 1. Run Diagnostic Self-Tests
Verify that entropy generators, hashing engines, and kernel permissions are operational:
\`\`\`bash
sih149 test
\`\`\`

### 2. Execute a Simulated Drive Wipe
Run an autonomous dry-run verification demo on a temporary 64MB in-memory loop device:
\`\`\`bash
sih149 demo --size-mb 64
\`\`\`
This injects simulated files, runs a NIST Clear zero-pass, validates entropy, runs the file carver to prove that 0 files survive, and displays the execution summary.

### 3. Enumerate Connected Hardware
List storage topologies and find target drive identifiers:
\`\`\`bash
sudo sih149 info
\`\`\``,
  },
  {
    id: 'workflow-wipe',
    title: 'Drive Sanitization Workflow',
    category: 'Guides',
    content: `RAZE-CARVE enforces safety interlocks to prevent catastrophic accidental destruction of the host operating system.

### Step 1: Identify the Target Drive Node
\`\`\`bash
sudo sih149 info
\`\`\`
Note the target identifier (e.g. \`/dev/sdb\` or \`/dev/nvme1n1\`). Verify the model number, capacity, and serial number.

### Step 2: Unmount Filesystem Partitions
Before writing raw blocks, ensure that no active partitions are mounted:
\`\`\`bash
sudo umount /dev/sdb* 2>/dev/null || true
\`\`\`

### Step 3: Select the Sanitization Standard
- **NIST SP 800-88 Rev 1 Clear**: Single pass of pseudo-random or zero bytes. Appropriate for magnetic and solid state media prior to internal reuse.
  \`\`\`bash
  sudo sih149 wipe --target /dev/sdb --method nist-clear --audit-out /var/log/wipe_sdb.pdf
  \`\`\`
- **DoD 5220.22-M (3-Pass)**: Pass 1: Fixed character (0x00), Pass 2: Inverted complement (0xFF), Pass 3: CSPRNG Random pattern.
  \`\`\`bash
  sudo sih149 wipe --target /dev/sdb --method dod-3pass --audit-out /var/log/wipe_sdb.pdf
  \`\`\`
- **NIST SP 800-88 Purge (Firmware Erase)**: Issues NVMe Crypto Scramble or Block Erase to sanitize overprovisioned and wear-leveled blocks.
  \`\`\`bash
  sudo sih149 firmware-erase --target /dev/nvme1n1 --type crypto
  \`\`\``,
  },
  {
    id: 'workflow-carve',
    title: 'File Carving & Recovery',
    category: 'Guides',
    content: `The RAZE-CARVE carving subsystem parses raw block streams to detect known file header signatures (magic numbers) and trailer boundaries.

### Carving Deleted Files from Raw Disk Image
\`\`\`bash
# Extract images, documents, and archives into target directory
mkdir -p ./recovered_evidence
sih149 recover \\
  --source /dev/sdc1 \\
  --out-dir ./recovered_evidence \\
  --types jpeg,png,pdf,docx,sqlite
\`\`\`

### TOML Custom Plugin Signatures
You can define custom enterprise or proprietary file signatures by placing a TOML file in the plugins directory:

\`\`\`toml
# plugins/signatures/custom_db.toml
name = "Enterprise Database Ledger"
extension = "edb"
header = [0x45, 0x44, 0x42, 0x5F, 0x30, 0x31]
footer = [0x54, 0x45, 0x52, 0x4D]
max_size_bytes = 104857600
\`\`\`
Load plugins during carving with \`--plugins-dir /path/to/plugins\`.`,
  },
  {
    id: 'evidence-verification',
    title: 'Evidence Verification',
    category: 'Guides',
    content: `RAZE-CARVE provides chain-of-custody verification tools for forensic evidence integrity.

### Hash Verification
\`\`\`bash
sih149 hash --target /dev/sdb --algorithm sha256 --output evidence_hash.txt
\`\`\`

### Entropy Analysis
Run post-sanitization entropy analysis to mathematically confirm complete data destruction:
\`\`\`bash
sih149 verify --target /dev/sdb --entropy-threshold 0.001
\`\`\`

### Audit Report Generation
Generate a compliance-ready audit report with timestamps, hash values, and verification results:
\`\`\`bash
sih149 report --target /dev/sdb --format pdf --output sanitization_report.pdf
\`\`\``,
  },
  {
    id: 'cli-reference',
    title: 'CLI Reference',
    category: 'Guides',
    content: `### Core Commands
\`\`\`bash
sih149 test          # Run diagnostic self-tests
sih149 info          # Enumerate connected storage devices
sih149 demo          # Run simulated sanitization demo
sih149 wipe          # Execute drive sanitization
sih149 recover       # Run file carving and recovery
sih149 verify        # Post-sanitization entropy verification
sih149 hash          # Compute cryptographic hashes
sih149 report        # Generate audit reports
sih149 firmware-erase # Issue firmware-level erase commands
\`\`\`

### Global Flags
- \`--verbose\` / \`-v\`: Enable detailed logging output
- \`--quiet\` / \`-q\`: Suppress non-essential output
- \`--dry-run\`: Simulate without writing to disk
- \`--force\`: Bypass safety interlocks (use with caution)
- \`--json\`: Output results in JSON format for scripting`,
  },
  {
    id: 'standards-matrix',
    title: 'Compliance & Forensic Standards',
    category: 'Reference',
    content: `RAZE-CARVE algorithms are mapped to government, military, and industry data sanitization guidelines:

| Standard | Method Identifier | Pass Count | Algorithm Details | Recommended Target |
| :--- | :--- | :--- | :--- | :--- |
| **NIST SP 800-88 Rev 1 Clear** | \`nist-clear\` | 1 | Zero or CSPRNG pseudo-random overwrite with readback validation | General media reuse |
| **NIST SP 800-88 Rev 1 Purge** | \`nist-purge\` | Hardware | NVMe Crypto Erase / ATA Enhanced Secure Erase | High-assurance decommissioning |
| **DoD 5220.22-M** | \`dod-3pass\` | 3 | 0x00, 0xFF, Random + Verification pass | Magnetic HDD / Defense media |
| **DoD 5220.22-M (ECE)** | \`dod-7pass\` | 7 | 0x55, 0xAA, Random, 0x92, 0x49, 0x24, Random | Classified magnetic drives |
| **BSI-GS (German BSI)** | \`bsi-gs\` | 2 | Random sequence followed by deterministic inverse | Enterprise European compliance |
| **Gutmann 35-Pass** | \`gutmann\` | 35 | 4 random passes, 27 MFM/RLL magnetic encoding patterns, 4 random passes | Legacy magnetic MFM/RLL drives |`,
  },
  {
    id: 'supported-devices',
    title: 'Supported Devices',
    category: 'Reference',
    content: `### Storage Interface Support Matrix

| Interface | Protocol | Max Throughput | Controller Access | Firmware Erase |
| :--- | :--- | :--- | :--- | :--- |
| **NVMe** | PCIe 3.0/4.0/5.0 | 7+ GB/s | Direct ioctl | ✅ Crypto/Block Erase |
| **SATA** | AHCI | 600 MB/s | ATA passthrough | ✅ Secure Erase |
| **SAS** | SCSI | 1.2 GB/s | SCSI CDB | ✅ Format Unit |
| **USB** | UAS/BOT | 1.25 GB/s | SAT translation | ⚠️ Bridge dependent |
| **UFS** | MIPI M-PHY | 2.9 GB/s | SCSI UFS | ✅ Purge/Erase |
| **eMMC** | MMC 5.1 | 400 MB/s | MMC ioctl | ✅ Secure Trim |

### Tested Storage Classes
- High-Performance PCIe 4.0/5.0 NVMe M.2 & U.2 Modules
- Enterprise High-Density SATA 3.0 Solid-State & Magnetic Disks
- Dual-Port Enterprise SAS 12Gb/s Storage Arrays
- Embedded UFS 3.1 & eMMC 5.1 Flash Memory Chips
- UAS / SAT Encapsulated Direct-Attached Storage Bridges`,
  },
  {
    id: 'api-reference',
    title: 'API Reference',
    category: 'Reference',
    content: `### Rust Library API

RAZE-CARVE exposes a public Rust API for integration into custom forensic workflows:

\`\`\`rust
use razecarve::{Device, SanitizeMethod, Config};

let device = Device::open("/dev/nvme0n1")?;
let config = Config::new()
    .method(SanitizeMethod::NistClear)
    .verify(true)
    .audit_path("/var/log/audit.pdf");

device.sanitize(&config)?;
\`\`\`

### REST API (Desktop Application)

The Tauri desktop application exposes a local REST API on port 9149:

\`\`\`bash
# List connected devices
curl http://localhost:9149/api/devices

# Start sanitization job
curl -X POST http://localhost:9149/api/sanitize \\
  -H "Content-Type: application/json" \\
  -d '{"target": "/dev/sdb", "method": "nist-clear"}'
\`\`\``,
  },
  {
    id: 'command-line-interface',
    title: 'Command Line Interface',
    category: 'Reference',
    content: `### sih149 — CLI Pipeline Engine

The primary command-line interface for all RAZE-CARVE operations.

\`\`\`bash
sih149 [COMMAND] [OPTIONS] [FLAGS]
\`\`\`

### Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| \`test\` | Run diagnostic self-tests | \`sih149 test\` |
| \`info\` | Enumerate storage devices | \`sudo sih149 info\` |
| \`wipe\` | Sanitize a target drive | \`sudo sih149 wipe --target /dev/sdb\` |
| \`recover\` | Carve files from raw media | \`sih149 recover --source /dev/sdc1\` |
| \`verify\` | Entropy verification | \`sih149 verify --target /dev/sdb\` |
| \`demo\` | Simulated sanitization | \`sih149 demo --size-mb 64\` |

### sforge — Terminal UI

The Ratatui-powered terminal interface for interactive operation:
\`\`\`bash
sudo sforge
\`\`\``,
  },
  {
    id: 'building',
    title: 'Compiling, Testing & Deploying',
    category: 'Development',
    content: `### Toolchain Setup
Ensure you have the latest stable Rust toolchain:
\`\`\`bash
rustup update stable
rustup target add x86_64-unknown-linux-gnu
\`\`\`

### Running the Test Suites
RAZE-CARVE maintains an extensive test suite verifying raw disk simulation, error injection, signature parsing, and cryptographic engines:

\`\`\`bash
# Run all unit and integration tests
cargo test --workspace

# Run security and clippy lints
cargo clippy --all-targets -- -D warnings

# Run formatting audit
cargo fmt -- --check
\`\`\`

### Building Release Binaries
\`\`\`bash
cargo build --release --workspace
\`\`\`
The compiled binaries are placed in \`target/release/\`:
- \`sih149\`: CLI and batch pipeline engine
- \`sforge\`: Ratatui-powered terminal interface
- \`secureforge-desktop\`: Tauri and embedded Axum server`,
  },
  {
    id: 'contributing',
    title: 'Contributing',
    category: 'Development',
    content: `### Contribution Guidelines

1. **Fork the repository** and create a feature branch from \`main\`.
2. **Write tests** for all new functionality — minimum 80% coverage required.
3. **Follow Rust conventions** — run \`cargo fmt\` and \`cargo clippy\` before submitting.
4. **Sign your commits** — GPG-signed commits are required for security-critical code.
5. **Submit a pull request** with a clear description of changes and motivation.

### Code of Conduct
RAZE-CARVE follows the Contributor Covenant Code of Conduct. All contributors are expected to maintain professional, inclusive, and respectful communication.

### Security Disclosures
Report security vulnerabilities privately via the Security Advisory process. Do not open public issues for security-sensitive bugs.`,
  },
  {
    id: 'architecture-deep-dive',
    title: 'Architecture Deep Dive',
    category: 'Development',
    content: `### Module Architecture

RAZE-CARVE is organized into the following Rust workspace crates:

\`\`\`
secureforge/
├── crates/
│   ├── sf-core/          # Core engine, I/O orchestration
│   ├── sf-sanitize/      # Sanitization algorithms
│   ├── sf-carve/         # File carving engine
│   ├── sf-verify/        # Entropy & hash verification
│   ├── sf-device/        # Device detection & management
│   └── sf-audit/         # Audit trail & report generation
├── sih149/               # CLI binary
├── sforge/               # TUI binary
└── desktop/              # Tauri desktop application
\`\`\`

### Data Flow Pipeline
1. Device enumeration and capability detection
2. Safety interlock verification (mount checks, permission validation)
3. Raw I/O channel establishment (O_DIRECT, O_SYNC)
4. Algorithm execution (sanitize/carve/verify)
5. Post-operation entropy sampling and hash computation
6. Audit report generation and signing`,
  },
  {
    id: 'troubleshooting',
    title: 'Troubleshooting Guide',
    category: 'Troubleshooting',
    content: `### 1. "Device or resource busy" (EBUSY)
**Cause**: The block device has active mounts, an active LVM volume group, or an open device-mapper / LUKS container.
**Solution**:
\`\`\`bash
sudo umount /dev/sdX* 2>/dev/null
sudo vgchange -an 2>/dev/null
sudo cryptsetup close <name> 2>/dev/null
\`\`\`

### 2. "Operation not permitted" (EPERM)
**Cause**: Raw disk access (\`/dev/sdX\`, \`/dev/nvmeXn1\`) requires kernel-level write authority.
**Solution**: Prepend commands with \`sudo\` or verify that the container is launched with \`--privileged\`.

### 3. NVMe Sanitize rejected over USB Bridge
**Cause**: Cheap USB-to-NVMe enclosures do not support the SCSI-to-NVMe Translation (SNT) specification, rejecting raw ioctl pass-through.
**Solution**: Connect the NVMe drive directly to an M.2 slot or PCIe host adapter.

### 4. CoW Filesystem Snapshot Retention (Btrfs / ZFS)
**Limitation**: Individual file shredding (\`sih149 shred\`) on copy-on-write filesystems can leave historical blocks in prior snapshots. For absolute cryptographic guarantees on CoW filesystems, perform whole-volume sanitization (\`sih149 wipe\`).`,
  },
  {
    id: 'known-limitations',
    title: 'Known Limitations',
    category: 'Troubleshooting',
    content: `### Current Limitations

- **macOS / Windows**: Native block device access is not supported. Use the Docker container or Live ISO for cross-platform operation.
- **Wear Leveling**: Flash storage (SSD/NVMe) may retain data in overprovisioned cells that are inaccessible via standard I/O. Use firmware-level erase (Purge) for complete sanitization.
- **Network Attached Storage**: NAS devices with proprietary controllers may not expose raw block access. Disconnect drives and attach directly for guaranteed sanitization.
- **Hardware Encryption**: Self-encrypting drives (SEDs) with TCG Opal may require vendor-specific unlock sequences before sanitization commands are accepted.`,
  },
  {
    id: 'faq',
    title: 'FAQ',
    category: 'Troubleshooting',
    content: `### Frequently Asked Questions

**Q: Is RAZE-CARVE certified for government use?**
A: RAZE-CARVE implements NIST SP 800-88 Rev 1 algorithms. Individual deployment certification depends on your organization's accreditation process.

**Q: Can RAZE-CARVE recover files after sanitization?**
A: By design, properly sanitized drives should yield zero recoverable files. The carving engine is intended for pre-sanitization recovery of evidence, not post-wipe recovery.

**Q: What's the difference between Clear and Purge?**
A: Clear uses software-based overwriting accessible to standard I/O. Purge uses firmware-level commands that sanitize areas inaccessible to software (overprovisioned cells, wear-leveled blocks).

**Q: Does RAZE-CARVE support remote operation?**
A: The desktop application exposes a local REST API. For remote operation, combine with SSH tunneling or deploy within a container orchestration platform.

**Q: How long does sanitization take?**
A: Speed depends on drive capacity, interface bandwidth, and sanitization method. A single-pass NIST Clear on a 2TB NVMe drive typically completes in under 3 minutes.`,
  },
];
