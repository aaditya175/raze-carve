import { useState } from 'react';
import {
  Folder,
  FolderOpen,
  FolderPlus,
  Sparkles,
  CheckCircle2,
  HardDrive,
  RotateCcw,
  ChevronRight,
  ShieldCheck,
  Zap,
} from 'lucide-react';

export interface FolderPreset {
  id: string;
  name: string;
  path: string;
  description: string;
  badge: string;
  icon: 'default' | 'ram' | 'downloads' | 'desktop';
}

export const FORENSIC_FOLDER_PRESETS: FolderPreset[] = [
  {
    id: 'default',
    name: 'Workspace Evidence',
    path: './recovered_evidence',
    description: 'Local workspace quarantine folder with project isolation',
    badge: 'DEFAULT',
    icon: 'default',
  },
  {
    id: 'ram',
    name: 'RAM /tmp Vault',
    path: '/tmp/razecarve_cases',
    description: 'Volatile memory storage for fast, zero-wear examination',
    badge: 'HIGH SPEED',
    icon: 'ram',
  },
  {
    id: 'downloads',
    name: 'Investigator Downloads',
    path: '~/Downloads/RazeCarve_Recovery',
    description: 'Direct export to current user workstation downloads',
    badge: 'CONVENIENT',
    icon: 'downloads',
  },
  {
    id: 'desktop',
    name: 'Desktop Case Vault',
    path: '~/Desktop/Evidence_Vault',
    description: 'Forensic staging repository pinned to operator desktop',
    badge: 'DEDICATED',
    icon: 'desktop',
  },
];

interface FolderSelectorProps {
  value: string;
  onChange: (path: string) => void;
  label?: string;
  disabled?: boolean;
}

