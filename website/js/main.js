/**
 * SecureForge Enterprise Forensic Website
 * Interactive Client Scripts
 */

document.addEventListener('DOMContentLoaded', () => {
  initTerminalTabs();
  initDownloadTabs();
  initCopyButtons();
  initHardwareSearch();
  initEntropySimulator();
});

/* ── Interactive Terminal Tabs in Hero ───────────────────────────────────── */
const terminalOutputs = {
  tui: `<span class="t-cyan">┌────────────────────────────────────────────────────────────────────────┐</span>
<span class="t-cyan">│</span>  <span class="t-bold">🔒 SECUREFORGE :: FORENSIC SANITIZATION & RECOVERY ENGINE</span>        <span class="t-cyan">│</span>
<span class="t-cyan">│</span>  <span class="t-gray">v0.1.0  ●  NIST SP 800-88 Rev 1  ●  DoD 5220.22-M  ●  RFC 3161 Certified</span><span class="t-cyan">│</span>
<span class="t-cyan">└────────────────────────────────────────────────────────────────────────┘</span>

<span class="t-bold">Attached Block Devices & Target Triage:</span>
  <span class="t-cyan">▸ /dev/nvme0n1</span>  Samsung 990 PRO 2TB     [NVMe 2.0]  <span class="t-green">ACTIVE (Locked System)</span>
  <span class="t-green">▸ /dev/sdb</span>      Kingston DataTraveler   [USB 3.2]   <span class="t-yellow">FORENSIC TARGET (Ready)</span>

<span class="t-bold">Select Active Subsystem:</span>
  [1] <span class="t-bold">Sanitization Engine</span>    (NIST Purge / DoD 5220.22-M / NVMe Cryptographic)
  [2] <span class="t-bold">Deep File Carver</span>       (Structural Chunk Parser: PNG/JPEG/SQLite/PDF)
  [3] <span class="t-bold">Entropy Heatmap</span>        (Real-time 0.000 - 7.999 b/B Sector Scanner)
  [4] <span class="t-bold">Chain-of-Custody</span>       (SHA-256 Merkle Ledger & RFC 3161 Timestamp)
  [5] <span class="t-bold">System Diagnostics</span>     (23 Subsystem Live Hardware Self-Tests)

<span class="t-gray">Navigate: [↑/↓] Select  [Enter] Launch Operation  [Tab] Toggle Pane  [Q] Exit</span>`,

  wipe: `<span class="t-gray">$</span> <span class="t-bold">sih149 wipe --device /dev/sdb --method dod3 --verify</span>

<span class="t-cyan">▸ Initializing DoD 5220.22-M (3-Pass) Sanitization Pipeline...</span>
  Target Storage: <span class="t-bold">/dev/sdb</span> (Kingston DataTraveler, 64 GB)
  Safety Guard:   <span class="t-green">PASS</span> (Target is not mounted, not system root, not swap)

  <span class="t-yellow">● Pass 1/3</span>: Writing Deterministic Zero Stream (0x00)...  <span class="t-green">[100%] 212 MB/s</span>
  <span class="t-yellow">● Pass 2/3</span>: Writing Deterministic Inverted (0xFF)...     <span class="t-green">[100%] 215 MB/s</span>
  <span class="t-yellow">● Pass 3/3</span>: Writing CSPRNG Keyed Cryptographic Stream... <span class="t-green">[100%] 208 MB/s</span>

<span class="t-cyan">▸ Hardware Verification & Sampling:</span>
  Surface Byte Verification: <span class="t-green">100% PASS (Zero remnant pattern detected)</span>
  Post-Wipe Shannon Entropy: <span class="t-bold">7.998 bits/byte</span> (Keyed CSPRNG confirmed)
  Slack Space Tail Scrubbed: <span class="t-green">✓ Enforced at hardware cluster boundary</span>

<span class="t-green">✔ [PASS] Case Certificate Generated:</span> <span class="t-bold">cases/wipe-178956/certificate.pdf</span>
  Audit Digest: <span class="t-cyan">9f83a21...b847ec0</span> (Bound to RFC 3161 Time-Stamp Token)`,

  recover: `<span class="t-gray">$</span> <span class="t-bold">sih149 recover --source /dev/sdb --out-dir /mnt/evidence</span>

<span class="t-cyan">▸ Initializing Deep Structure-Aware Carving Engine...</span>
  Source: <span class="t-bold">/dev/sdb</span>  ●  Signatures loaded: <span class="t-bold">16 Built-in + 4 Lua Plugins</span>

<span class="t-bold">Sector Scan Results & Structural Parsing:</span>
  <span class="t-green">✓ [0x00042000]</span> PNG Image   <span class="t-cyan">IHDR/IDAT/IEND</span>  100% Boundary (1,482,912 B)
  <span class="t-green">✓ [0x00084000]</span> JPEG Photo  <span class="t-cyan">SOI/APP0/EOI</span>    100% Boundary (3,211,048 B)
  <span class="t-green">✓ [0x00128000]</span> SQLite DB   <span class="t-cyan">Page Count 64</span>   100% Boundary (262,144 B)
  <span class="t-green">✓ [0x0019C000]</span> PDF Doc     <span class="t-cyan">%PDF..%%EOF</span>     100% Boundary (892,104 B)

<span class="t-bold">Selective Extraction Summary:</span>
  Extracted: <span class="t-bold">4 artifacts</span>  ●  Validation: <span class="t-green">100% Structurally Valid</span>
  SHA-256 Merkle Ledger: <span class="t-cyan">e3b0c44...b855</span> (Committed to SQLite WAL)
  Court Certificate:     <span class="t-bold">cases/rec-9182/recovery_certificate.pdf</span>`,

  demo: `<span class="t-gray">$</span> <span class="t-bold">sih149 demo</span>

<span class="t-cyan">▸ SecureForge Live Autonomous Forensic Demo</span>
  Workspace: <span class="t-gray">/tmp/secureforge-demo-1789561310</span>

  <span class="t-cyan">•</span> Synthetic evidence generated: <span class="t-bold">8192 B</span> (16 sectors of 512 B)
    ├─ Offset 0x0200 (Sector 1):  PNG graphic  (45 B) -> SHA-256: 6422564120b76bbc
    ├─ Offset 0x0400 (Sector 2):  JPEG photo   (22 B) -> SHA-256: 473715a932e74c8d
    ├─ Offset 0x0600 (Sector 3):  Corrupt slack (pseudo-random high entropy)
    └─ Offset 0x0800 (Sector 4):  SQLite db   (512 B) -> SHA-256: 1aa233d9d7decc1c

<span class="t-bold">Sector Carving & Bit-Exact Verification:</span>
┌────────────┬────────┬───────────┬─────────────┬────────────────────────────────────────────────────────┐
│ ARTIFACT   │ OFFSET │ CARVED    │ INTEGRITY   │ SHA-256 MATCH VERIFICATION                             │
├────────────┼────────┼───────────┼─────────────┼────────────────────────────────────────────────────────┤
│ PNG Image  │ 0x0200 │     45 B  │ <span class="t-green">✓ 100% MATCH</span> │ 6422564120b76bbc...2abaad1cdbf85339                    │
│ JPEG Photo │ 0x0400 │     22 B  │ <span class="t-green">✓ 100% MATCH</span> │ 473715a932e74c8d...5207bb08d818df14                    │
│ SQLite DB  │ 0x0800 │    512 B  │ <span class="t-green">✓ 100% MATCH</span> │ 1aa233d9d7decc1c...b25c26665a71e3b0                    │
└────────────┴────────┴───────────┴─────────────┴────────────────────────────────────────────────────────┘

<span class="t-green">✔ [PASS] DEMO COMPLETED SUCCESSFULLY: 100% FORENSIC INTEGRITY VERIFIED</span>`
};

