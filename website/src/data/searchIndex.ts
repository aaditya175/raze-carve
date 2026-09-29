import { SearchResult } from '../types';
import { COMMANDS_DATA } from './commandsData';
import { DOCS_DATA } from './docsData';
import { DOWNLOADS_DATA } from './downloadsData';
import { FAQ_DATA } from './faqData';

export const SEARCH_INDEX: SearchResult[] = [
  ...COMMANDS_DATA.map((cmd) => ({
    id: `cmd-${cmd.id}`,
    title: `${cmd.binary} ${cmd.name}`,
    category: 'Commands',
    snippet: `${cmd.summary}. Syntax: ${cmd.syntax}`,
    route: `#/commands?search=${cmd.name}`,
  })),
  ...DOCS_DATA.map((doc) => ({
    id: `doc-${doc.id}`,
    title: doc.title,
    category: `Docs: ${doc.category}`,
    snippet: doc.content.slice(0, 140).replace(/[#*`]/g, '') + '...',
    route: `#/docs?section=${doc.id}`,
  })),
  ...DOWNLOADS_DATA.map((dl) => ({
    id: `dl-${dl.id}`,
    title: dl.title,
    category: 'Downloads',
    snippet: `${dl.filename} (${dl.size}) - ${dl.description.slice(0, 100)}...`,
    route: `#/downloads?category=${dl.category}`,
  })),
  ...FAQ_DATA.map((faq, idx) => ({
    id: `faq-${idx}`,
    title: faq.question,
    category: `FAQ: ${faq.category}`,
    snippet: faq.answer.slice(0, 140) + '...',
    route: `#/docs?section=standards-matrix`,
  })),
  {
    id: 'page-privacy',
    title: 'Privacy Policy',
    category: 'Legal',
    snippet: 'Strict zero-telemetry policy. No personal data or diagnostic analytics collected.',
    route: '#/privacy',
  },
  {
    id: 'page-terms',
    title: 'Terms of Service',
    category: 'Legal',
    snippet: 'Open-source forensic operational licensing terms, AS-IS warranties, and compliance disclaimers.',
    route: '#/terms',
  },
  {
    id: 'page-security',
    title: 'Security & Vulnerability Disclosure',
    category: 'Legal',
    snippet: 'Responsible security disclosure guidelines, cryptographic integrity, and memory safety.',
    route: '#/security',
  },
  {
    id: 'page-accessibility',
    title: 'Accessibility Statement',
    category: 'Legal',
    snippet: 'WCAG 2.1 AA accessibility conformance commitment, keyboard navigation, and contrast specifications.',
    route: '#/accessibility',
  },
];
