import React, { useState, useEffect, useRef } from 'react';
import { Terminal, Copy, Check, RotateCcw, Play } from 'lucide-react';

interface TerminalTabInfo {
  id: 'sforge' | 'wipe' | 'carve' | 'demo';
  label: string;
  command: string;
  output: string;
}

const TERMINAL_DATA: Record<string, TerminalTabInfo> = {
  sforge: {
    id: 'sforge',
    label: 'sforge (TUI)',
    command: 'sudo sforge',
    output: `[INIT] RAZE-CARVE Forensic Workstation Core v0.1.0
[PASS] Kernel direct I/O privileges verified (CAP_SYS_RAWIO)
[INFO] Detected 2 physical storage controller(s):
       ├── /dev/nvme0n1 [PCIe NVMe Gen4 1TB]       (NVMe Sanitize Supported)
       └── /dev/sdb     [Enterprise SATA 2TB]      (ATA Secure Erase Supported)
[PASS] CSPRNG Entropy Source: /dev/urandom (H = 7.9998 b/B)
[READY] Interactive ratatui interface initialized on /dev/tty1`,
  },
  wipe: {
    id: 'wipe',
    label: 'sih149 wipe',
    command: 'sudo sih149 wipe --target /dev/sdb --method nist-clear --verify',
    output: `[!] Target: /dev/sdb (2000.3 GB) | Method: NIST SP 800-88 Rev 1 Clear
[*] Running pre-sanitization inspection...
    Pre-wipe Shannon entropy: 7.142 b/B (Active data detected)
[Pass 1/1] Overwriting with 0x00 pattern...
           100% [==============================] 2000.3 GB @ 194.8 MB/s
[*] Sampling 200,000 LBAs for Shannon Entropy validation...
    Post-wipe entropy: 0.0000 b/B (Deterministic Zero confirmed)
[✓] Generated signed forensic audit log: /var/log/cert_sdb.pdf`,
  },
  carve: {
    id: 'carve',
    label: 'sih149 recover',
    command: 'sih149 recover --source /dev/sdb1 --out-dir ./recovered_files --types jpeg,png,pdf',
    output: `[*] Source: /dev/sdb1 (32.00 GB) | Output: ./recovered_files/
[*] Scanning for file magic headers: [JPEG, PNG, PDF]...
    100% [==============================] 32.00 GB @ 245.1 MB/s
[*] Forensic Carving Results:
    ├── JPEG images:   312 files extracted (1.14 GB)
    ├── PNG graphics:   84 files extracted (142.6 MB)
    └── PDF documents:  49 files extracted (488.2 MB)
[✓] 445 files salvaged without filesystem allocation table reliance.`,
  },
  demo: {
    id: 'demo',
    label: 'sih149 demo',
    command: 'sih149 demo --size-mb 64',
    output: `[*] Creating 64MB in-memory loopback block device...
[*] Writing 20 synthetic forensic canary files (JPEG, ZIP, PDF)...
[*] Pre-wipe Shannon entropy: 6.841 b/B
[*] Applying NIST SP 800-88 Rev 1 single-pass zero overwrite...
[*] Running post-wipe Shannon entropy audit: 0.0000 b/B
[*] Running signature file carver to test recovery resistance...
    Files recovered: 0 (100% Data Destruction Confirmed)
[✓] Demo test completed successfully in 0.82 seconds.`,
  },
};

