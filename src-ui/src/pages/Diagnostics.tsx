import { useState, useEffect } from 'react';
import {
  Activity,
  CheckCircle2,
  XCircle,
  Play,
  RefreshCw,
  Cpu,
  Database,
  Lock,
  Search,
  Zap,
} from 'lucide-react';
import { DiagnosticsAPI } from '../lib/api';
import { DiagnosticSummary, DiagnosticResult } from '../lib/types';

export function Diagnostics() {
  const [summary, setSummary] = useState<DiagnosticSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');

  const runTests = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await DiagnosticsAPI.run();
      setSummary(data);
    } catch (err: any) {
      setError(err?.message || 'Failed to run diagnostic self-tests');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    runTests();
  }, []);

  const categories = [
    { id: 'all', label: 'All Assertions', icon: <Activity className="w-3.5 h-3.5" /> },
    { id: 'Wiper', label: 'Wiper Engine', icon: <Zap className="w-3.5 h-3.5" /> },
    { id: 'Safety', label: 'Safety & Traversal', icon: <Lock className="w-3.5 h-3.5" /> },
    { id: 'Carver', label: 'Forensic Carver', icon: <Search className="w-3.5 h-3.5" /> },
    { id: 'Crypto', label: 'Crypto & Hash Chain', icon: <Cpu className="w-3.5 h-3.5" /> },
    { id: 'Database', label: 'Case Database', icon: <Database className="w-3.5 h-3.5" /> },
  ];

  const filteredResults: DiagnosticResult[] = (summary?.results || []).filter((r) => {
    const matchCategory = selectedCategory === 'all' || r.category.toLowerCase() === selectedCategory.toLowerCase();
    const matchQuery =
      r.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.category.toLowerCase().includes(searchQuery.toLowerCase());
    return matchCategory && matchQuery;
  });

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 text-xs font-mono font-medium rounded bg-cyber-500/10 text-cyber-400 border border-cyber-500/20">
              SELF-TEST RUNNER
            </span>
            <span className="text-xs text-slate-400 font-mono">NIST 800-88 & Forensics Validation</span>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-white mt-1">System Diagnostics & Assertions</h1>
          <p className="text-sm text-slate-400">
            Live in-situ verification of cryptographic hashing, wiper patterns, directory traversal blocks, and carvers.
          </p>
        </div>

        <button
          onClick={runTests}
          disabled={loading}
          className="flex items-center justify-center gap-2 px-4 py-2.5 bg-cyber-500 hover:bg-cyber-400 disabled:opacity-50 text-surface-950 font-semibold text-sm rounded-xl shadow-lg shadow-cyber-500/20 transition-all cursor-pointer shrink-0"
        >
          {loading ? (
            <RefreshCw className="w-4 h-4 animate-spin" />
          ) : (
            <Play className="w-4 h-4 fill-current" />
          )}
          {loading ? 'Running Tests...' : 'Run All Self-Tests'}
        </button>
      </div>

      {error && (
        <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-sm flex items-center gap-3">
          <XCircle className="w-5 h-5 text-rose-400 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Summary KPI cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div className="p-4 rounded-xl bg-surface-900/60 border border-white/5 space-y-1">
          <span className="text-xs font-mono text-slate-400 uppercase tracking-wider">Total Tests</span>
          <div className="text-2xl font-bold text-white font-mono">{summary?.total ?? '—'}</div>
          <span className="text-xs text-slate-500">Live sub-assertions</span>
        </div>

        <div className="p-4 rounded-xl bg-surface-900/60 border border-emerald-500/20 space-y-1">
          <span className="text-xs font-mono text-emerald-400 uppercase tracking-wider">Passed</span>
          <div className="text-2xl font-bold text-emerald-400 font-mono">{summary?.passed ?? '—'}</div>
          <span className="text-xs text-emerald-500/80">100% operational</span>
        </div>

        <div className="p-4 rounded-xl bg-surface-900/60 border border-white/5 space-y-1">
          <span className="text-xs font-mono text-rose-400 uppercase tracking-wider">Failed</span>
          <div className={`text-2xl font-bold font-mono ${(summary?.failed || 0) > 0 ? 'text-rose-400' : 'text-slate-400'}`}>
            {summary?.failed ?? '—'}
          </div>
          <span className="text-xs text-slate-500">Subsystem faults</span>
        </div>

        <div className="p-4 rounded-xl bg-surface-900/60 border border-white/5 space-y-1">
          <span className="text-xs font-mono text-purple-400 uppercase tracking-wider">Total Duration</span>
          <div className="text-2xl font-bold text-purple-400 font-mono">
            {summary ? `${summary.total_duration_ms.toFixed(2)} ms` : '—'}
          </div>
          <span className="text-xs text-slate-500">Execution latency</span>
        </div>
      </div>

      {/* Filters & Search */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
        <div className="flex flex-wrap gap-1.5">
          {categories.map((cat) => (
            <button
              key={cat.id}
              onClick={() => setSelectedCategory(cat.id)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer border ${
                selectedCategory === cat.id
                  ? 'bg-cyber-500/20 text-cyber-300 border-cyber-500/40 shadow-sm shadow-cyber-500/10'
                  : 'bg-surface-900/40 text-slate-400 border-white/5 hover:bg-white/5 hover:text-slate-200'
              }`}
            >
              {cat.icon}
              {cat.label}
            </button>
          ))}
        </div>

        <div className="relative w-full md:w-64">
          <input
            type="text"
            placeholder="Search test names..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-3 py-1.5 text-xs bg-surface-900/80 border border-white/10 rounded-lg text-slate-200 placeholder-slate-500 focus:outline-none focus:border-cyber-500/50"
          />
          <Search className="w-3.5 h-3.5 text-slate-500 absolute left-3 top-2.5" />
        </div>
      </div>

      {/* Assertions Table */}
      <div className="rounded-xl border border-white/5 bg-surface-900/40 overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-surface-900/90 border-b border-white/5 text-slate-400 font-mono uppercase tracking-wider">
              <tr>
                <th className="px-4 py-3 w-28">Status</th>
                <th className="px-4 py-3 w-32">Category</th>
                <th className="px-4 py-3">Assertion Name</th>
                <th className="px-4 py-3 w-28 text-right">Latency</th>
                <th className="px-4 py-3">Verification Details</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {filteredResults.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-4 py-8 text-center text-slate-500">
                    {loading ? 'Running self-tests...' : 'No test results matched your criteria.'}
                  </td>
                </tr>
              ) : (
                filteredResults.map((r, i) => (
                  <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                    <td className="px-4 py-3">
                      {r.passed ? (
                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                          <CheckCircle2 className="w-3 h-3" />
                          PASS
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                          <XCircle className="w-3 h-3" />
                          FAIL
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      <span className="text-[11px] font-mono text-slate-400 px-2 py-0.5 rounded bg-surface-800/60 border border-white/5">
                        {r.category}
                      </span>
                    </td>
                    <td className="px-4 py-3 font-medium text-slate-200">{r.name}</td>
                    <td className="px-4 py-3 text-right font-mono text-slate-400">{r.duration_us} µs</td>
                    <td className="px-4 py-3 text-slate-400 font-mono text-[11px]">{r.message}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
