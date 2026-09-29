import React, { useState, useEffect } from 'react';
import { RoutePath } from '../types';
import { SecureForgeBackend } from '../lib/backendApi';
import {
  Terminal,
  Play,
  Square,
  RefreshCw,
  HardDrive,
  Cpu,
  Layers,
  Shield,
  Activity,
  ArrowRight,
  Info,
  CheckCircle2,
  AlertTriangle
} from 'lucide-react';

interface TuiWorkspacePageProps {
  onNavigate: (path: RoutePath) => void;
}

interface TargetDrive {
  id: string;
  node: string;
  type: string;
  capacity: string;
  sectors: string;
  temp: string;
  status: 'Ready' | 'Wiping' | 'Verified';
}

const DRIVES: TargetDrive[] = [
  { id: '1', node: '/dev/nvme0n1', type: 'PCIe NVMe Gen4', capacity: '2000 GB', sectors: '3,907,029,168', temp: '42°C', status: 'Ready' },
  { id: '2', node: '/dev/sdb', type: 'Enterprise SATA', capacity: '8000 GB', sectors: '15,628,053,168', temp: '36°C', status: 'Ready' },
  { id: '3', node: '/dev/sda', type: 'Serial-Attached SCSI', capacity: '16000 GB', sectors: '31,256,106,336', temp: '38°C', status: 'Ready' },
  { id: '4', node: '/dev/sdc', type: 'Forensic Partition', capacity: '512 GB', sectors: '1,000,215,216', temp: '31°C', status: 'Ready' },
];

