import { FAQItem } from '../types';

export const FAQ_DATA: FAQItem[] = [
  {
    category: 'Sanitization',
    question: 'How does RAZE-CARVE verify that data was actually destroyed on modern SSDs?',
    answer: 'Standard software overwriting on SSDs only writes to logical block addresses (LBAs), which flash controllers remap across dynamic wear-leveling pools, potentially leaving stale data in unaddressed cells. RAZE-CARVE tackles this through two mechanisms: (1) Native NVMe Sanitize / ATA Secure Erase commands which instruct the flash microcontroller itself to cryptographically erase or flash-block-erase all cells including overprovisioned space, and (2) Post-wipe Shannon entropy validation across sampled LBAs to mathematically verify that data has zero structural remnants.',
  },
  {
    category: 'Sanitization',
    question: 'Does single-pass zeroing comply with NIST SP 800-88 Rev 1?',
    answer: 'Yes. For modern high-density magnetic drives and solid-state drives, NIST SP 800-88 Rev 1 explicitly states that a single full-capacity overwrite pass (Clear) with deterministic zeroes or pseudo-random bits renders data unrecoverable by laboratory magnetic force microscopy or forensic software. Multi-pass methods (such as DoD 5220.22-M 3-pass or 7-pass) are also provided for defense agencies requiring legacy compliance protocols.',
  },
  {
    category: 'Carving',
    question: 'What file formats are supported by the built-in carving engine?',
    answer: 'RAZE-CARVE includes built-in signature parsers for 24+ common file types including JPEG, PNG, GIF, BMP, WebP, PDF, DOCX/XLSX/PPTX, ZIP, TAR, GZ, 7Z, ELF, MP3, MP4, MKV, WAV, and SQLite databases. Additional proprietary, industrial, or scientific file signatures can be loaded dynamically at runtime via simple TOML plugin definition files.',
  },
  {
    category: 'Hardware',
    question: 'Can RAZE-CARVE operate in fully air-gapped, isolated forensic cleanrooms?',
    answer: 'Absolutely. RAZE-CARVE has zero telemetry, zero cloud dependencies, and zero outbound network requirements. The bootable Live ISO runs entirely in volatile RAM, and all binaries are statically self-contained. Audit reports and digital certificates are compiled locally on-disk without calling external services.',
  },
  {
    category: 'Compliance',
    question: 'Are the generated audit reports tamper-evident?',
    answer: 'Yes. Every sanitization event records the physical drive serial number, LBA count, pre-wipe hash, post-wipe entropy, technician operator identifier, and timestamp. The final audit certificate is signed using a BLAKE3 cryptographic hash chain and can be optionally co-signed with an Ed25519 private key.',
  },
];
