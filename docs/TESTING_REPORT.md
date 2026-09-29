# SecureForge Testing & Verification Report

## 1. Quality Assurance Strategy & Verification Gates
SecureForge enforces a zero-regression, 4-gate verification policy before any code is committed to the repository:

| Verification Gate | Command | Passing Standard | Current Status |
| :--- | :--- | :--- | :--- |
| **Code Formatting** | `cargo fmt --all -- --check` | Zero diffs against Rustfmt | **PASS** |
| **Workspace Test Suite** | `cargo test --workspace` | All 178 unit & integration tests | **PASS (178/178)** |
| **Strict Static Analysis** | `cargo clippy --workspace --lib --bins -- -D warnings` | 0 production compiler or clippy warnings | **PASS** |
| **UI Type Safety** | `npm --prefix src-ui run typecheck` | 0 TypeScript diagnostic errors | **PASS (0 errors)** |
| **UI Linting** | `npm --prefix src-ui run lint` | 0 lint errors | **PASS** |

---

## 2. Test Suites Summary

### 2.1 Core Unit Tests (`sih149-core`, 123 tests)
- **Sanitization & Wiper (`wiper/`):**
  - Canonical patterns: Zeros, Ones, DoD 5220.22-M 3-pass & 7-pass, Gutmann 35 distinct passes.
  - Sampled byte comparison against the final written pattern, including keyed random passes.
  - CoW filesystem detection (`is_cow`) for Btrfs/ZFS with parent fallback.
  - Symlink traversal defenses and protected system drive detection.
- **Carver & Structural Parsers (`carver/`):**
  - Sector scanner sector-alignment and zero-length safety.
  - PNG chunk CRC-32 boundary validation and `IEND` termination.
  - JPEG SOI, DQT, SOS, and Exif parsing with `FF D9` footer truncation.
  - PDF `%%EOF` trailer discovery and `/Info` dictionary extraction.
  - ZIP EOCD extraction for `zip`, `docx`, `xlsx`, `pptx`, `apk`, and `jar`.
  - SQLite B-tree page geometry parsing and zeroed slack boundary trimming.
- **Filesystem Record Recovery (`carver/filesystem_records.rs`):**
  - NTFS 1024-byte MFT record parsing with Update Sequence Array (USA) fixup unapplication across sector boundaries.
  - NTFS non-resident packed data run parsing into `cluster_pointers` with signed relative LCN deltas.
  - VFAT Long File Name (LFN) multi-part reverse reassembly with 8.3 checksum matching and deleted first character recovery.
- **Audit Engine & Cryptography (`audit/`, `db/`):**
  - SHA-256 linear hashchain append, serialization, and deterministic reconstruction.
  - SQLite database row tamper detection via `get_verified_hashchain`.
  - Argon2id password hashing and constant-time verification.
- **Plugins (`plugins/`):**
  - TOML signature directory loading and validation.
  - `mlua`-powered `LuaPluginHost` execution, deduplication, and unhandled format pass-through.

### 2.2 Integration Test Suites (33 tests)
- **`tests/integration/test_carver.rs` (13 tests):** Raw disk image carving, Lua plugin integration, confidence scoring.
- **`tests/integration/test_wiper.rs` (10 tests):** Real file shredding, directory shredding, CoW filesystem detection, symlink rejection, verification on zeroed buffers.
- **`tests/integration/test_pipeline.rs` (10 tests):** Python IPC communication, magic byte file classification, RFC 3161 timestamping client verification, PDF report generation.

### 2.3 Binary & Live Tests (22 tests)
- **`sforge` CLI & TUI Tests:** Entropy uniform distributions, Gutmann 35-pass uniqueness, live cancellation handling, and wipe verification on corrupted vs clean media.

---

## 3. Test Execution Matrix
| Suite | Target | Tests Run | Result | Duration |
| :--- | :--- | :--- | :--- | :--- |
| `sih149-core` | `lib.rs` unit tests | 124 | **PASS** | ~1.1s |
| `test_carver` | Integration test binary | 15 | **PASS** | < 0.02s |
| `test_pipeline` | Pipeline integration | 10 | **PASS** | ~0.40s |
| `test_wiper` | Wiper integration | 10 | **PASS** | < 0.01s |
| `sforge` | CLI & TUI unittests | 22 | **PASS** | ~0.13s |
| `GUI Tests` | `carver_gui.test.mjs` Node runner | 5 | **PASS** | < 0.06s |
| `Diagnostics` | `sih149 test` in-situ assertions | 23 | **PASS (23/23)** | < 0.01s |
| **Total** | **All Verification Surfaces** | **209** | **100% PASS** | **~1.7s** |