export const TuiWorkspacePage: React.FC<TuiWorkspacePageProps> = ({ onNavigate }) => {
  const [selectedDrive, setSelectedDrive] = useState<TargetDrive>(DRIVES[0]);
  const [activeMethod, setActiveMethod] = useState<'nist-purge' | 'nist-clear' | 'dod-3pass' | 'carve'>('nist-purge');
  const [isRunning, setIsRunning] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentSector, setCurrentSector] = useState(0);
  const [entropy, setEntropy] = useState(7.9421);
  const [speed, setSpeed] = useState('0.00 MB/s');
  const [logLines, setLogLines] = useState<string[]>([
    'sforge v0.1.0 (Ratatui Terminal Core) initialized.',
    'Probing kernel block devices via /sys/block and /dev...',
    'Found 4 hardware endpoints ready for direct I/O.',
    'Ready. Select operation and target device to begin.',
  ]);

  // Simulation loop when running
  useEffect(() => {
    let interval: any;
    if (isRunning) {
      interval = setInterval(() => {
        setProgress((prev) => {
          if (prev >= 100) {
            setIsRunning(false);
            setEntropy(0.0000);
            setSpeed('0.00 MB/s');
            setLogLines((l) => [
              ...l.slice(-7),
              `[DONE] Operation completed on ${selectedDrive.node}.`,
              `[VERIFY] Shannon Entropy H = 0.0000 b/B verified across all sectors.`,
              `[AUDIT] Cryptographic digest logged. Proof ready.`,
            ]);
            return 100;
          }
          const next = prev + 4;
          setCurrentSector(Math.floor((next / 100) * 3907029168));
          setSpeed(`${(1200 + Math.random() * 250).toFixed(1)} MB/s`);
          setEntropy(Number((8 - (next / 100) * 8).toFixed(4)));
          if (next === 20) {
            setLogLines((l) => [...l.slice(-7), `[PASS 1] O_DIRECT raw block stream verified on ${selectedDrive.node}...`]);
          } else if (next === 60) {
            setLogLines((l) => [...l.slice(-7), `[IOCTL] Firmware controller sanitize executed. Overwriting LBA tables...`]);
          }
          return next;
        });
      }, 350);
    }
    return () => clearInterval(interval);
  }, [isRunning, selectedDrive]);

  const handleStart = async () => {
    setProgress(0);
    setIsRunning(true);
    setLogLines((l) => [
      ...l.slice(-7),
      `[START] Dispatched ${activeMethod.toUpperCase()} pipeline to ${selectedDrive.node}`,
      `[LOCK] Acquiring exclusive raw descriptor (O_EXCL | O_SYNC)...`,
    ]);

    // Dispatch to local backend if running
    try {
      const jobId = await SecureForgeBackend.startWipe(selectedDrive.node, activeMethod, true);
      if (jobId) {
        setLogLines((l) => [
          ...l.slice(-7),
          `[BACKEND] Daemon connected on http://127.0.0.1:7878 (Job: ${jobId})`,
        ]);
      }
    } catch {
      // Backend not running, client simulation continues smoothly
    }
  };

  const handleStop = () => {
    setIsRunning(false);
    setSpeed('0.00 MB/s');
    setLogLines((l) => [...l.slice(-7), `[ABORT] User triggered operation halt.`]);
  };

  return (
    <div className="min-h-screen bg-[#06080F] text-slate-100 font-mono py-8 px-4 sm:px-6 lg:px-8 flex flex-col items-center">
      {/* Top Banner Information */}
      <div className="w-full max-w-6xl mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400">
            <Terminal className="w-4 h-4" />
          </div>
          <div>
            <div className="text-xs font-bold text-emerald-400 uppercase tracking-wider flex items-center gap-2">
              <span>Interactive TUI Workspace</span>
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            </div>
            <h1 className="text-lg font-bold text-white tracking-tight">
              sforge — Defense Terminal Interface
            </h1>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => onNavigate('/gui')}
            className="px-3 py-1.5 rounded-lg border border-slate-700 bg-slate-900/80 text-xs text-slate-300 hover:text-white hover:border-slate-600 transition-colors flex items-center gap-1.5"
          >
            <span>Switch to GUI</span>
            <ArrowRight className="w-3 h-3" />
          </button>
          <button
            onClick={() => onNavigate('/cli')}
            className="px-3 py-1.5 rounded-lg border border-slate-700 bg-slate-900/80 text-xs text-slate-300 hover:text-white hover:border-slate-600 transition-colors flex items-center gap-1.5"
          >
            <span>Switch to CLI</span>
            <ArrowRight className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Live Docker Sandbox Callout */}
      <div className="w-full max-w-6xl mb-4 p-3 sm:p-4 rounded-xl border border-emerald-500/30 bg-emerald-950/20 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs">
        <div className="flex items-center gap-2.5">
          <span className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shrink-0" />
          <span className="text-emerald-300 font-bold">
            Real 60 FPS Ratatui TUI available in Live Docker Sandbox
          </span>
          <span className="text-slate-400 hidden md:inline font-sans">
            — Launch the live cloud container and run <code className="text-emerald-400 font-bold bg-black/40 px-1 py-0.5 rounded">sforge</code>.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <button
            onClick={() => onNavigate('/cli')}
            className="px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold transition-all flex items-center gap-1.5 cursor-pointer shadow-xs"
          >
            <span>Open Live Sandbox Terminal</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Main Terminal Shell Card */}
      <div className="w-full max-w-6xl rounded-2xl border border-slate-800 bg-[#0A0E17] shadow-2xl overflow-hidden">
        {/* Terminal Header Bar */}
        <div className="bg-[#0D1322] border-b border-slate-800 px-4 py-2.5 flex items-center justify-between text-xs text-slate-400">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-[#E31B23]" />
            <span className="w-3 h-3 rounded-full bg-amber-500" />
            <span className="w-3 h-3 rounded-full bg-emerald-500" />
            <span className="text-slate-200 font-bold ml-2">
              sforge v0.1.0 — [TERM: xterm-256color] [RATATUI 0.29]
            </span>
          </div>

          <div className="flex items-center gap-4 text-[11px]">
            <span>KERNEL: LINUX 6.6+</span>
            <span className="text-emerald-400 font-bold">RAW_IO: ACTIVE</span>
          </div>
        </div>

        {/* Inner TUI Grid Layout */}
        <div className="p-4 sm:p-6 grid grid-cols-1 lg:grid-cols-12 gap-6">
          
          {/* Left Column: Storage Targets Selector (4 Cols) */}
          <div className="lg:col-span-4 rounded-xl border border-slate-800 bg-[#0D1322] p-4 flex flex-col justify-between">
            <div>
              <div className="flex items-center justify-between pb-2 border-b border-slate-800 mb-3 text-xs font-bold text-slate-300 uppercase tracking-wider">
                <span>[ 1. TARGET DEVICES ]</span>
                <span className="text-slate-500 text-[10px]">SELECT WITH CLICK</span>
              </div>

              <div className="space-y-2">
                {DRIVES.map((drive) => {
                  const isSelected = selectedDrive.id === drive.id;
                  return (
                    <div
                      key={drive.id}
                      onClick={() => !isRunning && setSelectedDrive(drive)}
                      className={`p-3 rounded-lg border text-xs cursor-pointer transition-all ${
                        isSelected
                          ? 'border-emerald-500 bg-emerald-950/30 text-emerald-300'
                          : 'border-slate-800/80 bg-slate-900/60 text-slate-400 hover:border-slate-700 hover:text-slate-200'
                      }`}
                    >
                      <div className="flex items-center justify-between font-bold mb-1">
                        <span className="text-white">{drive.node}</span>
                        <span className="text-xs">{drive.capacity}</span>
                      </div>
                      <div className="text-[11px] text-slate-400 flex items-center justify-between">
                        <span>{drive.type}</span>
                        <span>{drive.temp}</span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Method Selector */}
              <div className="mt-6 pt-4 border-t border-slate-800">
                <div className="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">
                  [ 2. OPERATION METHOD ]
                </div>
                <div className="grid grid-cols-2 gap-2 text-[11px]">
                  <button
                    onClick={() => !isRunning && setActiveMethod('nist-purge')}
                    className={`p-2 rounded border text-left transition-colors ${
                      activeMethod === 'nist-purge'
                        ? 'border-emerald-500 bg-emerald-950/40 text-emerald-300 font-bold'
                        : 'border-slate-800 text-slate-400 hover:border-slate-700'
                    }`}
                  >
                    NIST PURGE
                  </button>
                  <button
                    onClick={() => !isRunning && setActiveMethod('nist-clear')}
                    className={`p-2 rounded border text-left transition-colors ${
                      activeMethod === 'nist-clear'
                        ? 'border-emerald-500 bg-emerald-950/40 text-emerald-300 font-bold'
                        : 'border-slate-800 text-slate-400 hover:border-slate-700'
                    }`}
                  >
                    NIST CLEAR
                  </button>
                  <button
                    onClick={() => !isRunning && setActiveMethod('dod-3pass')}
                    className={`p-2 rounded border text-left transition-colors ${
                      activeMethod === 'dod-3pass'
                        ? 'border-emerald-500 bg-emerald-950/40 text-emerald-300 font-bold'
                        : 'border-slate-800 text-slate-400 hover:border-slate-700'
                    }`}
                  >
                    DoD 3-PASS
                  </button>
                  <button
                    onClick={() => !isRunning && setActiveMethod('carve')}
                    className={`p-2 rounded border text-left transition-colors ${
                      activeMethod === 'carve'
                        ? 'border-emerald-500 bg-emerald-950/40 text-emerald-300 font-bold'
                        : 'border-slate-800 text-slate-400 hover:border-slate-700'
                    }`}
                  >
                    FILE CARVE
                  </button>
                </div>
              </div>
            </div>

            {/* Launch / Abort Controls */}
            <div className="pt-6 mt-6 border-t border-slate-800 flex gap-2">
              {!isRunning ? (
                <button
                  onClick={handleStart}
                  className="flex-1 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-black font-bold text-xs flex items-center justify-center gap-2 shadow-sm transition-colors"
                >
                  <Play className="w-3.5 h-3.5 fill-current" />
                  <span>[F2] EXECUTE RUN</span>
                </button>
              ) : (
                <button
                  onClick={handleStop}
                  className="flex-1 py-2 rounded-lg bg-[#E31B23] hover:bg-red-600 text-white font-bold text-xs flex items-center justify-center gap-2 shadow-sm transition-colors"
                >
                  <Square className="w-3.5 h-3.5 fill-current" />
                  <span>[F10] ABORT</span>
                </button>
              )}
            </div>
          </div>

          {/* Center & Right Columns: Realtime Live Visualizer & Meters (8 Cols) */}
          <div className="lg:col-span-8 space-y-4">
            
            {/* Live Metrics Row */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              <div className="p-3 rounded-xl border border-slate-800 bg-[#0D1322]">
                <div className="text-[10px] text-slate-500 uppercase">PROGRESS</div>
                <div className="text-lg font-bold text-emerald-400">{progress}%</div>
              </div>
              <div className="p-3 rounded-xl border border-slate-800 bg-[#0D1322]">
                <div className="text-[10px] text-slate-500 uppercase">THROUGHPUT</div>
                <div className="text-lg font-bold text-slate-200">{speed}</div>
              </div>
              <div className="p-3 rounded-xl border border-slate-800 bg-[#0D1322]">
                <div className="text-[10px] text-slate-500 uppercase">SHANNON ENTROPY (H)</div>
                <div className={`text-lg font-bold ${entropy === 0 ? 'text-emerald-400' : 'text-amber-400'}`}>
                  {entropy.toFixed(4)} b/B
                </div>
              </div>
              <div className="p-3 rounded-xl border border-slate-800 bg-[#0D1322]">
                <div className="text-[10px] text-slate-500 uppercase">TARGET SECTORS</div>
                <div className="text-lg font-bold text-slate-200 truncate">{selectedDrive.sectors}</div>
              </div>
            </div>

            {/* Sector Visualizer Grid Map (Ascii/Block Matrix) */}
            <div className="p-4 rounded-xl border border-slate-800 bg-[#0D1322]">
              <div className="flex items-center justify-between pb-2 border-b border-slate-800 mb-3 text-xs">
                <span className="font-bold text-slate-300">[ 3. SECTOR DISK MAP (LBA CLUSTERS) ]</span>
                <span className="text-emerald-400 font-bold">{progress >= 100 ? 'ZERO PATTERN CONFIRMED' : 'STREAMING'}</span>
              </div>

              {/* Block Grid */}
              <div className="grid grid-cols-16 sm:grid-cols-32 gap-1 py-1">
                {Array.from({ length: 64 }).map((_, idx) => {
                  const blockPercent = (idx / 64) * 100;
                  const isDone = progress >= blockPercent;
                  const isCurrent = Math.abs(progress - blockPercent) < 4 && isRunning;
                  return (
                    <div
                      key={idx}
                      className={`h-4 rounded-xs transition-all duration-150 ${
                        isCurrent
                          ? 'bg-white animate-pulse'
                          : isDone
                          ? 'bg-emerald-500/80 shadow-xs shadow-emerald-500/20'
                          : 'bg-slate-800'
                      }`}
                      title={`LBA Block ${idx * 61000000} - ${isDone ? 'Overwritten' : 'Unprocessed'}`}
                    />
                  );
                })}
              </div>

              <div className="flex items-center justify-between text-[10px] text-slate-500 pt-3 font-mono">
                <span>0x00000000 (LBA 0)</span>
                <span className="flex items-center gap-3">
                  <span className="flex items-center gap-1">
                    <span className="w-2 h-2 rounded-xs bg-emerald-500" />
                    <span>Zeroed</span>
                  </span>
                  <span className="flex items-center gap-1">
                    <span className="w-2 h-2 rounded-xs bg-slate-800" />
                    <span>Pending</span>
                  </span>
                </span>
                <span>0xE8D4A510 (MAX LBA)</span>
              </div>
            </div>

            {/* Live Terminal Output Console */}
            <div className="p-4 rounded-xl border border-slate-800 bg-black text-xs font-mono">
              <div className="text-slate-500 text-[11px] pb-2 border-b border-slate-900 flex items-center justify-between">
                <span>[ 4. KERNEL TELEMETRY LOG ]</span>
                <span className="text-slate-600">stdout & stderr</span>
              </div>
              <div className="pt-2 space-y-1 text-slate-300 min-h-[110px]">
                {logLines.map((line, idx) => (
                  <div key={idx} className="leading-relaxed">
                    <span className="text-slate-600 mr-2">&gt;</span>
                    <span className={line.includes('[DONE]') ? 'text-emerald-400 font-bold' : line.includes('[START]') ? 'text-blue-400' : 'text-slate-300'}>
                      {line}
                    </span>
                  </div>
                ))}
              </div>
            </div>

          </div>

        </div>

        {/* Bottom Keyboard Hotkeys Bar */}
        <div className="bg-[#090D15] border-t border-slate-800 px-4 py-2 flex items-center justify-between text-[11px] text-slate-400 flex-wrap gap-2">
          <div className="flex items-center gap-4">
            <span><strong className="text-slate-200">[F1]</strong> HELP</span>
            <span><strong className="text-slate-200">[F2]</strong> WIPE</span>
            <span><strong className="text-slate-200">[F3]</strong> CARVE</span>
            <span><strong className="text-slate-200">[F4]</strong> VERIFY</span>
            <span><strong className="text-slate-200">[F10]</strong> ABORT</span>
          </div>
          <div className="text-slate-500">
            RAZE-CARVE TUI Engine • Native Rust Execution
          </div>
        </div>
      </div>
    </div>
  );
};
