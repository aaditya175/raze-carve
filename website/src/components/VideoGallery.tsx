import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import {
  Film,
  Sparkles,
  Layers,
  CheckCircle2,
  HardDrive,
  ShieldCheck,
  Zap
} from 'lucide-react';

export interface VideoItem {
  id: string;
  badge: string;
  title: string;
  description: string;
  src: string;
  category: 'all' | 'hardware' | 'sanitization' | 'forensics' | 'system';
  categoryLabel: string;
  duration?: string;
  tags: string[];
}

export const PS149_VIDEOS: VideoItem[] = [
  {
    id: 'vid-1',
    badge: 'HARDWARE INTEGRATION TEST',
    title: 'Storage Controller Enumeration & O_DIRECT Probe',
    description: 'Direct verification of electrical & kernel IOCTL connections between NVMe, SATA, SAS, and USB endpoints with direct POSIX O_DIRECT memory bypass.',
    src: '/demo_videos/recording_2026-09-29_23.39.06.mp4',
    category: 'hardware',
    categoryLabel: 'Hardware & I/O',
    tags: ['POSIX O_DIRECT', 'IOCTL Probe', 'sih149 info'],
  },
  {
    id: 'vid-2',
    badge: 'NIST SP 800-88 PURGE',
    title: 'Full-Speed Multi-Pass Sanitization & Eradication Test',
    description: 'High-throughput block overwriting demonstrating deterministic NIST SP 800-88 Rev 1 media clear & DoD 5220.22-M 3-pass cryptographic wipe routines.',
    src: '/demo_videos/recording_2026-09-29_23.41.33.mp4',
    category: 'sanitization',
    categoryLabel: 'Sanitization',
    tags: ['NIST SP 800-88', 'DoD 5220.22-M', 'High Throughput'],
  },
  {
    id: 'vid-3',
    badge: 'REAL-TIME SENSOR & ENTROPY',
    title: 'Live Shannon Entropy Verification & Zero-Pattern Test',
    description: 'Mathematical spectral verification of post-wipe entropy across 12,800 sample windows confirming uniform flat-line zero state (H <= 0.001 b/B).',
    src: '/demo_videos/recording_2026-09-29_23.42.51.mp4',
    category: 'sanitization',
    categoryLabel: 'Sanitization',
    tags: ['Shannon Entropy', 'Zero Residuals', 'Spectral Analysis'],
  },
  {
    id: 'vid-4',
    badge: 'RAW SECTOR CARVING TEST',
    title: 'Signature Header & Trailer Evidence Salvage Routine',
    description: 'Streaming reconstruction of deleted JPEG, PNG, PDF, and SQLite artifacts from unpartitioned raw block devices with zero false-positive validation.',
    src: '/demo_videos/recording_2026-09-29_23.45.27.mp4',
    category: 'forensics',
    categoryLabel: 'Forensic Carving',
    tags: ['Magic Bytes', 'Zero False Positives', 'sih149 recover'],
  },
  {
    id: 'vid-5',
    badge: 'FIRMWARE CRYPTO PURGE',
    title: 'Hardware NVMe Cryptographic Key Scramble Test',
    description: 'Instantaneous media encryption key destruction triggering internal NAND firmware scramble and self-encrypting drive (SED) reset in under 2 seconds.',
    src: '/demo_videos/recording_2026-09-29_23.47.56.mp4',
    category: 'sanitization',
    categoryLabel: 'Sanitization',
    tags: ['NVMe Sanitize', 'Crypto Erase', 'Zero Wear'],
  },
  {
    id: 'vid-6',
    badge: 'DEFENSE TUI VALIDATION',
    title: 'Ratatui 60 FPS Terminal Defense UI (sforge) Walkthrough',
    description: 'Interactive keyboard-driven curses terminal console for air-gapped triage workstations, field technician laptops, and remote SSH administration.',
    src: '/demo_videos/recording_2026-09-29_23.50.44.mp4',
    category: 'system',
    categoryLabel: 'Terminal & TUI',
    tags: ['60 FPS', 'Curses/Ratatui', 'Air-Gapped Console'],
  },
  {
    id: 'vid-7',
    badge: 'CRYPTOGRAPHIC AUDIT TEST',
    title: 'Tamper-Evident BLAKE3 Audit Certificate Generation',
    description: 'Automated cryptographic signature logging and tamper-evident PDF verification certificate compilation with RFC 3161 digital timestamping.',
    src: '/demo_videos/recording_2026-09-29_23.52.39.mp4',
    category: 'forensics',
    categoryLabel: 'Forensic Carving',
    tags: ['BLAKE3 Hash Chain', 'RFC 3161', 'PDF Audit Report'],
  },
  {
    id: 'vid-8',
    badge: 'KERNEL WRITE-BLOCK PROBE',
    title: 'Kernel Write-Blocking & Air-Gapped Triage Routine',
    description: 'Read-only loopback mount initialization preventing accidental evidence alteration or file-system mount writes during initial incident response.',
    src: '/demo_videos/recording_2026-09-29_23.54.17.mp4',
    category: 'hardware',
    categoryLabel: 'Hardware & I/O',
    tags: ['Write-Blocker', 'Field Evidence', 'Integrity Lock'],
  },
  {
    id: 'vid-9',
    badge: 'DOCKER ISOLATION TEST',
    title: 'Isolated Container Hardware Pass-Through & Daemon Check',
    description: 'Deployment in an unprivileged container environment with physical device mapping via direct Linux /dev block pass-through.',
    src: '/demo_videos/recording_2026-09-29_23.55.00.mp4',
    category: 'system',
    categoryLabel: 'Terminal & TUI',
    tags: ['Docker Workstation', 'Hardware Mount', '--privileged'],
  },
  {
    id: 'vid-10',
    badge: 'FULL PIPELINE VALIDATION',
    title: 'End-to-End Operational Pipeline & Defense Verification',
    description: 'Comprehensive demonstration of the complete automated chain: discovery, sanitization, entropy audit, signature carving, and certification.',
    src: '/demo_videos/recording_2026-09-29_23.55.47.mp4',
    category: 'forensics',
    categoryLabel: 'Forensic Carving',
    tags: ['End-to-End', 'Automated QA', 'Complete Audit'],
  },
];

