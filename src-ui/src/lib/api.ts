import { isTauri, invoke } from '@tauri-apps/api/core';
import {
  DriveInfo,
  WipeConfig,
  WipeProgress,
  WipeResult,
  ShredConfig,
  ShredProgress,
  ShredResult,
  ScanConfig,
  ScanProgress,
  ScanResult,
  ExtractRequest,
  ExtractResult,
  FirmwareCapabilities,
  FirmwareEraseConfig,
  FirmwareEraseResult,
  CaseRecord,
  PluginItem,
  DiagnosticSummary,
  SurfaceScanResult,
} from './types';

// Runtime detection of Tauri desktop environment
export function checkIsTauri(): boolean {
  try {
    if (typeof window === 'undefined') return false;
    return isTauri() || '__TAURI_INTERNALS__' in window || '__TAURI__' in window;
  } catch {
    return false;
  }
}

export function getSavedToken(): string {
  if (typeof localStorage !== 'undefined') {
    const stored = localStorage.getItem('razecarve_api_token') || localStorage.getItem('secureforge_api_token');
    if (stored) return stored;
  }
  if (typeof sessionStorage !== 'undefined') {
    return sessionStorage.getItem('razecarve_api_token') || sessionStorage.getItem('secureforge_api_token') || '';
  }
  return '';
}

export function saveToken(token: string, remember: boolean = true) {
  const trimmed = token.trim();
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.setItem('razecarve_api_token', trimmed);
  sessionStorage.setItem('secureforge_api_token', trimmed);
  }
  if (remember && typeof localStorage !== 'undefined') {
    localStorage.setItem('razecarve_api_token', trimmed);
    localStorage.setItem('secureforge_api_token', trimmed);
  }
}

export function clearToken() {
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.removeItem('razecarve_api_token');
    sessionStorage.removeItem('secureforge_api_token');
  }
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem('razecarve_api_token');
    localStorage.removeItem('secureforge_api_token');
  }
}

