import React, { useState } from 'react';
import { RoutePath } from '../types';
import { AnimatedTerminal } from '../components/AnimatedTerminal';
import WarpText from '../components/WarpText';
import { ForensicWorkflow } from '../components/ForensicWorkflow';
import CountUp from '../components/CountUp';
import { VideoGallery } from '../components/VideoGallery';
import {
  Shield,
  HardDrive,
  Cpu,
  FileSearch,
  CheckCircle2,
  Terminal,
  Activity,
  Download,
  BookOpen,
  ArrowRight,
  FileText,
  ShieldCheck,
  Zap,
  Check,
  X,
  Database,
  Lock,
  ChevronRight,
  ChevronLeft,
  Users,
  Award,
  Layers,
  Sparkles,
  RefreshCw,
  Clock,
  Key,
  Globe,
  Sliders,
  Play,
  FileSpreadsheet,
  FolderOpen,
  CheckCircle,
  Hash,
  Folder,
} from 'lucide-react';

interface HomePageProps {
  onNavigate: (path: RoutePath) => void;
}

export const HomePage: React.FC<HomePageProps> = ({ onNavigate }) => {
  const [activeTab, setActiveTab] = useState<number>(0);
  const [showOverviewModal, setShowOverviewModal] = useState<boolean>(false);

  const standardsList = [
    {
      logoText: 'NIST',
      std: 'NIST SP 800-88',
      desc: 'Media Sanitization',
    },
    {
      logoText: 'DoD',
      std: 'DoD 5220.22-M',
      desc: 'Data Eradication',
    },
    {
      logoText: 'ISO',
      std: 'ISO/IEC 27001',
      desc: 'Information Security',
    },
    {
      logoText: 'RFC',
      std: 'RFC 3161',
      desc: 'Time-Stamping',
    },
    {
      logoText: 'Common Criteria',
      isCC: true,
      std: 'Common Criteria',
      desc: 'EAL4+ Security',
    },
    {
      logoText: 'IEEE',
      isIEEE: true,
      std: 'IEEE 2883',
      desc: 'Forensic Standards',
    },
  ];

  const capabilityModules = [
    {
      id: 0,
      tabLabel: 'Drive Eraser',
      icon: <HardDrive className="w-4 h-4" />,
      eyebrow: 'SECURE DATA DISPOSAL',
      headline: 'Erase with confidence.',
      description:
        'RAZE-CARVE sanitizes supported HDDs, SSDs, USB drives and external media with configurable erasure methods and post-operation verification.',
      capabilities: [
        {
          title: 'Verified Erasure',
          desc: 'Validate the sanitization operation before completion.',
        },
        {
          title: 'Audit-Ready Evidence',
          desc: 'Capture a complete record of the sanitization process.',
        },
        {
          title: 'Multi-Device Support',
          desc: 'Work across supported storage devices and removable media.',
        },
      ],
      appData: {
        headerBadge: 'DRIVE SANITIZATION',
        viewTitle: 'Sanitize Drive',
        viewSubtitle: 'Permanently erase data using compliant sanitization methods.',
        targetLabel: 'Target Device',
        deviceName: 'Enterprise Solid-State Drive',
        deviceSpecs: '931.5 GB | SATA',
        deviceStatus: 'READY',
        methodLabel: 'Sanitization Method',
        methodValue: 'NIST SP 800-88 (Clear & Purge)',
        verifyCheckbox: 'Verify erasure after completion',
        stat1Label: 'Estimated Time',
        stat1Value: '~ 24 minutes',
        stat2Label: 'Device Status',
        stat2Value: 'Ready for Erasure',
        actionLabel: 'Start Sanitization',
      },
    },
    {
      id: 1,
      tabLabel: 'File & Folder Eraser',
      icon: <FileText className="w-4 h-4" />,
      eyebrow: 'TARGETED DATA DESTRUCTION',
      headline: 'Precise, in-place file shredding.',
      description:
        'Target individual files, directories, and residual free space without altering undamaged storage volumes or operating system partitions.',
      capabilities: [
        {
          title: 'Targeted Inode Shredding',
          desc: 'Overwrite specific file blocks directly at the filesystem boundary.',
        },
        {
          title: 'Residual Space Sanitization',
          desc: 'Safely clear deleted file remnants and directory metadata.',
        },
        {
          title: 'Customizable Pass Configurations',
          desc: 'Choose single or multi-pass overwrite patterns per folder.',
        },
      ],
      appData: {
        headerBadge: 'FILE & FOLDER SHREDDER',
        viewTitle: 'Erase Files & Directories',
        viewSubtitle: 'Permanently scrub sensitive folders and metadata.',
        targetLabel: 'Selected Target Directory',
        deviceName: '/var/log/audit/sensitive_transcripts/',
        deviceSpecs: '48.2 MB • 142 files selected',
        deviceStatus: 'QUEUED',
        methodLabel: 'Shredding Method',
        methodValue: 'DoD 3-Pass Overwrite + Zero Fill',
        verifyCheckbox: 'Wipe directory table entries after erase',
        stat1Label: 'Total Files',
        stat1Value: '142 Files',
        stat2Label: 'Operation Status',
        stat2Value: 'Queued & Locked',
        actionLabel: 'Shred Selected Files',
      },
    },
    {
      id: 2,
      tabLabel: 'File Recovery',
      icon: <FileSearch className="w-4 h-4" />,
      eyebrow: 'DEEP FORENSIC CARVING',
      headline: 'Recover what matters most.',
      description:
        'Rebuild damaged and deleted artifacts from unallocated disk sectors using structure-aware carving across documents, databases, and media.',
      capabilities: [
        {
          title: 'Structure-Aware Parsing',
          desc: 'Reconstruct files based on internal chunk syntax and boundary markers.',
        },
        {
          title: 'Broad Format Support',
          desc: 'Extract PDFs, images, compressed archives, and SQLite databases.',
        },
        {
          title: 'Non-Destructive Extraction',
          desc: 'Safely read from write-blocked physical or raw disk image sources.',
        },
      ],
      appData: {
        headerBadge: 'EVIDENCE RECONSTRUCTION',
        viewTitle: 'Structure-Aware File Carver',
        viewSubtitle: 'Scanning disk sectors for signature boundaries and valid chunks.',
        targetLabel: 'Source Image / Partition',
        deviceName: 'Forensic 2.0 TB Physical Image',
        deviceSpecs: 'Raw Sector Range: 0x00000000 - 0x1FFFFFFF',
        deviceStatus: 'SCANNING',
        methodLabel: 'Carving Profile',
        methodValue: 'Deep Structure + Chunk Header Parser',
        verifyCheckbox: 'Validate CRC-32 checksums during scan',
        stat1Label: 'Signatures Detected',
        stat1Value: '12,842 files',
        stat2Label: 'Scan Progress',
        stat2Value: '68% Completed',
        actionLabel: 'Export Recovered Evidence',
      },
    },
    {
      id: 3,
      tabLabel: 'Verification',
      icon: <Activity className="w-4 h-4" />,
      eyebrow: 'MATHEMATICAL VALIDATION',
      headline: 'Verify before you sign off.',
      description:
        'Confirm that erased storage contains zero residual data or validate recovered evidence integrity before submitting reports.',
      capabilities: [
        {
          title: 'Entropy Density Analysis',
          desc: 'Mathematical scanning to differentiate encrypted data from zeroed space.',
        },
        {
          title: 'Zero-Byte Verification',
          desc: 'Rapid sampling across LBA sector ranges to confirm complete erasure.',
        },
        {
          title: 'Integrity Checksums',
          desc: 'Generate SHA-256 verification hashes for evidence validation.',
        },
      ],
      appData: {
        headerBadge: 'ENTROPY & ZERO-BYTE CHECK',
        viewTitle: 'Sanitization Verification',
        viewSubtitle: 'Mathematical randomness scan and sector verification.',
        targetLabel: 'Inspected Volume',
        deviceName: 'Crucial T700 1.0 TB NVMe',
        deviceSpecs: 'Post-Wipe Sector Verification Scan',
        deviceStatus: 'VERIFIED',
        methodLabel: 'Verification Method',
        methodValue: 'Full Range Sample + Entropy Scan',
        verifyCheckbox: 'Compare pre-wipe and post-wipe density curves',
        stat1Label: 'Shannon Entropy',
        stat1Value: '0.000 b/B (Zero)',
        stat2Label: 'Sanitization Result',
        stat2Value: '100% Verified Clean',
        actionLabel: 'Generate Verification Certificate',
      },
    },
    {
      id: 4,
      tabLabel: 'Audit & Reports',
      icon: <ShieldCheck className="w-4 h-4" />,
      eyebrow: 'EVIDENCE INTEGRITY & CUSTODY',
      headline: 'Defensible, audit-ready reports.',
      description:
        'Generate signed certificates and immutable audit records that document every sanitization and recovery operation from start to finish.',
      capabilities: [
        {
          title: 'Cryptographic Audit Log',
          desc: 'Record every action with SHA-256 integrity checksums.',
        },
        {
          title: 'Timestamp Verification',
          desc: 'Attach verifiable time records for full chain-of-custody proof.',
        },
        {
          title: 'Compliance Export',
          desc: 'Export clear, structured PDF and JSON reports ready for review.',
        },
      ],
      appData: {
        headerBadge: 'CHAIN OF CUSTODY AUDIT',
        viewTitle: 'Forensic Audit & Certificate Generator',
        viewSubtitle: 'Cryptographic record and compliance report ready for export.',
        targetLabel: 'Case Reference & Session',
        deviceName: 'CASE-2026-0914 (Operator: ID-4091)',
        deviceSpecs: 'SHA-256 Genesis: 3f2a6e9c...8b7f',
        deviceStatus: 'SEALED',
        methodLabel: 'Audit Standard',
        methodValue: 'SHA-256 Merkle Chain + RFC 3161 TSA',
        verifyCheckbox: 'Embed digital timestamp certificate in report',
        stat1Label: 'Logged Operations',
        stat1Value: '14 Operations',
        stat2Label: 'Chain Status',
        stat2Value: 'Cryptographically Sealed',
        actionLabel: 'Download Audit PDF Certificate',
      },
    },
  ];

  const currentMod = capabilityModules[activeTab];

  return (
    <div className="w-full bg-white text-[#101114] font-sans selection:bg-[#E31B23] selection:text-white overflow-x-hidden">
      {/* ========================================================================= */}
      {/* 1. HERO SECTION (CLEAN 2-COLUMN: MARKETING COPY + DIRECT PRODUCT VIDEO)   */}
      {/* ========================================================================= */}
      <section className="relative w-full bg-white border-b border-gray-100 overflow-hidden">
        <div className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 py-8 sm:py-12 lg:py-16">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-12 items-center min-h-[520px] lg:min-h-[580px]">
            {/* Left Column: Content (~46%) */}
            <div className="lg:col-span-5 space-y-6 max-w-xl">
              {/* Technical Eyebrow */}
              <div className="flex items-center gap-2.5">
                <span className="w-6 h-[2.5px] bg-[#E31B23] inline-block rounded-full" />
                <span className="text-[11px] sm:text-xs font-bold tracking-[0.16em] text-[#555B66] uppercase font-mono">
                  DIGITAL FORENSICS &nbsp;/&nbsp; SECURE ERASURE &nbsp;/&nbsp; EVIDENCE INTEGRITY
                </span>
              </div>

              {/* Headline with WarpText WebGL Interactive Shader */}
              <div className="w-full">
                <WarpText
                  text={'Secure Everything.\nRecover What Matters.'}
                  color="#101114"
                  align="left"
                  warpStrength={0.07}
                  warpScale={1.6}
                  speed={0.45}
                  pointerInfluence={0.42}
                  pointerStrength={0.38}
                  refraction={0.016}
                  ripple={true}
                  fontSize="clamp(2.4rem, 4.2vw, 3.6rem)"
                  fontWeight={900}
                  letterSpacing="-0.04em"
                  lineHeight={1.08}
                  style={{ height: '175px' }}
                  className="w-full"
                />
              </div>

              {/* Description */}
              <p className="text-base sm:text-lg text-[#555B66] leading-relaxed font-normal">
                RAZE-CARVE unifies secure data erasure, forensic recovery, file carving, verification, and audit reporting in one evidence-ready platform.
              </p>

              {/* CTAs */}
              <div className="flex flex-wrap items-center gap-5 pt-2">
                <button
                  onClick={() => onNavigate('/cli')}
                  className="inline-flex items-center justify-center gap-2 px-7 py-3.5 text-base font-bold text-white bg-[#E31B23] hover:bg-[#c9141b] active:bg-[#b01016] rounded-lg shadow-md shadow-red-500/20 hover:shadow-lg hover:shadow-red-500/30 transition-all duration-150 cursor-pointer"
                >
                  <span>Explore the Platform</span>
                  <ArrowRight className="w-4.5 h-4.5" />
                </button>

                <button
                  onClick={() => setShowOverviewModal(true)}
                  className="inline-flex items-center gap-1.5 text-base font-bold text-[#101114] hover:text-[#E31B23] border-b-2 border-slate-300 hover:border-[#E31B23] pb-0.5 transition-all duration-150 cursor-pointer group"
                >
                  <span>See how it works</span>
                  <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                </button>
              </div>

              {/* Supporting Line */}
              <div className="pt-2 flex items-center gap-3 text-xs sm:text-sm text-[#777E90] font-medium">
                <div className="w-6 h-6 rounded-full bg-red-50 border border-red-100 flex items-center justify-center text-[#E31B23] shrink-0">
                  <Users className="w-3.5 h-3.5" />
                </div>
                <span>Built for cybersecurity teams, digital forensics &amp; secure data disposal</span>
              </div>
            </div>

            {/* Right Column: Actual Product Demo Video (~54%) */}
            <div className="lg:col-span-7 relative w-full flex items-center justify-center">
              <div className="relative w-full rounded-2xl lg:rounded-3xl overflow-hidden bg-[#08090B] border border-slate-800/80 shadow-[0_20px_50px_rgba(0,0,0,0.25)] aspect-[16/10] sm:aspect-[16/9] max-h-[560px] flex items-center justify-center">
                <video
                  src="/hero-demo.mp4"
                  autoPlay
                  muted
                  loop
                  playsInline
                  className="w-full h-full object-cover object-center"
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 2. TRUSTED STANDARDS SECTION                                             */}
      {/* ========================================================================= */}
      <section className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 py-16 border-b border-gray-100">
        {/* Top Header Row with Split Text & Link */}
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6 mb-10">
          <div className="space-y-1.5 max-w-xl">
            <span className="text-xs font-mono font-bold tracking-[0.2em] text-[#E31B23] uppercase">
              TRUSTED STANDARDS
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-[#101114] tracking-tight">
              Built to comply with global cybersecurity &amp; forensic standards.
            </h2>
          </div>

          <div className="flex flex-col sm:flex-row sm:items-center gap-6 lg:border-l lg:border-gray-200 lg:pl-8">
            <p className="text-xs sm:text-sm text-[#555B66] max-w-md leading-relaxed">
              RAZE-CARVE follows internationally recognized standards to ensure defensible, audit-ready and legally valid outcomes.
            </p>
            <button
              onClick={() => onNavigate('/docs')}
              className="inline-flex items-center gap-1.5 text-xs sm:text-sm font-bold text-[#101114] hover:text-[#E31B23] whitespace-nowrap transition-colors cursor-pointer group"
            >
              <span>View all standards</span>
              <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
            </button>
          </div>
        </div>

        {/* 6 Standards Cards Row */}
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4">
          {standardsList.map((item, idx) => (
            <div
              key={idx}
              className="p-6 rounded-2xl border border-gray-200/80 bg-[#F9FAFB] text-center flex flex-col items-center justify-center space-y-2 hover:border-[#E31B23]/40 hover:bg-white hover:shadow-md transition-all duration-150 cursor-pointer group"
              onClick={() => onNavigate('/docs')}
            >
              <div className="h-8 flex items-center justify-center">
                {item.isCC ? (
                  <div className="flex items-center gap-1.5 text-slate-800 font-extrabold text-sm sm:text-base font-sans">
                    <span className="w-5 h-5 rounded-full border-2 border-slate-700 flex items-center justify-center text-[10px] font-mono font-bold">CC</span>
                    <span>Common Criteria</span>
                  </div>
                ) : item.isIEEE ? (
                  <div className="flex items-center gap-1.5 text-slate-800 font-extrabold text-base font-sans">
                    <span className="text-[#E31B23] text-sm">◆</span>
                    <span className="tracking-wider">IEEE</span>
                  </div>
                ) : (
                  <span className="text-xl sm:text-2xl font-black text-[#101114] tracking-tight font-sans">
                    {item.logoText}
                  </span>
                )}
              </div>

              <div className="text-xs sm:text-sm font-bold text-[#101114] font-sans">
                {item.std}
              </div>

              <div className="text-[11px] text-[#777E90] font-mono">
                {item.desc}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 3. UNIFIED ARCHITECTURE SHOWCASE (NO OUTER CARD • EDITORIAL REBALANCED)   */}
      {/* ========================================================================= */}
      <section className="relative w-full py-20 bg-white border-b border-gray-100 overflow-hidden">
        {/* Subtle Technical Dot Grid Background */}
        <div className="absolute inset-0 bg-[radial-gradient(#e5e7eb_1px,transparent_1px)] [background-size:24px_24px] opacity-40 pointer-events-none" />

        <div className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 relative z-10 space-y-12">
          {/* Section Header */}
          <div className="text-center max-w-3xl mx-auto space-y-4">
            <span className="text-xs font-mono font-extrabold tracking-[0.2em] text-[#E31B23] uppercase">
              UNIFIED ARCHITECTURE
            </span>
            <h2 className="text-3xl sm:text-4xl lg:text-[46px] font-black text-[#101114] tracking-tight leading-tight font-sans">
              One platform to replace <br />
              <span className="text-[#E31B23]">fragmented</span> forensic tools.
            </h2>
            <p className="text-sm sm:text-base lg:text-lg text-[#555B66] leading-relaxed max-w-2xl mx-auto">
              Commercial forensic toolchains require multiple disparate licenses costing $30,000/year. RAZE-CARVE consolidates silicon-level erasure, deep boundary carving, and audit verification into a single high-performance core.
            </p>
          </div>

          {/* Connected Capability Navigation Bar */}
          <div className="border-b border-gray-200 flex items-center justify-start sm:justify-center gap-1 sm:gap-4 overflow-x-auto scrollbar-none max-w-5xl mx-auto">
            {capabilityModules.map((mod) => (
              <button
                key={mod.id}
                onClick={() => setActiveTab(mod.id)}
                className={`py-3.5 px-4 sm:px-6 text-xs sm:text-sm font-bold transition-all flex items-center gap-2 whitespace-nowrap cursor-pointer ${
                  activeTab === mod.id
                    ? 'text-[#E31B23] border-b-2 border-[#E31B23] bg-red-50/50 rounded-t-lg'
                    : 'text-[#555B66] hover:text-[#101114] border-b-2 border-transparent hover:border-gray-300'
                }`}
              >
                {mod.icon}
                <span>{mod.tabLabel}</span>
              </button>
            ))}
          </div>

          {/* Showcase Display: Left Dark Product Dashboard (~54%) + Right Editorial Content (~40%) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-16 items-center pt-4">
            {/* Left Column: Direct Dark Software Dashboard (7 cols) */}
            <div className="lg:col-span-7 relative">
              <div className="rounded-2xl bg-[#0E1017] border border-slate-800 shadow-[0_20px_50px_-15px_rgba(0,0,0,0.3)] overflow-hidden font-sans">
                {/* Titlebar / Top Chrome */}
                <div className="bg-[#141722] border-b border-slate-800 px-4 sm:px-5 py-3 flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <img src="/logo-icon.png" alt="RAZE-CARVE" className="w-4 h-4 object-contain" />
                    <span className="text-xs font-bold text-white tracking-wider font-serif">
                      RAZE-CARVE
                    </span>
                    <span className="text-slate-600 text-xs">|</span>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-slate-900 text-slate-300 border border-slate-700 font-semibold uppercase">
                      {currentMod.appData.headerBadge}
                    </span>
                  </div>

                  <div className="flex items-center gap-3 text-xs font-mono">
                    <span className="text-slate-400 hidden sm:inline">CASE-2026-0914</span>
                    <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                  </div>
                </div>

                {/* Dashboard Main View Area */}
                <div className="p-6 sm:p-7 space-y-6 bg-gradient-to-b from-[#0E1017] to-[#0A0C12]">
                  {/* View Header */}
                  <div className="space-y-1">
                    <h3 className="text-lg sm:text-xl font-bold text-white tracking-tight">
                      {currentMod.appData.viewTitle}
                    </h3>
                    <p className="text-xs text-slate-400">
                      {currentMod.appData.viewSubtitle}
                    </p>
                  </div>

                  {/* Primary Target Highlight Card */}
                  <div className="p-4 sm:p-5 rounded-xl bg-[#141722] border border-slate-800/90 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                    <div className="flex items-center gap-3.5">
                      <div className="w-11 h-11 rounded-lg bg-black/50 border border-slate-700 flex items-center justify-center text-[#E31B23]">
                        {currentMod.icon}
                      </div>
                      <div>
                        <div className="text-xs text-slate-400 font-mono">
                          {currentMod.appData.targetLabel}
                        </div>
                        <div className="text-sm sm:text-base font-bold text-white">
                          {currentMod.appData.deviceName}
                        </div>
                        <div className="text-xs text-slate-400 font-mono mt-0.5">
                          {currentMod.appData.deviceSpecs}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-3 shrink-0">
                      <span className="inline-flex items-center gap-1.5 text-xs font-mono font-bold text-emerald-400 bg-emerald-950/60 px-2.5 py-1 rounded border border-emerald-800/40">
                        <span className="w-2 h-2 rounded-full bg-emerald-400" />
                        {currentMod.appData.deviceStatus}
                      </span>
                    </div>
                  </div>

                  {/* Operational Settings & Status Grid */}
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs font-mono">
                    <div className="p-4 rounded-xl bg-[#141722]/60 border border-slate-800 space-y-2.5">
                      <div className="text-slate-400 font-bold uppercase tracking-wider text-[10px]">
                        {currentMod.appData.methodLabel}
                      </div>
                      <div className="p-2.5 rounded bg-black/60 border border-slate-700 text-white font-semibold flex items-center justify-between">
                        <span className="truncate">{currentMod.appData.methodValue}</span>
                        <ChevronRight className="w-3.5 h-3.5 text-slate-400 shrink-0 ml-1" />
                      </div>
                      <div className="flex items-center gap-2 text-slate-300 pt-1">
                        <Check className="w-3.5 h-3.5 text-[#E31B23] shrink-0" />
                        <span className="text-[11px] truncate">{currentMod.appData.verifyCheckbox}</span>
                      </div>
                    </div>

                    <div className="p-4 rounded-xl bg-[#141722]/60 border border-slate-800 flex flex-col justify-between space-y-3">
                      <div className="space-y-1.5">
                        <div className="flex justify-between items-center text-slate-400 font-bold text-[11px]">
                          <span>{currentMod.appData.stat1Label}:</span>
                          <span className="text-white font-mono">{currentMod.appData.stat1Value}</span>
                        </div>
                        <div className="flex justify-between items-center text-slate-400 font-bold text-[11px]">
                          <span>{currentMod.appData.stat2Label}:</span>
                          <span className="text-emerald-400 font-mono">{currentMod.appData.stat2Value}</span>
                        </div>
                      </div>

                      <button 
                        onClick={() => onNavigate('/cli')}
                        className="w-full py-2.5 px-4 rounded-lg bg-[#E31B23] hover:bg-red-700 text-white font-bold text-xs flex items-center justify-center gap-2 shadow-sm transition-colors cursor-pointer"
                      >
                        <Play className="w-3.5 h-3.5 fill-current" />
                        <span>{currentMod.appData.actionLabel}</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Right Column: Editorial Copy & 3 Capabilities (5 cols) */}
            <div className="lg:col-span-5 space-y-6">
              <div className="space-y-3">
                <span className="text-xs font-mono font-bold tracking-[0.18em] text-[#E31B23] uppercase">
                  {currentMod.eyebrow}
                </span>
                <h3 className="text-3xl sm:text-4xl font-extrabold text-[#101114] tracking-tight leading-tight">
                  {currentMod.headline}
                </h3>
                <p className="text-base sm:text-lg text-[#555B66] leading-relaxed pt-1">
                  {currentMod.description}
                </p>
              </div>

              <div className="space-y-4 pt-4 border-t border-gray-100">
                {currentMod.capabilities.map((cap, idx) => (
                  <div key={idx} className="flex items-start gap-3.5">
                    <div className="w-8 h-8 rounded-lg bg-red-50 border border-red-100 flex items-center justify-center shrink-0 mt-0.5 text-[#E31B23]">
                      <Check className="w-4 h-4 stroke-[2.5]" />
                    </div>
                    <div>
                      <div className="text-base font-bold text-[#101114]">
                        {cap.title}
                      </div>
                      <div className="text-sm text-[#555B66] mt-0.5 leading-relaxed">
                        {cap.desc}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 4. FORENSIC WORKFLOW SECTION (EXACT 100% MATCH TO REFERENCE IMAGE)        */}
      {/* ========================================================================= */}
      <ForensicWorkflow
        onExploreWorkflow={() => onNavigate('/cli')}
        onLearnMore={() => onNavigate('/features')}
      />

      {/* ========================================================================= */}
      {/* 5. INTERACTIVE LIVE TERMINAL & WASM SANDBOX SECTION                      */}
      {/* ========================================================================= */}
      <section className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 py-16 space-y-8">
        <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4">
          <div className="space-y-2">
            <div className="text-xs font-mono font-bold text-[#E31B23] uppercase tracking-wider">
              INTERACTIVE EXPERIENCE
            </div>
            <h2 className="text-3xl sm:text-4xl font-extrabold text-[#101114] tracking-tight">
              Test drive RAZE-CARVE in real-time.
            </h2>
            <p className="text-[#555B66] text-sm sm:text-base max-w-xl">
              Experience the Terminal User Interface (sforge) and CLI commands directly in your browser without installing anything.
            </p>
          </div>

          <button
            onClick={() => onNavigate('/cli')}
            className="inline-flex items-center gap-2 px-6 py-3 text-xs sm:text-sm font-bold text-white bg-[#101114] hover:bg-black rounded-lg shadow-md transition-colors shrink-0 cursor-pointer"
          >
            <Terminal className="w-4 h-4" />
            <span>Open Live CLI &amp; Terminal</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Animated Terminal Component */}
        <AnimatedTerminal />
      </section>

      {/* ========================================================================= */}
      {/* 6. QUANTIFIED IMPACT METRICS (ANIMATED SPRING COUNT-UP STRIP)             */}
      {/* ========================================================================= */}
      <section className="w-full bg-[#08090B] text-white py-16 px-4 sm:px-8 border-y border-slate-800">
        <div className="max-w-[1536px] mx-auto grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
          <div className="space-y-2">
            <div className="text-4xl sm:text-5xl font-black text-[#E31B23] font-mono tracking-tight flex items-center justify-center">
              <CountUp to={100} duration={0.65} />
              <span>%</span>
            </div>
            <div className="text-xs sm:text-sm text-slate-400 font-medium">Verified Erasure Coverage</div>
          </div>
          <div className="space-y-2">
            <div className="text-4xl sm:text-5xl font-black text-[#00D26A] font-mono tracking-tight flex items-center justify-center">
              <CountUp to={420} duration={0.65} />
            </div>
            <div className="text-xs sm:text-sm text-slate-400 font-medium">Automated Test Assertions Passing</div>
          </div>
          <div className="space-y-2">
            <div className="text-4xl sm:text-5xl font-black text-white font-mono tracking-tight flex items-center justify-center gap-1.5">
              <CountUp from={7.999} to={0.000} decimals={3} duration={0.7} />
              <span className="text-2xl sm:text-3xl text-slate-300 font-normal">b/B</span>
            </div>
            <div className="text-xs sm:text-sm text-slate-400 font-medium">Mathematical Shannon Flatline Zero</div>
          </div>
          <div className="space-y-2">
            <div className="text-4xl sm:text-5xl font-black text-[#38BDF8] font-mono tracking-tight flex items-center justify-center gap-1.5">
              <span>&lt;</span>
              <CountUp from={10} to={1} duration={0.55} />
              <span className="text-2xl sm:text-3xl text-slate-300 font-normal">sec</span>
            </div>
            <div className="text-xs sm:text-sm text-slate-400 font-medium">Cryptographic Key Destruction</div>
          </div>
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 7. COMPETITIVE COMPARISON MATRIX                                         */}
      {/* ========================================================================= */}
      <section className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 py-20 space-y-8">
        <div className="text-center max-w-2xl mx-auto space-y-2">
          <div className="text-xs font-mono font-bold text-[#E31B23] uppercase tracking-wider">
            COMPETITIVE COMPARISON
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-[#101114] tracking-tight">
            How RAZE-CARVE compares to legacy tools.
          </h2>
        </div>

        <div className="overflow-x-auto rounded-2xl border border-gray-200 bg-white shadow-md">
          <table className="w-full text-left text-xs sm:text-sm">
            <thead className="bg-gray-50 text-[#101114] font-bold border-b border-gray-200">
              <tr>
                <th className="p-4 sm:p-5">Capability / Standard</th>
                <th className="p-4 sm:p-5 text-[#E31B23] font-black bg-red-50/40 font-serif">RAZE-CARVE</th>
                <th className="p-4 sm:p-5 text-[#555B66]">Commercial Forensics ($30k/yr)</th>
                <th className="p-4 sm:p-5 text-[#555B66]">Legacy CLI (dd, DBAN)</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              <tr>
                <td className="p-4 sm:p-5 font-semibold text-[#101114]">Hardware Storage Erasure Standards</td>
                <td className="p-4 sm:p-5 text-emerald-700 font-bold flex items-center gap-1.5 bg-red-50/20">
                  <Check className="w-4 h-4 text-emerald-600" /> NIST SP 800-88 &amp; DoD
                </td>
                <td className="p-4 sm:p-5 text-[#555B66]">Proprietary Hardware Box</td>
                <td className="p-4 sm:p-5 text-red-500 flex items-center gap-1.5">
                  <X className="w-4 h-4" /> Uncertified
                </td>
              </tr>
              <tr>
                <td className="p-4 sm:p-5 font-semibold text-[#101114]">Structure-Aware File Carving</td>
                <td className="p-4 sm:p-5 text-emerald-700 font-bold flex items-center gap-1.5 bg-red-50/20">
                  <Check className="w-4 h-4 text-emerald-600" /> Chunk &amp; Header Parsers
                </td>
                <td className="p-4 sm:p-5 text-[#555B66]">Separate $3K/yr License</td>
                <td className="p-4 sm:p-5 text-red-500 flex items-center gap-1.5">
                  <X className="w-4 h-4" /> Naive Magic Bytes Only
                </td>
              </tr>
              <tr>
                <td className="p-4 sm:p-5 font-semibold text-[#101114]">Entropy Heatmap Verification</td>
                <td className="p-4 sm:p-5 text-emerald-700 font-bold flex items-center gap-1.5 bg-red-50/20">
                  <Check className="w-4 h-4 text-emerald-600" /> Real-time 0.000-8.000 b/B
                </td>
                <td className="p-4 sm:p-5 text-[#555B66]">Static / Offline Analysis</td>
                <td className="p-4 sm:p-5 text-red-500 flex items-center gap-1.5">
                  <X className="w-4 h-4" /> None
                </td>
              </tr>
              <tr>
                <td className="p-4 sm:p-5 font-semibold text-[#101114]">Cryptographic Audit &amp; Timestamps</td>
                <td className="p-4 sm:p-5 text-emerald-700 font-bold flex items-center gap-1.5 bg-red-50/20">
                  <Check className="w-4 h-4 text-emerald-600" /> RFC 3161 Timestamp Support
                </td>
                <td className="p-4 sm:p-5 text-[#555B66]">PDF Stamp Only</td>
                <td className="p-4 sm:p-5 text-red-500 flex items-center gap-1.5">
                  <X className="w-4 h-4" /> None
                </td>
              </tr>
              <tr>
                <td className="p-4 sm:p-5 font-semibold text-[#101114]">Cost &amp; Licensing</td>
                <td className="p-4 sm:p-5 text-[#E31B23] font-black bg-red-50/20">100% Free &amp; Open-Source (MIT)</td>
                <td className="p-4 sm:p-5 text-[#555B66]">$3,000–$50,000 / Year</td>
                <td className="p-4 sm:p-5 text-[#555B66]">Free, but uncertified</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 7.5. PS 149 OPERATIONAL DEMONSTRATION VIDEOS                             */}
      {/* ========================================================================= */}
      <VideoGallery />

      {/* ========================================================================= */}
      {/* 8. HIGH-IMPACT BOTTOM CTA BANNER                                         */}
      {/* ========================================================================= */}
      <section className="max-w-[1536px] mx-auto px-4 sm:px-8 lg:px-12 pb-20">
        <div className="relative rounded-3xl bg-gradient-to-br from-[#11141F] via-[#161926] to-[#08090B] p-8 sm:p-12 lg:p-14 text-white overflow-hidden shadow-2xl border border-slate-800">
          {/* Subtle Ambient Red Glow */}
          <div className="absolute top-1/2 right-10 -translate-y-1/2 w-96 h-96 bg-red-600/15 rounded-full blur-3xl pointer-events-none" />

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-12 items-center relative z-10">
            {/* Left Column: Headline & Action Buttons (7 cols) */}
            <div className="lg:col-span-7 space-y-6 text-left">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-500/10 border border-red-500/30 text-[#E31B23] text-xs font-mono font-bold uppercase tracking-wider">
                <Shield className="w-3.5 h-3.5" />
                <span>Zero Trust Forensics Core</span>
              </div>

              <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black tracking-tight leading-[1.15] font-sans">
                Secure your data.<br />
                Recover critical evidence.<br />
                <span className="text-[#E31B23]">With defense-grade certainty.</span>
              </h2>

              <p className="text-slate-300 text-sm sm:text-base leading-relaxed max-w-xl">
                Start testing RAZE-CARVE in your browser immediately, or download our self-contained native binaries and bootable live rescue ISO.
              </p>

              <div className="flex flex-wrap items-center gap-4 pt-2">
                <button
                  onClick={() => onNavigate('/cli')}
                  className="inline-flex items-center gap-2 px-7 py-3.5 text-sm font-bold text-white bg-[#E31B23] hover:bg-[#c9141b] rounded-lg shadow-lg shadow-red-500/30 hover:shadow-red-500/50 transition-all cursor-pointer group"
                >
                  <span>Launch Live CLI &amp; Sandbox</span>
                  <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                </button>

                <button
                  onClick={() => onNavigate('/downloads')}
                  className="inline-flex items-center gap-2 px-7 py-3.5 text-sm font-bold text-white bg-white/10 hover:bg-white/20 border border-white/20 rounded-lg shadow-sm transition-all cursor-pointer"
                >
                  <Download className="w-4 h-4" />
                  <span>Get CLI &amp; Live ISO (1.7 GB)</span>
                </button>
              </div>

              {/* Security Badges */}
              <div className="flex flex-wrap items-center gap-6 pt-2 text-xs font-mono text-slate-400">
                <div className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-emerald-400" />
                  <span>NIST SP 800-88 Compliant</span>
                </div>
                <div className="flex items-center gap-2">
                  <Check className="w-4 h-4 text-emerald-400" />
                  <span>RFC 3161 Timestamped</span>
                </div>
              </div>
            </div>

            {/* Right Column: 3D Forensic Cryptographic Vault Image (5 cols) */}
            <div className="lg:col-span-5 relative flex items-center justify-center">
              <div className="relative w-full max-w-[480px] rounded-2xl overflow-hidden border border-slate-700/80 shadow-[0_20px_50px_rgba(0,0,0,0.5)] bg-slate-950 group">
                <img
                  src="/cta-vault.jpg"
                  alt="RAZE-CARVE Cryptographic Forensic Vault"
                  className="w-full h-auto object-cover transform group-hover:scale-105 transition-transform duration-500"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent pointer-events-none" />
                <div className="absolute bottom-3 left-3 right-3 flex items-center justify-between text-[10px] font-mono text-slate-300 bg-black/60 backdrop-blur-md px-3 py-1.5 rounded-lg border border-white/10">
                  <span className="flex items-center gap-1.5 font-bold text-white">
                    <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                    VAULT-01 / CRYPTOGRAPHIC ENGINE
                  </span>
                  <span className="text-red-400 font-bold">SEALED</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ========================================================================= */}
      {/* 9. OVERVIEW MODAL ("SEE HOW IT WORKS" POPUP)                             */}
      {/* ========================================================================= */}
      {showOverviewModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-xs animate-fade-in">
          <div className="relative w-full max-w-2xl rounded-2xl bg-white border border-gray-200 shadow-2xl p-6 sm:p-8 space-y-6 animate-fade-up text-left">
            <div className="flex items-center justify-between border-b border-gray-200 pb-4">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-red-50 text-[#E31B23]">
                  <Shield className="w-5 h-5" />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-[#101114]">RAZE-CARVE Forensic Platform</h3>
                  <p className="text-xs text-[#555B66]">Universal Forensic Data Sanitization &amp; File Carving System</p>
                </div>
              </div>
              <button
                onClick={() => setShowOverviewModal(false)}
                className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors cursor-pointer"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4 text-sm text-[#555B66] leading-relaxed">
              <p>
                <strong className="text-[#101114]">RAZE-CARVE</strong> is an integrated forensic toolkit engineered in memory-safe Rust to solve the modern storage sanitization and deep recovery crisis:
              </p>
              <ul className="space-y-2 text-xs sm:text-sm list-disc pl-5">
                <li>
                  <strong className="text-[#101114]">Storage Media Sanitization:</strong> Executes certified wiping algorithms compliant with NIST SP 800-88 and DoD standards.
                </li>
                <li>
                  <strong className="text-[#101114]">Structure-Aware Carving:</strong> Extracts valid files by parsing internal chunk hierarchies across PNG, PDF, ZIP, and SQLite.
                </li>
                <li>
                  <strong className="text-[#101114]">Shannon Entropy Verification:</strong> Quantifies sector randomness to verify cryptographic data destruction.
                </li>
                <li>
                  <strong className="text-[#101114]">Merkle Chain Auditing:</strong> Maintains an immutable append-only hashchain coupled with RFC 3161 digital timestamping.
                </li>
              </ul>
            </div>

            <div className="flex items-center justify-end gap-3 pt-4 border-t border-gray-200">
              <button
                onClick={() => setShowOverviewModal(false)}
                className="px-4 py-2 text-xs font-semibold text-[#555B66] hover:text-[#101114] transition-colors cursor-pointer"
              >
                Close
              </button>
              <button
                onClick={() => {
                  setShowOverviewModal(false);
                  onNavigate('/cli');
                }}
                className="px-5 py-2 text-xs font-bold text-white bg-[#E31B23] hover:bg-[#c9141b] rounded-lg shadow-sm transition-colors flex items-center gap-1.5 cursor-pointer"
              >
                <span>Launch Live CLI Terminal</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
