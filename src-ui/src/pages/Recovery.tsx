import { useState, useEffect } from 'react';
import { CarverAPI } from '../lib/api';
import { useDrives } from '../hooks/useDrives';
import { DriveCard } from '../components/DriveCard';
import { FileTable } from '../components/FileTable';
import { HexViewer } from '../components/HexViewer';
import { EntropyHeatmap } from '../components/EntropyHeatmap';
import { FolderSelector } from '../components/FolderSelector';
import { CarvedFile, ScanProgress, ScanResult, DriveInfo } from '../lib/types';
import {
  Play,
  Sparkles,
  FileCheck,
  HardDrive,
  FileCode,
  Trash2,
  RefreshCw,
  Info,
  ShieldAlert,
  FolderCheck,
  CheckCircle2,
  Eye,
  Download,
} from 'lucide-react';

import { MediaPreviewModal } from '../components/MediaPreviewModal';

export interface RecoveryProps {
  onNavigate?: (page: 'dash' | 'wipe' | 'shred' | 'carve' | 'sectors' | 'reports' | 'expert' | 'plugins' | 'diagnostics') => void;
}

export function Recovery({ onNavigate }: RecoveryProps = {}) {
  const { drives, loading: drivesLoading, refresh: refreshDrives } = useDrives();

  // Source selection mode: 'drive' (physical block device) or 'file' (dd image / raw dump)
  const [sourceMode, setSourceMode] = useState<'drive' | 'file'>('drive');
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(null);
  const [sourcePath, setSourcePath] = useState('');
  const [outputDir, setOutputDir] = useState('./recovered_evidence');
  const [minConfidence, setMinConfidence] = useState(50);
  const [selectedTypes, setSelectedTypes] = useState<string[]>(['jpg', 'png', 'pdf', 'sqlite', 'zip']);
  const [selectedModes, setSelectedModes] = useState<string[]>(['hybrid']);
  const [isInspectOnly, setIsInspectOnly] = useState(false);
  const [selectedFileIds, setSelectedFileIds] = useState<string[]>([]);
  const [isExtracting, setIsExtracting] = useState(false);
  const [extractionMessage, setExtractionMessage] = useState<string | null>(null);

  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [selectedFile, setSelectedFile] = useState<CarvedFile | null>(null);
  const [previewModalFile, setPreviewModalFile] = useState<CarvedFile | null>(null);
  const [hexDump, setHexDump] = useState<string | null>(null);
  const [hexError, setHexError] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Auto-select first available non-system drive, or first drive when drives list loads
  useEffect(() => {
    if (drives.length > 0 && !selectedDrive && !sourcePath) {
      const nonSystem = drives.find((d) => !d.is_system_drive);
      const chosen = nonSystem || drives[0];
      setSelectedDrive(chosen);
      setSourcePath(chosen.path);
    }
  }, [drives, selectedDrive, sourcePath]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    CarverAPI.onProgress((p) => setProgress(p)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleSelectDrive = (drive: DriveInfo) => {
    setSelectedDrive(drive);
    setSourcePath(drive.path);
  };

  const handleToggleType = (type: string) => {
    setSelectedTypes((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type]
    );
  };

  const handleToggleMode = (mode: string) => {
    if (mode === 'hybrid') {
      setSelectedModes(['hybrid']);
      return;
    }
    setSelectedModes((prev) => {
      const withoutHybrid = prev.filter((m) => m !== 'hybrid');
      if (withoutHybrid.includes(mode)) {
        const next = withoutHybrid.filter((m) => m !== mode);
        return next.length === 0 ? ['hybrid'] : next;
      } else {
        return [...withoutHybrid, mode];
      }
    });
  };

  const handleStartScan = async () => {
    if (!sourcePath.trim()) {
      setErrorMessage('Please select a storage drive or specify an evidence source path.');
      return;
    }

    setIsScanning(true);
    setResult(null);
    setSelectedFile(null);
    setSelectedFileIds([]);
    setHexDump(null);
    setHexError(null);
    setErrorMessage(null);
    setExtractionMessage(null);

    try {
      const res = await CarverAPI.start({
        source_path: sourcePath.trim(),
        output_dir: outputDir.trim() || './recovered_evidence',
        file_types: selectedTypes,
        min_confidence: minConfidence,
        recovery_modes: selectedModes,
        dry_run: isInspectOnly,
      });
      // Defensively normalize the result: every mode must expose a `files` array
      const normalized = {
        ...res,
        files: Array.isArray(res?.files) ? res.files : [],
      };
      setResult(normalized);
      if (isInspectOnly) {
        setExtractionMessage(
          `Inspection scan complete: ${normalized.files.length} artifact(s) discovered in-memory. Select files below to extract.`
        );
      }
      if (normalized.files.length > 0) {
        handleInspectFile(normalized.files[0]);
      }
    } catch (err: any) {
      setErrorMessage(err?.message || err?.toString?.() || 'Scan failed');
    } finally {
      setIsScanning(false);
      setProgress(null);
    }
  };

  const handleToggleSelectFile = (id: string) => {
    setSelectedFileIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  };

  const handleToggleSelectAll = () => {
    if (!result || !result.files) return;
    if (selectedFileIds.length === result.files.length) {
      setSelectedFileIds([]);
    } else {
      setSelectedFileIds(result.files.map((f) => f.id));
    }
  };

  const handleExtractFiles = async (filesToExtract: CarvedFile[]) => {
    if (filesToExtract.length === 0) return;
    setIsExtracting(true);
    setExtractionMessage(null);
    setErrorMessage(null);
    try {
      const res = await CarverAPI.extractSelected({
        source_path: sourcePath.trim(),
        output_dir: outputDir.trim() || './recovered_evidence',
        files: filesToExtract,
      });
      setExtractionMessage(
        `Successfully extracted ${res.extracted_count} file(s) (${(res.total_size_bytes / 1024).toFixed(1)} KB) to ${outputDir.trim() || './recovered_evidence'}`
      );
    } catch (err: any) {
      setErrorMessage(err?.message || err?.toString?.() || 'Extraction failed');
    } finally {
      setIsExtracting(false);
    }
  };

  const handleInspectFile = async (file: CarvedFile) => {
    setSelectedFile(file);
    setHexDump(null);
    setHexError(null);
    try {
      const hex = await CarverAPI.getHexPreview(sourcePath, file.offset_bytes, 256);
      if (!hex) throw new Error('Evidence source returned no bytes at this offset');
      setHexDump(hex);
    } catch (error: any) {
      setHexError(error?.message || 'Unable to read bytes from the evidence source');
    }
  };

  const recoveryStrategies = [
    {
      id: 'hybrid',
      name: 'Hybrid Multi-Strategy Ensemble (Recommended)',
      desc: 'Combines MFT/FAT metadata, syntactic container parsing, magic signatures, and entropy stream clustering for maximum file and metadata recovery.',
      badge: 'Best-in-Class',
    },
    {
      id: 'filesystem',
      name: 'Filesystem Metadata & Inode Carving',
      desc: 'Scans unallocated space for deleted NTFS $MFT records and FAT directory entries. Recovers original file names, creation/modification dates, and resident data.',
      badge: 'Metadata Recovery',
    },
    {
      id: 'structure',
      name: 'Deep Structural & Container Carving',
      desc: 'Parses internal container trees: ZIP Central Directory (extracts internal file trees & uncompressed sizes), SQLite B-Trees, PNG chunk CRC32, and JPEG EXIF.',
      badge: 'Structure-Aware',
    },
    {
      id: 'entropy',
      name: 'Statistical Shannon Entropy Stream Clustering',
      desc: 'Classifies continuous statistical entropy blocks (e.g. >7.2b/B) to carve fragmented or headerless compressed/encrypted data streams.',
      badge: 'Headerless Streams',
    },
    {
      id: 'magic',
      name: 'Signature Magic Bytes (Header & Footer Delimiters)',
      desc: 'Fast sector-aligned magic byte header search with matching footer termination boundaries.',
      badge: 'Fast Scan',
    },
  ];

  return (
    <div className="space-y-6 pb-12">
      {/* Header Banner */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 glass-panel p-6 rounded-2xl border border-white/10 bg-gradient-to-r from-surface-900/90 via-surface-900/60 to-surface-950/90">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-cyber-500/20 text-cyber-400 border border-cyber-500/30">
              Forensic Carving Suite
            </span>
            <span className="px-2 py-0.5 rounded-full text-[10px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Read-Only Safe
            </span>
          </div>
          <h1 className="text-2xl font-extrabold tracking-tight text-white">
            Deep Signature Scanner & Multi-Method Recovery Engine
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Recovers orphaned files and forensic metadata without solely depending on headers, using MFT/FAT inode carving, container syntax, and entropy clustering.
          </p>
        </div>

        {isScanning ? (
          <button
            onClick={() => CarverAPI.cancel()}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-rose-600/30 transition-all cursor-pointer"
          >
            <RefreshCw className="w-4 h-4 animate-spin" /> Stop Scan
          </button>
        ) : (
          <div className="flex items-center gap-2">
            <button
              onClick={() => setIsInspectOnly(!isInspectOnly)}
              className={`flex items-center gap-1.5 px-3.5 py-2.5 rounded-xl border text-xs font-semibold transition-all cursor-pointer ${
                isInspectOnly
                  ? 'bg-amber-500/20 text-amber-300 border-amber-500/50 shadow-md shadow-amber-500/10'
                  : 'bg-surface-900/80 text-slate-400 border-white/10 hover:text-slate-200 hover:bg-surface-800'
              }`}
              title="When enabled, scans and previews file structure in-memory without creating files on disk"
            >
              <Eye className="w-3.5 h-3.5" />
              {isInspectOnly ? 'Inspect Only (Dry Run)' : 'Direct Write'}
            </button>
            <button
              onClick={handleStartScan}
              className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-cyber-600/30 transition-all cursor-pointer"
            >
              <Play className="w-4 h-4 fill-white" />
              {isInspectOnly ? 'Start Inspection Scan' : 'Start Carving Scan'}
            </button>
          </div>
        )}
      </div>

      {/* Forensic Recovery vs Permanent Shredding Guidance Banner */}
      <div className="p-4 rounded-xl border border-cyber-500/20 bg-cyber-950/40 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-cyber-500/20 text-cyber-400 shrink-0 mt-0.5">
            <Info className="w-4 h-4" />
          </div>
          <div>
            <h2 className="text-xs font-bold text-slate-200">
              Forensic Data Recovery vs. Unrecoverable Shredding
            </h2>
            <p className="text-[11px] text-slate-400 mt-0.5 leading-relaxed max-w-2xl">
              <strong>Recovering Deleted Files:</strong> Standard deletion only clears filesystem pointers while raw data clusters stay intact in unallocated sectors — this engine carves them back.
              <br />
              <strong>Need to destroy a folder permanently?</strong> To erase sensitive folders/files so NO forensic carver can EVER recover them, use the <span className="text-rose-400 font-semibold">File & Folder Shredder</span> (DoD 7-pass overwrite + inode renaming).
            </p>
          </div>
        </div>
        {onNavigate && (
          <button
            onClick={() => onNavigate('shred')}
            className="shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 border border-rose-500/30 text-xs font-semibold transition-all cursor-pointer"
          >
            <Trash2 className="w-3.5 h-3.5 text-rose-400" />
            Go to File Shredder &rarr;
          </button>
        )}
      </div>

      {/* Live Scan Progress */}
      {isScanning && (
        <div className="glass-panel p-6 rounded-2xl border border-cyber-500/30 glow-border space-y-3 animate-pulse-slow">
          <div className="flex items-center justify-between text-xs font-mono">
            <div className="flex items-center gap-2 text-cyber-400 font-bold">
              <Sparkles className="w-4 h-4 animate-spin" />
              Scanning Sector Clusters on <span className="text-white font-mono">{sourcePath}</span>...
            </div>
            <span className="text-white font-bold">{progress?.percent.toFixed(1) || 0}%</span>
          </div>

          <div className="w-full h-2.5 bg-surface-950 rounded-full overflow-hidden border border-white/10">
            <div
              className="h-full bg-gradient-to-r from-cyber-500 to-emerald-400 transition-all duration-300"
              style={{ width: `${progress?.percent || 0}%` }}
            />
          </div>

          <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
            <span>Files Identified: <strong className="text-emerald-400">{progress?.files_found || 0}</strong></span>
            <span>Speed: <strong className="text-cyber-400">{(progress?.speed_mbps || 0).toFixed(1)} MB/s</strong></span>
          </div>
        </div>
      )}

      {/* Step 1: Storage Device or Evidence File Selection */}
      {!isScanning && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="w-5 h-5 rounded-full bg-cyber-500/20 text-cyber-400 flex items-center justify-center text-xs font-bold font-mono">1</span>
              <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
                Select Recovery Target (Disk or Image)
              </h2>
            </div>

            {/* Source Mode Switcher */}
            <div className="flex items-center gap-1 p-1 bg-surface-950 rounded-lg border border-white/10 text-xs font-medium">
              <button
                onClick={() => setSourceMode('drive')}
                className={`flex items-center gap-1.5 px-3 py-1 rounded-md transition-all cursor-pointer ${
                  sourceMode === 'drive'
                    ? 'bg-cyber-500/20 text-cyber-400 border border-cyber-500/40 font-bold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                <HardDrive className="w-3.5 h-3.5" />
                Physical Drive ({drives.length})
              </button>
              <button
                onClick={() => {
                  setSourceMode('file');
                  if (!sourcePath || sourcePath.startsWith('/dev/')) {
                    setSourcePath('tests/fixtures/mixed_with_jpeg.dd');
                  }
                }}
                className={`flex items-center gap-1.5 px-3 py-1 rounded-md transition-all cursor-pointer ${
                  sourceMode === 'file'
                    ? 'bg-cyber-500/20 text-cyber-400 border border-cyber-500/40 font-bold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                <FileCode className="w-3.5 h-3.5" />
                Forensic Image / File
              </button>
            </div>
          </div>

          {/* Mode 1: Physical Block Devices */}
          {sourceMode === 'drive' && (
            <div className="space-y-3">
              <div className="flex items-center justify-between text-xs text-slate-400">
                <span>Select a connected block device to carve unallocated clusters:</span>
                <button
                  onClick={refreshDrives}
                  disabled={drivesLoading}
                  className="flex items-center gap-1 text-cyber-400 hover:text-cyber-300 font-mono transition-colors cursor-pointer"
                >
                  <RefreshCw className={`w-3.5 h-3.5 ${drivesLoading ? 'animate-spin' : ''}`} />
                  Refresh Devices
                </button>
              </div>

              {drives.length === 0 ? (
                <div className="glass-panel p-8 rounded-xl border border-white/10 text-center space-y-2">
                  <HardDrive className="w-8 h-8 text-slate-500 mx-auto" />
                  <p className="text-xs font-semibold text-slate-300">No physical storage devices detected</p>
                  <p className="text-[11px] text-slate-500 max-w-sm mx-auto">
                    Plug in an external USB drive, SD card, or switch to "Forensic Image / File" mode to load an image dump.
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5">
                  {drives.map((drive) => (
                    <DriveCard
                      key={drive.path}
                      drive={drive}
                      selected={sourcePath === drive.path}
                      onClick={() => handleSelectDrive(drive)}
                    />
                  ))}
                </div>
              )}

              {/* Safety notice for selected drive */}
              {selectedDrive?.is_system_drive && (
                <div className="p-3.5 rounded-xl bg-amber-950/40 border border-amber-800/50 flex items-start gap-2.5 text-xs text-amber-300">
                  <ShieldAlert className="w-4 h-4 shrink-0 text-amber-400 mt-0.5" />
                  <div>
                    <span className="font-bold">System Boot Disk Selected ({selectedDrive.path})</span>
                    <p className="text-[11px] text-amber-400/80 mt-0.5">
                      RAZE-CARVE forensic scanner opens devices in strictly read-only mode (O_RDONLY). No host OS data or partition tables will be modified during scan.
                    </p>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Mode 2: Forensic Disk Image / DD Dump */}
          {sourceMode === 'file' && (
            <div className="glass-panel p-5 rounded-2xl border border-white/10 space-y-3">
              <label className="block text-xs font-semibold text-slate-300">
                Evidence Image File Path (Raw .dd, .raw, .img, or partition path)
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={sourcePath}
                  onChange={(e) => setSourcePath(e.target.value)}
                  placeholder="/home/user/evidence.dd or /dev/sdb1"
                  className="flex-1 bg-surface-950 border border-white/10 rounded-xl px-3.5 py-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyber-500"
                />
                <button
                  onClick={() => setSourcePath('tests/fixtures/mixed_with_jpeg.dd')}
                  className="px-3.5 py-2 rounded-xl bg-surface-900 border border-white/10 text-xs font-mono text-cyber-400 hover:border-cyber-500/50 transition-all cursor-pointer shrink-0"
                >
                  Load Sample Fixture
                </button>
              </div>
              <p className="text-[11px] text-slate-500 font-mono">
                Tip: You can create a forensic image of any drive using: <span className="text-slate-400">dd if=/dev/sdb of=/path/to/evidence.dd bs=4M status=progress</span>
              </p>
            </div>
          )}
        </div>
      )}

      {/* Step 2: Multi-Strategy Recovery Suite & Parameters */}
      {!isScanning && (
        <div className="glass-panel p-5 rounded-2xl border border-white/10 space-y-5">
          <div className="flex items-center gap-2">
            <span className="w-5 h-5 rounded-full bg-cyber-500/20 text-cyber-400 flex items-center justify-center text-xs font-bold font-mono">2</span>
            <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
              Recovery Algorithms & Extraction Strategies
            </h2>
          </div>

          {/* Select Recovery Techniques */}
          <div className="space-y-2">
            <label className="block text-xs font-semibold text-slate-400">
              Select Techniques (Multiple Allowed for Hybrid Ensemble):
            </label>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
              {recoveryStrategies.map((strat) => {
                const isSelected = selectedModes.includes('hybrid')
                  ? strat.id === 'hybrid'
                  : selectedModes.includes(strat.id);
                return (
                  <div
                    key={strat.id}
                    onClick={() => handleToggleMode(strat.id)}
                    className={`p-3 rounded-xl border transition-all cursor-pointer ${
                      isSelected
                        ? 'bg-cyber-950/80 border-cyber-500/60 ring-1 ring-cyber-500/30 text-white'
                        : 'bg-surface-950 border-white/5 text-slate-400 hover:border-white/15'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs font-bold flex items-center gap-2">
                        <CheckCircle2 className={`w-3.5 h-3.5 ${isSelected ? 'text-cyber-400' : 'text-slate-600'}`} />
                        {strat.name}
                      </span>
                      <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-surface-900 border border-white/5 text-slate-400">
                        {strat.badge}
                      </span>
                    </div>
                    <p className="text-[11px] text-slate-400 pl-5 leading-relaxed">
                      {strat.desc}
                    </p>
                  </div>
                );
              })}
            </div>
          </div>

          <div className="space-y-4 pt-2 border-t border-white/5">
            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1.5 font-mono">
                Active Evidence Source Path
              </label>
              <input
                type="text"
                value={sourcePath}
                onChange={(e) => setSourcePath(e.target.value)}
                placeholder="/dev/sdb or /path/to/evidence.dd"
                className="w-full bg-surface-950 border border-white/10 rounded-xl px-3.5 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyber-500"
              />
            </div>
            <FolderSelector
              value={outputDir}
              onChange={setOutputDir}
              disabled={isScanning || isExtracting}
            />
          </div>

          {/* Signature Type Filters & Confidence Slider */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-white/5">
            <div>
              <label className="block text-xs font-semibold text-slate-400 mb-2">Target File Signatures</label>
              <div className="flex flex-wrap gap-2">
                {[
                  { id: 'jpg', label: 'JPEG (.jpg)' },
                  { id: 'png', label: 'PNG (.png)' },
                  { id: 'pdf', label: 'PDF Docs (.pdf)' },
                  { id: 'sqlite', label: 'SQLite DB (.sqlite)' },
                  { id: 'zip', label: 'ZIP / Office (.zip)' },
                ].map((s) => (
                  <button
                    key={s.id}
                    onClick={() => handleToggleType(s.id)}
                    className={`px-3 py-1 rounded-lg text-xs font-mono transition-all cursor-pointer ${
                      selectedTypes.includes(s.id)
                        ? 'bg-cyber-500/20 text-cyber-400 border border-cyber-500/50 font-bold'
                        : 'bg-surface-950 text-slate-500 border border-white/5 hover:text-slate-300'
                    }`}
                  >
                    {s.label}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="text-xs font-semibold text-slate-400">Confidence Threshold</label>
                <span className="text-xs font-mono font-bold text-cyber-400">{minConfidence}%</span>
              </div>
              <input
                type="range"
                min={10}
                max={90}
                step={5}
                value={minConfidence}
                onChange={(e) => setMinConfidence(Number(e.target.value))}
                className="w-full accent-cyber-500 cursor-pointer"
              />
              <div className="flex justify-between text-[10px] text-slate-500 font-mono mt-1">
                <span>10% (Permissive)</span>
                <span>50% (Standard)</span>
                <span>90% (Strict Header/Footer)</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Scan Error Banner */}
      {errorMessage && !isScanning && (
        <div className="p-4 rounded-xl bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs font-mono">
          Scan failed: {errorMessage}
        </div>
      )}

      {/* Scan Results & Hex Inspector Grid */}
      {result && (
        <div className="space-y-6">
          {/* Entropy Heatmap of the Source */}
          {result.entropy_heatmap && result.entropy_heatmap.length > 0 && (
            <EntropyHeatmap data={result.entropy_heatmap} height={40} title="Carved Image Entropy Distribution" />
          )}

          {/* Results Summary Banner & Selective Extraction Actions */}
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 p-4 glass-panel rounded-xl border border-white/10">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-lg bg-emerald-950 text-emerald-400 border border-emerald-800">
                <FileCheck className="w-5 h-5" />
              </div>
              <div>
                <span className="text-sm font-bold text-white block">
                  {isInspectOnly ? 'Discovered' : 'Successfully Extracted'} {result.files.length} Artifacts ({((result.total_size_bytes || 0) / 1024 / 1024).toFixed(2)} MB)
                </span>
                <span className="text-xs text-slate-400 font-mono">
                  {isInspectOnly ? 'In-Memory Inspection Mode (0 files written)' : 'Direct Recovery Complete'} • {result.duration_secs.toFixed(2)}s • {selectedFileIds.length} file(s) selected
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2 flex-wrap">
              <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-950/80 border border-white/10 text-xs font-mono text-slate-300">
                <span className="text-slate-500">Dest:</span>
                <span className="text-cyber-400 font-bold truncate max-w-[160px]" title={outputDir}>{outputDir}</span>
              </div>
              <button
                disabled={isExtracting || selectedFileIds.length === 0}
                onClick={() => {
                  const toExtract = result.files.filter((f) => selectedFileIds.includes(f.id));
                  handleExtractFiles(toExtract);
                }}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-cyber-500 hover:bg-cyber-400 disabled:opacity-40 text-white text-xs font-bold transition-all cursor-pointer shadow-md shadow-cyber-500/20"
              >
                <Download className="w-3.5 h-3.5" />
                {isExtracting ? 'Extracting…' : `Extract Selected (${selectedFileIds.length})`}
              </button>
              <button
                disabled={isExtracting || result.files.length === 0}
                onClick={() => handleExtractFiles(result.files)}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 disabled:opacity-40 text-slate-200 border border-white/10 text-xs font-semibold transition-all cursor-pointer"
              >
                <FolderCheck className="w-3.5 h-3.5 text-emerald-400" />
                Extract All ({result.files.length})
              </button>
            </div>
          </div>

          {/* Extraction Toast Notification */}
          {extractionMessage && (
            <div className="p-3.5 rounded-xl bg-emerald-950/80 border border-emerald-800/60 text-emerald-300 text-xs font-mono flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>{extractionMessage}</span>
            </div>
          )}

          {/* Split View: Table on Left / Hex Viewer on Right */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div className="lg:col-span-7">
              <FileTable
                files={result.files}
                selectedFile={selectedFile}
                onSelectFile={handleInspectFile}
                onPreviewFile={(file) => setPreviewModalFile(file)}
                selectedFileIds={selectedFileIds}
                onToggleSelectFile={handleToggleSelectFile}
                onToggleSelectAll={handleToggleSelectAll}
                onExtractSingle={(file) => handleExtractFiles([file])}
              />
            </div>

            <div className="lg:col-span-5 space-y-4">
              {selectedFile ? (
                <div className="space-y-3">
                  {/* Quick Action: Open Full Media Preview */}
                  <button
                    onClick={() => setPreviewModalFile(selectedFile)}
                    className="w-full flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs shadow-lg shadow-cyber-500/20 transition-all cursor-pointer"
                  >
                    <Eye className="w-4 h-4" /> Open Full Media Preview (Image / Video / Text)
                  </button>

                  <div className="p-3.5 glass-panel rounded-xl border border-white/10 space-y-2 text-xs font-mono">
                    <div className="flex justify-between text-slate-400">
                      <span>Saved File:</span>
                      <span className="text-emerald-400 font-bold">{outputDir}/{selectedFile.filename}</span>
                    </div>
                    <div className="flex justify-between text-slate-400">
                      <span>Sector Offset:</span>
                      <span className="text-cyber-400">0x{selectedFile.offset_bytes.toString(16).toUpperCase()} ({selectedFile.offset_bytes} bytes)</span>
                    </div>
                    <div className="flex justify-between text-slate-400">
                      <span>Extracted Size:</span>
                      <span className="text-white">{(selectedFile.size_bytes / 1024).toFixed(1)} KB ({selectedFile.size_bytes} B)</span>
                    </div>
                    {selectedFile.recovery_method && (
                      <div className="flex justify-between text-slate-400 border-t border-white/5 pt-1.5">
                        <span>Technique:</span>
                        <span className="text-purple-400 font-bold">{selectedFile.recovery_method}</span>
                      </div>
                    )}
                    {selectedFile.original_metadata && (
                      <div className="p-2.5 rounded-lg bg-surface-950 border border-white/5 space-y-1 text-[11px] text-slate-300">
                        <span className="text-slate-400 font-semibold uppercase tracking-wider block text-[10px]">
                          Recovered Forensic Metadata:
                        </span>
                        <p className="text-cyber-300 leading-relaxed break-words">
                          {selectedFile.original_metadata}
                        </p>
                      </div>
                    )}
                  </div>
                  {hexError ? (
                    <div className="rounded-xl border border-rose-500/40 bg-rose-950/30 p-4 text-xs text-rose-200">
                      Unable to display source bytes: {hexError}
                    </div>
                  ) : (
                    <HexViewer
                      hexData={hexDump || undefined}
                      startOffset={selectedFile.offset_bytes}
                      title={`Hex Inspector — ${selectedFile.filename}`}
                    />
                  )}
                </div>
              ) : (
                <div className="glass-panel p-12 rounded-xl text-center text-xs text-slate-500">
                  Select a carved file from the table to inspect its raw binary bytes and structure.
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Media Preview Modal (Images, Videos, Audio, Documents, Text) */}
      <MediaPreviewModal
        file={previewModalFile}
        sourcePath={sourcePath}
        outputDir={outputDir}
        onClose={() => setPreviewModalFile(null)}
        onExtractSingle={(file) => handleExtractFiles([file])}
      />
    </div>
  );
}
