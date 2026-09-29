import React from 'react';
import { AlertCircle, AlertTriangle, CheckCircle, Info, ShieldAlert } from 'lucide-react';

interface CalloutProps {
  type?: 'note' | 'tip' | 'important' | 'warning' | 'caution';
  title?: string;
  children: React.ReactNode;
}

export const Callout: React.FC<CalloutProps> = ({ type = 'note', title, children }) => {
  const configs = {
    note: {
      icon: <Info className="w-5 h-5 text-sky-500 shrink-0 mt-0.5" />,
      border: 'border-sky-300 dark:border-sky-800',
      bg: 'bg-sky-50/70 dark:bg-sky-950/40',
      titleColor: 'text-sky-900 dark:text-sky-300',
      defaultTitle: 'Note',
    },
    tip: {
      icon: <CheckCircle className="w-5 h-5 text-emerald-500 shrink-0 mt-0.5" />,
      border: 'border-emerald-300 dark:border-emerald-800',
      bg: 'bg-emerald-50/70 dark:bg-emerald-950/40',
      titleColor: 'text-emerald-900 dark:text-emerald-300',
      defaultTitle: 'Pro Tip',
    },
    important: {
      icon: <AlertCircle className="w-5 h-5 text-indigo-500 shrink-0 mt-0.5" />,
      border: 'border-indigo-300 dark:border-indigo-800',
      bg: 'bg-indigo-50/70 dark:bg-indigo-950/40',
      titleColor: 'text-indigo-900 dark:text-indigo-300',
      defaultTitle: 'Important',
    },
    warning: {
      icon: <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />,
      border: 'border-amber-300 dark:border-amber-800',
      bg: 'bg-amber-50/70 dark:bg-amber-950/40',
      titleColor: 'text-amber-900 dark:text-amber-300',
      defaultTitle: 'Warning',
    },
    caution: {
      icon: <ShieldAlert className="w-5 h-5 text-rose-500 shrink-0 mt-0.5" />,
      border: 'border-rose-300 dark:border-rose-800',
      bg: 'bg-rose-50/70 dark:bg-rose-950/40',
      titleColor: 'text-rose-900 dark:text-rose-300',
      defaultTitle: 'Caution',
    },
  };

  const config = configs[type];

  return (
    <div className={`flex gap-3 p-4 my-4 rounded-lg border ${config.border} ${config.bg} text-sm leading-relaxed`}>
      {config.icon}
      <div className="flex-1 space-y-1">
        <div className={`font-semibold ${config.titleColor}`}>{title || config.defaultTitle}</div>
        <div className="text-slate-700 dark:text-slate-300">{children}</div>
      </div>
    </div>
  );
};