async function safeFetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  let token = getSavedToken();

  // If no token is saved and we are running in browser on localhost, auto-discover token
  if (!token && typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
    try {
      const autoRes = await fetch('/api/auth/token');
      if (autoRes.ok) {
        const autoJson = await autoRes.json();
        if (autoJson.token) {
          token = autoJson.token;
          saveToken(token, true);
        }
      }
    } catch {
      // ignore
    }
  }

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string> || {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  let res: Response;
  try {
    res = await fetch(url, { ...options, headers });
  } catch (netErr: any) {
    throw new Error(`Failed to connect to RAZE-CARVE backend server: ${netErr.message || netErr}`);
  }

  if (res.status === 401 && typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
    // Retry once with auto-discovered token
    try {
      const autoRes = await fetch('/api/auth/token');
      if (autoRes.ok) {
        const autoJson = await autoRes.json();
        if (autoJson.token) {
          token = autoJson.token;
          saveToken(token, true);
          headers['Authorization'] = `Bearer ${token}`;
          res = await fetch(url, { ...options, headers });
        }
      }
    } catch {
      // ignore
    }
  }

  if (res.status === 401) {
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('raze-carve-auth-required'));
    }
    throw new Error('Unauthorized: Missing or invalid Bearer token.');
  }

  const contentType = res.headers.get('content-type') || '';
  if (!contentType.includes('application/json')) {
    if (!res.ok) {
      const txt = await res.text();
      throw new Error(`Server returned HTTP ${res.status}: ${txt.slice(0, 100)}`);
    }
    throw new Error(
      `Unexpected server response (${contentType || 'HTML'}). Ensure RAZE-CARVE backend server is running on http://127.0.0.1:7878`
    );
  }

  const json = await res.json();
  if (!res.ok) {
    throw new Error(json.message || `Request failed with HTTP status ${res.status}`);
  }
  return json as T;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // 1. Direct Tauri IPC in desktop application
  if (checkIsTauri()) {
    return invoke<T>(cmd, args);
  }

  // 2. HTTP REST fallback when running in standard browser
  const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:7878';

  try {
    switch (cmd) {
      case 'list_drives': {
        const json: any = await safeFetchJson(`${baseUrl}/api/drives`);
        if (json.status === 'success') return json.data as T;
        throw new Error(json.message || 'Failed to query system drives');
      }

      case 'start_wipe': {
        const job: any = await safeFetchJson(`${baseUrl}/api/wipe`, {
          method: 'POST',
          body: JSON.stringify(args?.config || args),
        });
        const jobId = job?.job_id;
        if (!jobId) {
          throw new Error(job?.message || 'Wipe job was not accepted');
        }
        return (await pollJob<WipeResult>(jobId, 'wipe-progress')) as T;
      }

      case 'start_scan': {
        // The web backend routes carving through a job queue, so POST creates
        // a job and we poll /api/jobs/:id until it completes, then return a
        // ScanResult-shaped object so the UI works identically in web and
        // Tauri modes (the Tauri command returns a synchronous ScanResult).
        const cfg = (args?.config || args || {}) as {
          source_path?: string;
          output_dir?: string;
          file_types?: string[];
          min_confidence?: number;
          recovery_modes?: string[];
          dry_run?: boolean;
        };
        const job: any = await safeFetchJson(`${baseUrl}/api/scan`, {
          method: 'POST',
          body: JSON.stringify({
            source_path: cfg.source_path,
            output_dir: cfg.output_dir,
            file_types: cfg.file_types,
            min_confidence: cfg.min_confidence,
            recovery_modes: cfg.recovery_modes,
            dry_run: cfg.dry_run,
          }),
        });
        const jobId = job?.job_id;
        if (!jobId) {
          throw new Error(job?.message || 'Scan job was not accepted');
        }
        const scanResult = await pollJob(jobId, 'scan-progress');
        return (scanResult as unknown) as T;
      }

      case 'extract_selected_files': {
        const req = (args?.request || args || {}) as ExtractRequest;
        const json: any = await safeFetchJson(`${baseUrl}/api/scan/extract`, {
          method: 'POST',
          body: JSON.stringify(req),
        });
        return json as T;
      }

      case 'list_cases': {
        const json: any = await safeFetchJson(`${baseUrl}/api/cases`);
        return (json.data || []) as T;
      }

      case 'list_plugins': {
        const json: any = await safeFetchJson(`${baseUrl}/api/plugins`);
        return (json.data || []) as T;
      }

      case 'get_drive_entropy': {
        // The REST endpoint expects snake_case field names (device_path).
        const json: any = await safeFetchJson(`${baseUrl}/api/entropy`, {
          method: 'POST',
          body: JSON.stringify({
            device_path: args?.devicePath,
            chunks: args?.chunks,
          }),
        });
        return (json.data || []) as T;
      }

      case 'get_file_hex_preview': {
        const json: any = await safeFetchJson(`${baseUrl}/api/hex`, {
          method: 'POST',
          body: JSON.stringify({
            file_path: args?.filePath,
            offset: args?.offset,
            length: args?.length,
            allowed_root: args?.allowedRoot || null,
          }),
        });
        return (json.data || '') as T;
      }

      case 'get_audit_log': {
        const caseId = args?.caseId || args?.case_id;
        const json: any = await safeFetchJson(`${baseUrl}/api/cases/${caseId}/audit`);
        const data = json?.data;
        return (typeof data === 'string' ? data : JSON.stringify(data ?? json)) as T;
      }

      case 'get_drive_info': {
        const json: any = await safeFetchJson(`${baseUrl}/api/drives`);
        if (json.status === 'success' && Array.isArray(json.data)) {
          const match = json.data.find((d: DriveInfo) => d.path === args?.devicePath);
          if (match) return match as T;
        }
        throw new Error(`Device ${args?.devicePath} not found`);
      }

      case 'cancel_wipe': {
        await safeFetchJson(`${baseUrl}/api/wipe/cancel`, { method: 'POST' });
        return undefined as T;
      }

      case 'cancel_scan': {
        await safeFetchJson(`${baseUrl}/api/scan/cancel`, { method: 'POST' });
        return undefined as T;
      }

      case 'is_expert_configured': {
        const json: any = await safeFetchJson(`${baseUrl}/api/auth/status`);
        return (!!json.is_configured) as T;
      }

      case 'setup_expert_passphrase': {
        await safeFetchJson(`${baseUrl}/api/auth/setup`, {
          method: 'POST',
          body: JSON.stringify({ passphrase: args?.passphrase }),
        });
        return undefined as T;
      }

      case 'verify_expert_passphrase': {
        const json: any = await safeFetchJson(`${baseUrl}/api/auth/verify`, {
          method: 'POST',
          body: JSON.stringify({ passphrase: args?.passphrase }),
        });
        return (!!json.valid) as T;
      }

      case 'detect_firmware_capabilities': {
        const json: any = await safeFetchJson(`${baseUrl}/api/firmware/detect`, {
          method: 'POST',
          body: JSON.stringify({ device_path: args?.devicePath }),
        });
        return json.data as T;
      }

      case 'start_firmware_erase': {
        const json: any = await safeFetchJson(`${baseUrl}/api/firmware/erase`, {
          method: 'POST',
          body: JSON.stringify(args?.config || args),
        });
        return json.data as T;
      }

      case 'scan_drive_sectors': {
        const json: any = await safeFetchJson(`${baseUrl}/api/sectors/scan`, {
          method: 'POST',
          body: JSON.stringify({
            device_path: args?.devicePath,
            target_cells: args?.targetCells,
          }),
        });
        return json.data as T;
      }

      case 'estimate_wipe_time': {
        return 60 as T;
      }

      case 'shred_files': {
        const json: any = await safeFetchJson(`${baseUrl}/api/shred`, {
          method: 'POST',
          body: JSON.stringify(args?.config || args),
        });
        return (json?.data || json) as T;
      }

      case 'cancel_shred': {
        const json: any = await safeFetchJson(`${baseUrl}/api/shred/cancel`, { method: 'POST' });
        return (json?.data || json) as T;
      }

      case 'export_report': {
        const json: any = await safeFetchJson(`${baseUrl}/api/reports/export`, {
          method: 'POST',
          body: JSON.stringify(args),
        });
        if (typeof json === 'string') return json as T;
        return (json?.path || json?.data || json?.message || json) as T;
      }

      default:
        throw new Error(`Command '${cmd}' requires active desktop runtime or mapped REST route.`);
    }
  } catch (err: any) {
    console.warn(`[RAZE-CARVE API] ${cmd} error:`, err);
    throw err;
  }
}

