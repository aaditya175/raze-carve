export type RoutePath =
  | '/'
  | '/sandbox'
  | '/features'
  | '/docs'
  | '/commands'
  | '/downloads'
  | '/examples'
  | '/changelog'
  | '/about'
  | '/gui'
  | '/tui'
  | '/cli'
  | '/privacy'
  | '/terms'
  | '/cookies'
  | '/security'
  | '/accessibility';

export interface ForensicOperation {
  id: string;
  projectId: string;
  name: string;
  targetDevice: string;
  deviceModel: string;
  capacity: string;
  interfaceType: 'NVMe' | 'SATA' | 'SAS' | 'USB' | 'eMMC';
  complianceStandard: string;
  type: 'sanitization' | 'carving' | 'entropy' | 'firmware';
  progress: number;
  tasksCompleted: number;
  tasksTotal: number;
  status: 'in_progress' | 'completed' | 'queued' | 'verifying';
  nextMilestone: string;
  nextMilestoneDate: string;
  entropyScore?: number;
  sha256Digest?: string;
  recoveredFilesCount?: number;
  auditVerified: boolean;
}

export interface CommandOption {
  flag: string;
  short?: string;
  description: string;
  defaultValue?: string;
  required?: boolean;
}

export interface CommandArgument {
  name: string;
  required: boolean;
  description: string;
}

export interface CommandSpec {
  id: string;
  name: string;
  binary: 'sih149' | 'sforge' | 'raze-carve-desktop' | 'secureforge-desktop';
  summary: string;
  purpose: string;
  syntax: string;
  arguments: CommandArgument[];
  options: CommandOption[];
  examples: {
    title: string;
    command: string;
    description: string;
    expectedOutput: string;
  }[];
  commonMistakes: string[];
  relatedCommands: string[];
}

export interface DocSection {
  id: string;
  title: string;
  category: 'Getting Started' | 'Guides' | 'Reference' | 'Development' | 'Troubleshooting';
  content: string;
}

export interface DownloadArtifact {
  id: string;
  title: string;
  category: 'iso' | 'docker' | 'binaries' | 'source';
  filename: string;
  version: string;
  size: string;
  sha256?: string;
  downloadUrl?: string;
  description: string;
  command?: string;
  instructions: string[];
}

export interface FAQItem {
  question: string;
  answer: string;
  category: 'Sanitization' | 'Carving' | 'Hardware' | 'Compliance';
}

export interface SearchResult {
  id: string;
  title: string;
  category: string;
  snippet: string;
  route: string;
}
