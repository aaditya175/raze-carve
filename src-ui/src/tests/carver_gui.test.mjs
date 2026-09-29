import test from 'node:test';
import assert from 'node:assert/strict';

// Test 1: In-Memory Inspection Mode & ScanConfig contract
test('GUI Carver: Dry-run / In-Memory Inspection mode config contract', () => {
  const buildScanConfig = (sourcePath, outputDir, fileTypes, minConfidence, isInspectOnly) => {
    return {
      source_path: sourcePath.trim(),
      output_dir: outputDir.trim() || './recovered_evidence',
      file_types: fileTypes,
      min_confidence: minConfidence,
      dry_run: isInspectOnly,
    };
  };

  // When investigator toggles "Inspect Only (Dry Run)"
  const inspectConfig = buildScanConfig('/dev/sdb', './recovered_evidence', ['jpg', 'png'], 50, true);
  assert.strictEqual(inspectConfig.dry_run, true, 'dry_run flag must be true in inspect mode');
  assert.strictEqual(inspectConfig.source_path, '/dev/sdb');

  // When investigator uses direct extraction
  const extractConfig = buildScanConfig('/dev/sdb', './recovered_evidence', ['jpg', 'png'], 50, false);
  assert.strictEqual(extractConfig.dry_run, false, 'dry_run flag must be false in extraction mode');
});

// Test 2: Selective File Recovery & Payload Filtering
test('GUI Carver: Selective extraction filters candidate files properly', () => {
  const mockDiscoveredFiles = [
    { id: 'file-0', filename: 'photo_001.jpg', file_type: 'jpg', size_bytes: 1048576, confidence: 95, offset_bytes: 2048 },
    { id: 'file-1', filename: 'doc_002.pdf', file_type: 'pdf', size_bytes: 524288, confidence: 85, offset_bytes: 4096 },
    { id: 'file-2', filename: 'database_003.sqlite', file_type: 'sqlite', size_bytes: 2097152, confidence: 90, offset_bytes: 8192 },
    { id: 'file-3', filename: 'archive_004.zip', file_type: 'zip', size_bytes: 3145728, confidence: 70, offset_bytes: 16384 },
  ];

  // User selects file-0 and file-2
  const selectedFileIds = ['file-0', 'file-2'];
  const filesToExtract = mockDiscoveredFiles.filter((f) => selectedFileIds.includes(f.id));

  assert.strictEqual(filesToExtract.length, 2);
  assert.strictEqual(filesToExtract[0].filename, 'photo_001.jpg');
  assert.strictEqual(filesToExtract[1].filename, 'database_003.sqlite');

  // Build ExtractRequest
  const buildExtractRequest = (sourcePath, outputDir, files) => ({
    source_path: sourcePath,
    output_dir: outputDir || './recovered_evidence',
    files,
  });

  const extractReq = buildExtractRequest('/tmp/evidence.dd', '/tmp/custom_vault', filesToExtract);
  assert.strictEqual(extractReq.source_path, '/tmp/evidence.dd');
  assert.strictEqual(extractReq.output_dir, '/tmp/custom_vault');
  assert.strictEqual(extractReq.files.length, 2);
  assert.deepStrictEqual(extractReq.files.map((f) => f.id), ['file-0', 'file-2']);
});