function initTerminalTabs() {
  const tabs = document.querySelectorAll('.terminal-tab-btn');
  const body = document.getElementById('terminal-content');
  if (!body) return;

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      const target = tab.dataset.tab;
      if (terminalOutputs[target]) {
        body.innerHTML = `<pre>${terminalOutputs[target]}</pre>`;
      }
    });
  });
}

/* ── Interactive Downloads Hub Tabs ──────────────────────────────────────── */
function initDownloadTabs() {
  const tabBtns = document.querySelectorAll('.dl-tab-btn');
  const panes = document.querySelectorAll('.download-pane');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.dataset.target;
      tabBtns.forEach(b => b.classList.remove('active'));
      panes.forEach(p => p.classList.remove('active'));

      btn.classList.add('active');
      const targetPane = document.getElementById(targetId);
      if (targetPane) {
        targetPane.classList.add('active');
      }
    });
  });
}

/* ── One-Click Copy-to-Clipboard ─────────────────────────────────────────── */
function initCopyButtons() {
  const buttons = document.querySelectorAll('.copy-btn');
  buttons.forEach(btn => {
    btn.addEventListener('click', () => {
      const textToCopy = btn.dataset.copy || btn.previousElementSibling?.textContent;
      if (!textToCopy) return;

      navigator.clipboard.writeText(textToCopy.trim()).then(() => {
        const originalText = btn.innerHTML;
        btn.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#10B981" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg> Copied!
        `;
        btn.style.borderColor = '#10B981';
        btn.style.color = '#34D399';

        setTimeout(() => {
          btn.innerHTML = originalText;
          btn.style.borderColor = '';
          btn.style.color = '';
        }, 2000);
      });
    });
  });
}

/* ── Hardware & Standards Search / Filter ────────────────────────────────── */
function initHardwareSearch() {
  const searchInput = document.getElementById('hardware-search');
  const table = document.getElementById('hardware-table');
  if (!searchInput || !table) return;

  searchInput.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase().trim();
    const rows = table.querySelectorAll('tbody tr');

    rows.forEach(row => {
      const text = row.textContent.toLowerCase();
      if (text.includes(query)) {
        row.style.display = '';
      } else {
        row.style.display = 'none';
      }
    });
  });
}

/* ── Live Entropy Visualizer Demo ────────────────────────────────────────── */
function initEntropySimulator() {
  const slider = document.getElementById('entropy-slider');
  const valDisplay = document.getElementById('entropy-val');
  const bar = document.getElementById('entropy-fill');
  const desc = document.getElementById('entropy-desc');
  if (!slider || !valDisplay || !bar || !desc) return;

  slider.addEventListener('input', () => {
    const val = parseFloat(slider.value).toFixed(3);
    valDisplay.textContent = `${val} b/B`;
    const pct = (val / 8.0) * 100;
    bar.style.width = `${pct}%`;

    if (val < 0.1) {
      bar.style.background = '#10B981';
      desc.textContent = 'Status: Complete Zero Overwrite (NIST SP 800-88 Clear). Flatline randomness.';
    } else if (val < 4.5) {
      bar.style.background = '#0EA5E9';
      desc.textContent = 'Status: Uncompressed Text / Structured Binary (Low to Moderate Entropy).';
    } else if (val < 7.5) {
      bar.style.background = '#F59E0B';
      desc.textContent = 'Status: Compressed Media File (JPEG, MP3, PNG, ZIP chunks).';
    } else {
      bar.style.background = '#EF4444';
      desc.textContent = 'Status: High-Entropy CSPRNG Overwrite or Encrypted Volume (BitLocker / LUKS).';
    }
  });
}
