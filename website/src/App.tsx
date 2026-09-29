import React, { useState, useEffect } from 'react';
import { RoutePath } from './types';
import { ThemeProvider } from './context/ThemeContext';
import { Header } from './components/Header';
import { Footer } from './components/Footer';
import { SkipLink } from './components/SkipLink';
import { SearchModal } from './components/SearchModal';
import { HomePage } from './pages/HomePage';
import { FeaturesPage } from './pages/FeaturesPage';
import { DocsPage } from './pages/DocsPage';
import { CommandsPage } from './pages/CommandsPage';
import { DownloadsPage } from './pages/DownloadsPage';
import { ExamplesPage } from './pages/ExamplesPage';
import { ChangelogPage } from './pages/ChangelogPage';
import { AboutPage } from './pages/AboutPage';
import { SandboxPage } from './pages/SandboxPage';
import { LegalPage } from './pages/LegalPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { GuiCommandCenterPage } from './pages/GuiCommandCenterPage';
import { TuiWorkspacePage } from './pages/TuiWorkspacePage';
import { CliWorkspacePage } from './pages/CliWorkspacePage';

export default function App() {
  const [currentHash, setCurrentHash] = useState<string>(() => {
    if (typeof window !== 'undefined') {
      return window.location.hash || '#/';
    }
    return '#/';
  });

  const [searchModalOpen, setSearchModalOpen] = useState(false);

  useEffect(() => {
    const handleHashChange = () => {
      setCurrentHash(window.location.hash || '#/');
      window.scrollTo({ top: 0, behavior: 'smooth' });
    };

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // Keyboard shortcut for search (Cmd+K / Ctrl+K)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setSearchModalOpen(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Parse path and query
  const cleanHash = currentHash.replace(/^#/, '') || '/';
  const [pathPart, queryPart] = cleanHash.split('?');
  const path = (pathPart || '/') as RoutePath;

  const queryParams = new URLSearchParams(queryPart || '');
  const sectionParam = queryParams.get('section') || undefined;
  const searchParam = queryParams.get('search') || undefined;
  const categoryParam = queryParams.get('category') || undefined;

  // Dynamic document title update
  useEffect(() => {
    const titles: Record<string, string> = {
      '/': 'RAZE-CARVE — Enterprise Forensic Sanitization & Hardware Carving',
      '/sandbox': 'In-Browser Forensic Sandbox & Live Viewer — RAZE-CARVE',
      '/features': 'Platform Features & Architecture — RAZE-CARVE',
      '/docs': 'Technical Documentation & Guides — RAZE-CARVE',
      '/commands': 'Command Reference & CLI Syntax — RAZE-CARVE',
      '/downloads': 'Release Downloads Hub & Live ISO — RAZE-CARVE',
      '/examples': 'Operational Workflows & Guides — RAZE-CARVE',
      '/changelog': 'Changelog & Release Notes — RAZE-CARVE',
      '/about': 'About & Engineering Principles — RAZE-CARVE',
      '/gui': 'GUI Command Center & Forensic Operations — RAZE-CARVE',
      '/tui': 'Interactive Terminal Defense UI (sforge) — RAZE-CARVE',
      '/cli': 'Forensic Shell & Command Runner (sih149) — RAZE-CARVE',
      '/privacy': 'Privacy Policy — RAZE-CARVE',
      '/terms': 'Terms of Service — RAZE-CARVE',
      '/cookies': 'Cookie Statement — RAZE-CARVE',
      '/security': 'Security & Vulnerability Disclosure — RAZE-CARVE',
      '/accessibility': 'Accessibility Statement — RAZE-CARVE',
    };

    document.title = titles[path] || '404 Not Found — RAZE-CARVE';
  }, [path]);

  const navigateTo = (newRoute: string) => {
    if (newRoute.startsWith('#')) {
      window.location.hash = newRoute;
    } else {
      window.location.hash = `#${newRoute}`;
    }
  };

  const renderPage = () => {
    switch (path) {
      case '/':
        return <HomePage onNavigate={(p) => navigateTo(p)} />;
      case '/sandbox':
        return <SandboxPage onNavigate={(p) => navigateTo(p)} />;
      case '/features':
        return <FeaturesPage onNavigate={(p) => navigateTo(p)} />;
      case '/docs':
        return <DocsPage onNavigate={(p) => navigateTo(p)} initialSectionId={sectionParam} />;
      case '/commands':
        return <CommandsPage onNavigate={(p) => navigateTo(p)} initialSearch={searchParam} />;
      case '/downloads':
        return <DownloadsPage onNavigate={(p) => navigateTo(p)} initialCategory={categoryParam} />;
      case '/examples':
        return <ExamplesPage onNavigate={(p) => navigateTo(p)} />;
      case '/changelog':
        return <ChangelogPage onNavigate={(p) => navigateTo(p)} />;
      case '/about':
        return <AboutPage onNavigate={(p) => navigateTo(p)} />;
      case '/gui':
        return <GuiCommandCenterPage onNavigate={(p) => navigateTo(p)} />;
      case '/tui':
        return <TuiWorkspacePage onNavigate={(p) => navigateTo(p)} />;
      case '/cli':
        return <CliWorkspacePage onNavigate={(p) => navigateTo(p)} />;
      case '/privacy':
        return <LegalPage section="privacy" onNavigate={(p) => navigateTo(p)} />;
      case '/terms':
        return <LegalPage section="terms" onNavigate={(p) => navigateTo(p)} />;
      case '/cookies':
        return <LegalPage section="cookies" onNavigate={(p) => navigateTo(p)} />;
      case '/security':
        return <LegalPage section="security" onNavigate={(p) => navigateTo(p)} />;
      case '/accessibility':
        return <LegalPage section="accessibility" onNavigate={(p) => navigateTo(p)} />;
      default:
        return (
          <NotFoundPage
            currentPath={path}
            onNavigate={(p) => navigateTo(p)}
            onOpenSearch={() => setSearchModalOpen(true)}
          />
        );
    }
  };

  return (
    <ThemeProvider>
      <div className="min-h-screen flex flex-col bg-[#F8F9FA] text-slate-900 transition-colors duration-150">
        <SkipLink />
        <Header
          currentPath={path}
          onNavigate={(p) => navigateTo(p)}
          onOpenSearch={() => setSearchModalOpen(true)}
        />

        <main id="main-content" className={`flex-1 w-full ${path === '/' || path === '/features' || path === '/docs' || path === '/downloads' || path === '/gui' || path === '/tui' || path === '/cli' ? '' : 'max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-8'}`}>
          {renderPage()}
        </main>

        <Footer onNavigate={(p) => navigateTo(p)} />

        <SearchModal
          isOpen={searchModalOpen}
          onClose={() => setSearchModalOpen(false)}
          onNavigate={(route) => navigateTo(route)}
        />
      </div>
    </ThemeProvider>
  );
}
