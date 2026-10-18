import React, { useState, useRef, useEffect } from 'react';
import { RoutePath } from '../types';
import { SecureForgeBackend } from '../lib/backendApi';
import {
  Terminal as TerminalIcon,
  Play,
  Copy,
  Check,
  RotateCcw,
  ArrowRight,
  Code,
  HardDrive,
  ShieldCheck,
  Search,
  ExternalLink,
  Zap,
  CheckCircle2,
  RefreshCw
} from 'lucide-react';

interface CliWorkspacePageProps {
  onNavigate: (path: RoutePath) => void;
}

interface CommandPreset {
  title: string;
  command: string;
  description: string;
  category: 'info' | 'wipe' | 'carve' | 'verify';
}

const PRESETS: CommandPreset[] = [
  {
    title: 'Hardware Topology & Controller Check',
    command: 'sudo sih149 info',
    description: 'Enumerate block devices, direct ioctl endpoints, and firmware erase capabilities.',
    category: 'info',
  },
  {
    title: 'Diagnostic Self-Tests',
    command: 'sih149 test',
    description: 'Verify cryptographic engines, AES-NI hardware instructions, and entropy calculation.',
    category: 'info',
  },
  {
    title: 'NIST 800-88 Purge (Firmware Erase)',
    command: 'sudo sih149 wipe --target /dev/nvme0n1 --method nist-purge --audit-out /var/log/audit.pdf',
    description: 'Execute native NVMe Crypto Scramble to purge all NAND cells including overprovisioning.',
    category: 'wipe',
  },
  {
    title: 'DoD 5220.22-M 3-Pass Erasure',
    command: 'sudo sih149 wipe --target /dev/sdb --method dod-3pass --verify',
    description: 'Overwrites sectors in 3 passes (zero, complement, random) with post-wipe validation.',
    category: 'wipe',
  },
  {
    title: 'Autonomous Signature File Carving',
    command: 'sih149 recover --source /dev/sdc1 --out-dir ./evidence --types jpeg,pdf,sqlite',
    description: 'Scans raw unpartitioned disk sectors for magic bytes and extracts reconstructed files.',
    category: 'carve',
  },
  {
    title: 'Shannon Entropy Verification',
    command: 'sih149 verify --target /dev/nvme0n1 --entropy-threshold 0.001',
    description: 'Mathematically confirms zero residual structured data (target H <= 0.001 b/B).',
    category: 'verify',
  },
];

const SANDBOX_COMMANDS = [
  {
    num: '1',
    cmd: 'sforge',
    desc: 'Launch full 60 FPS Ratatui TUI defense console',
    badge: 'TUI',
  },
  {
    num: '2',
    cmd: 'sih149 recover -s /dev/loop10 -o /salvaged',
    desc: 'Carve deleted JPEG, PNG & PDF files from mounted evidence block',
    badge: 'CARVE',
  },
  {
    num: '3',
    cmd: 'sih149 wipe -d /dev/loop11 -m zero --verify',
    desc: 'Execute NIST SP 800-88 Zero Overwrite on high-entropy target',
    badge: 'SANATIZE',
  },
  {
    num: '4',
    cmd: 'lsblk / fdisk -l / df -h',
    desc: 'Inspect virtual loop devices (/dev/loop10, loop11, loop12)',
    badge: 'STORAGE',
  },
  {
    num: '5',
    cmd: 'reset-disks',
    desc: 'Re-seed virtual disk artifacts to initial state',
    badge: 'RESET',
  },
];

