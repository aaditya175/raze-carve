# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-12-29

### Added
- **In-Memory Forensic Carver Inspection & Dry-Run Mode**:
  - Added dry-run / in-memory inspection mode across core library, CLI (`--inspect`), TUI (`F5` toggle), and GUI (`Inspect Only` checkbox).
  - Enables zero-write evidentiary scanning without writing carved files to disk until specifically requested.
  - Implemented interactive candidate selection, file filtering, and hex preview in TUI and GUI workstations.
- **Cyber Destination Folder Selector (`FolderSelector.tsx`)**:
  - Implemented custom cyber-themed folder picker with breadcrumbs, quick presets (`Evidence Storage`, `Forensic Export`, `Case Archive`, `Timestamped Case`), and native system folder dialog invocation.
- **Automated Carver GUI Test Suite (`carver_gui.test.mjs`)**:
  - Built comprehensive GUI test suite verifying dry-run configuration, selective file filtering, folder presets, and web workstation event dispatching.
- **In-situ Forensic Diagnostics & Live Assertion Hardening (`diagnostics.rs`)**:
  - Aligned diagnostic assertions with fail-safe E01 container protection policy, verifying raw image seek execution and safe rejection of unparsed EWF streams.
  - Aligned slack-space boundary testing with user-space filesystem security policy, verifying refusal of unsafe file I/O slack overwriting.
  - Added full self-test verification unit test (`test_live_diagnostics_suite_passes_all_assertions`) guaranteeing 100% pass rate across all 23 live assertions.
- **Continuous Background Service & Headless Workstation (`secureforge-service.sh`)**:
  - Added `systemd` user service unit (`secureforge.service`) enabling zero-overhead background API execution and automatic web workstation serving.
  - Implemented persistent API token storage (`~/.config/secureforge/api_token`) with dedicated `/api/auth/token` discovery endpoint for seamless GUI auto-authentication.
- **Overleaf Academic Typography & Separate Evidentiary PDF Reports (`report_gen.py`)**:
  - Upgraded PDF pipeline to use headless Chromium with Overleaf-style serif typography (`EB Garamond`, `Cinzel`, `JetBrains Mono`) and LaTeX `booktabs` formatting.
  - Separated forensic outputs into distinct compliant certificates per NTRO SIH26149 requirements: dedicated **Certificate of Data Sanitization** (NIST SP 800-88 / DoD 5220.22-M) and dedicated **Forensic Recovery Report** (ISO/IEC 17025 / BSA Section 63).

### Fixed
- Fixed E01 diagnostic seeking failure by verifying raw evidence image seeking alongside fail-safe EWF container protection guards.
- Fixed wiper slack space diagnostic failure by asserting refusal of unsafe user-space slack overwriting in accordance with NIST SP 800-88 standards.

## [0.2.1] - 2026-12-25

### Added
- **Filesystem-Aware Metadata Scrubbers (`wiper::metadata`)**:
  - Implemented in-place NTFS `$MFT` record scrubber (`scrub_ntfs_mft_record`) to wipe resident `$DATA` payloads, `$FILE_NAME` timestamps, and cluster runlists, marking records with the corrupted `BAAD` signature.
  - Implemented FAT/VFAT directory entry scrubber (`scrub_fat_directory_entry`, `scrub_fat32_chain_entry`) to zero out filenames, timestamps, cluster addresses, and file sizes, marking entries with the `0xE5` deletion flag.
  - Implemented ext4 inode and directory entry scrubber (`scrub_ext4_inode`, `scrub_ext4_dir_entry`) to zero out extent headers (`0xF30A`), block pointers, timestamps, and unlinked dirent records.
  - Added unified `scrub_filesystem_metadata_buffer` and per-filesystem buffer scrubbers (`scrub_ntfs_metadata_buffer`, `scrub_fat_metadata_buffer`, `scrub_ext4_metadata_buffer`).
- **Native RFC 3161 Message Imprint Verification (`audit::signing`)**:
  - Implemented `verify_rfc3161_token_imprint` in Rust to validate 32-byte SHA-256 message imprints in DER-encoded TimeStampResp tokens without requiring Python or external tools.
  - Added anti-replay freshness nonce (`-nonce`) and strict input hex sanitization in `pipeline/timestamp.py`.
  - Enforced PKIStatus rejection detection in timestamp client response handling.
- **Windowed Variance Smoothing & Zero-Block Consolidation (`entropy_clustering`)**:
  - Added `cluster_entropy_streams_smoothed` with centered moving-window entropy smoothing to eliminate micro-dip stream fragmentation.
  - Computed intra-cluster variance (`variance`) in `EntropyCluster` to differentiate uniform ciphertext from variable compressed data.
  - Implemented `find_zero_block_runs` for contiguous sparse unallocated hole detection.