const CATEGORIES = [
  { id: 'all', label: 'All Modules (10)' },
  { id: 'hardware', label: 'Hardware Integration' },
  { id: 'sanitization', label: 'Sanitization & Purge' },
  { id: 'forensics', label: 'Forensic Recovery' },
  { id: 'system', label: 'TUI & Containerization' },
] as const;

export const VideoGallery: React.FC = () => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  const filteredVideos = selectedCategory === 'all'
    ? PS149_VIDEOS
    : PS149_VIDEOS.filter((v) => v.category === selectedCategory);

  return (
    <section className="py-16 sm:py-20 bg-[#F8F9FA] border-t border-b border-slate-200/80 font-sans relative overflow-hidden">
      <div className="max-w-[1480px] mx-auto px-4 sm:px-8 relative z-10 space-y-10">
        
        {/* Section Header (Light Theme) */}
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 pb-2 border-b border-slate-200/80">
          <div className="space-y-3">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono font-bold bg-red-50 text-[#E31B23] border border-red-200">
              <Film className="w-3.5 h-3.5" />
              <span>PS 149 OPERATIONAL DEMONSTRATION MODULES</span>
            </div>
            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black text-[#101114] tracking-tight">
              Hardware Testing &amp; Forensic Footage.
            </h2>
            <p className="text-sm sm:text-base text-[#555B66] max-w-2xl leading-relaxed">
              Uncut field video recordings validating NIST SP 800-88 sanitization, real-time Shannon entropy sweeps, raw sector carving, and embedded terminal operations.
            </p>
          </div>

          <div className="flex items-center gap-3 shrink-0">
            <div className="px-3.5 py-1.5 rounded-lg bg-white border border-slate-200 text-xs font-mono text-slate-700 flex items-center gap-2 shadow-xs">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span>10 MODULES • 60 FPS CAPTURE</span>
            </div>
          </div>
        </div>

        {/* Filter Tabs (Light Theme) */}
        <div className="flex items-center gap-2 overflow-x-auto scrollbar-none pb-1">
          {CATEGORIES.map((cat) => {
            const isActive = selectedCategory === cat.id;
            return (
              <button
                key={cat.id}
                onClick={() => setSelectedCategory(cat.id)}
                className={`px-4 py-2 rounded-xl text-xs font-bold font-mono transition-all cursor-pointer whitespace-nowrap ${
                  isActive
                    ? 'bg-[#E31B23] text-white shadow-sm shadow-red-500/20'
                    : 'bg-white text-slate-700 border border-slate-200 hover:text-slate-900 hover:bg-slate-50'
                }`}
              >
                {cat.label}
              </button>
            );
          })}
        </div>

        {/* 3-Column Video Card Grid with Stagger Motion Animation (Clean Editorial Light Theme) */}
        <motion.div
          layout
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
        >
          <AnimatePresence>
            {filteredVideos.map((video, idx) => (
              <motion.div
                key={video.id}
                layout
                initial={{ opacity: 0, y: 24 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95 }}
                transition={{ duration: 0.35, delay: idx * 0.06 }}
                className="rounded-2xl overflow-hidden bg-white border border-slate-200 shadow-sm hover:shadow-md hover:border-slate-300 transition-all duration-300 flex flex-col group hover:-translate-y-1.5"
              >
                {/* Video Player */}
                <div className="relative w-full aspect-video bg-black overflow-hidden border-b border-slate-100">
                  <video
                    src={video.src}
                    controls
                    preload="metadata"
                    playsInline
                    className="w-full h-full object-cover group-hover:scale-[1.01] transition-transform duration-300"
                  />
                </div>

                {/* Card Content (Light Theme Layout from Image 4) */}
                <div className="p-5 sm:p-6 flex-1 flex flex-col justify-between space-y-4">
                  <div className="space-y-2">
                    {/* Cyan / Teal Technical Category Badge */}
                    <div className="text-[11px] font-mono font-bold tracking-wider text-cyan-600 uppercase">
                      {video.badge}
                    </div>

                    {/* Card Title */}
                    <h3 className="text-base sm:text-lg font-bold text-slate-900 group-hover:text-[#E31B23] transition-colors tracking-tight font-sans leading-snug">
                      {video.title}
                    </h3>

                    {/* Descriptive Text */}
                    <p className="text-xs sm:text-sm text-slate-600 leading-relaxed font-sans line-clamp-3">
                      {video.description}
                    </p>
                  </div>

                  {/* Hash Tags & Footer */}
                  <div className="pt-3 border-t border-slate-100 flex flex-wrap items-center gap-1.5">
                    {video.tags.map((tag, tIdx) => (
                      <span
                        key={tIdx}
                        className="px-2 py-0.5 rounded text-[10px] font-mono bg-slate-50 text-slate-600 border border-slate-200/80"
                      >
                        #{tag}
                      </span>
                    ))}
                  </div>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>
        </motion.div>

      </div>
    </section>
  );
};
