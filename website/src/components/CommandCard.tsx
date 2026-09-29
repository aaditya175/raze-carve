import React, { useState } from 'react';
import { CommandSpec } from '../types';
import { CopyButton } from './CopyButton';
import { Badge } from './Badge';
import { Terminal, AlertCircle, ChevronDown, ChevronUp } from 'lucide-react';

interface CommandCardProps {
  command: CommandSpec;
  defaultExpanded?: boolean;
}

export const CommandCard: React.FC<CommandCardProps> = ({ command, defaultExpanded = false }) => {
  const [expanded, setExpanded] = useState(defaultExpanded);

  return (
    <article
      id={command.id}
      className="rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-xs overflow-hidden transition-all duration-200 scroll-mt-24 card-interactive"
    >
      <div className="p-5">
        <div className="flex flex-wrap items-center justify-between gap-3 mb-2">
          <div className="flex items-center gap-2.5">
            <span className="p-1.5 rounded bg-slate-100 dark:bg-slate-800 text-brand-600 dark:text-brand-400 font-mono text-sm">
              <Terminal className="w-4 h-4" />
            </span>
            <h3 className="text-lg font-bold text-slate-900 dark:text-slate-100 font-mono">
              {command.binary} {command.name}
            </h3>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant="brand" size="sm">{command.binary}</Badge>
            <button
              onClick={() => setExpanded(!expanded)}
              aria-expanded={expanded}
              aria-controls={`details-${command.id}`}
              className="p-1.5 text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100 rounded-md transition-colors"
              title={expanded ? 'Collapse details' : 'Expand details'}
            >
              <ChevronDown
                className={`w-4 h-4 transition-transform duration-200 ${
                  expanded ? 'rotate-180 text-brand-500' : ''
                }`}
              />
            </button>
          </div>
        </div>

        <p className="text-sm text-slate-600 dark:text-slate-300 mb-4">{command.summary}</p>

        {/* Syntax block */}
        <div className="flex items-center justify-between gap-2 sm:gap-3 p-3 bg-slate-950 text-slate-100 rounded-md font-mono text-xs">
          <div className="flex items-center gap-2 overflow-x-auto min-w-0 py-0.5">
            <span className="text-brand-400 font-semibold select-none">$</span>
            <code className="text-slate-200 whitespace-pre">{command.syntax}</code>
          </div>
          <CopyButton text={command.syntax} className="shrink-0" />
        </div>
      </div>

      {expanded && (
        <div id={`details-${command.id}`} className="px-5 pb-5 pt-1 border-t border-slate-100 dark:border-slate-800/80 space-y-5 text-sm animate-fade-in">
          {/* Detailed Purpose */}
          <div>
            <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-1.5">
              Technical Purpose
            </h4>
            <p className="text-slate-700 dark:text-slate-300 text-sm leading-relaxed">{command.purpose}</p>
          </div>

          {/* Options */}
          {command.options.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-2">
                Flags & Options
              </h4>
              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs border-collapse font-mono">
                  <thead>
                    <tr className="border-b border-slate-200 dark:border-slate-800 text-slate-500">
                      <th className="py-2 pr-4 font-semibold">Flag</th>
                      <th className="py-2 pr-4 font-semibold">Description</th>
                      <th className="py-2 font-semibold">Default</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100 dark:divide-slate-800/60">
                    {command.options.map((opt, i) => (
                      <tr key={i} className="hover:bg-slate-50 dark:hover:bg-slate-850/40">
                        <td className="py-2 pr-4 text-brand-600 dark:text-brand-400 whitespace-nowrap">
                          {opt.short && <span className="text-slate-400 mr-1">{opt.short},</span>}
                          {opt.flag}
                        </td>
                        <td className="py-2 pr-4 font-sans text-slate-600 dark:text-slate-300">
                          {opt.description}
                        </td>
                        <td className="py-2 text-slate-400 whitespace-nowrap">
                          {opt.defaultValue || '—'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Real Execution Examples */}
          {command.examples.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-2">
                Operational Example & Verified Output
              </h4>
              {command.examples.map((ex, idx) => (
                <div key={idx} className="space-y-2">
                  <div className="text-xs font-medium text-slate-700 dark:text-slate-300">{ex.title}</div>
                  <div className="p-3 bg-slate-950 text-slate-200 rounded font-mono text-xs flex justify-between items-center">
                    <code>{ex.command}</code>
                    <CopyButton text={ex.command} />
                  </div>
                  <div className="p-3 bg-slate-900 rounded border border-slate-800 font-mono text-xs text-slate-300 overflow-x-auto whitespace-pre leading-relaxed">
                    {ex.expectedOutput}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Common Mistakes */}
          {command.commonMistakes.length > 0 && (
            <div className="p-3 rounded-md bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900/60 text-xs">
              <div className="flex items-center gap-1.5 font-semibold text-amber-800 dark:text-amber-300 mb-1">
                <AlertCircle className="w-3.5 h-3.5" />
                <span>Common Operational Gotchas</span>
              </div>
              <ul className="list-disc list-inside space-y-1 text-amber-900/90 dark:text-amber-300/90 font-sans">
                {command.commonMistakes.map((m, idx) => (
                  <li key={idx}>{m}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </article>
  );
};
