import React from 'react';
import { RoutePath } from '../types';
import { Shield, ExternalLink, Heart } from 'lucide-react';

interface FooterProps {
  onNavigate: (path: RoutePath) => void;
}

export const Footer: React.FC<FooterProps> = ({ onNavigate }) => {
  return (
    <footer className="border-t border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 text-slate-600 dark:text-slate-400 text-xs">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          {/* Brand Col */}
          <div className="col-span-1 sm:col-span-2 space-y-3">
            <div className="flex items-center gap-2">
              <img
                src="/logo.png"
                alt="RAZE-CARVE"
                className="h-7 w-auto object-contain dark:hidden"
              />
              <img
                src="/logo-dark.png"
                alt="RAZE-CARVE"
                className="h-7 w-auto object-contain hidden dark:block"
              />
            </div>
            <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed max-w-sm">
              Defense-grade media sanitization, hardware firmware controller erasure, autonomous file carving, and cryptographic entropy audit verification.
            </p>
            <div className="text-[11px] font-mono text-slate-400 dark:text-slate-500">
              Complies with NIST SP 800-88 Rev 1 & DoD 5220.22-M
            </div>
            <div className="pt-1">
              <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-mono font-medium bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20">
                <Heart className="w-3 h-3 text-red-500 fill-red-500" />
                Made for SIH 2026 • Smart India Hackathon
              </span>
            </div>
          </div>

          {/* Product */}
          <div>
            <div className="font-semibold text-slate-900 dark:text-slate-200 uppercase tracking-wider text-[11px] mb-3">
              Product
            </div>
            <ul className="space-y-2">
              <li>
                <button onClick={() => onNavigate('/features')} className="hover:text-brand-500 transition-colors">
                  Capabilities
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/docs')} className="hover:text-brand-500 transition-colors">
                  Documentation
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/commands')} className="hover:text-brand-500 transition-colors">
                  Command Reference
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/downloads')} className="hover:text-brand-500 transition-colors">
                  Downloads Hub
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/examples')} className="hover:text-brand-500 transition-colors">
                  Workflows
                </button>
              </li>
            </ul>
          </div>

          {/* Legal */}
          <div>
            <div className="font-semibold text-slate-900 dark:text-slate-200 uppercase tracking-wider text-[11px] mb-3">
              Legal & Trust
            </div>
            <ul className="space-y-2">
              <li>
                <button onClick={() => onNavigate('/privacy')} className="hover:text-brand-500 transition-colors">
                  Privacy Policy
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/terms')} className="hover:text-brand-500 transition-colors">
                  Terms of Service
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/cookies')} className="hover:text-brand-500 transition-colors">
                  Cookie Statement
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/security')} className="hover:text-brand-500 transition-colors">
                  Security Disclosure
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/accessibility')} className="hover:text-brand-500 transition-colors">
                  Accessibility
                </button>
              </li>
            </ul>
          </div>

          {/* Open Source */}
          <div>
            <div className="font-semibold text-slate-900 dark:text-slate-200 uppercase tracking-wider text-[11px] mb-3">
              Developer
            </div>
            <ul className="space-y-2">
              <li>
                <a
                  href="https://github.com/aaditya175/raze-carve"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-brand-500 flex items-center gap-1"
                >
                  <span>GitHub Repository</span>
                  <ExternalLink className="w-3 h-3 text-slate-400" />
                </a>
              </li>
              <li>
                <button onClick={() => onNavigate('/changelog')} className="hover:text-brand-500 transition-colors">
                  Changelog (v0.1.0)
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate('/about')} className="hover:text-brand-500 transition-colors">
                  Architecture & Design
                </button>
              </li>
              <li>
                <a
                  href="https://github.com/aaditya175/raze-carve/issues"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-brand-500 flex items-center gap-1"
                >
                  <span>Issue Tracker</span>
                  <ExternalLink className="w-3 h-3 text-slate-400" />
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="pt-6 border-t border-slate-200 dark:border-slate-850 flex flex-col sm:flex-row items-center justify-between gap-4 text-slate-500 text-[11px]">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2">
            <span>© 2026 RAZE-CARVE Engineering Team. Released under Apache-2.0 / MIT licenses.</span>
            <span className="hidden sm:inline text-slate-400">•</span>
            <span className="inline-flex items-center gap-1 font-semibold text-red-600 dark:text-red-400">
              <Heart className="w-3 h-3 text-red-500 fill-red-500 inline" />
              Made for SIH 2026 (Smart India Hackathon)
            </span>
          </div>
          <div className="flex items-center gap-4">
            <span>Air-Gapped Verified</span>
            <span>•</span>
            <span>Zero Third-Party Telemetry</span>
            <span>•</span>
            <span>Strict POSIX Direct I/O</span>
          </div>
        </div>
      </div>
    </footer>
  );
};