export const AnimatedTerminal: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'sforge' | 'wipe' | 'carve' | 'demo'>('sforge');
  const [displayedCommand, setDisplayedCommand] = useState('');
  const [isTyping, setIsTyping] = useState(true);
  const [showOutput, setShowOutput] = useState(false);
  const [copied, setCopied] = useState(false);
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  // Check reduced motion
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
      setPrefersReducedMotion(mediaQuery.matches);
      const listener = (e: MediaQueryListEvent) => setPrefersReducedMotion(e.matches);
      mediaQuery.addEventListener('change', listener);
      return () => mediaQuery.removeEventListener('change', listener);
    }
  }, []);

  const fullCommand = TERMINAL_DATA[activeTab].command;
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const startAnimation = (cmd: string) => {
    if (timerRef.current) clearInterval(timerRef.current);

    if (prefersReducedMotion) {
      setDisplayedCommand(cmd);
      setIsTyping(false);
      setShowOutput(true);
      return;
    }

    setDisplayedCommand('');
    setIsTyping(true);
    setShowOutput(false);

    let currentIndex = 0;
    timerRef.current = setInterval(() => {
      currentIndex += 1;
      setDisplayedCommand(cmd.slice(0, currentIndex));

      if (currentIndex >= cmd.length) {
        if (timerRef.current) clearInterval(timerRef.current);
        setIsTyping(false);
        // Slight pause before executing output
        setTimeout(() => {
          setShowOutput(true);
        }, 180);
      }
    }, 28);
  };

  useEffect(() => {
    startAnimation(fullCommand);
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [activeTab, prefersReducedMotion]);

  const handleCopy = async () => {
    const textToCopy = `${TERMINAL_DATA[activeTab].command}\n\n${TERMINAL_DATA[activeTab].output}`;
    try {
      await navigator.clipboard.writeText(textToCopy);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleReplay = () => {
    startAnimation(fullCommand);
  };

  // Syntax highlight terminal lines
  const formatOutputLine = (line: string, index: number) => {
    if (line.startsWith('[PASS]') || line.startsWith('[✓]')) {
      return (
        <div key={index} className="text-emerald-400 font-medium">
          {line}
        </div>
      );
    }
    if (line.startsWith('[INIT]') || line.startsWith('[READY]')) {
      return (
        <div key={index} className="text-sky-400 font-medium">
          {line}
        </div>
      );
    }
    if (line.startsWith('[INFO]') || line.startsWith('[*]')) {
      return (
        <div key={index} className="text-slate-300">
          <span className="text-sky-400 font-semibold">{line.slice(0, 7)}</span>
          {line.slice(7)}
        </div>
      );
    }
    if (line.startsWith('[!]')) {
      return (
        <div key={index} className="text-amber-400 font-medium">
          {line}
        </div>
      );
    }
    if (line.includes('[===') || line.includes('100%')) {
      return (
        <div key={index} className="text-brand-300">
          {line}
        </div>
      );
    }
    return (
      <div key={index} className="text-slate-400">
        {line}
      </div>
    );
  };

  return (
    <div className="rounded-xl border border-slate-800 bg-slate-950 shadow-2xl overflow-hidden font-mono text-xs sm:text-sm">
      {/* Chrome Top Bar */}
      <div className="flex items-center justify-between px-3 sm:px-4 py-2.5 bg-slate-900 border-b border-slate-800 text-xs gap-2 select-none">
        {/* Window controls */}
        <div className="flex items-center gap-2 shrink-0">
          <span className="w-3 h-3 rounded-full bg-rose-500/80 inline-block"></span>
          <span className="w-3 h-3 rounded-full bg-amber-500/80 inline-block"></span>
          <span className="w-3 h-3 rounded-full bg-emerald-500/80 inline-block"></span>
          <span className="hidden md:inline-block ml-2 text-slate-400 font-sans text-xs font-medium">
            tty1 — raze-carve-live (x86_64)
          </span>
        </div>

        {/* Tab Pills */}
        <div className="flex items-center gap-1 overflow-x-auto py-0.5">
          {(['sforge', 'wipe', 'carve', 'demo'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-2.5 py-1 rounded text-xs transition-all duration-150 ${
                activeTab === tab
                  ? 'bg-slate-800 text-brand-400 font-bold shadow-xs'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-850'
              }`}
            >
              {TERMINAL_DATA[tab].label}
            </button>
          ))}
        </div>

        {/* Action icons */}
        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={handleReplay}
            title="Replay terminal animation"
            className="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded transition-colors"
            aria-label="Replay command animation"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={handleCopy}
            title="Copy command and output"
            className="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded transition-colors"
            aria-label="Copy terminal text"
          >
            {copied ? (
              <Check className="w-3.5 h-3.5 text-emerald-400" />
            ) : (
              <Copy className="w-3.5 h-3.5" />
            )}
          </button>
        </div>
      </div>

      {/* Terminal Viewport */}
      <div className="p-4 sm:p-6 bg-slate-950 text-slate-200 min-h-[260px] overflow-x-auto space-y-3">
        {/* Active Command Line with blinking cursor */}
        <div className="flex items-center gap-2 font-mono flex-wrap">
          <span className="text-slate-500 select-none">root@raze-carve-live:~#</span>
          <span className="text-brand-400 font-semibold">{displayedCommand}</span>
          {isTyping && (
            <span className="inline-block w-2 h-4 bg-brand-400 animate-blink align-middle select-none"></span>
          )}
        </div>

        {/* Output stream */}
        {showOutput && (
          <div className="space-y-1 pt-1 leading-relaxed font-mono animate-fade-in">
            {TERMINAL_DATA[activeTab].output.split('\n').map((line, idx) =>
              formatOutputLine(line, idx)
            )}
          </div>
        )}
      </div>

      {/* Terminal Bottom Status Bar */}
      <div className="px-4 py-1.5 bg-slate-900/90 border-t border-slate-800/80 flex items-center justify-between text-[11px] text-slate-400 font-mono">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            LIVE SHELL
          </span>
          <span className="hidden sm:inline text-slate-600">|</span>
          <span className="hidden sm:inline">NATIVE DIRECT I/O</span>
        </div>
        <span className="text-slate-500">UTF-8 / xterm-256color</span>
      </div>
    </div>
  );
};
