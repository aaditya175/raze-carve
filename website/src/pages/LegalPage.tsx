import React from 'react';
import { RoutePath } from '../types';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { ShieldCheck, Lock, EyeOff, FileText, CheckCircle2 } from 'lucide-react';

interface LegalPageProps {
  section: 'privacy' | 'terms' | 'cookies' | 'security' | 'accessibility';
  onNavigate: (path: RoutePath) => void;
}

export const LegalPage: React.FC<LegalPageProps> = ({ section }) => {
  const content = {
    privacy: {
      title: 'Privacy Policy',
      subtitle: 'Zero Telemetry & Local Execution Guarantee',
      crumbs: [{ label: 'Legal' }, { label: 'Privacy Policy' }],
      body: (
        <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
          <div className="p-4 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-xs font-mono text-emerald-800 dark:text-emerald-300">
            SUMMARY: RAZE-CARVE does not collect, transmit, store, or sell any personal information or telemetry data. All processing occurs strictly on your local hardware.
          </div>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">1. Information We Do NOT Collect</h2>
            <p>
              RAZE-CARVE is an offline, air-gapped forensic utility. The software contains <strong>zero third-party tracking libraries, zero diagnostic analytics, zero telemetry beacons, and zero advertising SDKs</strong>.
            </p>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>No personal identification information (PII)</li>
              <li>No IP addresses or network identifiers</li>
              <li>No storage hardware serial numbers transmitted externally</li>
              <li>No file names, carved file contents, or forensic metadata transmitted externally</li>
            </ul>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">2. Local Storage Usage</h2>
            <p>
              This website uses standard browser <code className="font-mono text-xs bg-slate-100 dark:bg-slate-800 px-1 py-0.5 rounded">localStorage</code> solely to remember your color scheme preference (<code className="font-mono text-xs">light</code> or <code className="font-mono text-xs">dark</code>). No personal data is stored in your browser.
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">3. Third-Party Services</h2>
            <p>
              The static website assets and source repository are hosted via standard open-source repositories (GitHub). When you visit external links to GitHub, their independent privacy policies govern their network requests.
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">4. Security Contact</h2>
            <p>
              For privacy inquiries or technical questions regarding data flow within RAZE-CARVE, open an issue on our GitHub repository or contact the core maintainers.
            </p>
          </section>
        </div>
      ),
    },
    terms: {
      title: 'Terms of Service',
      subtitle: 'Open Source Software & Forensic Operational Terms',
      crumbs: [{ label: 'Legal' }, { label: 'Terms of Service' }],
      body: (
        <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">1. Acceptance of Terms</h2>
            <p>
              By downloading, compiling, or using RAZE-CARVE (including the CLI <code className="font-mono text-xs">sih149</code>, TUI <code className="font-mono text-xs">sforge</code>, Desktop Workstation, or Live Bootable ISO), you agree to be bound by these Terms of Service and the accompanying open-source licenses (Apache-2.0 / MIT).
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">2. Permitted Use & Operator Responsibility</h2>
            <p>
              RAZE-CARVE is a defense-grade storage destruction and forensic data recovery tool. Media sanitization operations (<code className="font-mono text-xs">wipe</code>, <code className="font-mono text-xs">shred</code>, <code className="font-mono text-xs">firmware-erase</code>) are <strong>irreversible</strong>.
            </p>
            <p>
              You represent and warrant that:
            </p>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>You own or have explicit legal authorization to sanitize or carve data from target storage devices.</li>
              <li>You have verified the physical block device identifier before issuing destructive overwrite commands.</li>
              <li>You will not use RAZE-CARVE for unauthorized destruction of evidence or in violation of applicable laws.</li>
            </ul>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">3. Disclaimer of Warranty</h2>
            <p className="font-mono text-xs bg-slate-100 dark:bg-slate-900 p-3 rounded border border-slate-200 dark:border-slate-800">
              THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
            </p>
          </section>
        </div>
      ),
    },
    cookies: {
      title: 'Cookie Statement',
      subtitle: 'Zero Tracking Cookies Policy',
      crumbs: [{ label: 'Legal' }, { label: 'Cookie Statement' }],
      body: (
        <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
          <div className="p-4 rounded-lg bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800 text-xs font-mono text-sky-800 dark:text-sky-300">
            NOTICE: This website does NOT use any tracking cookies, third-party cookies, or advertising cookies.
          </div>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">What Are Cookies?</h2>
            <p>
              Cookies are small text files stored by websites on your computer or mobile device. They are commonly used to remember preferences, analyze web traffic, or track user activity across sites.
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">Our Use of Browser Storage</h2>
            <p>
              We do not set any HTTP cookies. We only access browser <code className="font-mono text-xs">localStorage</code> to maintain your selected UI theme:
            </p>
            <table className="w-full text-left text-xs font-mono border-collapse my-3">
              <thead>
                <tr className="border-b border-slate-200 dark:border-slate-800 text-slate-500">
                  <th className="py-2">Key</th>
                  <th className="py-2">Type</th>
                  <th className="py-2">Purpose</th>
                  <th className="py-2">Duration</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800/60">
                <tr>
                  <td className="py-2 text-brand-600 dark:text-brand-400">razecarve-theme</td>
                  <td className="py-2">localStorage</td>
                  <td className="py-2 font-sans">Stores user preference ('light' | 'dark' | 'system')</td>
                  <td className="py-2">Persistent until cleared</td>
                </tr>
              </tbody>
            </table>
          </section>
        </div>
      ),
    },
    security: {
      title: 'Security & Vulnerability Disclosure',
      subtitle: 'Responsible Disclosure & Cryptographic Assurance',
      crumbs: [{ label: 'Legal' }, { label: 'Security Policy' }],
      body: (
        <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">Security Posture</h2>
            <p>
              RAZE-CARVE is written entirely in <strong>Rust</strong>, enforcing memory safety at compile time to eliminate buffer overflows, dangling pointers, and data races.
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">Vulnerability Reporting</h2>
            <p>
              If you discover a potential security vulnerability, cryptographic weakness, or unintended device interaction in RAZE-CARVE:
            </p>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>Do NOT disclose the issue publicly in an open GitHub issue.</li>
              <li>Submit a confidential report via GitHub Security Advisories or contact the maintainers directly.</li>
              <li>Include reproduction steps, storage controller model, and command invocations.</li>
              <li>The team commits to reviewing reports within 48 business hours.</li>
            </ul>
          </section>
        </div>
      ),
    },
    accessibility: {
      title: 'Accessibility Statement',
      subtitle: 'Commitment to Inclusive Design & WCAG 2.1 AA Compliance',
      crumbs: [{ label: 'Legal' }, { label: 'Accessibility' }],
      body: (
        <div className="space-y-6 text-sm text-slate-700 dark:text-slate-300 leading-relaxed font-sans">
          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">Conformance Status</h2>
            <p>
              RAZE-CARVE is committed to ensuring digital accessibility for people with disabilities. We continually improve the user experience and apply relevant accessibility standards targeting <strong>Web Content Accessibility Guidelines (WCAG) 2.1 Level AA</strong>.
            </p>
          </section>

          <section className="space-y-2">
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100 font-mono">Implemented Measures</h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs font-sans">
              <div className="p-3 rounded border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
                <strong>Keyboard Navigation</strong>: Every interactive control, modal dialog, and command tab is reachable via Tab, Shift+Tab, and Enter.
              </div>
              <div className="p-3 rounded border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
                <strong>High-Contrast Ratios</strong>: Text elements meet or exceed 4.5:1 contrast ratios in both Dark and Light color profiles.
              </div>
              <div className="p-3 rounded border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
                <strong>Skip Link</strong>: A hidden skip-to-content link is provided for screen reader and keyboard-only users.
              </div>
              <div className="p-3 rounded border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
                <strong>Reduced Motion</strong>: Respects OS-level <code className="font-mono text-xs">prefers-reduced-motion</code> settings.
              </div>
            </div>
          </section>
        </div>
      ),
    },
  };

  const active = content[section];

  return (
    <div className="max-w-4xl mx-auto py-6 space-y-8">
      <div>
        <Breadcrumbs crumbs={active.crumbs} />
        <h1 className="text-2xl sm:text-4xl font-extrabold text-slate-900 dark:text-slate-100 font-mono mt-2">
          {active.title}
        </h1>
        <p className="text-sm sm:text-base text-slate-600 dark:text-slate-300 mt-2">
          {active.subtitle}
        </p>
      </div>

      <div className="p-6 sm:p-8 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm">
        {active.body}
      </div>
    </div>
  );
};
