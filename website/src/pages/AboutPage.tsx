import React from 'react';
import { RoutePath } from '../types';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { Shield, Lock, Terminal, Cpu, CheckCircle } from 'lucide-react';

interface AboutPageProps {
  onNavigate: (path: RoutePath) => void;
}

export const AboutPage: React.FC<AboutPageProps> = () => {
  return (
    <div className="max-w-4xl mx-auto py-6 space-y-8">
      <div>
        <Breadcrumbs crumbs={[{ label: 'Company' }, { label: 'About & Principles' }]} />
        <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono mt-2">
          Engineering Mission & Core Principles
        </h1>
        <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-2">
          Why RAZE-CARVE was created and the non-negotiable engineering principles governing its development.
        </p>
      </div>

      <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
        <section className="p-6 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-3">
          <h2 className="text-lg font-bold text-slate-900 dark:text-slate-100 font-mono">
            The Problem: The Failure of Software Overwriting on Flash Storage
          </h2>
          <p>
            For decades, data sanitization was straightforward: software simply wrote zeroes across every sector of a spinning magnetic disk. However, with the rise of modern solid-state drives (SSDs), NVMe controllers, eMMC, and UFS chips, traditional software overwriting became unreliable.
          </p>
          <p>
            Flash Translation Layers (FTLs) manage wear-leveling, overprovisioned reserve blocks, and dynamic cell retirement behind the scenes. When a standard utility writes to LBA 0 through 1,000,000, the controller can remap those writes to fresh blocks while leaving historical data sitting untouched in retired or spare NAND cells.
          </p>
          <p>
            RAZE-CARVE was engineered to eliminate this gap by coupling direct low-level microcontroller sanitization (NVMe Sanitize / ATA Secure Erase) with rigorous Shannon entropy verification and zero-copy forensic file carving.
          </p>
        </section>

        <section className="p-6 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-4">
          <h2 className="text-lg font-bold text-slate-900 dark:text-slate-100 font-mono">
            Our 5 Engineering Tenets
          </h2>

          <div className="space-y-3 text-xs sm:text-sm">
            <div className="flex items-start gap-3">
              <CheckCircle className="w-4 h-4 text-brand-500 shrink-0 mt-0.5" />
              <div>
                <strong>Never Assume Correctness</strong>: Every critical assertion is verified with evidence. Write return codes from the kernel are corroborated with post-sanitization entropy reads.
              </div>
            </div>

            <div className="flex items-start gap-3">
              <CheckCircle className="w-4 h-4 text-brand-500 shrink-0 mt-0.5" />
              <div>
                <strong>Air-Gapped by Design</strong>: Zero cloud connections, zero telemetry, zero analytics, zero external dependencies. The software must function reliably in an RF-shielded SCIF or forensic cleanroom.
              </div>
            </div>

            <div className="flex items-start gap-3">
              <CheckCircle className="w-4 h-4 text-brand-500 shrink-0 mt-0.5" />
              <div>
                <strong>Memory Safety & Direct I/O</strong>: Written in Rust to eliminate memory corruption vectors (buffer overflows, use-after-free) while executing deterministic direct POSIX I/O without allocations in tight hot loops.
              </div>
            </div>

            <div className="flex items-start gap-3">
              <CheckCircle className="w-4 h-4 text-brand-500 shrink-0 mt-0.5" />
              <div>
                <strong>Defense-in-Depth File Shredding</strong>: Strict canonicalization and prefix containment, symlink rejection (<code className="font-mono text-xs">O_NOFOLLOW</code>), synchronous flushing, and random directory journal renaming.
              </div>
            </div>

            <div className="flex items-start gap-3">
              <CheckCircle className="w-4 h-4 text-brand-500 shrink-0 mt-0.5" />
              <div>
                <strong>Authentic Open Source Integrity</strong>: Fully inspectable source code, deterministic builds, and reproducible ISO compilation.
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
};
