import React, { useState } from 'react';
import CursorGrid from './CursorGrid';
import {
  HardDrive,
  FileSearch,
  ShieldCheck,
  ArrowRight,
  Layers,
  FileText,
  Hash,
  Folder,
  CheckCircle2,
  Play,
  FileCode,
  Shield,
  Check,
  Lock,
  Cpu,
  Sparkles,
  ExternalLink
} from 'lucide-react';
import CountUp from './CountUp';

interface ForensicWorkflowProps {
  onExploreWorkflow?: () => void;
  onLearnMore?: () => void;
}

export const ForensicWorkflow: React.FC<ForensicWorkflowProps> = ({
  onExploreWorkflow,
  onLearnMore,
}) => {
  const [activeBranch, setActiveBranch] = useState<number | null>(null);

  return (
    <section className="relative w-full bg-white border-b border-gray-100 overflow-hidden font-sans">
      {/* Interactive Cursor Grid Background with Relatable RAZE-CARVE Red Accent */}
      <CursorGrid
        cellSize={58}
        color="#E31B23"
        radius={180}
        falloff="smooth"
        holdTime={400}
        fadeDuration={850}
        lineWidth={1.2}
        maxOpacity={0.7}
        fillOpacity={0.05}
        gridOpacity={0.04}
        cellRadius={6}
        clickPulse={true}
        pulseSpeed={650}
        className="z-0"
      />

      <div className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 py-16 sm:py-24 relative z-10 space-y-12">
        {/* ========================================================================= */}
        {/* 1. TOP HEADER ROW (SPLIT: EYEBROW + TITLE + TAGLINE)                     */}
        {/* ========================================================================= */}
        <div className="flex flex-col lg:flex-row lg:items-start justify-between gap-6 border-b border-gray-100 pb-10">
          <div className="space-y-3 max-w-2xl">
            {/* Red Horizontal Rule + Eyebrow */}
            <div className="flex items-center gap-3">
              <span className="w-8 h-[2.5px] bg-[#E31B23] inline-block rounded-full" />
              <span className="text-xs font-mono font-bold tracking-[0.2em] text-[#101114] uppercase">
                FORENSIC WORKFLOW
              </span>
            </div>

            {/* Main Headline */}
            <h2 className="text-3xl sm:text-4xl lg:text-[46px] font-black text-[#101114] tracking-tight leading-[1.12]">
              From raw media to{' '}
              <span className="text-[#E31B23]">verified evidence.</span>
            </h2>

            {/* Subtitle */}
            <p className="text-base sm:text-lg text-[#555B66] leading-relaxed max-w-xl">
              RAZE-CARVE brings secure erasure, forensic recovery, and evidence verification into one continuous workflow.
            </p>
          </div>

          {/* Right Tagline with Left Border Accent */}
          <div className="hidden lg:flex items-center border-l-2 border-gray-200 pl-6 py-2">
            <span className="font-mono text-xs font-bold text-[#777E90] uppercase tracking-[0.18em] leading-relaxed max-w-[220px]">
              TRUSTED TO UNCOVER <br />
              WHAT OTHERS MISS.
            </span>
          </div>
        </div>

        {/* ========================================================================= */}
        {/* 2. MAIN WORKFLOW DIAGRAM (LEFT BULLETS + CENTER/RIGHT ISOMETRIC FLOW)    */}
        {/* ========================================================================= */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-14 items-center">
          {/* ----------------------------------------------------------------------- */}
          {/* Left Column: 3 Feature Bullets + Action Button (~30% width)             */}
          {/* ----------------------------------------------------------------------- */}
          <div className="lg:col-span-4 space-y-8">
            <div className="space-y-6">
              {/* Bullet 1: Wide device support */}
              <div className="flex items-start gap-4 group">
                <div className="w-10 h-10 rounded-xl border border-gray-200 bg-[#F9FAFB] flex items-center justify-center text-slate-800 shrink-0 group-hover:border-[#E31B23]/40 group-hover:text-[#E31B23] transition-colors">
                  <Layers className="w-5 h-5" />
                </div>
                <div className="space-y-0.5">
                  <h4 className="text-base font-bold text-[#101114]">
                    Wide device support
                  </h4>
                  <p className="text-xs sm:text-sm text-[#555B66]">
                    HDD, SSD, USB and more.
                  </p>
                </div>
              </div>

              {/* Bullet 2: Forensic-grade processing */}
              <div className="flex items-start gap-4 group">
                <div className="w-10 h-10 rounded-xl border border-gray-200 bg-[#F9FAFB] flex items-center justify-center text-slate-800 shrink-0 group-hover:border-[#E31B23]/40 group-hover:text-[#E31B23] transition-colors">
                  <FileText className="w-5 h-5" />
                </div>
                <div className="space-y-0.5">
                  <h4 className="text-base font-bold text-[#101114]">
                    Forensic-grade processing
                  </h4>
                  <p className="text-xs sm:text-sm text-[#555B66]">
                    Works beyond the file system.
                  </p>
                </div>
              </div>

              {/* Bullet 3: Audit-ready results */}
              <div className="flex items-start gap-4 group">
                <div className="w-10 h-10 rounded-xl border border-gray-200 bg-[#F9FAFB] flex items-center justify-center text-slate-800 shrink-0 group-hover:border-[#E31B23]/40 group-hover:text-[#E31B23] transition-colors">
                  <ShieldCheck className="w-5 h-5" />
                </div>
                <div className="space-y-0.5">
                  <h4 className="text-base font-bold text-[#101114]">
                    Audit-ready results
                  </h4>
                  <p className="text-xs sm:text-sm text-[#555B66]">
                    Built for investigative integrity.
                  </p>
                </div>
              </div>
            </div>

            {/* Explore Workflow CTA Button */}
            <div className="pt-2">
              <button
                onClick={onExploreWorkflow}
                className="inline-flex items-center justify-center gap-2.5 px-6 py-3.5 text-sm font-bold text-white bg-[#101114] hover:bg-black rounded-lg shadow-md hover:shadow-lg transition-all duration-150 cursor-pointer group"
              >
                <span>Explore the workflow</span>
                <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
              </button>
            </div>
          </div>

          {/* ----------------------------------------------------------------------- */}
          {/* Right Column: Visual Continuous Workflow Diagram (~70% width)          */}
          {/* ----------------------------------------------------------------------- */}
          <div className="lg:col-span-8 relative bg-white/60 rounded-3xl p-4 sm:p-6 lg:p-8 border border-gray-200/80 shadow-xs">
            {/* Grid Layout: Input Media Stack -> Engine Glass Cube -> 3 Output Nodes */}
            <div className="grid grid-cols-1 md:grid-cols-12 gap-6 lg:gap-8 items-center">
              {/* =================================================================== */}
              {/* NODE 1: RAW STORAGE INPUT (HDD, SSD, USB)                           */}
              {/* =================================================================== */}
              <div className="md:col-span-4 flex flex-col items-center text-center space-y-4">
                {/* Hardware Devices Cluster Illustration */}
                <div className="relative w-full max-w-[220px] h-[170px] flex items-center justify-center">
                  {/* HDD Disk Graphic */}
                  <div className="absolute left-2 top-0 w-28 h-36 rounded-xl bg-gradient-to-br from-slate-700 to-slate-900 border border-slate-600 shadow-xl p-2 transform -rotate-6 transition-transform hover:rotate-0 hover:z-20">
                    <div className="w-full h-full rounded-lg bg-[#0F131C] p-2 flex flex-col justify-between text-left">
                      <div className="flex items-center justify-between">
                        <span className="text-[8px] font-mono text-slate-400 font-bold">SATA HDD 2TB</span>
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                      </div>
                      {/* Platter preview */}
                      <div className="w-14 h-14 rounded-full border-2 border-slate-600/70 bg-gradient-to-tr from-slate-800 to-slate-600 mx-auto flex items-center justify-center shadow-inner">
                        <div className="w-5 h-5 rounded-full bg-slate-950 border border-slate-400" />
                      </div>
                      <span className="text-[7px] font-mono text-slate-500">RAW LBA 0x00..FF</span>
                    </div>
                  </div>

                  {/* Generic SSD Graphic */}
                  <div className="absolute right-3 top-6 w-28 h-20 rounded-lg bg-gradient-to-br from-[#1E2330] to-[#12151F] border border-slate-600 shadow-2xl p-2.5 transform rotate-3 transition-transform hover:rotate-0 hover:z-20">
                    <div className="h-full flex flex-col justify-between">
                      <div className="flex items-center justify-between">
                        <span className="text-[9px] font-black text-white font-sans tracking-wider">NVMe SSD</span>
                        <span className="w-2 h-1 bg-[#E31B23] rounded-xs" />
                      </div>
                      <div className="text-[8px] font-mono text-slate-300">Solid State Drive</div>
                      <div className="flex items-center justify-between text-[7px] font-mono text-slate-500">
                        <span>PCIe Gen4</span>
                        <span>1.0 TB</span>
                      </div>
                    </div>
                  </div>

                  {/* USB Flash Drive Graphic */}
                  <div className="absolute bottom-1 left-10 w-24 h-9 rounded-md bg-gradient-to-r from-slate-900 via-slate-800 to-slate-900 border border-slate-700 shadow-2xl flex items-center px-2 z-10">
                    <div className="w-3.5 h-5 bg-slate-300 border border-slate-400 rounded-xs mr-2 shrink-0 flex flex-col justify-around py-0.5">
                      <div className="w-1.5 h-0.5 bg-slate-700 mx-auto" />
                      <div className="w-1.5 h-0.5 bg-slate-700 mx-auto" />
                    </div>
                    <div className="flex-1">
                      <div className="text-[8px] font-bold text-white font-sans">USB 3.2</div>
                      <div className="w-1.5 h-1.5 rounded-full bg-[#E31B23] animate-pulse mt-0.5" />
                    </div>
                  </div>
                </div>

                {/* Device Labels */}
                <div className="space-y-1">
                  <div className="text-xs sm:text-sm font-bold text-[#101114]">
                    HDD / SSD / USB
                  </div>
                  <div className="text-[11px] text-[#777E90] font-mono">
                    Any device. Any condition.
                  </div>
                </div>

                {/* Raw Media Badge Card */}
                <div className="w-full max-w-[190px] p-2.5 rounded-xl bg-gray-50 border border-gray-200/90 text-left space-y-1">
                  <div className="text-[11px] font-mono font-bold text-[#101114] uppercase flex items-center gap-1.5">
                    <Folder className="w-3.5 h-3.5 text-[#E31B23]" />
                    <span>RAW MEDIA</span>
                  </div>
                  <ul className="text-[10px] text-[#555B66] font-mono space-y-0.5 pl-5 list-disc">
                    <li>Deleted files</li>
                    <li>Hidden data</li>
                    <li>Unallocated space</li>
                  </ul>
                </div>
              </div>

              {/* =================================================================== */}
              {/* CONNECTOR 1 + CENTER ENGINE ISOMETRIC GLASS CUBE (4 cols)           */}
              {/* =================================================================== */}
              <div className="md:col-span-3 flex flex-col items-center justify-center relative py-4">
                {/* Glow Backdrop */}
                <div className="absolute inset-0 bg-red-500/10 rounded-full blur-2xl pointer-events-none" />

                {/* 3D Glassmorphic Engine Cube */}
                <div className="relative w-36 h-36 rounded-2xl bg-gradient-to-b from-white/95 via-red-50/40 to-white/90 border-2 border-red-500/40 shadow-[0_12px_30px_-8px_rgba(227,27,35,0.25)] flex flex-col items-center justify-center p-3 text-center transition-all duration-300 hover:scale-105 hover:border-red-500 group">
                  {/* Official Logo Center Icon */}
                  <div className="w-10 h-10 flex items-center justify-center mb-1 group-hover:scale-110 transition-transform">
                    <img src="/logo-icon.png" alt="RAZE-CARVE" className="w-9 h-9 object-contain" />
                  </div>

                  <div className="text-[11px] font-black tracking-wider text-[#101114] font-serif">
                    RAZE-CARVE
                  </div>
                  <div className="text-[9px] font-mono font-bold text-[#E31B23] tracking-wider uppercase mt-0.5">
                    FORENSIC ENGINE
                  </div>
                  <div className="text-[8px] font-mono text-[#777E90] mt-1">
                    Scan. Process. Validate.
                  </div>

                  {/* Corner Accent Dots */}
                  <span className="absolute top-2 left-2 w-1.5 h-1.5 rounded-full bg-red-400/60" />
                  <span className="absolute top-2 right-2 w-1.5 h-1.5 rounded-full bg-red-400/60" />
                  <span className="absolute bottom-2 left-2 w-1.5 h-1.5 rounded-full bg-red-400/60" />
                  <span className="absolute bottom-2 right-2 w-1.5 h-1.5 rounded-full bg-red-400/60" />
                </div>
              </div>

              {/* =================================================================== */}
              {/* NODE 3: 3 BRANCHING CAPABILITY CARDS (5 cols)                       */}
              {/* =================================================================== */}
              <div className="md:col-span-5 space-y-4">
                {/* ----------------------------------------------------------------- */}
                {/* Branch 01: SECURE ERASE                                           */}
                {/* ----------------------------------------------------------------- */}
                <div
                  onMouseEnter={() => setActiveBranch(1)}
                  onMouseLeave={() => setActiveBranch(null)}
                  className={`p-3.5 sm:p-4 rounded-2xl border transition-all duration-200 bg-white ${
                    activeBranch === 1
                      ? 'border-[#E31B23] shadow-md -translate-y-0.5'
                      : 'border-gray-200 hover:border-gray-300 shadow-xs'
                  }`}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-mono font-black text-[#E31B23]">01</span>
                        <h4 className="text-xs sm:text-sm font-black text-[#101114] tracking-tight uppercase">
                          SECURE ERASE
                        </h4>
                      </div>
                      <p className="text-[11px] text-[#555B66] leading-snug">
                        Permanently sanitize media using compliant methods.
                      </p>
                    </div>

                    {/* Status Badge */}
                    <div className="shrink-0 p-2 rounded-lg bg-red-50/80 border border-red-200/70 text-right">
                      <div className="flex items-center gap-1 text-[10px] font-mono font-bold text-[#E31B23]">
                        <span className="w-1.5 h-1.5 rounded-full bg-[#E31B23]" />
                        <span>Erased</span>
                      </div>
                      <div className="text-[8px] font-mono text-[#777E90] mt-0.5">
                        No recoverable data
                      </div>
                    </div>
                  </div>
                </div>

                {/* ----------------------------------------------------------------- */}
                {/* Branch 02: RECOVER                                                */}
                {/* ----------------------------------------------------------------- */}
                <div
                  onMouseEnter={() => setActiveBranch(2)}
                  onMouseLeave={() => setActiveBranch(null)}
                  className={`p-3.5 sm:p-4 rounded-2xl border transition-all duration-200 bg-white ${
                    activeBranch === 2
                      ? 'border-[#E31B23] shadow-md -translate-y-0.5'
                      : 'border-gray-200 hover:border-gray-300 shadow-xs'
                  }`}
                >
                  <div className="space-y-2.5">
                    <div className="flex items-start justify-between gap-3">
                      <div className="space-y-1">
                        <div className="flex items-center gap-2">
                          <span className="text-xs font-mono font-black text-[#E31B23]">02</span>
                          <h4 className="text-xs sm:text-sm font-black text-[#101114] tracking-tight uppercase">
                            RECOVER
                          </h4>
                        </div>
                        <p className="text-[11px] text-[#555B66] leading-snug">
                          Reconstruct deleted and fragmented files using advanced carving.
                        </p>
                      </div>

                      {/* Mini Media Preview Stack */}
                      <div className="shrink-0 relative w-12 h-10 flex items-center justify-center">
                        <div className="absolute right-0 top-0 w-8 h-8 rounded bg-slate-200 border border-slate-300 transform rotate-6" />
                        <div className="absolute left-0 bottom-0 w-9 h-9 rounded bg-[#101114] text-white flex items-center justify-center shadow-md">
                          <Play className="w-3.5 h-3.5 fill-current text-white" />
                        </div>
                      </div>
                    </div>

                    {/* File Formats Pill Tags */}
                    <div className="flex flex-wrap gap-1.5 pt-1 border-t border-gray-100">
                      {['Documents', 'Images', 'Videos', 'Archives', 'And more'].map((tag, i) => (
                        <span
                          key={i}
                          className="px-2 py-0.5 rounded text-[9px] font-mono font-medium bg-gray-100 text-slate-700"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>

                {/* ----------------------------------------------------------------- */}
                {/* Branch 03: VERIFY                                                 */}
                {/* ----------------------------------------------------------------- */}
                <div
                  onMouseEnter={() => setActiveBranch(3)}
                  onMouseLeave={() => setActiveBranch(null)}
                  className={`p-3.5 sm:p-4 rounded-2xl border transition-all duration-200 bg-white ${
                    activeBranch === 3
                      ? 'border-[#E31B23] shadow-md -translate-y-0.5'
                      : 'border-gray-200 hover:border-gray-300 shadow-xs'
                  }`}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-mono font-black text-[#E31B23]">03</span>
                        <h4 className="text-xs sm:text-sm font-black text-[#101114] tracking-tight uppercase">
                          VERIFY
                        </h4>
                      </div>
                      <p className="text-[11px] text-[#555B66] leading-snug">
                        Generate cryptographic hashes and complete audit logs.
                      </p>
                    </div>

                    {/* Certificate & Hash Status Badge */}
                    <div className="shrink-0 p-2 rounded-lg bg-emerald-50/80 border border-emerald-200/70 text-right">
                      <div className="text-[9px] font-mono font-bold text-slate-700">SHA-256</div>
                      <div className="text-[8px] font-mono text-slate-400">3f2a6e9c4d0b...</div>
                      <div className="flex items-center gap-1 text-[10px] font-mono font-bold text-emerald-600 mt-1 justify-end">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                        <span>Verified</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* ========================================================================= */}
        {/* 3. BOTTOM CAPABILITY STRIP (BORDERED 4-MODULE BAR + LEARN MORE)          */}
        {/* ========================================================================= */}
        <div className="pt-6 border-t border-gray-200 space-y-4">
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-6 lg:gap-10 flex-1">
              {/* Item 1: HDD / SSD / USB */}
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-lg bg-red-50 border border-red-100 flex items-center justify-center text-[#E31B23] shrink-0">
                  <HardDrive className="w-4.5 h-4.5" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] font-sans">
                    HDD / SSD / USB
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Wide device support
                  </div>
                </div>
              </div>

              {/* Item 2: RAW MEDIA */}
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-lg bg-red-50 border border-red-100 flex items-center justify-center text-[#E31B23] shrink-0">
                  <Folder className="w-4.5 h-4.5" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] font-sans">
                    RAW MEDIA
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Works without file-system metadata
                  </div>
                </div>
              </div>

              {/* Item 3: SHA-256 */}
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-lg bg-red-50 border border-red-100 flex items-center justify-center text-[#E31B23] shrink-0">
                  <Hash className="w-4.5 h-4.5" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] font-sans">
                    SHA-256
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Cryptographic verification
                  </div>
                </div>
              </div>

              {/* Item 4: AUDIT LOGS */}
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-lg bg-red-50 border border-red-100 flex items-center justify-center text-[#E31B23] shrink-0">
                  <FileText className="w-4.5 h-4.5" />
                </div>
                <div>
                  <div className="text-xs sm:text-sm font-bold text-[#101114] font-sans">
                    AUDIT LOGS
                  </div>
                  <div className="text-[11px] text-[#777E90]">
                    Complete operation records
                  </div>
                </div>
              </div>
            </div>

            {/* Learn More Link */}
            <button
              onClick={onLearnMore}
              className="inline-flex items-center gap-1.5 text-xs sm:text-sm font-bold text-[#101114] hover:text-[#E31B23] border-b border-[#101114] hover:border-[#E31B23] pb-0.5 whitespace-nowrap transition-colors cursor-pointer group shrink-0"
            >
              <span>Learn more</span>
              <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
            </button>
          </div>

          {/* Micro Footer Legend Note */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between text-[10px] font-mono text-[#94A3B8] pt-4 border-t border-gray-100 tracking-wider">
            <span>SECURE DATA. STRONGER TRUTH.</span>
            <span>BUILT FOR A SAFER DIGITAL WORLD.</span>
          </div>
        </div>
      </div>
    </section>
  );
};

export default ForensicWorkflow;
