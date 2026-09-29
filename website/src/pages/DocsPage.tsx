import React, { useState, useEffect } from 'react';
import { RoutePath } from '../types';
import { DOCS_DATA } from '../data/docsData';
import { CodeBlock } from '../components/CodeBlock';
import {
  Home,
  ChevronRight,
  ArrowLeft,
  ArrowRight,
  Shield,
  Cpu,
  FileText,
  Search,
  BookOpen,
  Menu,
  X,
  CheckCircle2,
  Terminal,
  Layers,
  HardDrive
} from 'lucide-react';

interface DocsPageProps {
  onNavigate: (path: RoutePath) => void;
  initialSectionId?: string;
}

export const DocsPage: React.FC<DocsPageProps> = ({ onNavigate, initialSectionId = 'intro' }) => {
  const [activeSectionId, setActiveSectionId] = useState<string>(initialSectionId || 'intro');
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [activeHeadingId, setActiveHeadingId] = useState<string>('introduction');

  useEffect(() => {
    if (initialSectionId) {
      setActiveSectionId(initialSectionId);
    }
  }, [initialSectionId]);

  const activeIndex = DOCS_DATA.findIndex((d) => d.id === activeSectionId);
  const currentDoc = DOCS_DATA[activeIndex >= 0 ? activeIndex : 0] || DOCS_DATA[0];
  const prevDoc = activeIndex > 0 ? DOCS_DATA[activeIndex - 1] : null;
  const nextDoc = activeIndex < DOCS_DATA.length - 1 ? DOCS_DATA[activeIndex + 1] : null;

  // Categories grouping
  const categories = [
    'Getting Started',
    'Guides',
    'Reference',
    'Development',
    'Troubleshooting',
  ] as const;

  // Headings for "ON THIS PAGE" for intro section
  const introHeadings = [
    { id: 'introduction', title: 'Introduction' },
    { id: 'architectural-tenets', title: 'Architectural Tenets' },
    { id: 'system-architecture', title: 'System Architecture' },
    { id: 'verification-integrity', title: 'Verification & Integrity' },
    { id: 'next-steps', title: 'Next Steps' },
  ];

  // Tenets Data for Intro Page
  const tenets = [
    {
      num: '01',
      title: 'Zero-Trust Verification',
      description:
        'Overwriting blocks is never assumed complete. RAZE-CARVE samples post-wipe and computes Shannon Entropy to verify that no residual patterns remain.',
      tags: 'SHANNON ENTROPY   •   SECTOR SAMPLING   •   VERIFIABLE ERASE',
      icon: <Shield className="w-5 h-5 text-slate-800" strokeWidth={1.5} />,
    },
    {
      num: '02',
      title: 'Controller-Direct Invocations',
      description:
        'Bypasses the OS filesystem caching and software translation layer to issue native NVMe Sanitize and ATA Secure Erase commands directly to device controllers.',
      tags: 'O_DIRECT   •   O_SYNC   •   NATIVE COMMANDS',
      icon: <Cpu className="w-5 h-5 text-slate-800" strokeWidth={1.5} />,
    },
    {
      num: '03',
      title: 'Defense-in-Depth File Shredding',
      description:
        'Employs synchronous I/O, strict symlink protection, and random 8-pass directory entry renaming to prevent TOCTOU attacks and ensure metadata destruction.',
      tags: 'TOCTOU PROTECTION   •   SECURE DELETION   •   METADATA SCRUB',
      icon: <FileText className="w-5 h-5 text-slate-800" strokeWidth={1.5} />,
    },
    {
      num: '04',
      title: 'Autonomous Signature Carving',
      description:
        'Features a zero-copy, high-performance scanner capable of salvaging orphaned fragments across 24+ common file types, even on corrupt or unpartitioned disks.',
      tags: 'ZERO-COPY SCANNER   •   24+ FILE TYPES   •   STRUCTURE-AWARE',
      icon: <Search className="w-5 h-5 text-slate-800" strokeWidth={1.5} />,
    },
  ];

  // Scroll to heading smoothly
  const scrollToHeading = (id: string) => {
    setActiveHeadingId(id);
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  // General markdown-like renderer for other docs sections
  const renderMarkdownContent = (rawText: string) => {
    const lines = rawText.split('\n');
    const elements: React.ReactNode[] = [];
    let inCodeBlock = false;
    let codeBuffer: string[] = [];
    let codeLanguage = 'bash';

    lines.forEach((line, index) => {
      if (line.startsWith('```')) {
        if (!inCodeBlock) {
          inCodeBlock = true;
          codeLanguage = line.replace('```', '').trim() || 'bash';
          codeBuffer = [];
        } else {
          inCodeBlock = false;
          elements.push(
            <CodeBlock key={`code-${index}`} code={codeBuffer.join('\n')} language={codeLanguage} />
          );
        }
        return;
      }

      if (inCodeBlock) {
        codeBuffer.push(line);
        return;
      }

      if (line.startsWith('### ')) {
        const text = line.replace('### ', '');
        const id = text.toLowerCase().replace(/[^a-z0-9]+/g, '-');
        elements.push(
          <h3
            key={index}
            id={id}
            className="text-base sm:text-lg font-bold text-slate-900 font-sans mt-7 mb-3 scroll-mt-24 flex items-center gap-2"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-[#E31B23]" />
            {text}
          </h3>
        );
      } else if (line.startsWith('## ')) {
        const text = line.replace('## ', '');
        const id = text.toLowerCase().replace(/[^a-z0-9]+/g, '-');
        elements.push(
          <h2
            key={index}
            id={id}
            className="text-xl sm:text-2xl font-bold text-slate-900 tracking-tight mt-10 mb-4 scroll-mt-24 border-b border-slate-100 pb-2"
          >
            {text}
          </h2>
        );
      } else if (line.startsWith('- ')) {
        elements.push(
          <li key={index} className="ml-5 list-disc text-slate-600 text-sm my-1.5 leading-relaxed">
            {line.replace('- ', '')}
          </li>
        );
      } else if (line.trim().startsWith('|')) {
        elements.push(
          <div key={index} className="font-mono text-xs text-slate-700 py-1 overflow-x-auto">
            {line}
          </div>
        );
      } else if (line.trim().length > 0) {
        elements.push(
          <p key={index} className="text-sm sm:text-[15px] text-slate-600 leading-relaxed my-3 font-sans">
            {line}
          </p>
        );
      }
    });

    return elements;
  };

  return (
    <div className="min-h-screen bg-white text-slate-900">
      {/* Explicit CSS to guarantee 3-column side-by-side grid without relying on Tailwind breakpoints */}
      <style>{`
        .sf-docs-grid {
          display: grid;
          grid-template-columns: 260px minmax(0, 1fr) 220px;
          gap: 2.5rem;
          align-items: start;
        }
        @media (max-width: 1280px) {
          .sf-docs-grid {
            grid-template-columns: 260px minmax(0, 1fr);
            gap: 2rem;
          }
          .sf-docs-toc {
            display: none !important;
          }
        }
        @media (max-width: 860px) {
          .sf-docs-grid {
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
          }
          .sf-docs-sidebar {
            display: none;
          }
          .sf-docs-sidebar.mobile-open {
            display: block !important;
            position: fixed;
            inset: 0;
            top: 4rem;
            z-index: 50;
            background: white;
            padding: 1.5rem;
            overflow-y: auto;
          }
        }
      `}</style>

      {/* Mobile Top Header / Drawer Toggle */}
      <div className="md:hidden flex items-center justify-between px-4 py-3 border-b border-slate-200 bg-slate-50 sticky top-16 z-20">
        <div className="flex items-center gap-2 text-xs font-medium text-slate-700">
          <BookOpen className="w-4 h-4 text-[#E31B23]" />
          <span className="text-slate-400">/</span>
          <span className="font-semibold text-slate-900 truncate max-w-[200px]">
            {currentDoc.title}
          </span>
        </div>
        <button
          onClick={() => setMobileNavOpen(!mobileNavOpen)}
          className="px-3 py-1.5 rounded-lg border border-slate-200 bg-white text-xs font-mono font-medium text-slate-700 shadow-xs flex items-center gap-1.5"
        >
          {mobileNavOpen ? <X className="w-3.5 h-3.5" /> : <Menu className="w-3.5 h-3.5" />}
          <span>{mobileNavOpen ? 'Close' : 'Menu'}</span>
        </button>
      </div>

      {/* Main Container - 3 Column Layout */}
      <div className="max-w-[1536px] mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
        <div className="sf-docs-grid">
          
          {/* =========================================================================
              LEFT COLUMN: Sidebar Navigation (~260px)
             ========================================================================= */}
          <aside
            className={`sf-docs-sidebar ${mobileNavOpen ? 'mobile-open' : ''} shrink-0 sticky top-24 max-h-[calc(100vh-7rem)] overflow-y-auto pr-2`}
            style={{ minWidth: 260, maxWidth: 260 }}
          >
            {/* Sidebar Title */}
            <div className="text-xs font-mono font-bold tracking-wider text-slate-900 pb-4 border-b border-slate-100 uppercase">
              DOCUMENTATION
            </div>

            {/* Categorized Navigation */}
            <div className="space-y-6 pt-4">
              {categories.map((cat) => {
                const catDocs = DOCS_DATA.filter((d) => d.category === cat);
                if (catDocs.length === 0) return null;

                return (
                  <div key={cat} className="space-y-1">
                    {/* Category Header */}
                    <div className="text-[11px] font-mono font-semibold tracking-wider text-slate-400 uppercase py-1">
                      {cat}
                    </div>

                    {/* Section Links */}
                    <ul className="space-y-0.5">
                      {catDocs.map((doc) => {
                        const isActive = activeSectionId === doc.id;
                        return (
                          <li key={doc.id}>
                            <button
                              onClick={() => {
                                setActiveSectionId(doc.id);
                                setMobileNavOpen(false);
                                window.scrollTo({ top: 0, behavior: 'smooth' });
                              }}
                              className={`w-full text-left px-3 py-1.5 rounded-lg text-[13px] sm:text-[13.5px] transition-colors duration-150 flex items-center relative ${
                                isActive
                                  ? 'bg-[#FFF1F2] text-[#E31B23] font-medium'
                                  : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50 font-normal'
                              }`}
                            >
                              {/* Red Indicator Pill on the Left for Active Item */}
                              {isActive && (
                                <span className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-4 bg-[#E31B23] rounded-r-full" />
                              )}
                              <span className="truncate">{doc.title}</span>
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  </div>
                );
              })}
            </div>

            {/* Bottom Tagline on Left Sidebar */}
            <div className="pt-10 pb-4 text-[11px] font-mono text-slate-400 tracking-wider space-y-0.5">
              <div>SECURE DATA.</div>
              <div>STRONGER TRUTH.</div>
            </div>
          </aside>

          {/* =========================================================================
              CENTER COLUMN: Main Documentation Reading Pane
             ========================================================================= */}
          <main className="min-w-0" style={{ maxWidth: 880 }}>
            {/* Top Bar: Breadcrumbs (Left) & Motto (Right) */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-6 border-b border-slate-100">
              {/* Breadcrumb Hierarchy */}
              <div className="flex items-center gap-2 text-xs font-mono text-slate-500 flex-wrap">
                <button
                  onClick={() => onNavigate('/')}
                  className="hover:text-slate-900 flex items-center gap-1 transition-colors"
                >
                  <Home className="w-3.5 h-3.5 text-slate-400" />
                </button>
                <ChevronRight className="w-3 h-3 text-slate-300" />
                <button
                  onClick={() => {
                    setActiveSectionId('intro');
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                  }}
                  className="hover:text-slate-900 transition-colors"
                >
                  Documentation
                </button>
                <ChevronRight className="w-3 h-3 text-slate-300" />
                <span className="text-slate-500">{currentDoc.category}</span>
                <ChevronRight className="w-3 h-3 text-slate-300" />
                <span className="text-slate-900 font-medium truncate">{currentDoc.title}</span>
              </div>

              {/* Right Tagline */}
              <div className="text-[11px] font-mono text-slate-400 tracking-wider flex items-center gap-1.5 self-start sm:self-center shrink-0">
                <span className="text-[#E31B23] font-bold">//</span>
                <span>FORENSIC TECHNOLOGY FOR A SAFER DIGITAL WORLD.</span>
              </div>
            </div>

            {/* Page Header */}
            <div className="pt-6 pb-6">
              <div className="text-xs font-mono font-bold tracking-wider text-[#E31B23] uppercase mb-2">
                {currentDoc.category}
              </div>
              <h1 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight leading-tight">
                {currentDoc.title}
              </h1>
            </div>

            {/* Content Display: Special High-Fidelity Layout for "Introduction & Architecture" */}
            {activeSectionId === 'intro' ? (
              <div className="space-y-10">
                {/* Introduction Lead Paragraph */}
                <div id="introduction" className="scroll-mt-24">
                  <p className="text-base sm:text-[16.5px] text-slate-600 leading-relaxed font-sans">
                    RAZE-CARVE is an enterprise forensic media sanitization and hardware recovery engine
                    engineered in Rust. It addresses the critical requirements of data destruction compliance
                    and file recovery across mechanical hard drives, NVMe SSDs, SATA SSDs, SAS arrays, UFS
                    chips, and eMMC devices.
                  </p>
                </div>

                {/* System Architecture Showcase Card */}
                <div
                  id="system-architecture"
                  className="scroll-mt-24 rounded-2xl border border-slate-200/90 bg-white p-5 sm:p-7 shadow-xs relative overflow-hidden"
                >
                  {/* Top Bar inside Architecture Box */}
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 pb-4 border-b border-slate-100">
                    <div>
                      <div className="text-xs font-mono font-bold text-[#E31B23] tracking-wide">
                        RAZE-CARVE CORE ARCHITECTURE
                      </div>
                      <div className="text-[11px] font-mono text-slate-400 tracking-wider">
                        MODULAR. VERIFIABLE. FORENSIC-GRADE.
                      </div>
                    </div>

                    <div className="text-[11px] font-mono text-slate-400 tracking-wider flex items-center gap-2">
                      <span>ACQUIRE</span>
                      <ChevronRight className="w-2.5 h-2.5 text-slate-300" />
                      <span>ERASE</span>
                      <ChevronRight className="w-2.5 h-2.5 text-slate-300" />
                      <span>RECOVER</span>
                      <ChevronRight className="w-2.5 h-2.5 text-slate-300" />
                      <span>VERIFY</span>
                    </div>
                  </div>

                  {/* Architecture Diagram Visual */}
                  <div className="mt-4 rounded-xl overflow-hidden border border-slate-100 bg-[#FBFBFD] relative group">
                    <img
                      src="/docs-architecture.jpg"
                      alt="RAZE-CARVE Core Architecture Diagram"
                      className="w-full h-auto object-contain transition-transform duration-300 group-hover:scale-[1.01]"
                    />
                  </div>
                </div>

                {/* Architectural Tenets Section */}
                <div id="architectural-tenets" className="scroll-mt-24 pt-2">
                  <h2 className="text-2xl font-bold text-slate-900 tracking-tight">
                    Architectural Tenets
                  </h2>
                  <p className="text-sm text-slate-500 mt-1 mb-6 font-sans">
                    Core principles that make RAZE-CARVE reliable, efficient, and forensically sound.
                  </p>

                  {/* 2x2 Grid of Tenet Cards */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-5">
                    {tenets.map((tenet) => (
                      <div
                        key={tenet.num}
                        className="rounded-2xl border border-slate-200/80 bg-white p-5 sm:p-6 shadow-xs hover:border-slate-300 transition-all duration-150 flex flex-col justify-between"
                      >
                        <div>
                          {/* Card Header: Number & Icon */}
                          <div className="flex items-start justify-between gap-3 mb-3">
                            <span className="text-sm font-mono font-bold text-[#E31B23]">
                              {tenet.num}
                            </span>
                            <div className="w-8 h-8 rounded-lg bg-slate-50 border border-slate-200/70 flex items-center justify-center shrink-0">
                              {tenet.icon}
                            </div>
                          </div>

                          {/* Title */}
                          <h3 className="text-[15px] sm:text-base font-bold text-slate-900 leading-snug mb-2">
                            {tenet.title}
                          </h3>

                          {/* Description */}
                          <p className="text-xs sm:text-[13px] text-slate-600 leading-relaxed font-sans mb-4">
                            {tenet.description}
                          </p>
                        </div>

                        {/* Bottom Tagline / Technical Pills */}
                        <div className="text-[10px] font-mono text-slate-400 tracking-wider pt-3 border-t border-slate-100">
                          {tenet.tags}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Verification & Integrity Section */}
                <div id="verification-integrity" className="scroll-mt-24 pt-6 border-t border-slate-100">
                  <h2 className="text-xl sm:text-2xl font-bold text-slate-900 tracking-tight mb-3">
                    Verification & Integrity
                  </h2>
                  <p className="text-sm sm:text-[15px] text-slate-600 leading-relaxed mb-4">
                    Every operation in RAZE-CARVE produces a cryptographically verifiable audit trail.
                    Post-wipe entropy sampling ensures that data destruction is mathematically confirmed,
                    not just assumed from write return codes.
                  </p>

                  <CodeBlock
                    code={`# Verify entropy of a sanitized drive\nsih149 verify --target /dev/nvme0n1 --entropy-threshold 0.001`}
                    language="bash"
                  />
                </div>

                {/* Next Steps Section */}
                <div id="next-steps" className="scroll-mt-24 pt-6 border-t border-slate-100">
                  <h2 className="text-xl sm:text-2xl font-bold text-slate-900 tracking-tight mb-4">
                    Next Steps
                  </h2>
                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <button
                      onClick={() => {
                        setActiveSectionId('requirements');
                        window.scrollTo({ top: 0, behavior: 'smooth' });
                      }}
                      className="p-4 rounded-xl border border-slate-200 hover:border-[#E31B23] transition-all text-left group bg-white"
                    >
                      <div className="text-xs font-mono text-[#E31B23] font-bold mb-1">STEP 01</div>
                      <div className="text-sm font-bold text-slate-900 group-hover:text-[#E31B23] transition-colors">
                        System Requirements
                      </div>
                      <div className="text-xs text-slate-500 mt-1">Check hardware & OS compatibility</div>
                    </button>

                    <button
                      onClick={() => {
                        setActiveSectionId('installation');
                        window.scrollTo({ top: 0, behavior: 'smooth' });
                      }}
                      className="p-4 rounded-xl border border-slate-200 hover:border-[#E31B23] transition-all text-left group bg-white"
                    >
                      <div className="text-xs font-mono text-[#E31B23] font-bold mb-1">STEP 02</div>
                      <div className="text-sm font-bold text-slate-900 group-hover:text-[#E31B23] transition-colors">
                        Installation & Setup
                      </div>
                      <div className="text-xs text-slate-500 mt-1">Binary, Docker, or Cargo build</div>
                    </button>

                    <button
                      onClick={() => {
                        setActiveSectionId('quick-start');
                        window.scrollTo({ top: 0, behavior: 'smooth' });
                      }}
                      className="p-4 rounded-xl border border-slate-200 hover:border-[#E31B23] transition-all text-left group bg-white"
                    >
                      <div className="text-xs font-mono text-[#E31B23] font-bold mb-1">STEP 03</div>
                      <div className="text-sm font-bold text-slate-900 group-hover:text-[#E31B23] transition-colors">
                        Quick Start Tutorial
                      </div>
                      <div className="text-xs text-slate-500 mt-1">Run simulated demo & self-test</div>
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              /* General Document Rendering for other sections */
              <div className="prose max-w-none pt-2 font-sans">
                {renderMarkdownContent(currentDoc.content)}
              </div>
            )}

            {/* =========================================================================
                BOTTOM PAGINATION: Previous / Next Buttons
               ========================================================================= */}
            <div className="flex items-center justify-between gap-4 mt-12 pt-8 border-t border-slate-200">
              {/* Previous Button */}
              {prevDoc ? (
                <button
                  onClick={() => {
                    setActiveSectionId(prevDoc.id);
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                  }}
                  className="flex items-center gap-3 p-3 sm:p-4 rounded-xl border border-slate-200 hover:border-slate-300 hover:bg-slate-50/50 transition-all text-left group max-w-[48%]"
                >
                  <div className="w-8 h-8 rounded-full border border-slate-200 flex items-center justify-center shrink-0 group-hover:border-slate-400 transition-colors">
                    <ArrowLeft className="w-4 h-4 text-slate-500 group-hover:text-slate-900" />
                  </div>
                  <div className="min-w-0">
                    <div className="text-[11px] font-mono text-slate-400 uppercase tracking-wider">
                      Previous
                    </div>
                    <div className="text-xs sm:text-sm font-bold text-slate-900 truncate">
                      {prevDoc.title}
                    </div>
                  </div>
                </button>
              ) : (
                <div />
              )}

              {/* Next Button */}
              {nextDoc && (
                <button
                  onClick={() => {
                    setActiveSectionId(nextDoc.id);
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                  }}
                  className="flex items-center gap-3 p-3 sm:p-4 rounded-xl border border-slate-200 hover:border-slate-300 hover:bg-slate-50/50 transition-all text-right group max-w-[48%] ml-auto"
                >
                  <div className="min-w-0">
                    <div className="text-[11px] font-mono text-slate-400 uppercase tracking-wider">
                      Next
                    </div>
                    <div className="text-xs sm:text-sm font-bold text-slate-900 truncate">
                      {nextDoc.title}
                    </div>
                  </div>
                  <div className="w-8 h-8 rounded-full border border-slate-200 flex items-center justify-center shrink-0 group-hover:border-slate-400 transition-colors">
                    <ArrowRight className="w-4 h-4 text-slate-500 group-hover:text-slate-900" />
                  </div>
                </button>
              )}
            </div>
          </main>

          {/* =========================================================================
              RIGHT COLUMN: "ON THIS PAGE" Table of Contents (~220px)
             ========================================================================= */}
          <aside className="sf-docs-toc shrink-0 sticky top-24 max-h-[calc(100vh-7rem)] overflow-y-auto pl-4 border-l border-slate-100" style={{ width: 220 }}>
            <div className="text-xs font-mono font-bold tracking-wider text-slate-900 pb-3 uppercase">
              ON THIS PAGE
            </div>

            <ul className="space-y-1.5 text-[13px] pt-1">
              {activeSectionId === 'intro' ? (
                introHeadings.map((heading) => {
                  const isActive = activeHeadingId === heading.id;
                  return (
                    <li key={heading.id}>
                      <button
                        onClick={() => scrollToHeading(heading.id)}
                        className={`text-left w-full py-1 transition-colors duration-150 flex items-center relative ${
                          isActive
                            ? 'text-slate-900 font-medium'
                            : 'text-slate-500 hover:text-slate-800'
                        }`}
                      >
                        {/* Red vertical bar on active item */}
                        {isActive && (
                          <span className="absolute -left-4 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-[#E31B23]" />
                        )}
                        <span>{heading.title}</span>
                      </button>
                    </li>
                  );
                })
              ) : (
                <li className="text-xs text-slate-400 font-mono">
                  <button
                    onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
                    className="text-left w-full py-1 text-slate-900 font-medium flex items-center relative"
                  >
                    <span className="absolute -left-4 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-[#E31B23]" />
                    <span>{currentDoc.title}</span>
                  </button>
                </li>
              )}
            </ul>
          </aside>

        </div>
      </div>
    </div>
  );
};
