import React from 'react';
import { RoutePath } from '../types';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { Badge } from '../components/Badge';
import { CheckCircle2, GitCommit, Calendar } from 'lucide-react';

interface ChangelogPageProps {
  onNavigate: (path: RoutePath) => void;
}

export const ChangelogPage: React.FC<ChangelogPageProps> = () => {
  return (
    <div className="max-w-4xl mx-auto py-6 space-y-8">
      <div>
        <Breadcrumbs crumbs={[{ label: 'Resources' }, { label: 'Changelog' }]} />
        <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono mt-2">
          Project Changelog & Release Notes
        </h1>
        <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-2">
          Documented changes, cryptographic improvements, and architectural milestones for RAZE-CARVE.
        </p>
      </div>

      <div className="space-y-8 border-l-2 border-slate-200 dark:border-slate-800 pl-6 sm:pl-8 ml-2">
        {/* Version 0.1.0 */}
        <article className="relative space-y-4">
          <div className="absolute -left-[31px] sm:-left-[39px] top-0.5 w-4 h-4 rounded-full bg-brand-500 border-4 border-white dark:border-slate-950"></div>

          <div className="flex flex-wrap items-center gap-3">
            <h2 className="text-xl font-bold font-mono text-slate-900 dark:text-slate-100">
              v0.1.0 — Initial Foundation Release
            </h2>
            <Badge variant="brand" size="sm">Current Stable</Badge>
            <span className="flex items-center gap-1 text-xs text-slate-500 font-mono">
              <Calendar className="w-3.5 h-3.5" />
              <span>September 2026</span>
            </span>
          </div>

          <div className="space-y-4 text-sm text-slate-700 dark:text-slate-300">
            <div>
              <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-500 font-mono mb-2">
                Core Capabilities Added
              </h3>
              <ul className="space-y-2 text-xs sm:text-sm">
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>NIST SP 800-88 Rev 1 & DoD 5220.22-M Engine</strong>: Multi-pass algorithmic overwriting engine implemented with pure Rust POSIX raw block interfaces (<code className="font-mono text-xs">O_DIRECT</code> / <code className="font-mono text-xs">O_SYNC</code>).</span>
                </li>
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>Mathematical Shannon Entropy Verification</strong>: Real-time bit distribution analysis to mathematically detect residual non-zero sectors post-sanitization.</span>
                </li>
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>NVMe Sanitize & ATA Secure Erase</strong>: Microcontroller-level Crypto Scramble and Block Erase commands for modern SSDs.</span>
                </li>
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>Signature-Based File Carver</strong>: Circular buffer scanner supporting 24+ file types with extensible TOML plugin rules.</span>
                </li>
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>Tri-Interface Distribution</strong>: Interactive Ratatui terminal (<code className="font-mono text-xs">sforge</code>), headless CLI (<code className="font-mono text-xs">sih149</code>), and web/desktop GUI (<code className="font-mono text-xs">raze-carve-desktop</code>).</span>
                </li>
                <li className="flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0 mt-0.5" />
                  <span><strong>Universal Live Bootable ISO</strong>: Air-gapped Arch Linux LTS bootable image with hybrid UEFI/BIOS support and automated tty1 login.</span>
                </li>
              </ul>
            </div>
          </div>
        </article>
      </div>
    </div>
  );
};