const MOCK_OUTPUTS: Record<string, string[]> = {
  'sudo sih149 info': [
    '[INFO] RAZE-CARVE v0.1.0 — Enumerating storage endpoints (O_DIRECT capability check)...',
    '-----------------------------------------------------------------------------------------',
    'DEVICE         TYPE       CAPACITY       SECTOR SIZE  IOCTL PASSTHROUGH  PURGE SUPPORT',
    '-----------------------------------------------------------------------------------------',
    '/dev/nvme0n1   NVMe 1.4   2,000.39 GB    512 bytes    Direct ioctl       YES (Crypto / Block)',
    '/dev/sdb       SATA 3.3   8,001.56 GB    4096 bytes   ATA Passthrough    YES (Enhanced Secure)',
    '/dev/sda       SAS 12G    16,000.00 GB   512 bytes    SCSI CDB Pass      YES (Format Unit)',
    '/dev/sdc1      USB-SAT    512.11 GB      512 bytes    SCSI-to-ATA        LIMITED (Bridge dep)',
    '-----------------------------------------------------------------------------------------',
    '[OK] All 4 block devices accessible with administrative permissions.',
  ],
  'sih149 test': [
    '[TEST 1/5] Checking AES-NI CPU cryptographic acceleration... [PASS] (Throughput: 3.84 GB/s)',
    '[TEST 2/5] Blake3 and SHA-256 digest consistency check...    [PASS] (Hash verified)',
    '[TEST 3/5] Shannon Entropy mathematical precision unit...    [PASS] (H(0x00) = 0.00000 b/B)',
    '[TEST 4/5] Magic byte signature parser (24 file types)...    [PASS] (All signatures loaded)',
    '[TEST 5/5] Kernel direct I/O test on /dev/zero...            [PASS] (O_DIRECT operational)',
    '-----------------------------------------------------------------------------------------',
    '[STATUS] All self-tests passed. RAZE-CARVE core engine ready for high-assurance tasks.',
  ],
  'sudo sih149 wipe --target /dev/nvme0n1 --method nist-purge --audit-out /var/log/audit.pdf': [
    '[INIT] Target node: /dev/nvme0n1 (PCIe NVMe Controller)',
    '[CHECK] Unmounting active partitions... Done.',
    '[IOCTL] Dispatching NVMe Sanitize command (Opcode: 0x84, Crypto Erase)...',
    '[PASS 1/1] Firmware controller scrambling encryption key across all NAND blocks...',
    '[VERIFY] Sampling 10,000 random sector clusters for residual entropy...',
    '[VERIFY] Shannon Entropy H = 0.000000 b/B. Uniform clear state confirmed.',
    '[REPORT] Signed cryptographic certificate exported to /var/log/audit.pdf',
    '[STATUS] COMPLETED: NIST SP 800-88 Rev 1 Purge executed in 14.8 seconds.',
  ],
  'sudo sih149 wipe --target /dev/sdb --method dod-3pass --verify': [
    '[INIT] Target node: /dev/sdb (High-Density Enterprise SATA)',
    '[PASS 1/3] Writing 0x00 deterministic zero pattern (8.0 TB)... [DONE] (420 MB/s)',
    '[PASS 2/3] Writing 0xFF inverted complement pattern (8.0 TB)... [DONE] (418 MB/s)',
    '[PASS 3/3] Writing CSPRNG pseudo-random sequence (8.0 TB)...     [DONE] (412 MB/s)',
    '[VERIFY] Post-wipe readback verification... [PASS] (No original patterns detected)',
    '[AUDIT] Hash digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    '[STATUS] COMPLETED: DoD 5220.22-M 3-Pass sanitization certified.',
  ],
  'sih149 recover --source /dev/sdc1 --out-dir ./evidence --types jpeg,pdf,sqlite': [
    '[INIT] Opening raw stream /dev/sdc1 (512 GB) in read-only mode (O_RDONLY | O_DIRECT)...',
    '[SCAN] Scanning byte stream for header signatures (JPEG: 0xFFD8, PDF: %PDF-, SQLite: SQLite format 3)...',
    '[RECOVERED] Found file #001: evidence_0001.pdf (Size: 1.42 MB at offset 0x004F2000)',
    '[RECOVERED] Found file #002: evidence_0002.jpg (Size: 3.88 MB at offset 0x0182C000)',
    '[RECOVERED] Found file #003: evidence_0003.sqlite (Size: 14.20 MB at offset 0x04C00000)',
    '-----------------------------------------------------------------------------------------',
    '[SUMMARY] Extracted 142 complete files to ./evidence/ (0 false positives).',
  ],
  'sih149 verify --target /dev/nvme0n1 --entropy-threshold 0.001': [
    '[INIT] Opening /dev/nvme0n1 for zero-trust mathematical sampling...',
    '[SAMPLE] Reading 12,800 window sectors across LBA range 0x00000000 to 0xE8D4A510...',
    '[MATH] Calculating Shannon Entropy: H(X) = -sum( P(xi) * log2(P(xi)) )...',
    '[RESULT] Measured Entropy: 0.000000 b/B (Threshold <= 0.001000 b/B)',
    '[AUDIT] ZERO RESIDUALS DETECTED. The media is forensically sanitized.',
  ],
};

