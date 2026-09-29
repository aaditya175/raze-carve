import { CommandSpec } from '../types';

export const COMMANDS_DATA: CommandSpec[] = [
  {
    id: 'sih149-test',
    name: 'test',
    binary: 'sih149',
    summary: 'Run automated hardware and cryptographic self-diagnostics',
    purpose: 'Executes comprehensive internal self-tests across CSPRNG entropy sources, AES-256 block ciphers, BLAKE3 and SHA-256 hash digest engines, and raw block device access permissions.',
    syntax: 'sih149 test [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--verbose', short: '-v', description: 'Display microsecond timing metrics and full test assertion traces' },
      { flag: '--json', description: 'Emit machine-readable JSON telemetry for CI/CD test harness integration' },
    ],
    examples: [
      {
        title: 'Run baseline diagnostic self-test',
        command: 'sih149 test',
        description: 'Validates cryptographic primitives and system privileges before starting sanitization operations.',
        expectedOutput: `[PASS] OsRng CSPRNG entropy quality: H = 7.9998 b/B
[PASS] AES-256-CTR cryptographic stream generator: 1.48 GB/s
[PASS] BLAKE3 streaming digest integrity: 3.12 GB/s
[PASS] SHA-256 validation vector match: verified
[PASS] POSIX O_NOFOLLOW / O_DIRECT raw block access: enabled
✓ 23/23 diagnostic assertions passed (0 failures)`,
      },
    ],
    commonMistakes: [
      'Running without root privileges on systems where /dev/urandom or direct hardware access is restricted.',
    ],
    relatedCommands: ['sih149 demo', 'sih149 info'],
  },
  {
    id: 'sih149-demo',
    name: 'demo',
    binary: 'sih149',
    summary: 'Execute an end-to-end synthetic virtual drive sanitization demo',
    purpose: 'Creates a simulated virtual loop device in memory or temporary storage, populates it with sensitive forensic canary tokens, wipes it according to NIST SP 800-88 Rev 1, and verifies zero-data retention with Shannon entropy calculation.',
    syntax: 'sih149 demo [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--size-mb', short: '-s', description: 'Virtual disk size in Megabytes', defaultValue: '64' },
      { flag: '--method', short: '-m', description: 'Sanitization method (zero, nist-clear, dod-3pass)', defaultValue: 'nist-clear' },
    ],
    examples: [
      {
        title: 'Run full verification demo on a 128MB virtual drive',
        command: 'sih149 demo --size-mb 128',
        description: 'Demonstrates live sanitization, entropy checks, and carving failure verification without touching real physical drives.',
        expectedOutput: `[*] Initializing virtual loop target: 128.00 MB
[*] Injecting 42 canary forensic artifact headers (JPEG, PDF, PNG)...
[*] Initial pre-wipe Shannon entropy: 6.421 b/B
[1/2] Executing NIST SP 800-88 Rev 1 Clear overwrite pass...
      100% [==============================] 128.00 MB @ 342.1 MB/s
[2/2] Calculating post-wipe Shannon entropy over 262,144 sectors...
      Post-wipe Shannon entropy: 0.0000 b/B (100% Zero-Deterministic)
[*] Running signature-based carver over target...
      Carver recovered: 0 files (100% Data Erasure Confirmed)
✓ Demo completed successfully. Zero residual data detected.`,
      },
    ],
    commonMistakes: [
      'Attempting to run on filesystems with insufficient space in /tmp for the temporary loop disk image.',
    ],
    relatedCommands: ['sih149 test', 'sih149 wipe'],
  },
  {
    id: 'sih149-info',
    name: 'info',
    binary: 'sih149',
    summary: 'Enumerate physical drives, partitions, and supported sanitization features',
    purpose: 'Queries kernel sysfs, udev, NVMe identification controllers, and ATA IDENTIFY DEVICE structures to report bus topologies, sector alignments, wear-leveling states, and hardware-level sanitize commands.',
    syntax: 'sih149 info [OPTIONS] [DEVICE]',
    arguments: [
      { name: 'DEVICE', required: false, description: 'Specific device path (e.g. /dev/nvme0n1 or /dev/sdb). Omit to list all.' },
    ],
    options: [
      { flag: '--json', description: 'Output device metadata schema as JSON' },
      { flag: '--sanitize-caps', description: 'Display NVMe Sanitize / ATA Secure Erase firmware capabilities' },
    ],
    examples: [
      {
        title: 'Inspect NVMe drive capabilities',
        command: 'sih149 info /dev/nvme0n1',
        description: 'Discovers whether the drive supports NVMe Crypto Scramble, Block Erase, or Overwrite.',
        expectedOutput: `Path:               /dev/nvme0n1
Model:              Enterprise NVMe Solid-State Drive
Serial:             S5GXNF0R123456
Capacity:           1,000,204,886,016 bytes (1.00 TB)
Logical Sector:     512 bytes | Physical Sector: 4096 bytes
NVMe Sanitize:      Crypto Erase (Supported), Block Erase (Supported)
Wear Indicator:     98% Life Remaining | Status: Healthy
Protected System:   NO (Target is safe for sanitization)`,
      },
    ],
    commonMistakes: [
      'Omitting sudo/root privileges, preventing the kernel from permitting low-level ioctl queries.',
    ],
    relatedCommands: ['sih149 wipe', 'sih149 firmware-erase'],
  },
  {
    id: 'sih149-wipe',
    name: 'wipe',
    binary: 'sih149',
    summary: 'Sanitize a block device using certified algorithmic multi-pass patterns',
    purpose: 'Performs forensically certified media sanitization on magnetic or solid-state media according to NIST SP 800-88 Rev 1 (Clear/Purge), DoD 5220.22-M (3-Pass or 7-Pass), BSI-GS, or Gutmann (35-Pass).',
    syntax: 'sih149 wipe --target <DEVICE> --method <METHOD> [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--target', short: '-t', description: 'Target block device node (e.g. /dev/sdX)', required: true },
      { flag: '--method', short: '-m', description: 'Wipe standard: nist-clear, nist-purge, dod-3pass, dod-7pass, gutmann, zero, random', defaultValue: 'nist-clear' },
      { flag: '--verify', description: 'Run full post-wipe Shannon entropy verification pass', defaultValue: 'true' },
      { flag: '--audit-out', description: 'Path to write signed JSON/PDF sanitization certificate' },
      { flag: '--force', description: 'Bypass interactive confirmation prompt (for headless batch scripts)' },
    ],
    examples: [
      {
        title: 'Execute NIST SP 800-88 Clear on secondary SATA drive',
        command: 'sudo sih149 wipe --target /dev/sdb --method nist-clear --audit-out /var/log/cert_sdb.pdf',
        description: 'Overwrites all addressable sectors with single zero pass, validates entropy, and emits cryptographically signed PDF certificate.',
        expectedOutput: `[!] CAUTION: Target /dev/sdb (500.1 GB) will be irreversibly destroyed.
[*] Confirmed: System drive guard passed (/dev/sdb is not root /).
[Pass 1/1] Overwriting with 0x00 pattern...
           100% [==============================] 500.10 GB @ 184.2 MB/s
[*] Running verification pass (sample interval 1/100 sectors)...
    Shannon Entropy: 0.0000 b/B (Deterministic Zero confirmed)
    Unsanitized sectors: 0
[*] Generating cryptographic audit trail (BLAKE3 signature)...
✓ Sanitization SUCCESSFUL. Audit certificate saved to /var/log/cert_sdb.pdf`,
      },
    ],
    commonMistakes: [
      'Specifying a mounted filesystem partition (e.g. /dev/sdb1) instead of the parent raw block device (/dev/sdb). Unmount all partitions first.',
      'Attempting to wipe the root OS drive. RAZE-CARVE active drive guards will reject the command unless booted from Live ISO.',
    ],
    relatedCommands: ['sih149 firmware-erase', 'sih149 report'],
  },
  {
    id: 'sih149-shred',
    name: 'shred',
    binary: 'sih149',
    summary: 'Forensically obliterate individual files and directory hierarchies',
    purpose: 'Overwrites target files using CSPRNG pseudo-random bytes and bitwise complements, forces synchronous filesystem sync (O_SYNC / fsync), renames filenames to 8 random hexadecimal aliases to purge filesystem journal and directory entry metadata, and unlinks.',
    syntax: 'sih149 shred [OPTIONS] <PATH>...',
    arguments: [
      { name: 'PATH', required: true, description: 'File or directory path to shred.' },
    ],
    options: [
      { flag: '--passes', short: '-p', description: 'Number of overwrite passes', defaultValue: '3' },
      { flag: '--recursive', short: '-r', description: 'Recursively shred all contents of directories' },
      { flag: '--zero', short: '-z', description: 'Final pass with zeroes to conceal shredding pattern' },
    ],
    examples: [
      {
        title: 'Securely destroy sensitive database dump and truncate inode metadata',
        command: 'sih149 shred -p 3 -z /mnt/evidence/passwords.dump',
        description: 'Performs 3 pseudo-random passes, 1 zero pass, scrubs metadata, and unlinks file.',
        expectedOutput: `[1/4] Pass 1/3: CSPRNG Random (H > 7.99 b/B)... done
[2/4] Pass 2/3: Bitwise Inverted pattern... done
[3/4] Pass 3/3: Random Cryptographic fill... done
[4/4] Pass 4/4: Deterministic 0x00 zero flush... done
[*] Scrubbing filesystem journal metadata:
    Renaming inode 849204: passwords.dump -> .0a9f1c2b -> .5e81d4a0 -> .00000000
[*] fsync() and unlinking inode... done
✓ Target shredded permanently.`,
      },
    ],
    commonMistakes: [
      'Shredding files located on copy-on-write (CoW) filesystems like Btrfs or ZFS without understanding that filesystem snapshots or out-of-place block allocations may retain historical extents.',
    ],
    relatedCommands: ['sih149 wipe'],
  },
  {
    id: 'sih149-recover',
    name: 'recover',
    binary: 'sih149',
    summary: 'Carve and recover unallocated disk sectors using magic byte signatures',
    purpose: 'Scans raw storage block devices or forensic raw disk images (RAW/DD/E01) for magic header and footer signatures (JPEG, PNG, PDF, ZIP, DOCX, ELF, MP4, SQLITE) to salvage orphaned or deleted files without relying on file allocation tables.',
    syntax: 'sih149 recover --source <IMAGE_OR_DEVICE> --out-dir <DIR> [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--source', short: '-s', description: 'Source block device or disk image file', required: true },
      { flag: '--out-dir', short: '-o', description: 'Directory to store recovered files', required: true },
      { flag: '--types', description: 'Comma-separated file extensions to target (e.g. jpeg,png,pdf,sqlite)', defaultValue: 'all' },
      { flag: '--plugins-dir', description: 'Path to custom TOML signature definition plugins' },
      { flag: '--max-size-mb', description: 'Maximum file carving size threshold', defaultValue: '100' },
    ],
    examples: [
      {
        title: 'Carve photos and documents from damaged partition image',
        command: 'sih149 recover --source /dev/sdb1 --out-dir ./recovered_files --types jpeg,png,pdf',
        description: 'Reads raw blocks and extracts matching file headers and footers to the destination folder.',
        expectedOutput: `[*] Scanning /dev/sdb1 (16.00 GB) for signatures: [JPEG, PNG, PDF]...
[*] Loaded 18 built-in signatures + 6 TOML plugin rules.
      Progress: 100% [==============================] 16.00 GB @ 210.4 MB/s
[*] Extraction Summary:
    - JPEG images:      142 files (482.1 MB)
    - PNG graphics:      38 files (12.4 MB)
    - PDF documents:     19 files (104.8 MB)
    Total Carved:       199 valid files
✓ Recovery complete. Output stored in ./recovered_files/`,
      },
    ],
    commonMistakes: [
      'Setting --out-dir to the same disk that is currently being carved. Always output carved files to a separate storage drive to prevent overwriting unallocated sectors.',
    ],
    relatedCommands: ['sih149 info'],
  },
  {
    id: 'sih149-firmware-erase',
    name: 'firmware-erase',
    binary: 'sih149',
    summary: 'Trigger hardware-level controller NVMe Sanitize or ATA Secure Erase',
    purpose: 'Sends direct kernel ioctls to onboard flash/disk microcontrollers to execute internal crypto-scramble (invalidating encryption keys in <1 second) or block erase of all NAND cells (including overprovisioned, retired, and wear-leveled areas unreachable by software writes).',
    syntax: 'sih149 firmware-erase --target <DEVICE> --type <TYPE> [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--target', short: '-t', description: 'Target controller node (e.g. /dev/nvme0n1 or /dev/sda)', required: true },
      { flag: '--type', description: 'Sanitize type: crypto (Cryptographic Erase), block (Block Erase), overwrite (Hardware Overwrite)', defaultValue: 'crypto' },
      { flag: '--force', description: 'Bypass confirmation prompts' },
    ],
    examples: [
      {
        title: 'Instant cryptographic erasure of an entire NVMe SSD',
        command: 'sudo sih149 firmware-erase --target /dev/nvme0n1 --type crypto',
        description: 'Instructs the NVMe controller to generate a new internal Media Encryption Key (MEK), rendering all past data unrecoverable in under 1 second.',
        expectedOutput: `[*] Querying NVMe Sanitize Capabilities for /dev/nvme0n1...
    Crypto Erase: SUPPORTED
    Block Erase:  SUPPORTED
[!] CRITICAL WARNING: NVMe Crypto Erase will permanently drop MEK.
[*] Issuing NVMe Sanitize Command (Action: Crypto Scramble)...
[✓] Controller Sanitize Status: Complete (Duration: 0.42 seconds)
[*] Verifying unallocated read return:
    Reading LBA 0..1000: All sectors return 0x00 / deallocated status
✓ Hardware-level Crypto Erase verified successfully.`,
      },
    ],
    commonMistakes: [
      'Attempting NVMe sanitize over USB enclosures that do not support SCSI-to-ATA/NVMe Translation (SAT/NVMe pass-through). Connect the drive directly to M.2 or PCIe.',
    ],
    relatedCommands: ['sih149 wipe', 'sih149 info'],
  },
  {
    id: 'sih149-report',
    name: 'report',
    binary: 'sih149',
    summary: 'Generate tamper-evident, cryptographically signed forensic audit reports',
    purpose: 'Compiles device hardware serials, disk geometry, sanitization logs, operator digital signatures, and BLAKE3 sector verification hashes into tamper-evident NIST SP 800-88 Rev 1 compliance certificates in PDF or JSON.',
    syntax: 'sih149 report --log <AUDIT_LOG_JSON> --output <OUTPUT_PATH> [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--log', short: '-l', description: 'Input audit log generated by wipe or firmware-erase', required: true },
      { flag: '--output', short: '-o', description: 'Output certificate file (.pdf or .json)', required: true },
      { flag: '--operator', description: 'Forensic technician identifier or name' },
      { flag: '--sign-key', description: 'Path to Ed25519 or RSA private key for cryptographic certificate signing' },
    ],
    examples: [
      {
        title: 'Generate verified PDF compliance certificate',
        command: 'sih149 report --log /var/log/razecarve/wipe_sdc.json -o /reports/cert_sdc.pdf --operator "Investigator J. Doe"',
        description: 'Creates a defense-grade PDF compliance document with SHA-256 validation QR code.',
        expectedOutput: `[*] Reading audit log: /var/log/razecarve/wipe_sdc.json
[*] Validating log signature chain: VALID
[*] Compiling forensic certificate for Enterprise SATA Drive 8TB (SN: ZA123456)...
    Method: NIST SP 800-88 Purge
    Pre-wipe Entropy: 7.214 b/B | Post-wipe Entropy: 0.000 b/B
    Verified Sectors: 15,628,053,168 / 15,628,053,168 (100%)
[✓] Report written: /reports/cert_sdc.pdf (SHA-256: d8f3a...91e)`,
      },
    ],
    commonMistakes: [
      'Providing an altered or truncated JSON audit log. RAZE-CARVE report generator enforces strict hash-chain validation.',
    ],
    relatedCommands: ['sih149 wipe'],
  },
  {
    id: 'sforge-tui',
    name: 'sforge',
    binary: 'sforge',
    summary: 'Launch the interactive defense-grade Terminal User Interface (TUI)',
    purpose: 'Provides a real-time, zero-X11 interactive terminal console with live block device topology trees, sector heatmaps, real-time Shannon entropy graphing, interactive wipe wizards, and signature carving monitors.',
    syntax: 'sforge [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--headless', description: 'Run TUI in non-interactive batch capture mode' },
      { flag: '--theme', description: 'Terminal color profile: defense (default), high-contrast, monochrome', defaultValue: 'defense' },
    ],
    examples: [
      {
        title: 'Launch the interactive console',
        command: 'sudo sforge',
        description: 'Opens the full-screen terminal interface with keyboard shortcuts (1-8).',
        expectedOutput: `[ Launches full-screen interactive curses/ratatui console with 8 operational panes ]`,
      },
    ],
    commonMistakes: [
      'Running in terminal emulators smaller than 80x24 characters. Ensure your terminal window is expanded.',
    ],
    relatedCommands: ['sih149 demo', 'sih149 test'],
  },
  {
    id: 'raze-carve-desktop',
    name: 'raze-carve-desktop',
    binary: 'raze-carve-desktop',
    summary: 'Launch the GUI Workstation or Headless REST API Server',
    purpose: 'Starts the forensic workstation web server on port 7878 with SSE (Server-Sent Events) live streaming and opens the native graphical interface.',
    syntax: 'raze-carve-desktop [OPTIONS]',
    arguments: [],
    options: [
      { flag: '--server', description: 'Run headless REST/WebSocket server on background daemon' },
      { flag: '--port', short: '-p', description: 'Network binding port for REST API', defaultValue: '7878' },
      { flag: '--no-browser', description: 'Do not automatically launch graphical browser window' },
    ],
    examples: [
      {
        title: 'Launch headless workstation API daemon',
        command: 'raze-carve-desktop --server --port 7878 --no-browser',
        description: 'Runs backend daemon to service remote web clients or API automation.',
        expectedOutput: `[INFO] RAZE-CARVE Workstation Core v0.1.0 listening on http://0.0.0.0:7878
[INFO] Authentication: Bearer token system active
[INFO] Device polling thread initialized`,
      },
    ],
    commonMistakes: [
      'Attempting to bind to port 7878 when another instance of raze-carve-desktop or razecarve.service is already active.',
    ],
    relatedCommands: ['sforge'],
  },
];
