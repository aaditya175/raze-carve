import React from 'react';
import { RoutePath } from '../types';
import { Button } from '../components/Button';
import { ShieldAlert, Home, Search, BookOpen, Terminal } from 'lucide-react';

interface NotFoundPageProps {
  currentPath: string;
  onNavigate: (path: RoutePath) => void;
  onOpenSearch: () => void;
}

export const NotFoundPage: React.FC<NotFoundPageProps> = ({ currentPath, onNavigate, onOpenSearch }) => {
  return (
    <div className="max-w-2xl mx-auto py-16 text-center space-y-6">
      <div className="inline-flex p-4 rounded-full bg-amber-50 dark:bg-amber-950/60 text-amber-600 dark:text-amber-400 border border-amber-200 dark:border-amber-800">
        <ShieldAlert className="w-8 h-8" />
      </div>

      <div className="space-y-2">
        <div className="text-xs font-mono font-semibold text-brand-600 dark:text-brand-400 uppercase tracking-wider">
          Error 404 • Resource Not Found
        </div>
        <h1 className="text-3xl font-extrabold text-slate-900 dark:text-slate-100 font-mono">
          Page Not Located
        </h1>
        <p className="text-sm text-slate-600 dark:text-slate-300 font-sans max-w-md mx-auto">
          The requested path <code className="font-mono text-xs bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded text-rose-500">{currentPath}</code> does not exist on the RAZE-CARVE documentation portal.
        </p>
      </div>

      <div className="flex flex-wrap items-center justify-center gap-3 pt-2">
        <Button onClick={() => onNavigate('/')} icon={<Home className="w-4 h-4" />}>
          Return Home
        </Button>
        <Button variant="outline" onClick={onOpenSearch} icon={<Search className="w-4 h-4" />}>
          Search Knowledge Base
        </Button>
        <Button variant="ghost" onClick={() => onNavigate('/docs')} icon={<BookOpen className="w-4 h-4" />}>
          Browse Documentation
        </Button>
      </div>
    </div>
  );
};