- **TUI Progress & ETA Hardening (`ui::wipe`, `ui::carver`, `worker.rs`)**:
  - Implemented `calculate_eta_seconds` in `ui::wipe` with fallback to average speed over elapsed runtime for slow media and zero-speed states.
  - Added 70/30 exponential moving average (EMA) smoothing and near-zero time delta guards in `worker.rs`.
  - Clamped carver progress ratios and guarded against NaN percentages in `ui::carver`.

### Fixed
- Fixed potential divide-by-zero and NaN issues in TUI carver progress and wiper ETA calculations.
- Fixed stream fragmentation caused by transient entropy dips in high-entropy compressed streams.
- Fixed silent saving of rejected RFC 3161 timestamp response tokens.

## [0.2.0] - 2026-12-24

### Added
- **NTFS Update Sequence Array (USA) Fixup Unapplication**: Unapplies 2-byte fixup sequence numbers across 512-byte sector boundaries in `filesystem_records`, restoring pristine in-record resident payloads and attribute data without corruption.
- **NTFS Non-Resident Data Run Parsing**: Decodes variable-length packed data runs with signed relative LCN deltas into structured `cluster_pointers` (`Vec<(cluster_count, start_lcn)>`) for non-resident files.
- **VFAT Long File Name (LFN) Reconstruction**: Implemented multi-part reverse-order LFN reassembly with 8.3 checksum matching; recovers full long filenames (e.g. `.docx`, `.xlsx`) and restores the overwritten first character of deleted 8.3 directory entries.
- **NIST SP 800-88 Rev. 1 Section 4.7 Media Tail Boundary Verification**: Enforced mandatory tail boundary checking in `verify_wipe` under sampled verification (<100%), guaranteeing that secondary GPT headers and volume end slack are verified.
- **Configurable Verification Sampling Rate**: Added `--verify-rate <PCT>` to CLI `sforge wipe` with verify rate propagation into tamper-evident audit logs.
- **Compound Archive Sizing & Validation**: Extended structure-aware boundary trimming in `trim_carved_file_boundary` for `docx`, `xlsx`, `pptx`, `apk`, `jar`, `sqlite3`, and `db`.
- **Copy-on-Write (CoW) Filesystem Detection**: Added `detect_cow_filesystem` with parent directory fallback for Btrfs/ZFS, populating `is_cow` in `WipeFileResult` and `ShredFileResult` with operational warnings.
- **Cryptographic Hashchain SQLite Integrity Verification**: Added `get_verified_hashchain` to `CaseDatabase`, validating linear parent hash linkages and detecting database row tampering or out-of-order insertions.
- **Centralized Canonical File Signatures**: Centralized built-in magic signatures in `SignatureDatabase::builtin()` eliminating duplicated definitions across CLI, GUI, and TUI.
- **Unified Lua Plugin Host**: Integrated `LuaPluginHost` into CLI, GUI, and TUI carver workflows with script deduplication and guarded extension dispatch.

### Fixed
- Fixed sector scanner division-by-zero vulnerability when handling zero sector sizes.
- Fixed entropy clustering panic on zero block size and trailing remainder block alignment.
- Fixed Gutmann pass 35 false-positive pattern collision bug in wiper verification.
- Fixed FAT directory timestamp seconds counter truncation.
- Fixed false `Plugin not found` log warnings by returning `Ok(true)` for unhandled file extensions.
- Fixed small-buffer adaptive Shannon entropy thresholds for tail blocks (<64KB) in wipe verification.

### Changed
- Unified carved file record persistence across CLI, GUI, and TUI via `UnifiedCarvedArtifact`.
- Hardened PDF trailer parsing to extract `/Info` metadata dictionary and PNG chunk CRC boundary checks.
- Expanded workspace test coverage to 170 unit and integration tests with zero clippy warnings.

## [0.1.0] - 2026-12-15

### Added
- Project scaffolding and multi-crate workspace structure (`sih149-core`, `sih149-cli`, `sih149-tui`, `secureforge-desktop`).
- Core disk abstraction trait `DiskSource` supporting block devices, raw disk images, and Expert Witness Format (E01) forensic images.
- Multi-pass sanitization engine (Zeros, Ones, DoD 5220.22-M 3-pass & 7-pass, Gutmann 35-pass, NVMe Crypto Erase, ATA Secure Erase).
- Structure-aware file carving engine (JPEG, PNG, PDF, ZIP, SQLite).
- SHA-256 linear hashchain audit engine with RFC 3161 TSA timestamping and PKI digital signatures.
- Ratatui interactive terminal user interface (TUI) and Tauri desktop GUI with React frontend.


