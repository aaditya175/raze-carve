import { useState, useEffect } from 'react';
import { CarvedFile } from '../lib/types';
import { getSavedToken } from '../lib/api';
import {
  X,
  Download,
  FileText,
  Image as ImageIcon,
  Film,
  Music,
  ZoomIn,
  ZoomOut,
  Maximize2,
  FileCode,
  ShieldCheck,
  CheckCircle2,
  Loader2,
} from 'lucide-react';

interface Props {
  file: CarvedFile | null;
  sourcePath: string;
  outputDir: string;
  onClose: () => void;
  onExtractSingle?: (file: CarvedFile) => void;
}

export function MediaPreviewModal({
  file,
  sourcePath,
  outputDir,
  onClose,
  onExtractSingle,
}: Props) {
  const [zoom, setZoom] = useState(1);
  const [textContent, setTextContent] = useState<string | null>(null);
  const [textLoading, setTextLoading] = useState(false);
  const [textError, setTextError] = useState<string | null>(null);

  useEffect(() => {
    setZoom(1);
    setTextContent(null);
    setTextError(null);

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [file, onClose]);

  const baseUrl =
    typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:7878';
  const token = getSavedToken();

  const diskPath = file ? `${outputDir.replace(/\/$/, '')}/${file.filename}` : '';
  const params = new URLSearchParams({
    source_path: sourcePath,
    offset: file?.offset_bytes.toString() || '0',
    size: file?.size_bytes.toString() || '0',
    file_path: diskPath,
    mime: file?.file_type.toLowerCase() || '',
  });
  if (token) {
    params.set('token', token);
  }

  const previewUrl = `${baseUrl}/api/preview?${params.toString()}`;

  const ext = file
    ? file.filename.split('.').pop()?.toLowerCase() || file.file_type.toLowerCase()
    : '';
  const isImage = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg'].includes(ext);
  const isVideo = ['mp4', 'webm', 'mov', 'm4v', 'mkv'].includes(ext);
  const isAudio = ['mp3', 'wav', 'ogg', 'flac'].includes(ext);
  const isText = ['txt', 'log', 'csv', 'json', 'xml', 'html', 'htm', 'md', 'ini', 'cfg'].includes(ext);
  const isPdf = ext === 'pdf';

  // Fetch text contents if text file - MUST run before any early return to comply with Rules of Hooks
  useEffect(() => {
    if (!file || !isText) return;
    setTextLoading(true);
    setTextError(null);
    fetch(previewUrl)
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP error ${res.status}`);
        return res.text();
      })
      .then((text) => setTextContent(text.slice(0, 100000))) // Cap at 100KB preview
      .catch((err) => setTextError(err.message))
      .finally(() => setTextLoading(false));
  }, [isText, previewUrl, file]);

  if (!file) return null;

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fade-in">
      <div
        className="glass-panel w-full max-w-4xl max-h-[90vh] rounded-2xl border border-white/20 shadow-2xl flex flex-col overflow-hidden bg-surface-950/95"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Modal Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-white/10 bg-surface-900/90">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-surface-800 border border-white/5 text-cyber-400">
              {isImage ? (
                <ImageIcon className="w-5 h-5" />
              ) : isVideo ? (
                <Film className="w-5 h-5 text-purple-400" />
              ) : isAudio ? (
                <Music className="w-5 h-5 text-amber-400" />
              ) : isText ? (
                <FileText className="w-5 h-5 text-emerald-400" />
              ) : (
                <FileCode className="w-5 h-5 text-slate-400" />
              )}
            </div>
            <div>
              <div className="flex items-center gap-2 flex-wrap">
                <span className="font-mono text-sm font-bold text-white truncate max-w-md">
                  {file.filename}
                </span>
                <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-cyber-950 text-cyber-400 border border-cyber-800/50">
                  {file.file_type}
                </span>
                {file.recovery_method && (
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-purple-950/80 text-purple-400 border border-purple-800/50">
                    {file.recovery_method}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-3 text-xs text-slate-400 font-mono mt-0.5">
                <span>{formatBytes(file.size_bytes)}</span>
                <span>•</span>
                <span className="text-cyber-400">
                  LBA: 0x{file.offset_bytes.toString(16).toUpperCase()}
                </span>
                <span>•</span>
                <span className="text-emerald-400 flex items-center gap-1">
                  <ShieldCheck className="w-3 h-3" />
                  {file.confidence}% Confidence
                </span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {isImage && (
              <div className="flex items-center gap-1 bg-surface-800 border border-white/5 rounded-lg p-1 mr-2">
                <button
                  onClick={() => setZoom((z) => Math.max(0.5, z - 0.25))}
                  className="p-1 text-slate-400 hover:text-white rounded hover:bg-surface-700 transition-colors"
                  title="Zoom Out"
                >
                  <ZoomOut className="w-4 h-4" />
                </button>
                <span className="text-[10px] font-mono px-1.5 text-slate-300">
                  {Math.round(zoom * 100)}%
                </span>
                <button
                  onClick={() => setZoom((z) => Math.min(3, z + 0.25))}
                  className="p-1 text-slate-400 hover:text-white rounded hover:bg-surface-700 transition-colors"
                  title="Zoom In"
                >
                  <ZoomIn className="w-4 h-4" />
                </button>
                <button
                  onClick={() => setZoom(1)}
                  className="p-1 text-slate-400 hover:text-white rounded hover:bg-surface-700 transition-colors"
                  title="Reset Zoom"
                >
                  <Maximize2 className="w-3.5 h-3.5" />
                </button>
              </div>
            )}
            <button
              onClick={onClose}
              className="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-surface-800 transition-colors cursor-pointer"
              title="Close (Esc)"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Modal Media Body */}
        <div className="flex-1 overflow-auto p-6 flex items-center justify-center min-h-[360px] max-h-[65vh] bg-surface-950/80">
          {isImage ? (
            <div className="relative overflow-auto max-w-full max-h-full flex items-center justify-center p-2 checkerboard-bg rounded-xl border border-white/5">
              <img
                src={previewUrl}
                alt={file.filename}
                style={{ transform: `scale(${zoom})`, transformOrigin: 'center center' }}
                className="max-h-[55vh] max-w-full object-contain rounded shadow-2xl transition-transform duration-150"
              />
            </div>
          ) : isVideo ? (
            <div className="w-full flex flex-col items-center">
              <video
                controls
                autoPlay
                src={previewUrl}
                className="max-h-[55vh] w-full max-w-2xl rounded-xl border border-white/10 bg-black shadow-2xl"
              >
                Your browser does not support the video tag.
              </video>
            </div>
          ) : isAudio ? (
            <div className="w-full max-w-md p-6 glass-panel rounded-2xl border border-white/10 space-y-4 text-center">
              <div className="w-16 h-16 rounded-full bg-amber-500/20 text-amber-400 flex items-center justify-center mx-auto border border-amber-500/30">
                <Music className="w-8 h-8" />
              </div>
              <div>
                <span className="font-mono text-sm font-bold text-white block">
                  {file.filename}
                </span>
                <span className="text-xs text-slate-400 font-mono">
                  Audio Stream • {formatBytes(file.size_bytes)}
                </span>
              </div>
              <audio controls autoPlay src={previewUrl} className="w-full mt-2">
                Your browser does not support the audio tag.
              </audio>
            </div>
          ) : isText ? (
            <div className="w-full h-full min-h-[300px] flex flex-col">
              {textLoading ? (
                <div className="flex items-center justify-center py-20 gap-2 text-cyber-400 text-xs font-mono">
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Loading text content...
                </div>
              ) : textError ? (
                <div className="p-4 rounded-xl bg-rose-950/80 border border-rose-800 text-rose-300 text-xs font-mono">
                  Error loading text preview: {textError}
                </div>
              ) : (
                <pre className="w-full max-h-[55vh] overflow-auto p-4 rounded-xl bg-surface-900 border border-white/10 text-xs font-mono text-slate-200 leading-relaxed whitespace-pre-wrap select-text">
                  {textContent || 'Empty file content'}
                </pre>
              )}
            </div>
          ) : isPdf ? (
            <iframe
              src={previewUrl}
              title={file.filename}
              className="w-full h-[55vh] rounded-xl border border-white/10 bg-white"
            />
          ) : (
            <div className="text-center p-8 glass-panel rounded-xl border border-white/10 space-y-3 max-w-md">
              <FileCode className="w-12 h-12 text-slate-500 mx-auto" />
              <div>
                <span className="font-mono text-sm font-bold text-white block">
                  {file.filename}
                </span>
                <p className="text-xs text-slate-400 mt-1">
                  Binary stream format ({file.file_type.toUpperCase()}). Use the raw Hex Inspector on the main screen to analyze sector bytes.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Metadata & Footer Controls */}
        <div className="px-6 py-4 border-t border-white/10 bg-surface-900/90 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
          <div className="text-xs text-slate-400 font-mono space-y-0.5 truncate max-w-lg">
            {file.original_metadata && (
              <div className="truncate text-slate-300">
                <span className="text-slate-500">Metadata:</span> {file.original_metadata}
              </div>
            )}
            <div className="truncate text-[11px] text-slate-500">
              Target: {sourcePath}
            </div>
          </div>

          <div className="flex items-center gap-2.5 shrink-0">
            {onExtractSingle && (
              <button
                onClick={() => {
                  onExtractSingle(file);
                }}
                className="flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-surface-800 hover:bg-surface-700 text-slate-200 border border-white/10 text-xs font-semibold transition-all cursor-pointer"
              >
                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                Extract to Disk
              </button>
            )}
            <a
              href={previewUrl}
              download={file.filename}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs shadow-lg shadow-cyber-500/20 transition-all cursor-pointer"
            >
              <Download className="w-3.5 h-3.5" />
              Download / Save
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