const webListeners = new Map<string, Set<(payload: any) => void>>();

export function dispatchWebEvent(event: string, payload: any) {
  const listeners = webListeners.get(event);
  if (listeners) {
    listeners.forEach((cb) => {
      try {
        cb(payload);
      } catch (err) {
        console.error(`[RAZE-CARVE Web] Error in ${event} listener:`, err);
      }
    });
  }
}

async function tauriListen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (checkIsTauri()) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen<T>(event, (e) => handler(e.payload));
    return () => {
      unlisten();
    };
  }

  // Fallback for Web/REST mode
  let set = webListeners.get(event);
  if (!set) {
    set = new Set();
    webListeners.set(event, set);
  }
  set.add(handler as (payload: any) => void);

  return () => {
    set.delete(handler as (payload: any) => void);
  };
}

async function pollJob<T>(jobId: string, eventName?: string): Promise<T> {
  const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:7878';
  const deadline = Date.now() + 1_800_000; // 30 minutes timeout for forensic operations
  while (Date.now() < deadline) {
    const job: any = await safeFetchJson(`${baseUrl}/api/jobs/${jobId}`);
    if (job?.progress && eventName) {
      dispatchWebEvent(eventName, job.progress);
    }
    if (job?.status === 'done' || job?.status === 'completed') {
      return (job?.result || job) as T;
    }
    if (job?.status === 'failed' || job?.status === 'error') {
      throw new Error(job?.message || 'Forensic operation failed');
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error('Forensic operation timed out');
}

export const DrivesAPI = {
  list: (): Promise<DriveInfo[]> => tauriInvoke('list_drives'),
  getInfo: (devicePath: string): Promise<DriveInfo> => tauriInvoke('get_drive_info', { devicePath }),
};

export const WiperAPI = {
  start: (config: WipeConfig): Promise<WipeResult> => tauriInvoke('start_wipe', { config }),
  cancel: (): Promise<void> => tauriInvoke('cancel_wipe'),
  estimateTime: (devicePath: string, method: string): Promise<number> =>
    tauriInvoke('estimate_wipe_time', { devicePath, method }),
  onProgress: (cb: (p: WipeProgress) => void) => tauriListen<WipeProgress>('wipe-progress', cb),
};

export const ShredderAPI = {
  shred: (config: ShredConfig): Promise<ShredResult> => tauriInvoke('shred_files', { config }),
  onProgress: (cb: (p: ShredProgress) => void) => tauriListen<ShredProgress>('shred-progress', cb),
};

export const CarverAPI = {
  start: (config: ScanConfig): Promise<ScanResult> => tauriInvoke('start_scan', { config }),
  cancel: (): Promise<void> => tauriInvoke('cancel_scan'),
  extractSelected: (request: ExtractRequest): Promise<ExtractResult> =>
    tauriInvoke('extract_selected_files', { request }),
  getHexPreview: (filePath: string, offset: number, length: number, allowedRoot?: string): Promise<string> =>
    tauriInvoke('get_file_hex_preview', { filePath, offset, length, allowedRoot }),
  onProgress: (cb: (p: ScanProgress) => void) => tauriListen<ScanProgress>('scan-progress', cb),
};

export const FirmwareAPI = {
  detect: (devicePath: string): Promise<FirmwareCapabilities> =>
    tauriInvoke('detect_firmware_capabilities', { devicePath }),
  erase: (config: FirmwareEraseConfig): Promise<FirmwareEraseResult> =>
    tauriInvoke('start_firmware_erase', { config }),
};

export const AuthAPI = {
  isConfigured: (): Promise<boolean> => tauriInvoke('is_expert_configured'),
  setup: (passphrase: string): Promise<void> => tauriInvoke('setup_expert_passphrase', { passphrase }),
  verify: (passphrase: string): Promise<boolean> => tauriInvoke('verify_expert_passphrase', { passphrase }),
};

export const ReportsAPI = {
  list: (): Promise<CaseRecord[]> => tauriInvoke('list_cases'),
  export: (caseId: string, format: string, outputPath: string, investigator?: string): Promise<string> =>
    tauriInvoke('export_report', { caseId, format, outputPath, investigator }),
  getAuditLog: (caseId: string): Promise<string> => tauriInvoke('get_audit_log', { caseId }),
};

export const PluginsAPI = {
  list: (): Promise<PluginItem[]> => tauriInvoke('list_plugins'),
};

export const EntropyAPI = {
  get: (devicePath: string, chunks?: number): Promise<number[]> =>
    tauriInvoke('get_drive_entropy', { devicePath, chunks }),
};

export const DiagnosticsAPI = {
  run: async (): Promise<DiagnosticSummary> => {
    if (checkIsTauri()) {
      return tauriInvoke<DiagnosticSummary>('run_system_diagnostics');
    }
    return safeFetchJson<DiagnosticSummary>('/api/diagnostics');
  },
};

export const SectorMapAPI = {
  scan: async (devicePath: string, targetCells?: number): Promise<SurfaceScanResult> => {
    if (checkIsTauri()) {
      return tauriInvoke<SurfaceScanResult>('scan_drive_sectors', { devicePath, targetCells });
    }
    const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:7878';
    const json: any = await safeFetchJson(`${baseUrl}/api/sectors/scan`, {
      method: 'POST',
      body: JSON.stringify({ device_path: devicePath, target_cells: targetCells }),
    });
    return json.data as SurfaceScanResult;
  },
};