const CLOUDFLARE_SANDBOX_URL = 'https://secure-forge-sandbox.duckdns.org/';

export const CliWorkspacePage: React.FC<CliWorkspacePageProps> = ({ onNavigate }) => {
  const [activeTab, setActiveTab] = useState<'live-sandbox' | 'simulator'>('live-sandbox');
  const [currentCommand, setCurrentCommand] = useState('sudo sih149 info');
  const [history, setHistory] = useState<{ cmd: string; output: string[] }[]>([
    {
      cmd: 'sudo sih149 info',
      output: MOCK_OUTPUTS['sudo sih149 info'],
    },
  ]);
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);
  const terminalEndRef = useRef<HTMLDivElement>(null);

  const executeCommand = async (cmdText: string) => {
    const trimmed = cmdText.trim();
    if (!trimmed) return;

    if (trimmed.includes('wipe')) {
      try {
        await SecureForgeBackend.startWipe('/dev/nvme0n1', 'nist-purge', true);
      } catch {
        // Backend offline, fallback simulation handles output
      }
    } else if (trimmed.includes('report') && trimmed.includes('--operator')) {
      SecureForgeBackend.downloadCertificate(
        '/dev/nvme0n1',
        '2.0 TB',
        'NIST SP 800-88 Purge',
        0.0000
      );
    }

    let output = MOCK_OUTPUTS[trimmed];
    if (!output) {
      const matchingKey = Object.keys(MOCK_OUTPUTS).find((k) => trimmed.includes(k.split(' ')[1]));
      output = matchingKey
        ? MOCK_OUTPUTS[matchingKey]
        : [
            `sih149: command executed: '${trimmed}'`,
            `[EXEC] Process returned code 0. Direct I/O finished with verified checksum.`,
          ];
    }

    setHistory((prev) => [...prev, { cmd: trimmed, output }]);
    setCurrentCommand('');
  };

  useEffect(() => {
    terminalEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [history]);

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCmd(text);
    setTimeout(() => setCopiedCmd(null), 2000);
  };

  return (
    <div className="min-h-screen bg-[#070A11] text-slate-100 font-mono py-8 px-4 sm:px-6 lg:px-8 flex flex-col items-center">
      {/* Top Banner Header */}
      <div className="w-full max-w-7xl mb-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-[#E31B23]/20 border border-[#E31B23]/40 flex items-center justify-center text-[#E31B23]">
            <TerminalIcon className="w-5 h-5" />
          </div>
          <div>
            <div className="text-xs font-bold text-[#E31B23] uppercase tracking-wider flex items-center gap-2">
              <span>Interactive CLI &amp; Cloud Sandbox</span>
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              <span className="text-[10px] text-emerald-400 font-mono">DOCKER CONTAINER LIVE</span>
            </div>
            <h1 className="text-xl font-bold text-white tracking-tight">
              sih149 — Forensic Command-Line Shell &amp; TUI Sandbox
            </h1>
          </div>
        </div>

        {/* Live Status Indicator & Quick External Link */}
        <div className="flex items-center gap-2">
          <a
            href={CLOUDFLARE_SANDBOX_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="px-3.5 py-1.5 rounded-lg border border-slate-700 bg-slate-900/90 text-xs text-white hover:border-red-500/50 hover:bg-slate-800 transition-all flex items-center gap-1.5 cursor-pointer shadow-xs"
          >
            <span>Open in Fullscreen Tab</span>
            <ExternalLink className="w-3.5 h-3.5 text-slate-400" />
          </a>
        </div>
      </div>

      {/* =========================================================================
          TAB 1: LIVE CLOUD DOCKER SANDBOX (EMBEDDED)
         ========================================================================= */}
        <div className="w-full max-w-7xl space-y-4">
          {/* Sandbox Controls & Status Bar */}
          <div className="p-3 sm:p-4 rounded-xl border border-slate-800 bg-[#0C101A] flex flex-col md:flex-row md:items-center justify-between gap-3 text-xs">
            <div className="flex items-center gap-2.5">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shrink-0" />
              <div>
                <span className="font-bold text-white">RAZE-CARVE FORENSIC SANDBOX WORKSTATION</span>
                <span className="text-slate-400 ml-2 hidden sm:inline font-sans">
                  Isolated Docker container with virtual loopback disks (/dev/loop10, /dev/loop11, /dev/loop12).
                </span>
              </div>
            </div>

            <div className="flex items-center gap-2 shrink-0">
              <a
                href={CLOUDFLARE_SANDBOX_URL}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#E31B23] hover:bg-[#c9141b] text-white font-bold transition-all shadow-sm cursor-pointer"
              >
                <ExternalLink className="w-3.5 h-3.5" />
                <span>Open in Fullscreen Tab</span>
              </a>
            </div>
          </div>

          {/* Quick-Click Sandbox Command Helper Bar */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-2.5">
            {SANDBOX_COMMANDS.map((item) => (
              <div
                key={item.num}
                onClick={() => handleCopy(item.cmd)}
                className="p-2.5 rounded-lg border border-slate-800 bg-[#0C101A] hover:border-slate-700 hover:bg-slate-900/60 cursor-pointer transition-all group"
                title="Click to copy command to clipboard"
              >
                <div className="flex items-center justify-between text-[11px] font-bold text-white mb-1">
                  <span className="text-[#E31B23] font-mono">#{item.num} {item.badge}</span>
                  {copiedCmd === item.cmd ? (
                    <span className="text-emerald-400 text-[10px] flex items-center gap-0.5">
                      <Check className="w-3 h-3" /> Copied!
                    </span>
                  ) : (
                    <Copy className="w-3 h-3 text-slate-500 group-hover:text-white" />
                  )}
                </div>
                <div className="text-[11px] font-mono text-emerald-300 font-semibold truncate bg-black/60 px-1.5 py-0.5 rounded border border-slate-800">
                  {item.cmd}
                </div>
                <div className="text-[10px] text-slate-400 mt-1 line-clamp-1 font-sans">
                  {item.desc}
                </div>
              </div>
            ))}
          </div>

          {/* Live Terminal iFrame Container */}
          <div className="relative w-full rounded-2xl overflow-hidden border border-slate-800 bg-black shadow-2xl">
            {/* Terminal Header */}
            <div className="bg-[#0B0E17] border-b border-slate-800 px-4 py-2.5 flex items-center justify-between text-xs text-slate-400">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 rounded-full bg-[#E31B23]" />
                <span className="w-2.5 h-2.5 rounded-full bg-amber-500" />
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
                <span className="text-slate-300 font-bold ml-2 font-mono">
                  secureforge-sandbox:~$ (ttyd live stream)
                </span>
              </div>
              <div className="text-[11px] font-mono text-slate-500">
                Connected via secure-forge-sandbox.duckdns.org
              </div>
            </div>

            {/* Embedded Live Web Terminal */}
            <iframe
              src={CLOUDFLARE_SANDBOX_URL}
              title="RAZE-CARVE Live Docker Sandbox Workstation"
              className="w-full h-[620px] sm:h-[700px] border-0 bg-black"
              allow="clipboard-read; clipboard-write"
            />
          </div>
        </div>

          </div>
  );
};
