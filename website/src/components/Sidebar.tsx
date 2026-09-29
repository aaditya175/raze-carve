import React from 'react';

interface SidebarItem {
  id: string;
  title: string;
  category?: string;
}

interface SidebarProps {
  items: SidebarItem[];
  activeId: string;
  onSelect: (id: string) => void;
  title?: string;
}

export const Sidebar: React.FC<SidebarProps> = ({ items, activeId, onSelect, title }) => {
  // Group items by category if available
  const grouped = items.reduce<Record<string, SidebarItem[]>>((acc, item) => {
    const cat = item.category || 'General';
    if (!acc[cat]) acc[cat] = [];
    acc[cat].push(item);
    return acc;
  }, {});

  return (
    <aside className="w-full lg:w-64 shrink-0 font-sans">
      <div className="sticky top-20 max-h-[calc(100vh-6rem)] overflow-y-auto pr-3 py-2 space-y-6">
        {title && (
          <div className="text-xs font-semibold uppercase tracking-wider text-slate-400 dark:text-slate-500 px-3">
            {title}
          </div>
        )}

        {Object.entries(grouped).map(([category, catItems]) => (
          <div key={category} className="space-y-1.5">
            <div className="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400 px-3 py-1">
              {category}
            </div>
            <nav className="space-y-0.5" aria-label={`Navigation: ${category}`}>
              {catItems.map((item) => {
                const isActive = item.id === activeId;
                return (
                  <button
                    key={item.id}
                    onClick={() => onSelect(item.id)}
                    className={`w-full text-left text-xs font-medium px-3 py-1.5 rounded-md transition-colors duration-100 flex items-center justify-between ${
                      isActive
                        ? 'bg-sky-50 dark:bg-sky-950/60 text-brand-600 dark:text-brand-400 font-semibold border-l-2 border-brand-500'
                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-850'
                    }`}
                  >
                    <span className="truncate">{item.title}</span>
                  </button>
                );
              })}
            </nav>
          </div>
        ))}
      </div>
    </aside>
  );
};
