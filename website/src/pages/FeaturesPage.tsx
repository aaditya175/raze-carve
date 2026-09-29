import React, { useState } from 'react';
import { RoutePath } from '../types';
import {
  HardDrive,
  FileSearch,
  ShieldCheck,
  ArrowRight,
  FileText,
  Hash,
  Folder,
  Zap,
  Check,
  Terminal,
  Activity,
  Cpu,
  Lock,
  Play,
  Download,
  RefreshCw,
  Layers,
  Database,
  CheckCircle2,
  Sliders,
  Award,
  Copy
} from 'lucide-react';
import SplitText from '../components/SplitText';

interface FeaturesPageProps {
  onNavigate: (path: RoutePath) => void;
}

export const FeaturesPage: React.FC<FeaturesPageProps> = ({ onNavigate }) => {
  // Active Layer selection in Section 2 (0: Device I/O, 1: Sanitization, 2: Recovery, 3: Verification)
  const [activeLayer, setActiveLayer] = useState<number>(0);
  const [copied, setCopied] = useState<boolean>(false);

  const handleCopyCli = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const layers = [
    {
      id: 0,
      num: '01',
      layerTag: 'LAYER 01',
      cardIcon: <HardDrive className="w-5 h-5 text-slate-800" />,
      title: 'Device I/O Layer',
      cardSubtitle: 'Direct device control using low-level interfaces.',
      subtitle: 'Direct access to storage devices using low-level, OS-bypass interfaces.',
      description:
        'RAZE-CARVE communicates directly with storage devices using POSIX kernel I/O (O_DIRECT, O_SYNC) and raw sector descriptors, bypassing the OS page cache to ensure every byte is read and written with forensic precision.',
      image: '/layer-01-nvme.jpg',
      imageTopLabel: ['SATA / NVME / USB', 'READ / WRITE / IDENTIFY'],
      imageBottomLabel: ['DIRECT I/O', 'BYPASS'],
      pills: [
        {
          icon: <Database className="w-4 h-4 text-[#E31B23]" />,
          title: 'Raw block access',
          sub: 'No OS cache'
        },
        {
          icon: <Cpu className="w-4 h-4 text-[#E31B23]" />,
          title: 'Multi-interface',
          sub: 'SATA, NVMe, USB'
        },
        {
          icon: <Lock className="w-4 h-4 text-[#E31B23]" />,
          title: 'HPA/DCO support',
          sub: 'Access hidden areas'
        },
        {
          icon: <CheckCircle2 className="w-4 h-4 text-[#E31B23]" />,
          title: 'Verified I/O',
          sub: 'Integrity checked'
        }
      ],
      cli: {
        command: 'sforge device inspect --target /dev/nvme0n1 --bypass-cache --verify-caps',
        lines: [
          { type: 'cmd', text: '$ sforge device inspect --target /dev/nvme0n1 --bypass-cache --verify-caps' },
          { type: 'init', tag: '[INIT]', text: 'Opening /dev/nvme0n1 ...' },
          { type: 'pass', tag: '[PASS]', text: 'O_DIRECT enabled' },
          { type: 'pass', tag: '[PASS]', text: 'O_SYNC enabled' },
          { type: 'info', tag: '[INFO]', text: 'Device: Enterprise NVMe SSD 2TB' },
          { type: 'info', tag: '[INFO]', text: 'Sector size: 512 bytes' },
          { type: 'info', tag: '[INFO]', text: 'Capacity: 2,000,398,934,016 bytes (2 TB)' },
          { type: 'ready', tag: '[READY]', text: 'Device ready for forensic operations' }
        ]
      },
      spec: {
        devName: '/dev/nvme0n1',
        devModel: 'Enterprise PCIe Gen4 SSD 2TB',
        badge: 'ONLINE',
        items: [
          { label: 'Interface', val: 'NVMe (PCIe 4.0 x4)' },
          { label: 'Model', val: 'Direct I/O NVMe Controller' },
          { label: 'Capacity', val: '2,000,398,934,016 bytes' },
          { label: 'Sector Size', val: '512 bytes' },
          { label: 'HPA / DCO', val: 'Not detected' },
          { label: 'Cache Policy', val: 'O_DIRECT (bypass OS)' }
        ]
      },
      supportedInterfaces: 'SATA / NVMe / USB / ATA / SCSI'
    },
    {
      id: 1,
      num: '02',
      layerTag: 'LAYER 02',
      cardIcon: <Zap className="w-5 h-5 text-slate-800" />,
      title: 'Sanitization Layer',
      cardSubtitle: 'Controller-level erase and multi-pass methods.',
      subtitle: 'Firmware controller eradication and cryptographic multi-pass sanitization.',
      description:
        'Executes hardware-level Purge and Clear commands including NVMe Cryptographic Erase (destroying media encryption keys in < 1s) and NIST SP 800-88 Rev 1 multi-pass pseudorandom overwrites, bypassing the Flash Translation Layer.',
      image: '/layer-02-sanitizer.jpg',
      imageTopLabel: ['NIST SP 800-88', 'DOD 5220.22-M PURGE'],
      imageBottomLabel: ['CONTROLLER LEVEL', 'ERADICATOR'],
      pills: [
        {
          icon: <Zap className="w-4 h-4 text-[#E31B23]" />,
          title: 'NVMe Crypto Erase',
          sub: '< 1 sec key wipe'
        },
        {
          icon: <ShieldCheck className="w-4 h-4 text-[#E31B23]" />,
          title: 'NIST & DoD Standards',
          sub: 'SP 800-88 / 5220.22-M'
        },
        {
          icon: <Sliders className="w-4 h-4 text-[#E31B23]" />,
          title: 'FTL Bypass Mode',
          sub: 'Overprovisioning wipe'
        },
        {
          icon: <CheckCircle2 className="w-4 h-4 text-[#E31B23]" />,
          title: 'Zero Verification',
          sub: '100% Flatline verified'
        }
      ],
      cli: {
        command: 'sforge sanitize --target /dev/nvme0n1 --method nist-purge --verify-full',
        lines: [
          { type: 'cmd', text: '$ sforge sanitize --target /dev/nvme0n1 --method nist-purge --verify-full' },
          { type: 'init', tag: '[INIT]', text: 'Targeting controller /dev/nvme0n1 (2TB NVMe)' },
          { type: 'pass', tag: '[AUTH]', text: 'Hardware crypto-erase authorization granted' },
          { type: 'pass', tag: '[PASS]', text: 'AES-256 media encryption key destroyed in 340ms' },
          { type: 'info', tag: '[WIPE]', text: 'Executing NIST Clear overwrite pass (0x00 pattern)' },
          { type: 'info', tag: '[PROG]', text: 'Overwritten 3,907,029,168 sectors (100% complete)' },
          { type: 'pass', tag: '[PASS]', text: 'Post-wipe entropy verification: 0.000 b/B' },
          { type: 'ready', tag: '[DONE]', text: 'Sanitization certificate generated & signed' }
        ]
      },
      spec: {
        devName: 'Eradicator X-1 Engine',
        devModel: 'NIST SP 800-88 Controller',
        badge: 'PURGED',
        items: [
          { label: 'Sanitization Method', val: 'NVMe Crypto + NIST Purge' },
          { label: 'Standards', val: 'NIST 800-88 / DoD 5220.22-M' },
          { label: 'Key Destruction', val: '340ms (AES-256 Media Key)' },
          { label: 'Total LBAs Cleared', val: '3,907,029,168 Sectors' },
          { label: 'Entropy Verification', val: '0.000 b/B (Flatline Zero)' },
          { label: 'Certificate Hash', val: 'SHA-256 Merkle Sealed' }
        ]
      },
      supportedInterfaces: 'NIST CLEAR / PURGE / CRYPTO ERASE / ATA SECURE ERASE'
    },
    {
      id: 2,
      num: '03',
      layerTag: 'LAYER 03',
      cardIcon: <FileSearch className="w-5 h-5 text-slate-800" />,
      title: 'Recovery Layer',
      cardSubtitle: 'Signature-based file carving for deleted and fragmented data.',
      subtitle: 'Signature-based and structure-aware carving for deleted and fragmented data.',
      description:
        'Rebuilds orphaned, formatted, and fragmented files directly from raw unallocated sectors. Analyzes internal syntax trees for PNG, PDF, ZIP, and SQLite databases without relying on damaged master file tables or corrupted metadata.',
      image: '/layer-03-carver.jpg',
      imageTopLabel: ['STRUCTURE PARSER', 'RAW DATA STREAM'],
      imageBottomLabel: ['CHUNK ANALYSIS', 'ZERO-COPY BUFFER'],
      pills: [
        {
          icon: <FileSearch className="w-4 h-4 text-[#E31B23]" />,
          title: '140+ File Formats',
          sub: 'Extensible TOML DB'
        },
        {
          icon: <Layers className="w-4 h-4 text-[#E31B23]" />,
          title: 'Structure Validation',
          sub: 'PNG, PDF, ZIP, SQLite'
        },
        {
          icon: <Database className="w-4 h-4 text-[#E31B23]" />,
          title: 'Zero-Copy Engine',
          sub: 'Circular ring buffer'
        },
        {
          icon: <CheckCircle2 className="w-4 h-4 text-[#E31B23]" />,
          title: 'Chunk Coherence',
          sub: '99.8% recovery accuracy'
        }
      ],
      cli: {
        command: 'sforge carver scan --source /dev/sdb --output /evidence/recovered --deep',
        lines: [
          { type: 'cmd', text: '$ sforge carver scan --source /dev/sdb --output /evidence/recovered --deep' },
          { type: 'init', tag: '[INIT]', text: 'Scanning unallocated disk blocks on /dev/sdb ...' },
          { type: 'pass', tag: '[MATCH]', text: '42 PNG headers detected (IHDR->IEND verified)' },
          { type: 'pass', tag: '[MATCH]', text: '14 PDF documents reconstructed (xref validated)' },
          { type: 'info', tag: '[PARSE]', text: '8 SQLite database tables extracted with headers' },
          { type: 'info', tag: '[RATE]', text: 'Throughput: 1.48 GB/s (Zero-copy DMA streaming)' },
          { type: 'pass', tag: '[PASS]', text: 'CRC-32 checksums verified on 64/64 recovered files' },
          { type: 'ready', tag: '[READY]', text: 'Recovery completed: 64 files saved to /evidence' }
        ]
      },
      spec: {
        devName: 'DFRX-C1000 Carver Engine',
        devModel: 'Structure-Aware File Reconstruction',
        badge: 'RECOVERED',
        items: [
          { label: 'Supported Signatures', val: '140+ Formats (TOML DB)' },
          { label: 'Validation Engine', val: 'Internal Syntax & CRC-32' },
          { label: 'Memory Architecture', val: 'Zero-Copy Circular Buffer' },
          { label: 'Processed Sectors', val: '1,953,525,168 LBAs' },
          { label: 'Files Extracted', val: '64 Validated Artifacts' },
          { label: 'Extraction Accuracy', val: '99.8% Chunk Coherence' }
        ]
      },
      supportedInterfaces: 'RAW / E01 / DD / NTFS / EXT4 / FAT32 / EXFAT / APFS'
    },
    {
      id: 3,
      num: '04',
      layerTag: 'LAYER 04',
      cardIcon: <ShieldCheck className="w-5 h-5 text-slate-800" />,
      title: 'Verification Layer',
      cardSubtitle: 'Cryptographic hashing, entropy analysis and audit logging.',
      subtitle: 'Cryptographic hashing, entropy analysis and tamper-evident audit logging.',
      description:
        'Mathematical validation engine that calculates discrete Shannon entropy H(X) across sector samples to guarantee 0.000 b/B flatline on wiped storage, generating SHA-256 audit reports with RFC 3161 digital timestamping.',
      image: '/layer-04-verification.jpg',
      imageTopLabel: ['SHA-256 MERKLE ROOT', 'RFC 3161 TIMESTAMP'],
      imageBottomLabel: ['SHANNON ENTROPY', 'FLATLINE 0.000 b/B'],
      pills: [
        {
          icon: <Activity className="w-4 h-4 text-[#E31B23]" />,
          title: 'Shannon Entropy',
          sub: '0.000 b/B flatline check'
        },
        {
          icon: <ShieldCheck className="w-4 h-4 text-[#E31B23]" />,
          title: 'RFC 3161 Timestamp',
          sub: 'Cryptographically sealed'
        },
        {
          icon: <Hash className="w-4 h-4 text-[#E31B23]" />,
          title: 'Merkle Tree Ledger',
          sub: 'SHA-256 chain of custody'
        },
        {
          icon: <Award className="w-4 h-4 text-[#E31B23]" />,
          title: 'Court-Admissible',
          sub: 'Tamper-evident JSON/PDF'
        }
      ],
      cli: {
        command: 'sforge entropy scan --target /dev/nvme0n1 --verify-flatline --rfc3161-seal',
        lines: [
          { type: 'cmd', text: '$ sforge entropy scan --target /dev/nvme0n1 --verify-flatline --rfc3161-seal' },
          { type: 'init', tag: '[INIT]', text: 'Sampling 2,000,398,934,016 bytes across all LBAs ...' },
          { type: 'pass', tag: '[PASS]', text: 'Discrete Shannon Entropy H(X): 0.0000 b/B' },
          { type: 'pass', tag: '[PASS]', text: 'Uniformity test: 100% zero-byte flatline verified' },
          { type: 'info', tag: '[HASH]', text: 'SHA-256: 3f2a6e9c4d0b1e7a8f9c2d3e4b5a6f7e8d9c...' },
          { type: 'info', tag: '[TSA]', text: 'RFC 3161 TSA authority signed token: VERIFIED' },
          { type: 'pass', tag: '[PASS]', text: 'Evidence certificate generated: /reports/audit.pdf' },
          { type: 'ready', tag: '[SEALED]', text: 'Forensic ledger cryptographically sealed' }
        ]
      },
      spec: {
        devName: 'HWV-256 Validator',
        devModel: 'Shannon Entropy & Cryptographic Ledger',
        badge: 'VERIFIED',
        items: [
          { label: 'Calculated Entropy', val: '0.0000 b/B (Flatline Zero)' },
          { label: 'Sampling Resolution', val: '100% LBA Coverage' },
          { label: 'Timestamp Standard', val: 'RFC 3161 TSA Sealed' },
          { label: 'Hash Algorithm', val: 'SHA-256 & SHA-512' },
          { label: 'Evidence Ledger', val: 'Merkle Tree Hash Chain' },
          { label: 'Audit Status', val: 'Court-Admissible Evidence' }
        ]
      },
      supportedInterfaces: 'RFC 3161 / NIST SP 800-88 / ISO/IEC 27037 / FIPS 140-3'
    }
  ];

  const currentLayer = layers[activeLayer];

  return (
    <div className="w-full bg-white text-[#101114] font-sans selection:bg-[#E31B23] selection:text-white">
      {/* ========================================================================= */}
      {/* SECTION 1: CORE ENGINE HERO (ONE WHOLE SCREEN VIEWPORT)                   */}
      {/* ========================================================================= */}
      <section className="w-full min-h-[calc(100vh-68px)] bg-white flex flex-col justify-between">
        <div className="max-w-[1536px] w-full mx-auto px-4 sm:px-8 lg:px-12 py-10 sm:py-16 flex-1 flex flex-col justify-between">
          
          {/* Main 2-Column Hero Grid (Left Copy + Right Architecture Diagram / Visual) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 xl:gap-14 items-center my-auto">
            
            {/* Left Column (~42% Width on Desktop) */}
            <div className="lg:col-span-5 space-y-6">
              {/* Eyebrow: Red line + CORE ENGINE */}
              <div className="flex items-center gap-3">
                <span className="w-7 h-[2px] bg-[#E31B23] inline-block rounded-full" />
                <span className="text-xs font-mono font-bold tracking-[0.2em] text-[#E31B23] uppercase">
                  CORE ENGINE
                </span>
              </div>

              {/* Main Headline with SplitText Letter Animation */}
              <div className="space-y-1 sm:space-y-1.5">
                <SplitText
                  text="The engine behind"
                  tag="h1"
                  className="text-4xl sm:text-5xl lg:text-[48px] xl:text-[54px] font-black text-[#101114] tracking-tight leading-[1.08] block"
                  delay={30}
                  duration={1.15}
                  ease="power3.out"
                  splitType="chars"
                  from={{ opacity: 0, y: 35 }}
                  to={{ opacity: 1, y: 0 }}
                  textAlign="left"
                />
                <SplitText
                  text="secure erasure and"
                  tag="h1"
                  className="text-4xl sm:text-5xl lg:text-[48px] xl:text-[54px] font-black text-[#101114] tracking-tight leading-[1.08] block"
                  delay={30}
                  duration={1.15}
                  ease="power3.out"
                  splitType="chars"
                  from={{ opacity: 0, y: 35 }}
                  to={{ opacity: 1, y: 0 }}
                  textAlign="left"
                />
                <SplitText
                  text="forensic recovery."
                  tag="h1"
                  className="text-4xl sm:text-5xl lg:text-[48px] xl:text-[54px] font-black text-[#E31B23] tracking-tight leading-[1.08] block"
                  delay={35}
                  duration={1.25}
                  ease="power3.out"
                  splitType="chars"
                  from={{ opacity: 0, y: 35 }}
                  to={{ opacity: 1, y: 0 }}
                  textAlign="left"
                />
              </div>

              {/* Subtitle */}
              <p className="text-base sm:text-lg text-[#555B66] leading-relaxed max-w-lg">
                RAZE-CARVE combines low-level device control, intelligent carving, verification and audit integrity in one forensic engine.
              </p>

              {/* CTA Buttons */}
              <div className="flex flex-wrap items-center gap-3.5 pt-2">
                <button
                  onClick={() => onNavigate('/cli')}
                  className="inline-flex items-center justify-center gap-2 px-6 py-3 text-sm font-bold text-white bg-[#E31B23] hover:bg-[#c9141b] rounded-lg shadow-sm hover:shadow-md transition-all cursor-pointer group"
                >
                  <span>Explore the Platform</span>
                  <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                </button>

                <button
                  onClick={() => onNavigate('/docs')}
                  className="inline-flex items-center justify-center gap-2 px-5 py-3 text-sm font-bold text-[#101114] bg-white hover:bg-gray-50 border border-gray-200 hover:border-gray-300 rounded-lg shadow-xs transition-all cursor-pointer"
                >
                  <FileText className="w-4 h-4 text-slate-600" />
                  <span>Read the Technical Docs</span>
                </button>
              </div>
            </div>

            {/* Right Column (~58% Width on Desktop) */}
            <div className="lg:col-span-7 flex flex-col justify-between space-y-6">
              
              {/* Top Right Tagline */}
              <div className="flex justify-end">
                <div className="flex items-center border-l-2 border-[#E31B23] pl-3 py-0.5">
                  <span className="font-mono text-[10px] sm:text-xs font-bold text-[#777E90] uppercase tracking-[0.16em] leading-tight">
                    DEEPER CONTROL. <br />
                    CLEARER EVIDENCE. <br />
                    A SAFER DIGITAL WORLD.
                  </span>
                </div>
              </div>

              {/* Hardware Circuit Render Visual */}
              <div className="rounded-2xl border border-gray-200 overflow-hidden shadow-sm bg-[#08090B] relative group">
                <img
                  src="/core-engine-hardware.jpg"
                  alt="RAZE-CARVE Core Engine Hardware Architecture"
                  className="w-full h-auto max-h-[360px] object-contain mx-auto"
                />
                <div className="p-3 bg-white border-t border-gray-100 flex items-center justify-between text-xs">
                  <span className="font-mono text-slate-500 font-semibold">
                    // LOW-LEVEL NVMe FLASH &amp; DIRECT I/O ASIC ARCHITECTURE
                  </span>
                  <button
                    onClick={() => onNavigate('/cli')}
                    className="text-[#E31B23] font-bold hover:underline flex items-center gap-1 cursor-pointer"
                  >
                    <span>Simulate in CLI Terminal</span>
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

            </div>

          </div>

          {/* ========================================================================= */}
          {/* MIDDLE 4-MODULE CAPABILITY STRIP                                          */}
          {/* ========================================================================= */}
          <div className="mt-12 sm:mt-16 lg:mt-24 pt-4 sm:pt-6 pb-4">
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 lg:gap-8">
              
              {/* Module 1 */}
              <div 
                onClick={() => setActiveLayer(0)}
                className="flex items-center gap-3.5 cursor-pointer group"
              >
                <div className="w-10 h-10 rounded-xl bg-gray-50 border border-gray-200 group-hover:border-[#E31B23] flex items-center justify-center text-[#101114] group-hover:text-[#E31B23] transition-colors shrink-0">
                  <Database className="w-5 h-5 stroke-[1.8]" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] group-hover:text-[#E31B23] transition-colors">
                    RAW BLOCK ACCESS
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Direct device control (no OS cache)
                  </div>
                </div>
              </div>

              {/* Module 2 */}
              <div 
                onClick={() => setActiveLayer(1)}
                className="flex items-center gap-3.5 cursor-pointer group"
              >
                <div className="w-10 h-10 rounded-xl bg-gray-50 border border-gray-200 group-hover:border-[#E31B23] flex items-center justify-center text-[#101114] group-hover:text-[#E31B23] transition-colors shrink-0">
                  <Zap className="w-5 h-5 stroke-[1.8]" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] group-hover:text-[#E31B23] transition-colors">
                    NIST SP 800-88 PURGE
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Controller-level cryptographic wipe
                  </div>
                </div>
              </div>

              {/* Module 3 */}
              <div 
                onClick={() => setActiveLayer(2)}
                className="flex items-center gap-3.5 cursor-pointer group"
              >
                <div className="w-10 h-10 rounded-xl bg-gray-50 border border-gray-200 group-hover:border-[#E31B23] flex items-center justify-center text-[#101114] group-hover:text-[#E31B23] transition-colors shrink-0">
                  <FileSearch className="w-5 h-5 stroke-[1.8]" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] group-hover:text-[#E31B23] transition-colors">
                    STRUCTURE-AWARE CARVER
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Zero-copy chunk reconstruction
                  </div>
                </div>
              </div>

              {/* Module 4 */}
              <div 
                onClick={() => setActiveLayer(3)}
                className="flex items-center gap-3.5 cursor-pointer group"
              >
                <div className="w-10 h-10 rounded-xl bg-gray-50 border border-gray-200 group-hover:border-[#E31B23] flex items-center justify-center text-[#101114] group-hover:text-[#E31B23] transition-colors shrink-0">
                  <ShieldCheck className="w-5 h-5 stroke-[1.8]" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] group-hover:text-[#E31B23] transition-colors">
                    AUDIT READY LEDGER
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    RFC 3161 Merkle timestamp proofs
                  </div>
                </div>
              </div>

            </div>
          </div>

        </div>
      </section>

      {/* ========================================================================= */}
      {/* SECTION 2: ENGINEERING LAYERS (EXACT MATCH TO REFERENCE IMAGE)            */}
      {/* ========================================================================= */}
      <section className="max-w-[1440px] w-full mx-auto px-4 sm:px-6 lg:px-8 py-12 sm:py-16 space-y-8">
        
        {/* Header Row (Left Eyebrow/Headline/Subtitle + Right Monospace Tagline) */}
        <div className="flex flex-col lg:flex-row lg:items-start justify-between gap-6 pb-2">
          
          {/* Left: Eyebrow + Title + Subtitle */}
          <div className="space-y-3 max-w-2xl">
            <div className="text-xs font-mono font-bold tracking-[0.2em] text-[#E31B23] uppercase">
              ENGINEERING LAYERS
            </div>
            <h2 className="text-3xl sm:text-4xl lg:text-[42px] font-black text-[#101114] tracking-tight leading-[1.12]">
              Four layers. One <span className="text-[#E31B23]">forensic core.</span>
            </h2>
            <p className="text-sm sm:text-base text-[#555B66] leading-relaxed max-w-xl">
              Each layer is purpose-built, tightly integrated, and designed for forensic accuracy, operational safety, and verifiable results.
            </p>
          </div>

          {/* Right: Tagline with red left bar */}
          <div className="flex items-center border-l-2 border-[#E31B23] pl-4 py-1 self-start lg:mt-2">
            <span className="font-mono text-xs font-bold text-[#777E90] uppercase tracking-[0.16em] leading-relaxed">
              BUILT FOR INVESTIGATORS. <br />
              TRUSTED BY PROFESSIONALS.
            </span>
          </div>
        </div>

        {/* 2-COLUMN MAIN WORKBENCH LAYOUT — inline style grid to guarantee column sizing */}
        <div
          className="gap-6 items-start"
          style={{
            display: 'grid',
            gridTemplateColumns: 'minmax(280px, 340px) 1fr',
            gap: '2rem',
          }}
        >
          
          {/* LEFT COLUMN: 4 Interactive Layer Cards (~28-30% Width, fixed max) */}
          <div className="space-y-3.5" style={{ minWidth: 0 }}>
            {layers.map((layer) => {
              const isActive = activeLayer === layer.id;
              return (
                <div
                  key={layer.id}
                  onClick={() => setActiveLayer(layer.id)}
                  className={`p-4 rounded-2xl border transition-all duration-150 cursor-pointer flex items-center justify-between gap-3.5 select-none ${
                    isActive
                      ? 'bg-red-50/20 border-[#E31B23] shadow-xs'
                      : 'bg-white border-slate-200 hover:border-slate-300 shadow-xs'
                  }`}
                >
                  {/* Left: Number + Icon */}
                  <div className="flex items-center gap-3.5 min-w-0">
                    <span className="text-sm font-mono font-black text-[#E31B23] w-5 shrink-0">
                      {layer.num}
                    </span>

                    <div className="w-10 h-10 rounded-xl bg-slate-50 border border-slate-200 flex items-center justify-center shrink-0">
                      {layer.cardIcon}
                    </div>

                    {/* Middle: Title + Subtitle */}
                    <div className="min-w-0">
                      <h3 className="text-sm sm:text-[15px] font-bold text-slate-900 leading-tight truncate">
                        {layer.title}
                      </h3>
                      <p className="text-xs text-slate-500 leading-snug truncate">
                        {layer.cardSubtitle}
                      </p>
                    </div>
                  </div>

                  {/* Right: Circular Trigger Button */}
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center shrink-0 transition-all ${
                      isActive
                        ? 'bg-[#E31B23] text-white shadow-xs'
                        : 'border border-slate-200 text-slate-400'
                    }`}
                  >
                    <ArrowRight className="w-3.5 h-3.5" />
                  </div>
                </div>
              );
            })}

            {/* Bottom Tagline below Left Cards */}
            <div className="flex items-center gap-2 text-[11px] font-mono text-slate-400 tracking-wider pt-2 pl-1">
              <span className="w-[2px] h-3.5 bg-slate-300 inline-block" />
              <span>// INTEGRATED. VERIFIABLE. FORENSIC-GRADE.</span>
            </div>
          </div>

          {/* RIGHT COLUMN: Interactive Showcase Panel (takes all remaining space) */}
          <div className="rounded-3xl border border-slate-200/90 bg-[#FAFAFC] p-6 sm:p-8 lg:p-10 shadow-xs space-y-6 lg:space-y-8" style={{ minWidth: 0 }}>
            
            {/* Top Product Header: Left Title & Technical Description + Right 3D Visual */}
            <div className="flex flex-col md:flex-row items-start justify-between gap-6 lg:gap-8">
              
              {/* Left Side: Layer Tag, Title, Subtitle, and Technical Explanation */}
              <div className="flex-1 min-w-0 space-y-3">
                <div className="text-xs font-mono font-bold tracking-[0.2em] text-[#E31B23] uppercase">
                  {currentLayer.layerTag}
                </div>
                
                <h3 className="text-2xl sm:text-3xl lg:text-[34px] font-black text-slate-900 tracking-tight leading-tight">
                  {currentLayer.title}
                </h3>
                
                <p className="text-sm sm:text-base font-semibold text-slate-700 leading-relaxed">
                  {currentLayer.subtitle}
                </p>
                
                <p className="text-xs sm:text-sm text-slate-500 leading-relaxed pt-1">
                  {currentLayer.description}
                </p>
              </div>

              {/* Right Side: Storage Device Visual (fixed 280px width) */}
              <div style={{ width: 280, flexShrink: 0 }} className="hidden md:flex flex-col items-end self-start">
                
                {/* Top Floating Annotation */}
                <div className="font-mono text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right leading-tight pb-1.5">
                  <div>{currentLayer.imageTopLabel[0]}</div>
                  <div>{currentLayer.imageTopLabel[1]}</div>
                </div>

                {/* 3D Hardware Render Image */}
                <div className="w-full aspect-[4/3] rounded-2xl bg-white border border-slate-200/80 shadow-xs flex items-center justify-center p-3 relative group overflow-hidden">
                  <img
                    key={currentLayer.id}
                    src={currentLayer.image}
                    alt={currentLayer.title}
                    className="w-full h-full object-contain transition-transform duration-300 group-hover:scale-105"
                  />
                </div>

                {/* Bottom Floating Annotation */}
                <div className="font-mono text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right leading-tight pt-1.5 flex items-center gap-1.5">
                  <span className="w-3 h-[1px] bg-[#E31B23]" />
                  <div>
                    {currentLayer.imageBottomLabel[0]} <span className="text-slate-300">•</span> {currentLayer.imageBottomLabel[1]}
                  </div>
                </div>
              </div>

            </div>

            {/* Capability Indicators (4 horizontal cards) */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              {currentLayer.pills.map((pill, idx) => (
                <div
                  key={idx}
                  className="p-3 sm:p-3.5 rounded-xl bg-white border border-slate-200/90 shadow-xs flex items-center gap-3 min-w-0"
                >
                  <div className="w-8 h-8 rounded-lg bg-red-50/60 border border-red-100 flex items-center justify-center shrink-0">
                    {pill.icon}
                  </div>
                  <div className="min-w-0">
                    <div className="text-xs font-bold text-slate-900 truncate">
                      {pill.title}
                    </div>
                    <div className="text-[11px] text-slate-500 truncate">
                      {pill.sub}
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Lower Area: CLI Terminal + Hardware Spec Inspector Card */}
            <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: '1.25rem' }}>
              
              {/* Left Column (CLI Terminal) */}
              <div className="rounded-2xl bg-[#090D16] border border-slate-800 p-4 sm:p-5 font-mono text-xs text-slate-200 shadow-md flex flex-col justify-between" style={{ minWidth: 0 }}>
                
                {/* Terminal Header */}
                <div className="flex items-center justify-between border-b border-slate-800 pb-3 mb-3">
                  <div className="flex items-center gap-2">
                    <span className="w-2.5 h-2.5 rounded-full bg-[#E31B23]" />
                    <span className="text-slate-300 font-bold text-xs font-mono">RAZE-CARVE CLI</span>
                  </div>
                  <button
                    onClick={() => handleCopyCli(currentLayer.cli.command)}
                    className="flex items-center gap-1.5 text-[11px] text-slate-400 hover:text-white px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 transition-colors cursor-pointer"
                  >
                    {copied ? (
                      <>
                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                        <span className="text-emerald-400">Copied!</span>
                      </>
                    ) : (
                      <>
                        <Copy className="w-3.5 h-3.5" />
                        <span>Copy</span>
                      </>
                    )}
                  </button>
                </div>

                {/* Terminal Lines */}
                <div className="space-y-1.5 leading-relaxed overflow-x-auto text-[11.5px]">
                  {currentLayer.cli.lines.map((line, lIdx) => (
                    <div key={lIdx} className="flex items-start gap-2 whitespace-nowrap">
                      {line.type === 'cmd' && (
                        <span className="text-slate-100 font-bold">{line.text}</span>
                      )}
                      {line.type === 'init' && (
                        <>
                          <span className="text-sky-400 font-bold">{line.tag}</span>
                          <span className="text-slate-300">{line.text}</span>
                        </>
                      )}
                      {line.type === 'pass' && (
                        <>
                          <span className="text-emerald-400 font-bold">{line.tag}</span>
                          <span className="text-slate-300">{line.text}</span>
                        </>
                      )}
                      {line.type === 'info' && (
                        <>
                          <span className="text-amber-400 font-bold">{line.tag}</span>
                          <span className="text-slate-300">{line.text}</span>
                        </>
                      )}
                      {line.type === 'ready' && (
                        <>
                          <span className="text-emerald-400 font-bold">{line.tag}</span>
                          <span className="text-emerald-300 font-bold">{line.text}</span>
                        </>
                      )}
                    </div>
                  ))}
                </div>

              </div>

              {/* Right Column (Spec Inspector Box) */}
              <div className="rounded-2xl bg-white border border-slate-200/90 p-4 sm:p-5 font-mono shadow-xs flex flex-col justify-between" style={{ minWidth: 0 }}>
                
                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-100 pb-3 mb-3">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="w-7 h-7 rounded-lg bg-slate-100 flex items-center justify-center text-slate-700 shrink-0">
                      <HardDrive className="w-4 h-4" />
                    </div>
                    <div className="min-w-0">
                      <div className="text-xs font-bold text-slate-900 truncate">{currentLayer.spec.devName}</div>
                      <div className="text-[10px] text-slate-400 truncate">{currentLayer.spec.devModel}</div>
                    </div>
                  </div>
                  <span className="px-2 py-0.5 rounded bg-emerald-50 border border-emerald-200 text-emerald-700 text-[10px] font-bold shrink-0">
                    {currentLayer.spec.badge}
                  </span>
                </div>

                {/* Key-Value Spec List */}
                <div className="space-y-2 text-xs">
                  {currentLayer.spec.items.map((item, sIdx) => (
                    <div key={sIdx} className="flex items-center justify-between gap-2">
                      <span className="text-slate-400 text-[11px] font-normal">{item.label}</span>
                      <span className="text-slate-800 text-[11px] font-bold text-right truncate">
                        {item.val}
                      </span>
                    </div>
                  ))}
                </div>

              </div>

            </div>

            {/* Bottom Action Buttons & Supported Interfaces Row */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pt-3 border-t border-slate-200/80">
              
              {/* Left Action Buttons */}
              <div className="flex flex-wrap items-center gap-3">
                <button
                  onClick={() => onNavigate('/cli')}
                  className="inline-flex items-center gap-2 px-5 py-2.5 bg-[#E31B23] hover:bg-[#c9141b] text-white text-xs sm:text-sm font-bold rounded-lg shadow-sm transition-all cursor-pointer group"
                >
                  <Terminal className="w-4 h-4" />
                  <span>Run in Sandbox</span>
                  <ArrowRight className="w-3.5 h-3.5 transition-transform group-hover:translate-x-1" />
                </button>

                <button
                  onClick={() => onNavigate('/docs')}
                  className="inline-flex items-center gap-2 px-4 py-2.5 bg-white hover:bg-slate-50 text-slate-800 border border-slate-200 hover:border-slate-300 text-xs sm:text-sm font-bold rounded-lg shadow-xs transition-all cursor-pointer"
                >
                  <FileText className="w-4 h-4 text-slate-500" />
                  <span>View Documentation</span>
                </button>
              </div>

              {/* Right: Supported Interfaces Tag */}
              <div className="text-left sm:text-right font-mono text-[10px] sm:text-[11px] text-slate-400 uppercase leading-tight">
                <div className="font-bold text-slate-500">SUPPORTED INTERFACES</div>
                <div>{currentLayer.supportedInterfaces}</div>
              </div>

            </div>

          </div>

        </div>

        {/* Footer Meta Note (Exact Match to Reference Image 3) */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between text-[11px] font-mono text-[#94A3B8] pt-8 border-t border-gray-200 tracking-wider">
          <span>SECURE DATA. STRONGER TRUTH.</span>
          <span>// RAZE-CARVE CORE ENGINE v1.0</span>
        </div>

      </section>
    </div>
  );
};

export default FeaturesPage;