export function FolderSelector({
  value,
  onChange,
  label = 'Evidence Extraction Directory',
  disabled = false,
}: FolderSelectorProps) {
  const [isCustomEditing, setIsCustomEditing] = useState(false);
  const [customPath, setCustomPath] = useState(value);
  const [pickerStatus, setPickerStatus] = useState<string | null>(null);

  const activePreset = FORENSIC_FOLDER_PRESETS.find((p) => p.path === value);

  const handleSelectPreset = (preset: FolderPreset) => {
    if (disabled) return;
    onChange(preset.path);
    setCustomPath(preset.path);
    setIsCustomEditing(false);
  };

  const handleCustomSubmit = () => {
    const trimmed = customPath.trim() || './recovered_evidence';
    onChange(trimmed);
    setIsCustomEditing(false);
  };

  const handleNativeBrowse = async () => {
    if (disabled) return;
    try {
      if (typeof window !== 'undefined' && 'showDirectoryPicker' in window) {
        // Modern File System Access API
        const dirHandle = await (window as unknown as { showDirectoryPicker: () => Promise<{ name: string }> }).showDirectoryPicker();
        if (dirHandle?.name) {
          const chosen = `./${dirHandle.name}`;
          onChange(chosen);
          setCustomPath(chosen);
          setPickerStatus(`Selected directory: ${dirHandle.name}`);
          setTimeout(() => setPickerStatus(null), 4000);
          return;
        }
      }
    } catch (err: unknown) {
      // User cancelled picker or permission rejected
      if (err instanceof Error && err.name !== 'AbortError') {
        console.warn('Native folder picker fallback:', err);
      }
    }
    // Fallback toggle custom path editing if native picker unavailable or cancelled
    setIsCustomEditing(true);
  };

  const handleAppendTimestampCase = () => {
    if (disabled) return;
    const now = new Date();
    const pad = (n: number) => String(n).padStart(2, '0');
    const stamp = `case_${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}`;
    const base = (value.endsWith('/') ? value.slice(0, -1) : value) || './recovered_evidence';
    const nextPath = `${base}/${stamp}`;
    onChange(nextPath);
    setCustomPath(nextPath);
  };

  // Split path for cyber breadcrumbs display
  const pathParts = value
    .split(/[/|\\]/)
    .filter(Boolean);

  return (
    <div className="space-y-3">
      {/* Header with Title and Quick Status */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <label className="text-xs font-semibold text-slate-300 flex items-center gap-2 font-mono">
          <Folder className="w-4 h-4 text-cyber-400" />
          <span>{label}</span>
        </label>
        <div className="flex items-center gap-2">
          {pickerStatus && (
            <span className="text-[11px] font-mono text-emerald-400 flex items-center gap-1 animate-fadeIn">
              <CheckCircle2 className="w-3 h-3" /> {pickerStatus}
            </span>
          )}
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-mono uppercase tracking-wider bg-cyber-500/10 text-cyber-400 border border-cyber-500/30">
            <ShieldCheck className="w-3 h-3 text-emerald-400" />
            {activePreset ? activePreset.badge : 'CUSTOM DESTINATION'}
          </span>
        </div>
      </div>

      {/* Main Folder Card */}
      <div className="glass-panel p-4 rounded-2xl border border-cyber-500/30 glow-border bg-gradient-to-br from-surface-900/90 via-surface-950/90 to-surface-900/80 space-y-3">
        {/* Active Path Display & Native Action Bar */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3 rounded-xl bg-surface-950/80 border border-white/5">
          <div className="flex items-center gap-3 overflow-hidden">
            <div className="p-2.5 rounded-xl bg-cyber-500/10 border border-cyber-500/30 text-cyber-400 shrink-0">
              <FolderOpen className="w-5 h-5 animate-pulse" />
            </div>
            <div className="min-w-0">
              <div className="text-[11px] font-mono text-slate-400 flex items-center gap-1">
                <span>Active Target Path</span>
                <ChevronRight className="w-3 h-3 text-slate-600" />
              </div>
              <div className="font-mono text-xs font-bold text-white tracking-wide truncate select-all" title={value}>
                {value}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
            <button
              type="button"
              disabled={disabled}
              onClick={handleNativeBrowse}
              className="px-3 py-1.5 rounded-xl text-xs font-mono font-bold bg-cyber-600/30 hover:bg-cyber-500/40 text-cyber-300 hover:text-white border border-cyber-500/40 transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shadow-sm"
              title="Browse native folder on workstation"
            >
              <FolderOpen className="w-3.5 h-3.5" />
              <span>Browse...</span>
            </button>
            <button
              type="button"
              disabled={disabled}
              onClick={() => setIsCustomEditing(!isCustomEditing)}
              className={`px-3 py-1.5 rounded-xl text-xs font-mono transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-40 ${
                isCustomEditing
                  ? 'bg-cyber-500 text-white font-bold'
                  : 'bg-surface-800 text-slate-400 hover:text-white border border-white/5'
              }`}
            >
              <span>{isCustomEditing ? 'Done' : 'Edit Path'}</span>
            </button>
          </div>
        </div>

        {/* Interactive Breadcrumb Visualizer */}
        {pathParts.length > 0 && (
          <div className="flex flex-wrap items-center gap-1.5 text-[11px] font-mono px-1 py-0.5">
            <span className="text-slate-500">Root:</span>
            {pathParts.map((part, index) => (
              <span key={index} className="inline-flex items-center gap-1.5">
                <span className="px-2 py-0.5 rounded-md bg-surface-900 border border-white/5 text-slate-300 font-mono text-[11px]">
                  {part}
                </span>
                {index < pathParts.length - 1 && <span className="text-slate-600">/</span>}
              </span>
            ))}
          </div>
        )}

        {/* Custom Path Editing Drawer */}
        {isCustomEditing && (
          <div className="p-3 rounded-xl bg-surface-950/90 border border-cyber-500/40 space-y-2.5 animate-fadeIn">
            <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
              <span>Enter Custom Destination Path</span>
              <button
                type="button"
                onClick={handleAppendTimestampCase}
                className="text-[11px] text-cyber-400 hover:text-cyber-300 flex items-center gap-1 cursor-pointer"
              >
                <FolderPlus className="w-3 h-3" /> + Timestamp Case Subfolder
              </button>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleCustomSubmit()}
                placeholder="./recovered_evidence or /path/to/vault"
                className="w-full bg-surface-900 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white placeholder-slate-600 focus:outline-none focus:border-cyber-400"
                autoFocus
              />
              <button
                type="button"
                onClick={handleCustomSubmit}
                className="px-3.5 py-2 rounded-xl text-xs font-mono font-bold bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white cursor-pointer shrink-0 transition-all shadow-md shadow-cyber-600/30"
              >
                Apply
              </button>
              <button
                type="button"
                onClick={() => {
                  setCustomPath('./recovered_evidence');
                  onChange('./recovered_evidence');
                  setIsCustomEditing(false);
                }}
                className="p-2 rounded-xl bg-surface-800 text-slate-400 hover:text-white border border-white/5 cursor-pointer shrink-0"
                title="Reset to default ./recovered_evidence"
              >
                <RotateCcw className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Quick Preset Selector Cards */}
        <div>
          <div className="text-[11px] font-mono text-slate-400 mb-2 flex items-center gap-1.5">
            <Sparkles className="w-3 h-3 text-cyber-400" />
            <span>Forensic Destination Presets</span>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2">
            {FORENSIC_FOLDER_PRESETS.map((p) => {
              const isSelected = value === p.path;
              return (
                <button
                  key={p.id}
                  type="button"
                  disabled={disabled}
                  onClick={() => handleSelectPreset(p)}
                  className={`p-2.5 rounded-xl text-left border transition-all cursor-pointer flex flex-col justify-between space-y-1.5 disabled:opacity-40 disabled:cursor-not-allowed ${
                    isSelected
                      ? 'bg-cyber-950/80 border-cyber-500 shadow-md shadow-cyber-500/20 glow-border'
                      : 'bg-surface-950/60 border-white/5 hover:border-white/20 hover:bg-surface-900/60'
                  }`}
                >
                  <div className="flex items-center justify-between gap-1">
                    <span className="text-xs font-bold text-white font-mono flex items-center gap-1.5 truncate">
                      {p.icon === 'ram' && <Zap className="w-3.5 h-3.5 text-amber-400 shrink-0" />}
                      {p.icon === 'downloads' && <HardDrive className="w-3.5 h-3.5 text-emerald-400 shrink-0" />}
                      {p.icon === 'desktop' && <Folder className="w-3.5 h-3.5 text-cyber-400 shrink-0" />}
                      {p.icon === 'default' && <FolderOpen className="w-3.5 h-3.5 text-cyber-400 shrink-0" />}
                      <span className="truncate">{p.name}</span>
                    </span>
                    {isSelected && <CheckCircle2 className="w-3.5 h-3.5 text-cyber-400 shrink-0" />}
                  </div>
                  <div className="text-[10px] font-mono text-slate-400 truncate" title={p.path}>
                    {p.path}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

export default FolderSelector;
