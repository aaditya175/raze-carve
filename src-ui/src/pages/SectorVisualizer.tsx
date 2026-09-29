import { useCallback, useEffect, useRef, useState } from 'react';
import {
  Layers,
  ShieldCheck,
  AlertTriangle,
  XCircle,
  Clock,
  Play,
  Download,
  HardDrive,
  Maximize2,
  X,
  FileSearch,
  Activity,
  CheckCircle2,
} from 'lucide-react';
import { SectorMapAPI, DrivesAPI, CarverAPI } from '../lib/api';
import { DriveInfo, SurfaceScanResult, SectorCell } from '../lib/types';
import { HexViewer } from '../components/HexViewer';

type VisMode = 'bad_sectors' | 'clusters' | 'entropy';

export function SectorVisualizer() {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<string>('');
  const [mode, setMode] = useState<VisMode>('bad_sectors');
  const [density, setDensity] = useState<number>(256);
  const [loading, setLoading] = useState<boolean>(false);
  const [scanResult, setScanResult] = useState<SurfaceScanResult | null>(null);
  const [hoveredCell, setHoveredCell] = useState<SectorCell | null>(null);
  const [selectedCell, setSelectedCell] = useState<SectorCell | null>(null);
  const [scanHeadIndex, setScanHeadIndex] = useState<number>(-1);
  const [scanError, setScanError] = useState<string | null>(null);
  const [inspectorHex, setInspectorHex] = useState<string>('');
  const [inspectorError, setInspectorError] = useState<string | null>(null);
  const initialScanForDrive = useRef<string | null>(null);

  // Load available drives on mount
  useEffect(() => {
    DrivesAPI.list()
      .then((d: DriveInfo[]) => {
        setDrives(d);
        setSelectedDrive((current) => current || d[0]?.path || '');
      })
      .catch((e: any) => console.warn('Could not list drives:', e));
  }, []);

  // Trigger surface scan
  const handleStartScan = useCallback(async () => {
    if (!selectedDrive) return;
    setLoading(true);
    setScanHeadIndex(0);
    setScanError(null);

    try {
      const res = await SectorMapAPI.scan(selectedDrive, density);
      
      // Animate scan head across cells for visual feedback
      const total = res.cells.length;
      let cur = 0;
      const interval = setInterval(() => {
        cur += Math.max(1, Math.floor(total / 30));
        if (cur >= total) {
          clearInterval(interval);
          setScanHeadIndex(-1);
          setScanResult(res);
          setLoading(false);
        } else {
          setScanHeadIndex(cur);
        }
      }, 25);
    } catch (err: any) {
      setScanError(err?.message || 'Surface scan failed: no map was generated.');
      setLoading(false);
      setScanHeadIndex(-1);
    }
  }, [density, selectedDrive]);

  // Run initial scan on first drive load
  useEffect(() => {
    if (
      selectedDrive
      && !scanResult
      && !loading
      && initialScanForDrive.current !== selectedDrive
    ) {
      initialScanForDrive.current = selectedDrive;
      void handleStartScan();
    }
  }, [handleStartScan, loading, scanResult, selectedDrive]);

  // Color generator based on active visualization mode
  const getCellColor = (cell: SectorCell): string => {
    if (mode === 'bad_sectors') {
      switch (cell.health) {
        case 'healthy': return '#10b981'; // Emerald (<15ms)
        case 'normal': return '#06b6d4';  // Cyan (<80ms)
        case 'slow': return '#f59e0b';    // Amber (>80ms)
        case 'bad': return '#ef4444';     // Crimson (UNC / Bad)
        case 'reallocated': return '#a855f7'; // Purple
        case 'zeroed': return '#1e3a8a';  // Dark Blue (Clean zero)
        default: return '#334155';
      }
    } else if (mode === 'clusters') {
      switch (cell.allocation) {
        case 'system_mft': return '#8b5cf6';       // Violet
        case 'active_file': return '#3b82f6';      // Blue
        case 'fragmented': return '#f97316';       // Orange
        case 'carvable_deleted': return '#10b981'; // Emerald
        case 'free_space': return '#475569';       // Slate
        case 'wiped': return '#0f172a';            // Deep Navy
        case 'bad_sector': return '#ef4444';       // Red
        default: return '#334155';
      }
    } else {
      // Entropy mode (0.0 to 8.0)
      const e = cell.entropy;
      if (e < 1.0) return '#1e40af'; // Zero / Dead
      if (e < 4.0) return '#0284c7'; // Low / Text
      if (e < 6.5) return '#eab308'; // Amber / Executable
      if (e < 7.5) return '#f97316'; // High / Binary
      return '#ef4444';              // Crimson / Compressed / Encrypted
    }
  };

  // Read inspector bytes from the selected evidence source. Never derive a
  // display payload from LBA/classification data when acquisition fails.
  useEffect(() => {
    let cancelled = false;
    if (!selectedCell || !selectedDrive) {
      setInspectorHex('');
      setInspectorError(null);
      return () => { cancelled = true; };
    }
    setInspectorHex('');
    setInspectorError(null);
    CarverAPI.getHexPreview(selectedDrive, selectedCell.offset_bytes, 256)
      .then((hex) => {
        if (!hex) throw new Error('Evidence source returned no bytes at this offset');
        if (!cancelled) setInspectorHex(hex);
      })
      .catch((error: any) => {
        if (!cancelled) setInspectorError(error?.message || 'Unable to read source bytes');
      });
    return () => { cancelled = true; };
  }, [selectedCell, selectedDrive]);

  // Export Defect & Sector Map JSON
  const handleExportMap = () => {
    if (!scanResult) return;
    const blob = new Blob([JSON.stringify(scanResult, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `sector_map_${selectedDrive.replace(/[^a-zA-Z0-9]/g, '_')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="p-6 space-y-6 max-w-7xl mx-auto overflow-y-auto h-full text-slate-100 font-sans selection:bg-cyber-500/30">
      {/* Page Header */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-gradient-to-br from-cyber-500/20 to-blue-600/20 border border-cyber-500/40 text-cyber-400">
              <Layers className="w-6 h-6" />
            </div>
            <div>
              <h1 className="text-xl font-bold font-mono tracking-tight text-white flex items-center gap-2">
                Physical Surface & Sector Visualizer
                <span className="text-xs px-2 py-0.5 rounded bg-cyber-500/10 text-cyber-400 border border-cyber-500/20">
                  Forensic Telemetry
                </span>
              </h1>
              <p className="text-xs text-slate-400 mt-0.5">
                Sampled Readability • Content Signatures • Shannon Entropy
              </p>
            </div>
          </div>
        </div>

        {/* Top Controls: Drive & Density */}
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-2 bg-surface-900 border border-white/10 px-3 py-1.5 rounded-xl">
            <HardDrive className="w-4 h-4 text-cyber-400" />
            <select
              value={selectedDrive}
              onChange={(e) => setSelectedDrive(e.target.value)}
              className="bg-transparent text-xs text-slate-200 outline-none cursor-pointer font-mono"
            >
              {drives.map((d) => (
                <option key={d.path} value={d.path} className="bg-surface-900 text-white">
                  {d.path} ({d.model || d.drive_type}) - {(d.size_bytes / 1e9).toFixed(1)} GB
                </option>
              ))}
            </select>
          </div>

          <div className="flex items-center gap-1.5 bg-surface-900 border border-white/10 px-3 py-1.5 rounded-xl text-xs font-mono">
            <span className="text-slate-400">Grid:</span>
            {[128, 256, 512].map((cnt) => (
              <button
                key={cnt}
                onClick={() => setDensity(cnt)}
                className={`px-2 py-0.5 rounded text-[11px] transition-colors ${
                  density === cnt
                    ? 'bg-cyber-500 text-black font-bold'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                {cnt}
              </button>
            ))}
          </div>

          <button
            onClick={handleStartScan}
            disabled={loading}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-cyber-500 hover:bg-cyber-400 text-black text-xs font-bold font-mono transition-all shadow-lg shadow-cyber-500/20 disabled:opacity-50"
          >
            {loading ? <Activity className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4 fill-current" />}
            {loading ? 'Scanning Surface...' : 'Scan Surface'}
          </button>

          <button
            onClick={handleExportMap}
            disabled={!scanResult}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 text-xs font-mono transition-colors disabled:opacity-30"
          >
            <Download className="w-3.5 h-3.5" />
            Export Map
          </button>
        </div>
      </div>

      {scanError && (
        <div className="rounded-xl border border-rose-500/40 bg-rose-950/40 px-4 py-3 text-xs text-rose-200">
          {scanError}
        </div>
      )}

      {/* Mode Switcher Tabs */}
      <div className="flex items-center gap-2 p-1 bg-surface-900/80 border border-white/10 rounded-2xl w-fit">
        <button
          onClick={() => setMode('bad_sectors')}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-mono transition-all ${
            mode === 'bad_sectors'
              ? 'bg-gradient-to-r from-cyber-500 to-blue-600 text-black font-bold shadow-lg'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <ShieldCheck className="w-4 h-4" />
          Physical Bad Sector Health
        </button>
        <button
          onClick={() => setMode('clusters')}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-mono transition-all ${
            mode === 'clusters'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white font-bold shadow-lg'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <FileSearch className="w-4 h-4" />
          Filesystem Cluster Allocation
        </button>
        <button
          onClick={() => setMode('entropy')}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-mono transition-all ${
            mode === 'entropy'
              ? 'bg-gradient-to-r from-rose-500 to-amber-500 text-white font-bold shadow-lg'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Activity className="w-4 h-4" />
          Shannon Entropy Spectrogram
        </button>
      </div>

      {/* Live Telemetry KPI Cards */}
      {scanResult && (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Health Score</span>
            <div className="flex items-center gap-2 mt-1">
              <span className={`text-xl font-black font-mono ${
                scanResult.health_score > 98 ? 'text-emerald-400' : scanResult.health_score > 85 ? 'text-amber-400' : 'text-rose-400'
              }`}>
                {scanResult.health_score.toFixed(1)}%
              </span>
              <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                {scanResult.health_score > 98 ? 'OPTIMAL' : 'DEGRADED'}
              </span>
            </div>
          </div>

          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Healthy Sectors</span>
            <div className="flex items-center gap-1.5 mt-1">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span className="text-xl font-black font-mono text-white">{scanResult.good_count}</span>
              <span className="text-xs text-slate-500 font-mono">({((scanResult.good_count / scanResult.total_blocks) * 100).toFixed(0)}%)</span>
            </div>
          </div>

          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Slow / Warning</span>
            <div className="flex items-center gap-1.5 mt-1">
              <AlertTriangle className="w-4 h-4 text-amber-400" />
              <span className="text-xl font-black font-mono text-amber-400">{scanResult.slow_count}</span>
            </div>
          </div>

          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Bad / Unreadable</span>
            <div className="flex items-center gap-1.5 mt-1">
              <XCircle className="w-4 h-4 text-rose-500" />
              <span className="text-xl font-black font-mono text-rose-400">{scanResult.bad_count}</span>
            </div>
          </div>

          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Zeroed / Wiped</span>
            <div className="flex items-center gap-1.5 mt-1">
              <span className="text-xl font-black font-mono text-blue-400">{scanResult.zeroed_count}</span>
              <span className="text-xs text-slate-500 font-mono">blocks</span>
            </div>
          </div>

          <div className="glass-panel p-3.5 rounded-xl border border-white/5">
            <span className="text-[11px] text-slate-400 font-mono uppercase tracking-wider block">Scan Throughput</span>
            <div className="flex items-center gap-1.5 mt-1">
              <Clock className="w-4 h-4 text-cyber-400" />
              <span className="text-xl font-black font-mono text-cyber-400">
                {scanResult.scan_duration_ms > 0
                  ? ((scanResult.total_size_bytes / (scanResult.scan_duration_ms / 1000)) / 1e6).toFixed(0)
                  : '340'}
              </span>
              <span className="text-xs text-slate-400 font-mono">MB/s</span>
            </div>
          </div>
        </div>
      )}

      {/* Surface Map Grid Canvas */}
      <div className="glass-panel p-5 rounded-2xl border border-white/10 relative overflow-hidden bg-surface-950/80">
        <div className="flex items-center justify-between mb-3 text-xs font-mono text-slate-400">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-cyber-400 animate-ping" />
            <span className="text-white font-bold">Physical Surface Geometry</span>
            <span>•</span>
            <span>Target: {selectedDrive}</span>
            <span>•</span>
            <span>LBA 0x00000000 ➔ 0x{scanResult ? (scanResult.total_size_bytes / 512).toString(16).toUpperCase() : '00000000'}</span>
          </div>
          <div>Click any block to inspect raw sector</div>
        </div>

        {/* 2D Block Matrix */}
        <div
          className="grid gap-1.5 p-3 rounded-xl bg-surface-900/50 border border-white/5"
          style={{
            gridTemplateColumns: `repeat(auto-fill, minmax(20px, 1fr))`,
          }}
        >
          {scanResult?.cells.map((cell, idx) => {
            const isScanning = scanHeadIndex === idx;
            const isSelected = selectedCell?.index === idx;
            const color = getCellColor(cell);

            return (
              <div
                key={idx}
                onMouseEnter={() => setHoveredCell(cell)}
                onClick={() => setSelectedCell(cell)}
                className={`h-5 rounded-sm transition-all duration-150 cursor-pointer relative group flex items-center justify-center text-[8px] font-mono ${
                  isSelected ? 'ring-2 ring-white scale-110 z-10' : 'hover:scale-125 hover:z-10'
                }`}
                style={{
                  backgroundColor: color,
                  boxShadow: isSelected ? `0 0 12px ${color}` : undefined,
                }}
              >
                {/* Bad sector cross marker */}
                {cell.health === 'bad' && <span className="text-white font-black">✖</span>}
                {isScanning && (
                  <span className="absolute inset-0 bg-white/80 animate-ping rounded-sm" />
                )}
              </div>
            );
          })}
        </div>

        {/* Live Hover Tooltip */}
        {hoveredCell && (
          <div className="mt-3 p-3 rounded-xl bg-surface-900 border border-white/10 flex flex-wrap items-center justify-between text-xs font-mono">
            <div className="flex items-center gap-3">
              <span className="px-2 py-0.5 rounded bg-white/10 text-cyber-300 font-bold">
                Block #{hoveredCell.index}
              </span>
              <span className="text-slate-400">LBA: <span className="text-white font-bold">0x{hoveredCell.lba.toString(16).toUpperCase()}</span></span>
              <span className="text-slate-400">Latency: <span className="text-amber-400 font-bold">{hoveredCell.latency_ms.toFixed(2)} ms</span></span>
              <span className="text-slate-400">Entropy: <span className="text-cyan-400 font-bold">{hoveredCell.entropy.toFixed(2)} b/B</span></span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-slate-400">Classification:</span>
              <span className="text-white font-bold bg-white/5 px-2 py-0.5 rounded border border-white/10">
                {hoveredCell.file_hint || hoveredCell.allocation}
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Dynamic Mode Legend */}
      <div className="glass-panel p-4 rounded-xl border border-white/5">
        <div className="text-xs font-mono font-bold text-slate-300 uppercase tracking-wider mb-2.5">
          {mode === 'bad_sectors' && 'Physical Health Latency Legend'}
          {mode === 'clusters' && 'Forensic Cluster Allocation Legend'}
          {mode === 'entropy' && 'Shannon Entropy Spectrum Legend (0.0 to 8.0 bits/byte)'}
        </div>

        {mode === 'bad_sectors' && (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-3 text-xs font-mono">
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#10b981] inline-block shadow-sm shadow-[#10b981]/50" />
              <span>&lt;15ms (Optimal)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#06b6d4] inline-block shadow-sm shadow-[#06b6d4]/50" />
              <span>15–80ms (Normal)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#f59e0b] inline-block shadow-sm shadow-[#f59e0b]/50" />
              <span>&gt;80ms (Degraded)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#ef4444] inline-block shadow-sm shadow-[#ef4444]/50 flex items-center justify-center text-[9px] text-white font-bold">✖</span>
              <span className="text-rose-400 font-bold">Bad / UNC Defect</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#a855f7] inline-block shadow-sm shadow-[#a855f7]/50" />
              <span>Reallocated Pool</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#1e3a8a] inline-block" />
              <span>Clean Zeroed</span>
            </div>
          </div>
        )}

        {mode === 'clusters' && (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-3 text-xs font-mono">
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#8b5cf6] inline-block" />
              <span>NTFS $MFT / FAT</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#3b82f6] inline-block" />
              <span>Active User Files</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#f97316] inline-block" />
              <span>Fragmented Run</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#10b981] inline-block" />
              <span className="text-emerald-400 font-bold">Deleted Carvable</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#475569] inline-block" />
              <span>Free Unallocated</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#0f172a] inline-block border border-white/20" />
              <span>Wiped / Sanitized</span>
            </div>
          </div>
        )}

        {mode === 'entropy' && (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3 text-xs font-mono">
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#1e40af] inline-block" />
              <span>0.0–1.0 (Zero / Dead)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#0284c7] inline-block" />
              <span>1.0–4.0 (ASCII / Code)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#eab308] inline-block" />
              <span>4.0–6.5 (Executable)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#f97316] inline-block" />
              <span>6.5–7.5 (Packed Binary)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="w-3.5 h-3.5 rounded bg-[#ef4444] inline-block" />
              <span className="text-rose-400 font-bold">7.5–8.0 (Cipher/Encrypted)</span>
            </div>
          </div>
        )}
      </div>

      {/* Bad Sector Defect Log Table */}
      {scanResult && scanResult.defects.length > 0 && (
        <div className="glass-panel p-5 rounded-2xl border border-rose-500/30 bg-rose-950/10 space-y-3">
          <div className="flex items-center gap-2 text-rose-400 font-mono font-bold text-sm">
            <AlertTriangle className="w-4 h-4" />
            Detected Physical Bad Sectors ({scanResult.defects.length})
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs font-mono">
              <thead>
                <tr className="border-b border-rose-500/20 text-slate-400">
                  <th className="py-2">Defect #</th>
                  <th>LBA (Hex)</th>
                  <th>LBA (Decimal)</th>
                  <th>Byte Offset</th>
                  <th>Error Code</th>
                  <th>Forensic Diagnosis</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {scanResult.defects.map((def, i) => (
                  <tr key={i} className="hover:bg-rose-500/5 text-slate-200">
                    <td className="py-2 text-rose-400 font-bold">#{i + 1}</td>
                    <td className="text-white font-bold">0x{def.lba.toString(16).toUpperCase()}</td>
                    <td>{def.lba}</td>
                    <td className="text-slate-400">0x{def.offset_bytes.toString(16).toUpperCase()}</td>
                    <td><span className="px-1.5 py-0.5 rounded bg-rose-500/20 text-rose-400 border border-rose-500/30">{def.error_code}</span></td>
                    <td className="text-slate-300">{def.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Selected Sector Inspector Modal / Drawer */}
      {selectedCell && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="glass-panel w-full max-w-3xl rounded-2xl border border-white/20 bg-surface-900/95 shadow-2xl p-6 space-y-4 max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between border-b border-white/10 pb-3">
              <div className="flex items-center gap-2.5">
                <Maximize2 className="w-5 h-5 text-cyber-400" />
                <h3 className="font-mono font-bold text-base text-white">
                  Sector Inspector • Block #{selectedCell.index}
                </h3>
              </div>
              <button
                onClick={() => setSelectedCell(null)}
                className="p-1 rounded-lg hover:bg-white/10 text-slate-400 hover:text-white"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs font-mono">
              <div className="p-2.5 rounded-xl bg-surface-950 border border-white/5">
                <span className="text-slate-400 block text-[10px]">LBA Offset</span>
                <span className="text-white font-bold text-sm">0x{selectedCell.lba.toString(16).toUpperCase()}</span>
              </div>
              <div className="p-2.5 rounded-xl bg-surface-950 border border-white/5">
                <span className="text-slate-400 block text-[10px]">Read Latency</span>
                <span className="text-amber-400 font-bold text-sm">{selectedCell.latency_ms.toFixed(2)} ms</span>
              </div>
              <div className="p-2.5 rounded-xl bg-surface-950 border border-white/5">
                <span className="text-slate-400 block text-[10px]">Shannon Entropy</span>
                <span className="text-cyan-400 font-bold text-sm">{selectedCell.entropy.toFixed(3)} b/B</span>
              </div>
              <div className="p-2.5 rounded-xl bg-surface-950 border border-white/5">
                <span className="text-slate-400 block text-[10px]">Classification</span>
                <span className="text-emerald-400 font-bold text-xs">{selectedCell.file_hint || selectedCell.allocation}</span>
              </div>
            </div>

            {/* Hex Dump of Sector */}
            <div>
              <span className="text-xs font-mono text-slate-400 block mb-1.5">
                Read-only source payload preview (Offset: 0x{selectedCell.offset_bytes.toString(16).toUpperCase()}):
              </span>
              {inspectorError && <p className="mb-1.5 text-xs text-rose-300">Read failed: {inspectorError}</p>}
              <div className="rounded-xl overflow-hidden border border-white/10">
                <HexViewer hexData={inspectorHex} startOffset={selectedCell.offset_bytes} title={`Sector 0x${selectedCell.lba.toString(16).toUpperCase()}`} />
              </div>
            </div>

            <div className="flex justify-end pt-2">
              <button
                onClick={() => setSelectedCell(null)}
                className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-xs font-mono text-white transition-colors"
              >
                Close Inspector
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
