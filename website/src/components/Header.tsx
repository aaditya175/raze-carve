import React, { useState, useEffect } from 'react';
import { RoutePath } from '../types';
import {
  Shield,
  Search,
  Menu,
  X,
  ArrowRight
} from 'lucide-react';

export interface HeaderProps {
  currentPath: RoutePath;
  onNavigate: (path: RoutePath) => void;
  onOpenSearch: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  currentPath,
  onNavigate,
  onOpenSearch,
}) => {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  // Force pure light mode across the app as requested
  useEffect(() => {
    document.documentElement.classList.remove('dark');
  }, []);

  const handleNavigate = (path: RoutePath) => {
    onNavigate(path);
    setIsMobileMenuOpen(false);
  };

  // Dedicated Product Interface: CLI Live Terminal only (GUI & TUI removed as requested)
  const productInterfaces = [
    {
      label: 'CLI',
      fullName: 'Direct Shell & Live Cloud Terminal',
      path: '/cli' as RoutePath,
      dotColor: 'bg-[#E31B23]', // RAZE-CARVE Red dot
    },
  ];

  return (
    <header className="sticky top-2 sm:top-4 w-full z-50 px-4 sm:px-8 transition-all font-sans">
      {/* Floating Pill Container - Slightly increased size & comfortable padding */}
      <div className="max-w-[1420px] mx-auto rounded-2xl sm:rounded-full bg-white/95 backdrop-blur-md border border-slate-200/90 shadow-sm px-6 sm:px-8 py-3.5 sm:py-4 flex items-center justify-between transition-all">
        
        {/* =========================================================================
            LEFT: Brand Logo
           ========================================================================= */}
        <div
          className="flex items-center cursor-pointer group select-none shrink-0"
          onClick={() => handleNavigate('/')}
        >
          <img
            src="/logo.png"
            alt="RAZE-CARVE"
            className="h-8 sm:h-9 w-auto object-contain transition-transform group-hover:scale-[1.02]"
          />
        </div>

        {/* =========================================================================
            CENTER: Navigation Items with Colored Dot Badges (Desktop)
           ========================================================================= */}
        <nav className="hidden lg:flex items-center space-x-2 text-[14px] font-semibold text-slate-700">
          
          {/* Platform / Home Link */}
          <button
            onClick={() => handleNavigate('/')}
            className={`px-3 py-1.5 rounded-lg hover:text-slate-900 transition-colors cursor-pointer ${
              currentPath === '/' ? 'text-[#E31B23]' : ''
            }`}
          >
            Platform
          </button>

          {/* Vertical Divider 1 */}
          <span className="h-4 w-[1px] bg-slate-200 mx-2" />

          {/* =======================================================================
              THE 3 PRODUCT INTERFACES: GUI, TUI, CLI (with colored dots)
             ======================================================================= */}
          <div className="flex items-center space-x-1.5">
            {productInterfaces.map((item) => {
              const isActive = currentPath === item.path;
              return (
                <button
                  key={item.label}
                  onClick={() => handleNavigate(item.path)}
                  className={`flex items-center gap-2 px-3.5 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer ${
                    isActive
                      ? 'bg-slate-100 text-slate-900 shadow-xs border border-slate-200/80'
                      : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                  }`}
                  title={`${item.label} — ${item.fullName}`}
                >
                  <span className={`w-2 h-2 rounded-full ${item.dotColor} shrink-0`} />
                  <span>{item.label}</span>
                  {item.label === 'CLI' && (
                    <span className="px-1.5 py-0.5 bg-emerald-100 text-emerald-700 text-[9px] font-mono font-bold rounded-full uppercase tracking-wider flex items-center gap-1">
                      <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                      <span>LIVE</span>
                    </span>
                  )}
                </button>
              );
            })}
          </div>

          {/* Vertical Divider 2 */}
          <span className="h-4 w-[1px] bg-slate-200 mx-2" />

          {/* Secondary Standard Links */}
          <button
            onClick={() => handleNavigate('/features')}
            className={`px-3 py-1.5 rounded-lg hover:text-slate-900 transition-colors cursor-pointer ${
              currentPath === '/features' ? 'text-[#E31B23]' : ''
            }`}
          >
            Features
          </button>

          <button
            onClick={() => handleNavigate('/docs')}
            className={`px-3 py-1.5 rounded-lg hover:text-slate-900 transition-colors cursor-pointer ${
              currentPath === '/docs' ? 'text-[#E31B23]' : ''
            }`}
          >
            Documentation
          </button>

          <button
            onClick={() => handleNavigate('/downloads')}
            className={`px-3 py-1.5 rounded-lg hover:text-slate-900 transition-colors cursor-pointer ${
              currentPath === '/downloads' ? 'text-[#E31B23]' : ''
            }`}
          >
            Downloads
          </button>

          <button
            onClick={() => handleNavigate('/about')}
            className={`px-3 py-1.5 rounded-lg hover:text-slate-900 transition-colors cursor-pointer ${
              currentPath === '/about' ? 'text-[#E31B23]' : ''
            }`}
          >
            About
          </button>
        </nav>

        {/* =========================================================================
            RIGHT: Search Trigger & Action Button (Pure Light Mode, No Toggle)
           ========================================================================= */}
        <div className="flex items-center space-x-3">
          
          {/* Search Trigger Button */}
          <button
            onClick={onOpenSearch}
            className="p-2 text-slate-500 hover:text-slate-900 rounded-full hover:bg-slate-100 transition-colors cursor-pointer"
            aria-label="Search documentation"
            title="Search (Cmd+K)"
          >
            <Search className="w-4 h-4" />
          </button>


          {/* Mobile Menu Hamburger Toggle */}
          <button
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            className="lg:hidden p-2 rounded-xl text-slate-700 hover:bg-slate-100 transition-colors cursor-pointer"
            aria-label="Toggle navigation menu"
          >
            {isMobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>

      </div>

      {/* =========================================================================
          MOBILE MENU DRAWER
         ========================================================================= */}
      {isMobileMenuOpen && (
        <div className="lg:hidden max-w-[1420px] mx-auto mt-2 rounded-2xl bg-white border border-slate-200 shadow-xl p-5 animate-fade-down">
          {/* Mobile Product Badges */}
          <div className="pb-3 border-b border-slate-100">
            <div className="text-[11px] font-mono text-slate-400 uppercase tracking-wider mb-2 font-semibold">
              CHOOSE INTERFACE
            </div>
            <div className="grid grid-cols-1 gap-2">
              {productInterfaces.map((item) => (
                <button
                  key={item.label}
                  onClick={() => handleNavigate(item.path)}
                  className="flex items-center justify-center gap-2 p-2.5 rounded-xl bg-slate-50 border border-slate-200/80 text-xs font-bold text-slate-800"
                >
                  <span className={`w-2 h-2 rounded-full ${item.dotColor} shrink-0`} />
                  <span>{item.label}</span>
                  {item.label === 'CLI' && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse ml-0.5" />}
                </button>
              ))}
            </div>
          </div>

          {/* Main Links */}
          <div className="py-3 space-y-1 text-sm font-semibold border-b border-slate-100 text-slate-700">
            <button
              onClick={() => handleNavigate('/')}
              className="w-full text-left py-2 px-3 rounded-lg hover:bg-slate-50"
            >
              Platform Overview
            </button>
            <button
              onClick={() => handleNavigate('/features')}
              className="w-full text-left py-2 px-3 rounded-lg hover:bg-slate-50"
            >
              Platform Features
            </button>
            <button
              onClick={() => handleNavigate('/docs')}
              className="w-full text-left py-2 px-3 rounded-lg hover:bg-slate-50"
            >
              Documentation
            </button>
            <button
              onClick={() => handleNavigate('/downloads')}
              className="w-full text-left py-2 px-3 rounded-lg hover:bg-slate-50"
            >
              Downloads Hub & Live ISO
            </button>
            <button
              onClick={() => handleNavigate('/about')}
              className="w-full text-left py-2 px-3 rounded-lg hover:bg-slate-50"
            >
              About RAZE-CARVE
            </button>
          </div>


        </div>
      )}
    </header>
  );
};
