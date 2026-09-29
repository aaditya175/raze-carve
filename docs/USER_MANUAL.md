# SecureForge User Manual

## 1. Introduction & Overview
SecureForge is a cross-platform utility for media sanitization and structure-aware recovery. It records locally verifiable SHA-256 audit chains and exposes hardware erase commands where the operating system and drive support them. It does not make a certification, legal-admissibility, or third-party timestamp-verification claim.

## 2. Installation & Quick Start
**Prerequisites:** Rust toolchain (1.75+), Python 3.10+, and administrative privileges (`sudo` / `CAP_SYS_ADMIN`).
```bash
# Clone repository
git clone https://github.com/Normie69K/SecureForge.git
cd SecureForge

# Build workspace binaries (CLI: sforge, TUI: sforge-tui, Desktop: secureforge-desktop)
cargo build --release

# Quick verification: run CLI
./target/release/sforge --help
```

### Quick Commands:
1. **List Block Devices:** `sforge info`
2. **Wipe Media (DoD 3-Pass with 10% Sampled Verification):**
   ```bash
   sudo sforge wipe --device /dev/sdb --method dod3 --verify --verify-rate 10 --yes
   ```
3. **Forensic Recovery from Disk Image:**
   ```bash
   sforge recover --source evidence.raw --output ./carved_output --min-confidence 65
   ```
4. **Inspect Tamper-Evident Audit Report:**
   ```bash
   sforge report --case-id wipe-1703350000
   ```

---

## 3. CLI Reference (`sforge`)

### `sforge info`
Inspects storage topology, geometry, and hardware identifiers.
- `sforge info [--device PATH] [--json]`

### `sforge wipe`
Executes multi-pass block-device erasure. Firmware erase is a separate `firmware-erase` command.
- `sforge wipe [OPTIONS]`
  - `--device <PATH>`: Path to block device (e.g. `/dev/sdb`).
  - `--method <METHOD>`: Sanitization method (`zero`, `random`, `dod3`, `dod7`, `gutmann`). Default: `dod3`.
  - `--verify`: Enable post-wipe read-back verification.
  - `--verify-rate <PCT>`: Sampling rate percentage (`1` to `100`). Default: `10`. Under sampling, the media tail boundary is always verified per NIST SP 800-88 Section 4.7.
  - `--yes`: Bypass confirmation prompt (`ERASE`).
  - `--expert`: Emits a compatibility warning; it never bypasses mounted or system-drive safeguards.
  - `--output-report <PATH>`: Path to write `.json` or `.pdf` audit certificate.

### `sforge recover`
Carves files and extracts residual filesystem records from raw images or unallocated block devices. E01 evidence is detected and rejected unless it has first been exported to a verified raw image by a complete EWF tool.
- `sforge recover [OPTIONS]`
  - `--source <PATH>`: Evidence source (block device or raw disk image).
  - `--output <DIR>`: Directory where carved artifacts will be saved.
  - `--types <EXT1,EXT2,...>`: Comma-delimited list of extensions to recover (e.g. `jpg,png,pdf,docx,sqlite`).
  - `--min-confidence <0-100>`: Minimum confidence percentage threshold. Default: `50`.
  - `--no-structure-check`: Disable deep structural boundary parsers (fallback to magic byte footers).

### `sforge report`
Inspects and exports case database audit logs.
- `sforge report [--list] [--case-id <ID>] [--export <PATH>] [--format pdf|json|html]`

---

## 4. Desktop GUI & Ratatui TUI Guide
- **Interactive TUI (`sforge-tui`):** Terminal UI with drive entropy heatmap, real-time live cancellation, and progress gauges.
- **Desktop GUI (`secureforge-desktop`):** Tauri application with visual disk explorer, live drive health telemetry, structured file preview, and PDF certificate export.
- **Expert Mode:** Protects high-risk operations behind Argon2id password verification, preventing accidental overwrites of OS disks.

---

## 5. Sanitization Methods Reference
| Method | Specification | Passes | Mechanism | Recommended Media |
| :--- | :--- | :--- | :--- | :--- |
| **Zero Fill** | NIST 800-88 Clear | 1 | Single pass of `0x00` | Quick sanitation / SSDs |
| **Ones Fill** | NIST 800-88 Clear | 1 | Single pass of `0xFF` | Alternative Clear pass |
| **DoD 3-Pass** | DoD 5220.22-M | 3 | `0x00` -> `0xFF` -> PRNG | Magnetic hard drives |
| **DoD 7-Pass** | DoD 5220.22-M (ECE) | 7 | 7 alternating passes | High-security magnetic HDDs |
| **Gutmann** | Peter Gutmann (1996) | 35 | Specialized magnetic patterns | Legacy MFM / RLL drives |
| **NVMe Crypto Erase** | NVMe Spec (`Format NVM`) | 1 | Internal key destruction | Modern NVMe SSDs |
| **ATA Secure Erase** | ATA Spec (Security Erase) | 1 | Controller-level block flash | SATA SSDs / HDDs |

---

## 6. Verification & NIST SP 800-88 Rev. 1 Compliance
When `--verify` is specified:
1. **Deterministic Patterns:** Performs exact byte-for-byte read-back verification against the final pass pattern.
2. **Pseudo-Random Passes:** Compares sampled bytes against the exact keyed stream written during the final pass; entropy alone is never treated as overwrite proof.
3. **Mandatory Tail Boundary Inspection:** When sampling (<100%), SecureForge verifies the beginning of media, interior blocks, and the **final addressable tail boundary**. This guarantees that secondary GPT backup tables and partition tail slack are verified.

---

## 7. Filesystem Record Recovery & Long File Names (VFAT LFN)
- **NTFS `$MFT` Inodes:** Recovers deleted resident files and decodes non-resident data runs into cluster pointer ranges with signed relative LCN deltas. Update Sequence Array (USA) fixups are unapplied to restore uncorrupted sector tail bytes.
- **VFAT Long File Names:** Collects multi-part reverse-order LFN entries, validates the Microsoft 8.3 checksum, and reconstructs full long filenames (e.g. `Budget_2026_Final.xlsx`), recovering the first character lost to the `0xE5` deletion marker.

---

## 8. Metadata Limits & Timestamp Tokens
- **Filesystem Metadata Scrubbing:** Standard overwriting targets block payloads, but residual filenames, timestamps, and cluster pointers often remain in directory entries and inode records. SecureForge's metadata scrubbers purge residual records:
  - NTFS: Sanitizes `$FILE_NAME` and `$DATA` resident attributes, wipes runlists, and stamps `BAAD`.
  - FAT: Replaces entry first bytes with `0xE5` and zeroes out filename, timestamp, and cluster fields.
  - ext4: Zeros out inode blocks, extent headers (`0xF30A`), and unlinked dirent records.
- **Timestamp tokens:** A supplied RFC 3161 token is recorded with its hash, but the report generator does not validate TSA signer trust, status, or message imprint and labels the token unverified.

---

## 9. Troubleshooting & Warnings
- **Copy-on-Write (CoW) Filesystems (Btrfs, ZFS):** File shredding is refused because ordinary overwrite cannot prove removal of prior extents or snapshots.
- **Drive Frozen (ATA Security):** If BIOS locks the security state, suspend and resume the system or hot-plug the SATA cable to clear the lock.
- **Unregistered Plugins:** Unregistered file extensions pass through the carving pipeline cleanly without generating missing plugin warnings.

