import React, { useState, useEffect } from 'react';
import { RoutePath } from '../types';
import { COMMANDS_DATA } from '../data/commandsData';
import { CommandCard } from '../components/CommandCard';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { Reveal } from '../components/Reveal';
import { Search, Terminal, Filter } from 'lucide-react';

interface CommandsPageProps {
  onNavigate: (path: RoutePath) => void;
  initialSearch?: string;
}

export const CommandsPage: React.FC<CommandsPageProps> = ({ initialSearch = '' }) => {
  const [filterBinary, setFilterBinary] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>(initialSearch);

  useEffect(() => {
    if (initialSearch) {
      setSearchQuery(initialSearch);
    }
  }, [initialSearch]);

  const filteredCommands = COMMANDS_DATA.filter((cmd) => {
    const matchesBinary = filterBinary === 'all' || cmd.binary === filterBinary;
    const q = searchQuery.toLowerCase();
    const matchesSearch =
      !q ||
      cmd.name.toLowerCase().includes(q) ||
      cmd.binary.toLowerCase().includes(q) ||
      cmd.summary.toLowerCase().includes(q) ||
      cmd.syntax.toLowerCase().includes(q);
    return matchesBinary && matchesSearch;
  });

  return (
    <div className="max-w-5xl mx-auto py-6 space-y-8">
      <div>
        <Breadcrumbs crumbs={[{ label: 'Reference' }, { label: 'Command Reference' }]} />
        <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono mt-2">
          Command Line & TUI Reference
        </h1>
        <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-2 max-w-3xl">
          Comprehensive flag, argument, and output specifications for the <code className="text-brand-600 dark:text-brand-400 font-mono font-semibold">sih149</code> CLI batch utility, <code className="text-brand-600 dark:text-brand-400 font-mono font-semibold">sforge</code> interactive TUI console, and <code className="text-brand-600 dark:text-brand-400 font-mono font-semibold">raze-carve-desktop</code> daemon.
        </p>
      </div>

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 p-4 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/60">
        {/* Binary Filter Tabs */}
        <div className="flex items-center gap-1.5 w-full sm:w-auto overflow-x-auto text-xs font-mono">
          {(['all', 'sih149', 'sforge', 'raze-carve-desktop'] as const).map((bin) => (
            <button
              key={bin}
              onClick={() => setFilterBinary(bin)}
              className={`px-3 py-1.5 rounded-md transition-colors ${
                filterBinary === bin
                  ? 'bg-slate-900 text-brand-400 dark:bg-brand-500 dark:text-slate-950 font-bold'
                  : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-800'
              }`}
            >
              {bin === 'all' ? 'All Binaries' : bin}
            </button>
          ))}
        </div>

        {/* Search Input */}
        <div className="relative w-full sm:w-72">
          <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
          <input
            type="text"
            placeholder="Search commands, flags, or syntax..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-3 py-1.5 text-xs bg-white dark:bg-slate-950 border border-slate-200 dark:border-slate-800 rounded-md outline-none text-slate-900 dark:text-slate-100 font-sans"
          />
        </div>
      </div>

      {/* Commands List */}
      <div className="space-y-6">
        {filteredCommands.length > 0 ? (
          filteredCommands.map((cmd, idx) => (
            <Reveal key={cmd.id} delayMs={Math.min(idx * 40, 200)}>
              <CommandCard command={cmd} defaultExpanded={false} />
            </Reveal>
          ))
        ) : (
          <div className="p-12 text-center border border-dashed border-slate-300 dark:border-slate-800 rounded-lg text-sm text-slate-500">
            No commands found matching "{searchQuery}".
          </div>
        )}
      </div>
    </div>
  );
};
