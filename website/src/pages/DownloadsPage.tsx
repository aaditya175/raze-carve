import React, { useState } from 'react';
import { RoutePath } from '../types';
import { DOWNLOADS_DATA } from '../data/downloadsData';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { CopyButton } from '../components/CopyButton';
import { CodeBlock } from '../components/CodeBlock';
import { Callout } from '../components/Callout';
import {
  Download,
  HardDrive,
  Terminal,
  Box,
  Code,
  CheckCircle,
  ShieldCheck,
  Shield,
  Check,
  ArrowRight,
  Cpu,
  Layers,
  Sparkles,
  Info,
  CheckCircle2,
  Lock,
  ExternalLink
} from 'lucide-react';

interface DownloadsPageProps {
  onNavigate: (path: RoutePath) => void;
  initialCategory?: string;
}

export const DownloadsPage: React.FC<DownloadsPageProps> = ({
  onNavigate,
  initialCategory = 'iso',
}) => {
  const [activeCategory, setActiveCategory] = useState<'iso' | 'docker' | 'binaries' | 'source'>(
    (initialCategory as any) || 'iso'
  );

  const activeArtifacts = DOWNLOADS_DATA.filter((d) => d.category === activeCategory);

  return (
    <div className="space-y-12 pb-16 font-sans">
      {/* =========================================================================
          1. HERO SECTION (CLEAN EDITORIAL LIGHT THEME MATCHING HOMEPAGE & FEATURES)
         ========================================================================= */}
      <section className="relative w-full bg-white border-b border-gray-200/80 pb-10 sm:pb-12">
        <div className="max-w-6xl mx-auto space-y-6">
          {/* Breadcrumbs Navigation */}
          <div className="flex items-center gap-2 text-xs font-mono text-slate-500">
            <span className="hover:text-slate-800 cursor-pointer" onClick={() => onNavigate('/')}>
              Product
            </span>
            <span>/</span>
            <span className="text-[#E31B23] font-bold">Downloads Hub</span>
          </div>

          {/* Technical Eyebrow matching HomePage */}
          <div className="flex items-center gap-2.5">
            <span className="w-6 h-[2.5px] bg-[#E31B23] inline-block rounded-full" />
            <span className="text-[11px] sm:text-xs font-bold tracking-[0.16em] text-[#555B66] uppercase font-mono">
              OFFICIAL PRODUCTION RELEASES &nbsp;/&nbsp; AIR-GAPPED MEDIA &nbsp;/&nbsp; v0.1.0
            </span>
          </div>

          {/* Headline */}
          <div className="space-y-1">
            <h1 className="text-3xl sm:text-5xl lg:text-5xl font-black text-[#101114] tracking-tight leading-[1.08]">
              Release Distribution Hub.
            </h1>
            <p className="text-xl sm:text-2xl font-bold text-[#E31B23] tracking-tight">
              Cryptographically verified media for air-gapped forensic operations.
            </p>
          </div>

          {/* Subtitle */}
          <p className="text-base sm:text-lg text-[#555B66] leading-relaxed max-w-3xl font-sans">
            Download authenticated release packages for air-gapped bare-metal operations, containerized environments, or forensic host servers. Dual-mode UEFI &amp; BIOS bootable media with complete storage controller driver stacks.
          </p>

          {/* Clean Information Pillars (Light-Mode Cards) */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
            <div className="p-4 rounded-xl bg-slate-50 border border-slate-200/80 space-y-1.5 hover:border-slate-300 transition-colors">
              <div className="flex items-center gap-2 text-xs font-mono font-bold text-slate-900">
                <HardDrive className="w-4 h-4 text-[#E31B23] shrink-0" />
                <span>Air-Gapped Live ISO</span>
              </div>
              <p className="text-xs text-[#555B66] leading-relaxed font-sans">
                Boots in &lt;5s directly into volatile RAM with automatic root kiosk login and forensic write-blocking.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-slate-50 border border-slate-200/80 space-y-1.5 hover:border-slate-300 transition-colors">
              <div className="flex items-center gap-2 text-xs font-mono font-bold text-slate-900">
                <Box className="w-4 h-4 text-[#E31B23] shrink-0" />
                <span>Docker Isolation</span>
              </div>
              <p className="text-xs text-[#555B66] leading-relaxed font-sans">
                Direct device pass-through (<code className="text-slate-800 font-mono font-bold bg-white px-1 py-0.5 rounded border border-slate-200">/dev</code>) without installing host dependencies.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-slate-50 border border-slate-200/80 space-y-1.5 hover:border-slate-300 transition-colors">
              <div className="flex items-center gap-2 text-xs font-mono font-bold text-slate-900">
                <ShieldCheck className="w-4 h-4 text-[#E31B23] shrink-0" />
                <span>NIST &amp; DoD Compliant</span>
              </div>
              <p className="text-xs text-[#555B66] leading-relaxed font-sans">
                Validated against NIST SP 800-88 Rev 1, DoD 5220.22-M, and RFC 3161 cryptographic audit timestamping.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* =========================================================================
          2. DOWNLOADS HUB SECTION
         ========================================================================= */}
      <section className="max-w-6xl mx-auto space-y-8">
        
        {/* Navigation Category Tabs */}
        <div className="flex items-center gap-2 border-b border-slate-200 dark:border-slate-800 pb-2 text-xs sm:text-sm font-mono overflow-x-auto whitespace-nowrap">
          <button
            onClick={() => setActiveCategory('iso')}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all cursor-pointer font-bold ${
              activeCategory === 'iso'
                ? 'bg-[#E31B23] text-white shadow-sm'
                : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
            }`}
          >
            <HardDrive className="w-4 h-4" />
            <span>1. Bootable Live ISO (1.7 GB)</span>
          </button>

          <button
            onClick={() => setActiveCategory('docker')}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all cursor-pointer font-bold ${
              activeCategory === 'docker'
                ? 'bg-[#E31B23] text-white shadow-sm'
                : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
            }`}
          >
            <Box className="w-4 h-4" />
            <span>2. Docker Container</span>
          </button>

          <button
            onClick={() => setActiveCategory('binaries')}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all cursor-pointer font-bold ${
              activeCategory === 'binaries'
                ? 'bg-[#E31B23] text-white shadow-sm'
                : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
            }`}
          >
            <Terminal className="w-4 h-4" />
            <span>3. Linux x86_64 Binaries</span>
          </button>

          <button
            onClick={() => setActiveCategory('source')}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all cursor-pointer font-bold ${
              activeCategory === 'source'
                ? 'bg-[#E31B23] text-white shadow-sm'
                : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
            }`}
          >
            <Code className="w-4 h-4" />
            <span>4. Source Code</span>
          </button>
        </div>

        {/* =========================================================================
            ARTIFACT CARDS LIST
           ========================================================================= */}
        <div className="space-y-6">
          {activeArtifacts.map((artifact) => {
            const isIso = artifact.category === 'iso';
            const isBinary = artifact.category === 'binaries';
            const isDocker = artifact.category === 'docker';

            return (
              <div
                key={artifact.id}
                className="p-6 sm:p-8 rounded-2xl border border-slate-200 bg-white shadow-sm space-y-6"
              >
                {/* Header Row */}
                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6 border-b border-slate-100 pb-6">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-mono font-bold text-[#E31B23] uppercase tracking-wider">
                        Version {artifact.version} • {artifact.size}
                      </span>
                      {isIso && (
                        <span className="px-2 py-0.5 rounded-md bg-emerald-50 text-emerald-700 border border-emerald-200 text-[10px] font-mono font-bold">
                          RECOMMENDED FOR FIELD TRIAGE
                        </span>
                      )}
                      {isBinary && (
                        <span className="px-2 py-0.5 rounded-md bg-slate-100 text-slate-700 border border-slate-200 text-[10px] font-mono font-bold">
                          COMPILED ELF BINARY
                        </span>
                      )}
                    </div>
                    <h2 className="text-xl sm:text-2xl font-black text-slate-900 font-sans tracking-tight">
                      {artifact.title}
                    </h2>
                    <div className="text-xs font-mono text-slate-500 break-all">
                      {artifact.filename}
                    </div>
                  </div>

                  {/* Actions Area */}
                  <div className="flex flex-col sm:flex-row sm:items-center gap-3 shrink-0">
                    {/* KEEP THE DOWNLOAD BUTTON ONLY FOR BOOTABLE LIVE ISO */}
                    {isIso && artifact.downloadUrl && (
                      <a
                        href={artifact.downloadUrl}
                        download={artifact.filename}
                        target={artifact.downloadUrl.startsWith('http') ? '_blank' : undefined}
                        rel={artifact.downloadUrl.startsWith('http') ? 'noopener noreferrer' : undefined}
                        className="inline-flex items-center justify-center gap-2 px-6 py-3.5 text-xs sm:text-sm font-bold text-white bg-[#E31B23] hover:bg-[#c9141b] rounded-xl shadow-md transition-all active:scale-95 cursor-pointer w-full sm:w-auto"
                      >
                        <Download className="w-4 h-4" />
                        <span>Download ISO Package ({artifact.size})</span>
                      </a>
                    )}

                    {/* FOR BINARIES: AS REQUESTED, REMOVE ALL DOWNLOAD BINARY BUTTONS */}
                    {isBinary && (
                      <div className="inline-flex items-center gap-2 px-4 py-2.5 text-xs font-mono font-bold text-slate-700 bg-slate-100 rounded-xl border border-slate-200">
                        <CheckCircle2 className="w-4 h-4 text-emerald-600" />
                        <span>Pre-packaged in Live ISO &amp; Docker</span>
                      </div>
                    )}

                    {/* FOR DOCKER: COPY COMMAND */}
                    {isDocker && (
                      <div className="flex items-center gap-2">
                        <CopyButton
                          text={artifact.command || 'docker pull normie69k/secureforge:latest'}
                          label="Copy Run Command"
                        />
                      </div>
                    )}

                    {/* INTEGRITY BADGE */}
                    <span className="inline-flex items-center justify-center gap-1.5 px-3 py-2 text-xs font-mono font-semibold rounded-xl bg-emerald-50 text-emerald-700 border border-emerald-200 w-full sm:w-auto">
                      <ShieldCheck className="w-4 h-4 text-emerald-600" />
                      <span>Integrity Verified</span>
                    </span>
                  </div>
                </div>

                {/* Description */}
                <p className="text-sm sm:text-base text-slate-600 leading-relaxed font-sans">
                  {artifact.description}
                </p>

                {/* SHA-256 Checksum block */}
                {artifact.sha256 && (
                  <div className="p-4 rounded-xl border border-slate-200 bg-slate-50 text-xs font-mono space-y-2">
                    <div className="flex items-center justify-between text-slate-500">
                      <span className="font-bold">AUTHENTIC CRYPTOGRAPHIC SHA-256 DIGEST</span>
                      <CopyButton text={artifact.sha256} label="Copy Hash" />
                    </div>
                    <div className="break-all select-all font-semibold text-slate-800 bg-white p-2 rounded-lg border border-slate-200">
                      {artifact.sha256}
                    </div>
                  </div>
                )}

                {/* Execution / Flashing Command */}
                {artifact.command && (
                  <div>
                    <div className="text-xs font-bold uppercase tracking-wider text-slate-500 mb-2 font-mono">
                      {isIso ? 'Flashing / DD Command' : 'Execution Syntax'}
                    </div>
                    <CodeBlock code={artifact.command} language="bash" />
                  </div>
                )}

                {/* Deployment Guidelines */}
                <div className="space-y-2 pt-1">
                  <div className="text-xs font-bold uppercase tracking-wider text-slate-500 font-mono">
                    Operational Guidelines
                  </div>
                  <ul className="space-y-1.5 text-xs text-slate-700 font-sans">
                    {artifact.instructions.map((inst, idx) => (
                      <li key={idx} className="flex items-start gap-2">
                        <CheckCircle className="w-3.5 h-3.5 text-emerald-500 shrink-0 mt-0.5" />
                        <span>{inst}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            );
          })}
        </div>

        {/* =========================================================================
            3. USB FLASHING GUIDES (VISIBLE WHEN ISO SELECTED)
           ========================================================================= */}
        {activeCategory === 'iso' && (
          <div className="p-6 sm:p-8 rounded-2xl border border-slate-200 bg-slate-50 space-y-6">
            <div>
              <span className="text-xs font-mono font-bold text-[#E31B23] uppercase tracking-wider">
                STEP-BY-STEP MEDIA PREPARATION
              </span>
              <h3 className="text-xl font-bold text-slate-900 mt-1 font-sans">
                Flashing Guides for USB Thumb Drives &amp; Forensic Media
              </h3>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-sans">
              <div className="p-4 rounded-xl bg-white border border-slate-200 space-y-2">
                <div className="font-bold text-slate-900 flex items-center gap-2">
                  <span className="w-5 h-5 rounded-full bg-slate-100 flex items-center justify-center font-mono text-[11px]">A</span>
                  <span>Linux / macOS Terminal (dd)</span>
                </div>
                <p className="text-slate-600">
                  Write directly to raw block storage device without cache writeback:
                </p>
                <div className="bg-slate-950 text-emerald-400 p-2.5 rounded-lg font-mono text-[11px] overflow-x-auto">
                  sudo dd if=raze-carve-live-0.1.0-x86_64.iso of=/dev/sdX bs=4M status=progress conv=fsync
                </div>
              </div>

              <div className="p-4 rounded-xl bg-white border border-slate-200 space-y-2">
                <div className="font-bold text-slate-900 flex items-center gap-2">
                  <span className="w-5 h-5 rounded-full bg-slate-100 flex items-center justify-center font-mono text-[11px]">B</span>
                  <span>Multi-Boot Drive (Ventoy)</span>
                </div>
                <p className="text-slate-600">
                  Keep multiple ISOs on a single forensic thumbdrive with zero reformatting:
                </p>
                <div className="bg-slate-950 text-emerald-400 p-2.5 rounded-lg font-mono text-[11px] overflow-x-auto">
                  cp raze-carve-live-0.1.0-x86_64.iso /media/ventoy/
                </div>
              </div>

              <div className="p-4 rounded-xl bg-white border border-slate-200 space-y-2">
                <div className="font-bold text-slate-900 flex items-center gap-2">
                  <span className="w-5 h-5 rounded-full bg-slate-100 flex items-center justify-center font-mono text-[11px]">C</span>
                  <span>Windows (Rufus)</span>
                </div>
                <p className="text-slate-600">
                  Open Rufus, choose the ISO, select <strong>GPT / UEFI</strong>, and pick <strong>DD Image Mode</strong> when prompted.
                </p>
              </div>

              <div className="p-4 rounded-xl bg-white border border-slate-200 space-y-2">
                <div className="font-bold text-slate-900 flex items-center gap-2">
                  <span className="w-5 h-5 rounded-full bg-slate-100 flex items-center justify-center font-mono text-[11px]">D</span>
                  <span>Cross-Platform (BalenaEtcher)</span>
                </div>
                <p className="text-slate-600">
                  Launch BalenaEtcher on Windows/Mac, select <strong>Flash from file</strong>, pick the ISO, select your target drive, and click Flash!
                </p>
              </div>
            </div>
          </div>
        )}

      </section>
    </div>
  );
};