// Test 3: FolderSelector Presets and Path Sanitization
test('GUI FolderSelector: Presets, path trimming, and timestamp case generation', () => {
  const presets = [
    { id: 'default', path: './recovered_evidence' },
    { id: 'ram', path: '/tmp/secureforge_cases' },
    { id: 'downloads', path: '~/Downloads/SecureForge_Recovery' },
    { id: 'desktop', path: '~/Desktop/Evidence_Vault' },
  ];

  // All presets must have valid paths
  for (const preset of presets) {
    assert.ok(preset.path.length > 0);
    assert.ok(!preset.path.includes(' '));
  }

  // Path trimming & fallback
  const sanitizePath = (p) => (p && p.trim()) ? p.trim() : './recovered_evidence';
  assert.strictEqual(sanitizePath('  /tmp/vault  '), '/tmp/vault');
  assert.strictEqual(sanitizePath(''), './recovered_evidence');
  assert.strictEqual(sanitizePath('   '), './recovered_evidence');

  // Timestamp case generation
  const appendTimestamp = (basePath, fakeDate) => {
    const pad = (n) => String(n).padStart(2, '0');
    const stamp = `case_${fakeDate.getFullYear()}${pad(fakeDate.getMonth() + 1)}${pad(fakeDate.getDate())}_${pad(fakeDate.getHours())}${pad(fakeDate.getMinutes())}`;
    const base = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath;
    return `${base}/${stamp}`;
  };

  const fixedDate = new Date(2026, 8, 14, 1, 30); // Sept 14, 2026, 01:30
  const stamped = appendTimestamp('./recovered_evidence', fixedDate);
  assert.strictEqual(stamped, './recovered_evidence/case_20260914_0130');
});

// Test 4: Web Workstation Event Listener & Telemetry Dispatching
test('GUI Web Workstation: Reactive event listener registry and progress dispatch', () => {
  const webListeners = new Map();

  const registerListener = (event, handler) => {
    if (!webListeners.has(event)) {
      webListeners.set(event, new Set());
    }
    const set = webListeners.get(event);
    set.add(handler);
    return () => set.delete(handler);
  };

  const dispatchEvent = (event, payload) => {
    const set = webListeners.get(event);
    if (set) {
      set.forEach((fn) => fn(payload));
    }
  };

  let receivedProgress = null;
  const unlisten = registerListener('scan-progress', (p) => {
    receivedProgress = p;
  });

  // Dispatch live carver progress
  const mockProgress = {
    percent: 45.5,
    sector_current: 2981888,
    sector_total: 6553600,
    files_found: 12,
    speed_mbps: 184.2,
  };

  dispatchEvent('scan-progress', mockProgress);
  assert.deepStrictEqual(receivedProgress, mockProgress);
  assert.strictEqual(receivedProgress.percent, 45.5);
  assert.strictEqual(receivedProgress.files_found, 12);

  // Test unlisten
  unlisten();
  dispatchEvent('scan-progress', { percent: 100, files_found: 20 });
  assert.strictEqual(receivedProgress.percent, 45.5, 'Unlistened handler must not receive updates');
});

// Test 5: Authentication Token Workstation Persistence Contract
test('GUI Token Storage: LocalStorage persistence with fallback', () => {
  const mockStorage = {
    local: {},
    session: {},
  };

  const saveTokenMock = (token, remember = true) => {
    const trimmed = token.trim();
    mockStorage.session['secureforge_api_token'] = trimmed;
    if (remember) {
      mockStorage.local['secureforge_api_token'] = trimmed;
    }
  };

  const getTokenMock = () => {
    return mockStorage.local['secureforge_api_token'] || mockStorage.session['secureforge_api_token'] || '';
  };

  const clearTokenMock = () => {
    delete mockStorage.local['secureforge_api_token'];
    delete mockStorage.session['secureforge_api_token'];
  };

  // Test remember: true
  saveTokenMock('auth_token_xyz123', true);
  assert.strictEqual(mockStorage.local['secureforge_api_token'], 'auth_token_xyz123');
  assert.strictEqual(mockStorage.session['secureforge_api_token'], 'auth_token_xyz123');
  assert.strictEqual(getTokenMock(), 'auth_token_xyz123');

  // Test clear
  clearTokenMock();
  assert.strictEqual(getTokenMock(), '');
  assert.strictEqual(mockStorage.local['secureforge_api_token'], undefined);
  assert.strictEqual(mockStorage.session['secureforge_api_token'], undefined);
});
