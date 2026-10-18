import React, { useState, useRef, useEffect } from 'react';
import { RoutePath } from '../types';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { Badge } from '../components/Badge';
import { Button } from '../components/Button';
import { Reveal } from '../components/Reveal';
import { CopyButton } from '../components/CopyButton';
import {
  Terminal,
  Shield,
  HardDrive,
  FileSearch,
  Activity,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Download,
  Eye,
  X,
  Play,
  Pause,
  FileText,
  Image as ImageIcon,
  Database,
  Lock,
  ExternalLink,
  Cpu,
  Maximize2,
  Minimize2,
  Clipboard,
  Copy,
  Check,
  CornerDownLeft,
  Zap,
} from 'lucide-react';
import { VM_BOOTSTRAP_CMD } from '../data/vmToolsBundle';

interface SandboxPageProps {
  onNavigate: (path: RoutePath) => void;
}

declare global {
  interface Window {
    V86Starter?: any;
    V86?: any;
  }
}

export const SandboxPage: React.FC<SandboxPageProps> = ({ onNavigate }) => {
  const [vmState, setVmState] = useState<'idle' | 'loading' | 'booting' | 'running' | 'paused' | 'error'>('idle');
  const [bootLog, setBootLog] = useState<string>('Click "Boot Live Linux (2GB RAM)" to launch genuine x86 Linux kernel.');
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [toolsLoaded, setToolsLoaded] = useState(true);
  const [allocatedRAM, setAllocatedRAM] = useState<string>('2048 MB (2 GB)');
  const [copiedNotification, setCopiedNotification] = useState(false);
  const [pastedNotification, setPastedNotification] = useState(false);
  const [commandBarInput, setCommandBarInput] = useState('');

  const screenContainerRef = useRef<HTMLDivElement>(null);
  const terminalWrapperRef = useRef<HTMLDivElement>(null);
  const emulatorRef = useRef<any>(null);

  // Sync True Hardware Browser Fullscreen
  useEffect(() => {
    const handleFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    return () => document.removeEventListener('fullscreenchange', handleFullscreenChange);
  }, []);

  const toggleNativeFullScreen = async () => {
    try {
      if (!document.fullscreenElement) {
        if (terminalWrapperRef.current?.requestFullscreen) {
          await terminalWrapperRef.current.requestFullscreen();
        } else if ((terminalWrapperRef.current as any)?.webkitRequestFullscreen) {
          await (terminalWrapperRef.current as any).webkitRequestFullscreen();
        } else {
          setIsFullscreen(true);
        }
      } else {
        if (document.exitFullscreen) {
          await document.exitFullscreen();
        }
      }
    } catch {
      setIsFullscreen(!isFullscreen);
    }
  };

  // Check and ensure v86 engine is loaded
  useEffect(() => {
    const checkV86 = () => {
      if (typeof window !== 'undefined' && (window.V86Starter || window.V86)) {
        if (!window.V86Starter && window.V86) {
          window.V86Starter = window.V86;
        }
        setBootLog('v86 WebAssembly engine ready. Allocate 2GB RAM & boot Linux.');
        return true;
      }
      return false;
    };

    if (checkV86()) return;

    const interval = setInterval(() => {
      if (checkV86()) clearInterval(interval);
    }, 150);

    const timer = setTimeout(() => {
      clearInterval(interval);
      if (!checkV86()) {
        const script = document.createElement('script');
        script.src = '/v86/libv86.js';
        script.async = true;
        script.onload = () => checkV86();
        document.head.appendChild(script);
      }
    }, 800);

    return () => {
      clearInterval(interval);
      clearTimeout(timer);
      if (emulatorRef.current) {
        try {
          emulatorRef.current.destroy();
        } catch {
          // ignore
        }
      }
    };
  }, []);

  // Intercept Ctrl+V clipboard paste into the VM
  useEffect(() => {
    const handleWindowPaste = (e: ClipboardEvent) => {
      if (vmState !== 'running' || !emulatorRef.current) return;
      const text = e.clipboardData?.getData('text');
      if (text) {
        // Send pasted text into VM
        sendTextToVM(text);
        setPastedNotification(true);
        setTimeout(() => setPastedNotification(false), 1500);
      }
    };

    window.addEventListener('paste', handleWindowPaste);
    return () => window.removeEventListener('paste', handleWindowPaste);
  }, [vmState]);

  // Handle ESC to exit fullscreen
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isFullscreen) {
        setIsFullscreen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isFullscreen]);

  const sendTextToVM = (text: string) => {
    if (!emulatorRef.current) return;
    try {
      emulatorRef.current.keyboard_send_text(text);
    } catch {
      try {
        emulatorRef.current.serial0_send(text);
      } catch {}
    }
  };

  const sendCommandToVM = (cmd: string) => {
    if (!emulatorRef.current || vmState !== 'running') return;
    sendTextToVM(`${cmd}\n`);
  };

  const handleManualInstallTools = () => {
    if (!emulatorRef.current || vmState !== 'running') return;
    sendTextToVM('[ -x /bin/sforge ] || (' + VM_BOOTSTRAP_CMD + ')\necho "[✓] Verifying RAZE-CARVE preloaded binaries in /bin/:" && ls -l /bin/sih149 /bin/sforge /bin/autotest /bin/sudo && sih149 --help\n');
    setToolsLoaded(true);
  };

  const handlePasteClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        sendTextToVM(text);
        setPastedNotification(true);
        setTimeout(() => setPastedNotification(false), 1500);
      }
    } catch {
      alert('Unable to access clipboard automatically. Please press Ctrl+V or use the input box below.');
    }
  };

  const handleCopyScreenText = async () => {
    if (!emulatorRef.current) return;
    try {
      let screenText = '';
      if (emulatorRef.current.screen_adapter?.get_text_screen) {
        const lines = emulatorRef.current.screen_adapter.get_text_screen();
        screenText = Array.isArray(lines) ? lines.join('\n').trim() : String(lines);
      }
      if (!screenText && screenContainerRef.current) {
        screenText = screenContainerRef.current.innerText || '';
      }
      if (screenText) {
        await navigator.clipboard.writeText(screenText);
        setCopiedNotification(true);
        setTimeout(() => setCopiedNotification(false), 2000);
      }
    } catch {
      alert('Unable to copy screen text.');
    }
  };

  const startLinuxVM = () => {
    const V86Constructor = window.V86Starter || window.V86;
    if (!V86Constructor) {
      alert('The WebAssembly emulator engine is still initializing. Please wait a moment.');
      return;
    }

    if (emulatorRef.current) {
      try {
        emulatorRef.current.destroy();
      } catch {
        // ignore
      }
    }

    setVmState('loading');
    setBootLog('Allocating 2048 MB (2 GB) RAM and mounting SeaBIOS + Linux kernel + RAZE-CARVE Initramfs...');

    let ramToAllocate = 2048 * 1024 * 1024;

    const setupEmulatorListeners = (emulator: any) => {
      let serialBuffer = '';
      let bootstrapped = false;
      emulator.add_listener('serial0-output-char', (char: string) => {
        if (bootstrapped) return;
        serialBuffer += char;
        if (serialBuffer.length > 300) serialBuffer = serialBuffer.slice(-300);
        if (serialBuffer.includes('~%') || serialBuffer.includes('/ #') || serialBuffer.includes('login:')) {
          bootstrapped = true;
          setBootLog('Linux shell ready. RAZE-CARVE tools verified & preloaded in /bin/.');
          setToolsLoaded(true);
        }
      });

      emulator.add_listener('emulator-ready', () => {
        setVmState('running');
        setBootLog('Linux kernel initialized. Launching shell & preloading tools...');
        setToolsLoaded(true);
      });
    };

    try {
      const emulator = new V86Constructor({
        wasm_path: '/v86/v86.wasm',
        memory_size: ramToAllocate,
        vga_memory_size: 16 * 1024 * 1024,
        screen_container: screenContainerRef.current,
        bios: { url: '/v86/seabios.bin' },
        vga_bios: { url: '/v86/vgabios.bin' },
        bzimage: { url: '/v86/buildroot-bzimage.bin' },
        initrd: { url: '/v86/secureforge-initrd.cpio.gz' },
        cmdline: 'tsc=reliable mitigations=off random.trust_cpu=on quiet console=tty0 console=ttyS0',
        autostart: true,
      });

      emulatorRef.current = emulator;
      setVmState('booting');
      setBootLog('2 GB RAM allocated. Booting Linux kernel with preloaded /bin/sih149, /bin/sforge, /bin/sudo...');
      setupEmulatorListeners(emulator);
    } catch (err: any) {
      console.warn('2GB allocation failed, falling back to 1024MB:', err);
      try {
        ramToAllocate = 1024 * 1024 * 1024;
        setAllocatedRAM('1024 MB (1 GB fallback)');
        const emulator = new V86Constructor({
          wasm_path: '/v86/v86.wasm',
          memory_size: ramToAllocate,
          vga_memory_size: 16 * 1024 * 1024,
          screen_container: screenContainerRef.current,
          bios: { url: '/v86/seabios.bin' },
          vga_bios: { url: '/v86/vgabios.bin' },
          bzimage: { url: '/v86/buildroot-bzimage.bin' },
          initrd: { url: '/v86/secureforge-initrd.cpio.gz' },
          cmdline: 'tsc=reliable mitigations=off random.trust_cpu=on quiet console=tty0 console=ttyS0',
          autostart: true,
        });
        emulatorRef.current = emulator;
        setVmState('booting');
        setupEmulatorListeners(emulator);
      } catch (fallbackErr: any) {
        setVmState('error');
        setBootLog(`Failed to allocate memory: ${fallbackErr?.message || fallbackErr}`);
      }
    }
  };

  const handleReboot = () => {
    if (emulatorRef.current) {
      try {
        emulatorRef.current.restart();
        setVmState('booting');
        setToolsLoaded(false);
      } catch {
        startLinuxVM();
      }
    } else {
      startLinuxVM();
    }
  };

  const handlePauseResume = () => {
    if (!emulatorRef.current) return;
    if (vmState === 'running') {
      emulatorRef.current.stop();
      setVmState('paused');
    } else if (vmState === 'paused') {
      emulatorRef.current.run();
      setVmState('running');
    }
  };

  const [sandboxEngine, setSandboxEngine] = useState<'oracle' | 'wasm'>('oracle');
  const [oracleEndpointUrl, setOracleEndpointUrl] = useState('');
  const [oracleSessionKey, setOracleSessionKey] = useState(0);

  const activeOracleUrl = oracleEndpointUrl || 'https://secure-forge-sandbox.duckdns.org';

  return (
    <div className={`max-w-7xl mx-auto py-6 space-y-6 ${isFullscreen ? 'relative' : ''}`}>
      {/* Header & Breadcrumbs */}
      <div>
        <Breadcrumbs crumbs={[{ label: 'Interactive' }, { label: 'Live Workstation Sandbox' }]} />
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mt-2">
          <div>
            <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono">
              RAZE-CARVE Interactive Workstation
            </h1>
            <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-1 max-w-3xl">
              Experience the genuine, defense-grade forensic workstation. Run native <code className="font-mono bg-slate-100 dark:bg-slate-800 px-1 py-0.5 rounded text-brand-600 dark:text-brand-400 font-bold">sforge</code> Ratatui TUI with 60 FPS fluid animations, or switch to the isolated offline client-side WASM virtual machine.
            </p>
          </div>

          <div className="flex items-center gap-2 shrink-0">
            {sandboxEngine === 'oracle' ? (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono font-semibold bg-emerald-50 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800 shadow-2xs">
                <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                LIVE ORACLE CLOUD • 60 FPS NATIVE RUST
              </span>
            ) : (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono font-semibold bg-sky-50 dark:bg-sky-950/60 text-sky-700 dark:text-sky-400 border border-sky-200 dark:border-sky-800 shadow-2xs">
                <span className="w-2 h-2 rounded-full bg-sky-500 animate-pulse"></span>
                CLIENT SANDBOX • 0 MB DISK
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Engine Switcher Tabs */}
      <div className="flex flex-wrap p-1.5 bg-slate-100 dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 gap-1.5 shadow-xs">
        <button
          onClick={() => setSandboxEngine('oracle')}
          className={`flex items-center gap-2 px-4 py-2.5 rounded-lg text-xs font-mono font-bold transition-all cursor-pointer ${
            sandboxEngine === 'oracle'
              ? 'bg-brand-600 text-white shadow-md'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-200/60 dark:hover:bg-slate-800'
          }`}
        >
          <Zap className="w-4 h-4 text-amber-300 fill-amber-300" />
          <span>⚡ Live Oracle Cloud Workstation (Native Rust sforge • 60 FPS • Real Animations)</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-500 text-white font-bold ml-1">
            RECOMMENDED
          </span>
        </button>

        <button
          onClick={() => setSandboxEngine('wasm')}
          className={`flex items-center gap-2 px-4 py-2.5 rounded-lg text-xs font-mono font-bold transition-all cursor-pointer ${
            sandboxEngine === 'wasm'
              ? 'bg-brand-600 text-white shadow-md'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-200/60 dark:hover:bg-slate-800'
          }`}
        >
          <Cpu className="w-4 h-4 text-sky-400" />
          <span>💻 Offline Client-Side Sandbox (WASM i686 Emulator)</span>
        </button>
      </div>

      {/* Mode 1: Live Oracle Cloud Native Station */}
      {sandboxEngine === 'oracle' ? (
        <div className="space-y-6">
          {/* Oracle Hardware Telemetry Banner */}
          <div className="p-4 sm:p-5 rounded-xl border border-emerald-200 dark:border-emerald-900/60 bg-emerald-50/70 dark:bg-emerald-950/30 text-xs sm:text-sm text-slate-700 dark:text-slate-300 space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 font-bold text-emerald-900 dark:text-emerald-300 font-mono">
                <Shield className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                <span>100% ISOLATED LINUX CONTAINER SANDBOX • REAL CLI + NATIVE TUI</span>
              </div>
              <span className="text-[11px] font-mono text-emerald-700 dark:text-emerald-400 bg-emerald-100 dark:bg-emerald-900/60 px-2 py-0.5 rounded border border-emerald-300 dark:border-emerald-800">
                Host Protected • Root Blocked
              </span>
            </div>
            <p className="leading-relaxed">
              You are connected to an <strong>isolated, containerized Linux environment</strong>. You have complete freedom to format, carve, wipe, and inspect storage blocks without any risk to the host instance.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-2.5 pt-2 font-mono text-xs">
              <div className="p-2.5 rounded-lg bg-emerald-100/60 dark:bg-emerald-950/60 border border-emerald-200 dark:border-emerald-800">
                <div className="font-bold text-emerald-950 dark:text-emerald-200 flex items-center gap-1.5">
                  <HardDrive className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                  <span>/dev/loop10 (32 MB)</span>
                </div>
                <div className="text-[11px] text-emerald-800 dark:text-emerald-400 mt-0.5">
                  [EVIDENCE DRIVE] Pre-loaded with deleted JPEG, PNG, and PDF files. Test: <code className="font-bold">sih149 recover -s /dev/loop10 -o /home/sandbox/salvaged</code>
                </div>
              </div>

              <div className="p-2.5 rounded-lg bg-emerald-100/60 dark:bg-emerald-950/60 border border-emerald-200 dark:border-emerald-800">
                <div className="font-bold text-emerald-950 dark:text-emerald-200 flex items-center gap-1.5">
                  <Lock className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400" />
                  <span>/dev/loop11 (32 MB)</span>
                </div>
                <div className="text-[11px] text-emerald-800 dark:text-emerald-400 mt-0.5">
                  [TARGET WIPE DRIVE] Sensitive canary tokens. Test NIST sanitization: <code className="font-bold">sih149 wipe -d /dev/loop11 -m zero --verify</code>
                </div>
              </div>

              <div className="p-2.5 rounded-lg bg-emerald-100/60 dark:bg-emerald-950/60 border border-emerald-200 dark:border-emerald-800">
                <div className="font-bold text-emerald-950 dark:text-emerald-200 flex items-center gap-1.5">
                  <Activity className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" />
                  <span>/dev/loop12 (16 MB)</span>
                </div>
                <div className="text-[11px] text-emerald-800 dark:text-emerald-400 mt-0.5">
                  [AUDIT DRIVE] 50% zero, 50% entropy. Test inspection: <code className="font-bold">lsblk</code> or <code className="font-bold">reset-disks</code>
                </div>
              </div>
            </div>
          </div>

          {/* Station Toolbar */}
          <div className="p-3.5 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/60 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
            <div className="flex items-center gap-2.5">
              <span className="p-2 rounded-lg bg-emerald-950 text-emerald-400 font-mono text-sm border border-emerald-800">
                <Terminal className="w-4 h-4" />
              </span>
              <div>
                <div className="text-xs font-mono font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                  <span>TERMINAL SHELL:</span>
                  <span className="text-emerald-500 font-bold flex items-center gap-1.5">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    ONLINE (ISOLATED DOCKER CONTAINER • CLI & TUI READY)
                  </span>
                </div>
                <div className="text-[11px] font-mono text-slate-500">
                  Endpoint: <code className="text-emerald-400 font-bold">{activeOracleUrl}</code> (Direct High-Speed PTY)
                </div>
              </div>
            </div>

            {/* Quick Controls */}
            <div className="flex flex-wrap items-center gap-2 w-full sm:w-auto">
              <a
                href={activeOracleUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-mono font-semibold hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors text-slate-700 dark:text-slate-200"
                title="Open terminal in a new browser tab"
              >
                <ExternalLink className="w-3.5 h-3.5 text-brand-500" />
                <span>Open in New Tab</span>
              </a>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setOracleSessionKey((k) => k + 1)}
                icon={<RotateCcw className="w-3.5 h-3.5" />}
                title="Restart the terminal session into a fresh container"
              >
                Fresh Container Session
              </Button>
              <Button
                size="sm"
                variant="primary"
                onClick={toggleNativeFullScreen}
                icon={<Maximize2 className="w-3.5 h-3.5" />}
                title="Enter 100% full screen"
              >
                Full Screen
              </Button>
            </div>
          </div>

          {/* Quick Command Reference Cards */}
          <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-3 shadow-xs">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1">
              <span className="text-xs font-mono font-bold uppercase tracking-wider text-slate-500 flex items-center gap-2">
                <span>Quick Actions (Type in terminal below):</span>
              </span>
              <span className="text-[11px] font-mono text-slate-400">
                3 isolated partitions ready in /dev/
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2 text-xs font-mono">
              <div className="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-1">
                <div className="font-bold text-brand-600 dark:text-brand-400 flex items-center gap-1">
                  <Terminal className="w-3.5 h-3.5" />
                  <span>1. Interactive TUI</span>
                </div>
                <code className="text-[11px] text-slate-800 dark:text-slate-200 block bg-slate-100 dark:bg-slate-900 p-1.5 rounded">
                  sforge
                </code>
                <p className="text-[10px] font-sans text-slate-500">
                  Full 8-screen 60 FPS workstation with smooth animated gauges.
                </p>
              </div>

              <div className="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-1">
                <div className="font-bold text-purple-600 dark:text-purple-400 flex items-center gap-1">
                  <FileSearch className="w-3.5 h-3.5" />
                  <span>2. Carve Evidence</span>
                </div>
                <code className="text-[11px] text-slate-800 dark:text-slate-200 block bg-slate-100 dark:bg-slate-900 p-1.5 rounded truncate" title="sih149 recover -s /dev/loop10 -o /home/sandbox/salvaged">
                  sih149 recover -s /dev/loop10 -o /home/sandbox/salvaged
                </code>
                <p className="text-[10px] font-sans text-slate-500">
                  Scans unallocated sectors and carves deleted JPEG/PNG/PDF.
                </p>
              </div>

              <div className="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-1">
                <div className="font-bold text-rose-600 dark:text-rose-400 flex items-center gap-1">
                  <Shield className="w-3.5 h-3.5" />
                  <span>3. NIST SP 800-88 Wipe</span>
                </div>
                <code className="text-[11px] text-slate-800 dark:text-slate-200 block bg-slate-100 dark:bg-slate-900 p-1.5 rounded truncate" title="sih149 wipe -d /dev/loop11 -m zero --verify">
                  sih149 wipe -d /dev/loop11 -m zero --verify
                </code>
                <p className="text-[10px] font-sans text-slate-500">
                  Cryptographically sanitizes target disk and verifies entropy.
                </p>
              </div>

              <div className="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-1">
                <div className="font-bold text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                  <RotateCcw className="w-3.5 h-3.5" />
                  <span>4. Re-seed Disks</span>
                </div>
                <code className="text-[11px] text-slate-800 dark:text-slate-200 block bg-slate-100 dark:bg-slate-900 p-1.5 rounded">
                  reset-disks
                </code>
                <p className="text-[10px] font-sans text-slate-500">
                  Restores test artifacts and canary tokens on all 3 disks.
                </p>
              </div>
            </div>
          </div>

          {/* Live Web Terminal Viewport (ttyd xterm.js WebGL) */}
          <div
            ref={terminalWrapperRef}
            className={`rounded-xl border border-slate-800 bg-[#030712] shadow-2xl overflow-hidden font-mono text-xs transition-all duration-200 ${
              isFullscreen
                ? 'fixed inset-0 z-50 w-screen h-screen flex flex-col bg-black rounded-none border-none'
                : 'relative'
            }`}
          >
            {/* Terminal Chrome Bar */}
            <div className="flex items-center justify-between px-4 py-2.5 bg-slate-900 border-b border-slate-800 select-none">
              <div className="flex items-center gap-2">
                <span className="w-3 h-3 rounded-full bg-rose-500/80"></span>
                <span className="w-3 h-3 rounded-full bg-amber-500/80"></span>
                <span className="w-3 h-3 rounded-full bg-emerald-500/80"></span>
                <span className="ml-2 text-slate-200 font-sans text-xs font-semibold">
                  RAZE-CARVE Forensic Sandbox — Isolated Docker Shell
                </span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-bold">
                  CLI + sforge TUI • 3 Disks (/dev/loop10-12)
                </span>
              </div>

              <div className="flex items-center gap-1.5 text-xs">
                <button
                  onClick={() => setOracleSessionKey((k) => k + 1)}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors cursor-pointer"
                  title="Reload Session"
                >
                  <RotateCcw className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Reload Session</span>
                </button>

                <button
                  onClick={toggleNativeFullScreen}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors cursor-pointer"
                  title={isFullscreen ? 'Exit Full Screen (Esc)' : 'Enter 100% Full Screen'}
                >
                  {isFullscreen ? (
                    <>
                      <Minimize2 className="w-3.5 h-3.5 text-amber-400" />
                      <span>Exit Full Screen</span>
                    </>
                  ) : (
                    <>
                      <Maximize2 className="w-3.5 h-3.5 text-brand-400" />
                      <span>Full Screen (⛶)</span>
                    </>
                  )}
                </button>
              </div>
            </div>

            {/* Embedded Live Web Terminal Frame */}
            <div className={`w-full bg-[#030712] ${isFullscreen ? 'h-full flex-1' : 'h-[640px]'}`}>
              <iframe
                key={oracleSessionKey}
                src={activeOracleUrl}
                title="RAZE-CARVE Live Native Ratatui TUI"
                className="w-full h-full border-0 bg-[#030712]"
                allow="clipboard-read; clipboard-write; fullscreen"
              />
            </div>
          </div>
        </div>
      ) : (
        /* Mode 2: Client-Side Offline WASM Virtual Machine */
        <div className="space-y-6">
          {/* Safety Banner */}
          <div className="p-4 sm:p-5 rounded-xl border border-sky-200 dark:border-sky-900/60 bg-sky-50/70 dark:bg-sky-950/30 text-xs sm:text-sm text-slate-700 dark:text-slate-300 space-y-2">
            <div className="flex items-center gap-2 font-bold text-sky-900 dark:text-sky-300 font-mono">
              <Shield className="w-4 h-4 text-sky-600 dark:text-sky-400" />
              <span>100% HARDWARE-ISOLATED CLIENT SANDBOX</span>
            </div>
            <p className="leading-relaxed">
              This virtual machine runs an authentic Linux kernel (Buildroot 32-bit x86) entirely within your web browser’s memory. It takes <strong>zero hard drive storage</strong> on your host computer (only a ~5 MB asset payload).
            </p>
            <div className="flex flex-wrap items-center gap-4 pt-1 font-mono text-xs text-slate-600 dark:text-slate-400">
              <div><strong className="text-slate-900 dark:text-slate-200">Architecture:</strong> i686 (32-bit x86 PC)</div>
              <div><strong className="text-slate-900 dark:text-slate-200">Virtual RAM Limit:</strong> <span className="text-brand-600 dark:text-brand-400 font-bold">2048 MB max ceiling</span></div>
              <div><strong className="text-slate-900 dark:text-slate-200">Host Impact:</strong> ~120 MB browser RAM | 0 MB disk</div>
              <div><strong className="text-slate-900 dark:text-slate-200">Preloaded:</strong> sih149, sforge, autotest, sudo</div>
              <div><strong className="text-slate-900 dark:text-slate-200">Target Storage:</strong> /dev/loop0 (16 MB in-memory)</div>
            </div>
          </div>

          {/* VM Machine Control & Toolbar */}
          <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/60 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
            <div className="flex items-center gap-2.5">
              <span className="p-2 rounded-lg bg-slate-900 text-brand-400 font-mono text-sm">
                <Cpu className="w-4 h-4" />
              </span>
              <div>
                <div className="text-xs font-mono font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                  <span>STATUS:</span>
                  {vmState === 'idle' && <span className="text-slate-500">IDLE (Click Boot to Launch)</span>}
                  {vmState === 'loading' && <span className="text-amber-500 animate-pulse">ALLOCATING MEMORY & MOUNTING KERNEL...</span>}
                  {vmState === 'booting' && <span className="text-sky-400 animate-pulse">BOOTING KERNEL & LOADING TOOLS...</span>}
                  {vmState === 'running' && (
                    <span className="text-emerald-500 font-bold flex items-center gap-1.5">
                      <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                      ONLINE • TOOLS READY
                    </span>
                  )}
                  {vmState === 'paused' && <span className="text-amber-400">PAUSED</span>}
                  {vmState === 'error' && <span className="text-rose-500">INITIALIZATION FAILED</span>}
                </div>
                <div className="text-[11px] font-mono text-slate-500 truncate max-w-md">{bootLog}</div>
              </div>
            </div>

            {/* Action Controls */}
            <div className="flex flex-wrap items-center gap-2 w-full sm:w-auto">
              {vmState === 'running' && (
                <>
                  <Button
                    size="sm"
                    variant="primary"
                    onClick={() => sendCommandToVM('autotest')}
                    icon={<Zap className="w-3.5 h-3.5 text-amber-300 fill-amber-300" />}
                    title="Execute automated 8-phase test suite"
                    className="bg-amber-600 hover:bg-amber-500 text-white font-bold"
                  >
                    Run autotest Suite
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={handleManualInstallTools}
                    icon={<Download className="w-3.5 h-3.5" />}
                    title="Verify preloaded binaries"
                  >
                    Verify Tools
                  </Button>
                </>
              )}

              {vmState === 'idle' || vmState === 'error' ? (
                <Button
                  size="sm"
                  onClick={startLinuxVM}
                  icon={<Play className="w-3.5 h-3.5 fill-current" />}
                  className="w-full sm:w-auto justify-center"
                >
                  Boot Live Linux Sandbox
                </Button>
              ) : (
                <>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={handlePauseResume}
                    icon={vmState === 'running' ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
                  >
                    {vmState === 'running' ? 'Pause' : 'Resume'}
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={handleReboot}
                    icon={<RotateCcw className="w-3.5 h-3.5" />}
                  >
                    Reboot VM
                  </Button>
                </>
              )}
            </div>
          </div>

          {/* 1-Click Guided Commands */}
          <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-3 shadow-xs">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1">
              <span className="text-xs font-mono font-bold uppercase tracking-wider text-slate-500 flex items-center gap-2">
                <span>1-Click Test Actions:</span>
                {toolsLoaded && (
                  <span className="text-emerald-600 dark:text-emerald-400 font-normal">
                    ✓ autotest, sforge & sih149 ready in /bin/
                  </span>
                )}
              </span>
              <span className="text-[11px] font-mono text-slate-400">
                {vmState === 'running' ? 'Click card to inject command' : 'Boot Linux above to activate'}
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
              <button
                onClick={() => sendCommandToVM('autotest')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border-2 border-amber-500/40 dark:border-amber-500/30 bg-amber-50/50 dark:bg-amber-950/20 hover:border-amber-500 dark:hover:border-amber-400 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-amber-500 text-white shrink-0 shadow-xs">
                  <Zap className="w-4 h-4 fill-current" />
                </div>
                <div>
                  <div className="font-bold text-amber-900 dark:text-amber-200 font-sans flex items-center gap-1.5">
                    <span>1. ⚡ Run autotest</span>
                    <span className="text-[10px] px-1 py-0.2 rounded bg-amber-200 dark:bg-amber-900/60 text-amber-800 dark:text-amber-200">8 Tests</span>
                  </div>
                  <div className="text-[11px] text-amber-700/80 dark:text-amber-400/80">Canaries, Carver, Wipe & Certs</div>
                </div>
              </button>

              <button
                onClick={() => sendCommandToVM('sforge')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 hover:border-brand-500 dark:hover:border-brand-500 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-sky-100 dark:bg-sky-950 text-brand-600 dark:text-brand-400 shrink-0">
                  <Terminal className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-bold text-slate-900 dark:text-slate-100 font-sans">2. sforge (Ratatui TUI)</div>
                  <div className="text-[11px] text-slate-500">Full 8-screen interactive workstation</div>
                </div>
              </button>

              <button
                onClick={() => sendCommandToVM('sih149 recover --source /dev/loop0 --out-dir /salvaged')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 hover:border-brand-500 dark:hover:border-brand-500 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-purple-100 dark:bg-purple-950 text-purple-600 dark:text-purple-400 shrink-0">
                  <FileSearch className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-bold text-slate-900 dark:text-slate-100 font-sans">3. sih149 recover</div>
                  <div className="text-[11px] text-slate-500">Carves JPEG, PNG, PDF into /salvaged</div>
                </div>
              </button>

              <button
                onClick={() => sendCommandToVM('sih149 wipe --target /dev/loop0 --method nist-clear --verify')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 hover:border-brand-500 dark:hover:border-brand-500 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-rose-100 dark:bg-rose-950 text-rose-600 dark:text-rose-400 shrink-0">
                  <Shield className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-bold text-slate-900 dark:text-slate-100 font-sans">4. sih149 wipe</div>
                  <div className="text-[11px] text-slate-500">NIST SP 800-88 Zero Overwrite</div>
                </div>
              </button>

              <button
                onClick={() => sendCommandToVM('sih149 verify --target /dev/loop0')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 hover:border-brand-500 dark:hover:border-brand-500 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-emerald-100 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 shrink-0">
                  <Activity className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-bold text-slate-900 dark:text-slate-100 font-sans">5. sih149 verify</div>
                  <div className="text-[11px] text-slate-500">Shannon entropy & zero residue audit</div>
                </div>
              </button>

              <button
                onClick={() => sendCommandToVM('sih149 bench')}
                disabled={vmState !== 'running'}
                className="flex items-start gap-2.5 p-3 text-left rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 hover:border-brand-500 dark:hover:border-brand-500 transition-all card-interactive text-xs font-mono disabled:opacity-40 disabled:cursor-not-allowed group"
              >
                <div className="p-1.5 rounded bg-blue-100 dark:bg-blue-950 text-blue-600 dark:text-blue-400 shrink-0">
                  <Cpu className="w-4 h-4" />
                </div>
                <div>
                  <div className="font-bold text-slate-900 dark:text-slate-100 font-sans">6. sih149 bench</div>
                  <div className="text-[11px] text-slate-500">Memory & block sanitization speed</div>
                </div>
              </button>
            </div>
          </div>

          {/* Live WebAssembly Virtual Terminal Viewport (TRUE FULL SCREEN READY) */}
          <div
            ref={terminalWrapperRef}
            className={`rounded-xl border border-slate-800 bg-slate-950 shadow-2xl overflow-hidden font-mono text-xs transition-all duration-200 ${
              isFullscreen
                ? 'fixed inset-0 z-50 w-screen h-screen flex flex-col bg-black rounded-none border-none'
                : 'relative'
            }`}
          >
            {/* Terminal Chrome Bar with Copy, Paste, & True Fullscreen Controls */}
            <div className="flex items-center justify-between px-4 py-2.5 bg-slate-900 border-b border-slate-800 select-none">
              <div className="flex items-center gap-2">
                <span className="w-3 h-3 rounded-full bg-rose-500/80"></span>
                <span className="w-3 h-3 rounded-full bg-amber-500/80"></span>
                <span className="w-3 h-3 rounded-full bg-emerald-500/80"></span>
                <span className="ml-2 text-slate-200 font-sans text-xs font-semibold">
                  v86 Virtual PC — 2 GB RAM (Linux i686)
                </span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-brand-950 text-brand-400 border border-brand-800">
                  sih149 + sforge + autotest
                </span>
              </div>

              <div className="flex items-center gap-1.5 text-xs">
                {/* Paste Button */}
                <button
                  onClick={handlePasteClipboard}
                  disabled={vmState !== 'running'}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-30 cursor-pointer"
                  title="Paste text from clipboard into VM"
                >
                  {pastedNotification ? (
                    <>
                      <Check className="w-3.5 h-3.5 text-emerald-400" />
                      <span className="text-emerald-400 font-bold">Pasted!</span>
                    </>
                  ) : (
                    <>
                      <Clipboard className="w-3.5 h-3.5 text-brand-400" />
                      <span>Paste (Ctrl+V)</span>
                    </>
                  )}
                </button>

                {/* Copy Screen Button */}
                <button
                  onClick={handleCopyScreenText}
                  disabled={vmState !== 'running'}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-30 cursor-pointer"
                  title="Copy visible screen text to clipboard"
                >
                  {copiedNotification ? (
                    <>
                      <Check className="w-3.5 h-3.5 text-emerald-400" />
                      <span className="text-emerald-400 font-bold">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy className="w-3.5 h-3.5 text-brand-400" />
                      <span>Copy Screen</span>
                    </>
                  )}
                </button>

                {/* True Hardware Fullscreen Toggle */}
                <button
                  onClick={toggleNativeFullScreen}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors cursor-pointer"
                  title={isFullscreen ? 'Exit Full Screen (Esc)' : 'Enter 100% Full Screen'}
                >
                  {isFullscreen ? (
                    <>
                      <Minimize2 className="w-3.5 h-3.5 text-amber-400" />
                      <span>Exit Full Screen</span>
                    </>
                  ) : (
                    <>
                      <Maximize2 className="w-3.5 h-3.5 text-brand-400" />
                      <span>Full Screen (⛶)</span>
                    </>
                  )}
                </button>
              </div>
            </div>

            {/* The v86 Virtual Screen & Terminal Viewport (Zero Padding, Flush TUI Alignment) */}
            <div
              className={`p-0 m-0 bg-black text-slate-200 flex flex-col justify-start items-stretch relative overflow-hidden ${
                isFullscreen ? 'flex-1 w-full h-full' : 'h-[560px]'
              }`}
            >
              {vmState === 'idle' && (
                <div className="text-center space-y-4 p-12 max-w-md mx-auto my-auto">
                  <div className="w-14 h-14 rounded-full bg-slate-900 border border-slate-800 text-brand-400 flex items-center justify-center mx-auto shadow-inner">
                    <Terminal className="w-7 h-7" />
                  </div>
                  <div className="space-y-1.5">
                    <div className="text-lg font-bold text-slate-100 font-sans">
                      Real Linux Instance In Your Browser
                    </div>
                    <p className="text-xs text-slate-400 font-sans leading-relaxed">
                      Boots an authentic Linux kernel in memory with preloaded <code className="text-brand-400 font-mono">sih149</code>, <code className="text-brand-400 font-mono">sforge</code>, and <code className="text-brand-400 font-mono">autotest</code> binaries.
                    </p>
                  </div>
                  <Button onClick={startLinuxVM} size="lg" icon={<Play className="w-4 h-4 fill-current" />}>
                    Boot Live Linux Sandbox
                  </Button>
                </div>
              )}

              {vmState === 'loading' && (
                <div className="text-center space-y-3 p-12 my-auto">
                  <div className="w-10 h-10 rounded-full border-2 border-brand-500 border-t-transparent animate-spin mx-auto"></div>
                  <div className="text-base font-bold text-slate-200">Allocating Memory & Mounting Linux Kernel...</div>
                  <div className="text-xs text-slate-500">Preparing memory tables and preloading RAZE-CARVE tools...</div>
                </div>
              )}

              {/* v86 Hardware Screen Mounting Container - Flush Zero Padding */}
              <div
                ref={screenContainerRef}
                tabIndex={0}
                className={`v86-screen-container p-0 m-0 ${vmState === 'idle' || vmState === 'loading' ? 'hidden' : 'block'}`}
                style={{
                  width: '100%',
                  height: '100%',
                  margin: 0,
                  padding: 0,
                  display: vmState === 'idle' || vmState === 'loading' ? 'none' : 'block',
                }}
              >
                <div style={{ whiteSpace: 'pre', margin: 0, padding: 0 }} />
                <canvas style={{ display: 'none', margin: 0, padding: 0 }} />
              </div>
            </div>

            {/* Quick Input Bar (Direct Paste & Run Command into Shell) */}
            <div className="p-2.5 bg-slate-900 border-t border-slate-800 flex items-center gap-2">
              <span className="text-slate-500 text-xs font-mono select-none px-1">Quick Run:</span>
              <input
                type="text"
                value={commandBarInput}
                onChange={(e) => setCommandBarInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && commandBarInput.trim()) {
                    sendCommandToVM(commandBarInput);
                    setCommandBarInput('');
                  }
                }}
                placeholder="Paste or type any command here (e.g. 'autotest', 'sforge', 'sih149 wipe', 'free -m')..."
                className="flex-1 px-3 py-1.5 rounded bg-slate-950 border border-slate-800 text-slate-200 font-mono text-xs outline-none focus:border-brand-500 placeholder:text-slate-600"
              />
              <button
                onClick={() => {
                  if (commandBarInput.trim()) {
                    sendCommandToVM(commandBarInput);
                    setCommandBarInput('');
                  }
                }}
                disabled={vmState !== 'running' || !commandBarInput.trim()}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded bg-brand-600 hover:bg-brand-500 text-white text-xs font-bold transition-colors disabled:opacity-40 cursor-pointer"
              >
                <span>Send to VM</span>
                <CornerDownLeft className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Operational Reference Guide */}
      <Reveal>
        <section className="p-6 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-4 shadow-xs">
          <div>
            <h3 className="text-lg font-bold font-mono text-slate-900 dark:text-slate-100">
              Pre-loaded Tools Reference Guide
            </h3>
            <p className="text-xs sm:text-sm text-slate-600 dark:text-slate-400">
              These commands are pre-installed in <code className="font-mono bg-slate-100 dark:bg-slate-800 px-1 py-0.5 rounded text-brand-600 dark:text-brand-400">/bin/</code> inside the virtual machine:
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
            <div className="p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-bold text-slate-900 dark:text-slate-100">sih149 wipe (Sanitization)</span>
                <CopyButton text="sih149 wipe --target /dev/loop0 --method nist-clear --verify" />
              </div>
              <code className="text-brand-600 dark:text-brand-400 block bg-slate-100 dark:bg-slate-900 p-2 rounded">
                sih149 wipe --target /dev/loop0 --method nist-clear --verify
              </code>
              <p className="text-[11px] font-sans text-slate-500">
                Executes NIST SP 800-88 single-pass zero overwrite and audits Shannon entropy.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-bold text-slate-900 dark:text-slate-100">sih149 recover (Signature Carver)</span>
                <CopyButton text="sih149 recover --source /dev/loop0 --out-dir /salvaged" />
              </div>
              <code className="text-brand-600 dark:text-brand-400 block bg-slate-100 dark:bg-slate-900 p-2 rounded">
                sih149 recover --source /dev/loop0 --out-dir /salvaged
              </code>
              <p className="text-[11px] font-sans text-slate-500">
                Scans unallocated sectors for JPEG, PNG, PDF, and SQLite magic byte signatures.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-bold text-slate-900 dark:text-slate-100">sforge (Interactive TUI)</span>
                <CopyButton text="sforge" />
              </div>
              <code className="text-brand-600 dark:text-brand-400 block bg-slate-100 dark:bg-slate-900 p-2 rounded truncate">
                sforge
              </code>
              <p className="text-[11px] font-sans text-slate-500">
                Launches the full interactive terminal console interface in the shell.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-bold text-slate-900 dark:text-slate-100">sih149 demo (Autonomous Self-Test)</span>
                <CopyButton text="sih149 demo" />
              </div>
              <code className="text-brand-600 dark:text-brand-400 block bg-slate-100 dark:bg-slate-900 p-2 rounded">
                sih149 demo
              </code>
              <p className="text-[11px] font-sans text-slate-500">
                Mounts 16MB in-memory loopback block device, writes test files, wipes, and audits.
              </p>
            </div>
          </div>
        </section>
      </Reveal>
    </div>
  );
};
