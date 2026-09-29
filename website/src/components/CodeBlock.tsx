import React from 'react';
import { CopyButton } from './CopyButton';

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  showLineNumbers?: boolean;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({
  code,
  language = 'bash',
  filename,
  showLineNumbers = false,
}) => {
  const lines = code.trim().split('\n');

  return (
    <div className="rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-900 text-slate-100 overflow-hidden my-4 shadow-sm">
      <div className="flex items-center justify-between px-4 py-2 bg-slate-950/80 border-b border-slate-800 text-xs text-slate-400 font-mono">
        <div className="flex items-center gap-2">
          <span className="inline-block w-2.5 h-2.5 rounded-full bg-slate-700"></span>
          {filename ? (
            <span className="text-slate-200 font-medium">{filename}</span>
          ) : (
            <span className="uppercase tracking-wider text-[10px] text-slate-500">{language}</span>
          )}
        </div>
        <CopyButton text={code.trim()} />
      </div>
      <div className="overflow-x-auto p-4 text-xs sm:text-sm font-mono leading-relaxed selection:bg-brand-500/30">
        {showLineNumbers ? (
          <table className="w-full border-collapse">
            <tbody>
              {lines.map((line, idx) => (
                <tr key={idx} className="hover:bg-slate-800/40">
                  <td className="pr-4 text-slate-600 select-none text-right w-8">{idx + 1}</td>
                  <td className="whitespace-pre">{line}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <pre className="whitespace-pre">
            <code>{code.trim()}</code>
          </pre>
        )}
      </div>
    </div>
  );
};
