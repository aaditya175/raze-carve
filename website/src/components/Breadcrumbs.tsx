import React from 'react';
import { ChevronRight, Home } from 'lucide-react';

interface Crumb {
  label: string;
  href?: string;
}

interface BreadcrumbsProps {
  crumbs: Crumb[];
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({ crumbs }) => {
  return (
    <nav aria-label="Breadcrumb" className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 my-3 font-mono">
      <a href="#/" className="hover:text-brand-500 flex items-center gap-1" aria-label="Home">
        <Home className="w-3.5 h-3.5" />
      </a>
      {crumbs.map((crumb, idx) => (
        <React.Fragment key={idx}>
          <ChevronRight className="w-3 h-3 text-slate-400 dark:text-slate-600" />
          {crumb.href ? (
            <a href={crumb.href} className="hover:text-brand-500 hover:underline">
              {crumb.label}
            </a>
          ) : (
            <span className="text-slate-800 dark:text-slate-200 font-medium" aria-current="page">
              {crumb.label}
            </span>
          )}
        </React.Fragment>
      ))}
    </nav>
  );
};
