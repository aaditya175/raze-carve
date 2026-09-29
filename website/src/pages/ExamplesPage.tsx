import React from 'react';
import { RoutePath } from '../types';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { CodeBlock } from '../components/CodeBlock';
import { Callout } from '../components/Callout';
import { Button } from '../components/Button';
import { Terminal, Shield, ArrowRight } from 'lucide-react';

interface ExamplesPageProps {
  onNavigate: (path: RoutePath) => void;
}

export const ExamplesPage: React.FC<ExamplesPageProps> = ({ onNavigate }) => {
  return (
    <div className="max-w-5xl mx-auto py-6 space-y-12">
      <div>
        <Breadcrumbs crumbs={[{ label: 'Product' }, { label: 'Operational Workflows' }]} />
        <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono mt-2">
          Real-World Forensic & Sanitization Workflows
        </h1>
        <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-2 max-w-3xl">
          Practical step-by-step terminal guides for enterprise decommissioning, incident response carving, and defense compliance sanitization.
        </p>
      </div>

      {/* Workflow 1 */}
      <section className="p-6 sm:p-8 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-4">
        <div className="text-xs font-mono font-semibold uppercase text-brand-600 dark:text-brand-400">
          Workflow Scenario A
        </div>
        <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100 font-mono">
          Enterprise Cloud Server Decommissioning (NVMe Crypto Erase)
        </h2>
        <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">
          When returning leased cloud bare-metal servers or retiring high-speed enterprise NVMe SSDs, software multi-pass overwrites cause excessive write wear and take hours. Using RAZE-CARVE, you trigger an onboard cryptographic erase that scrambles the Media Encryption Key in under a second.
        </p>

        <CodeBlock
          language="bash"
          code={`# 1. Enumerate and identify target NVMe device
sudo sih149 info /dev/nvme1n1

# 2. Issue hardware cryptographic erase
sudo sih149 firmware-erase --target /dev/nvme1n1 --type crypto

# 3. Verify zero-return on sampled LBAs and export signed PDF certificate
sudo sih149 wipe --target /dev/nvme1n1 --method nist-purge --audit-out /var/log/nvme_decom.pdf`}
        />

        <Callout type="tip" title="Controller Verification">
          NVMe Crypto Erase causes the internal microcontroller to generate a new random 256-bit AES Media Encryption Key. All historical ciphertext stored on the 3D NAND flash cells immediately becomes mathematically indecipherable.
        </Callout>
      </section>

      {/* Workflow 2 */}
      <section className="p-6 sm:p-8 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-4">
        <div className="text-xs font-mono font-semibold uppercase text-brand-600 dark:text-brand-400">
          Workflow Scenario B
        </div>
        <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100 font-mono">
          Post-Incident Response File Carving from Damaged Disk Image
        </h2>
        <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">
          In digital forensics investigations where an adversary deleted logs or damaged filesystem partition tables (e.g. wiped superblock), use the RAZE-CARVE file carver to scan raw sector streams for critical database, document, and image artifacts.
        </p>

        <CodeBlock
          language="bash"
          code={`# Create isolated output directory
mkdir -p /mnt/forensic_cases/case_042/carved

# Carve SQLite databases, PDFs, and compressed archives
sih149 recover \\
  --source /mnt/images/compromised_host.raw \\
  --out-dir /mnt/forensic_cases/case_042/carved \\
  --types sqlite,pdf,zip,docx \\
  --max-size-mb 500`}
        />
      </section>

      {/* Workflow 3 */}
      <section className="p-6 sm:p-8 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 space-y-4">
        <div className="text-xs font-mono font-semibold uppercase text-brand-600 dark:text-brand-400">
          Workflow Scenario C
        </div>
        <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100 font-mono">
          Defense Media Sanitization (DoD 5220.22-M 3-Pass)
        </h2>
        <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">
          For defense contractors and classified systems decommissioning magnetic hard disk drives, the DoD 5220.22-M standard mandates a 3-pass overwrite sequence: fixed characters, inverted bitwise complements, and pseudo-random noise.
        </p>

        <CodeBlock
          language="bash"
          code={`# Execute DoD 5220.22-M 3-Pass with live terminal telemetry
sudo sih149 wipe \\
  --target /dev/sdc \\
  --method dod-3pass \\
  --verify \\
  --audit-out /var/log/dod_wipe_sdc.json`}
        />
      </section>
    </div>
  );
};
