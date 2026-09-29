export type DriveType = 'HDD' | 'SSD' | 'NVMe' | 'USB' | 'ZRAM' | 'LOOP' | 'Unknown';
export type SmartStatus = 'Healthy' | 'Warning' | 'Critical' | 'Unknown';

export interface DriveInfo {
  name: string;
  path: string;
  size_bytes: number;
  model: string;
  serial: string;
  drive_type: DriveType;
  is_mounted: boolean;
  mount_points: string[];
  is_system_drive: boolean;
  smart_status: SmartStatus;
}

export type WipeMethod =
  | 'zero'
  | 'random'
  | 'dod3'
  | 'dod7'
  | 'gutmann'
  | 'nvme-crypto'
  | 'nvme-block'
  | 'ata'
  | 'ata-enhanced';

export interface WipeConfig {
  device_path: string;
  method: string;
  verify: boolean;
  expert_passphrase?: string;
}

export interface WipeProgress {
  sector_current: number;
  sector_total: number;
  percent: number;
  speed_mbps: number;
  eta_seconds: number;
  phase: string;
}

export interface WipeResult {
  success: boolean;
  sectors_wiped: number;
  bad_sectors: number;
  duration_secs: number;
  method_used: string;
  verified: boolean;
}

export interface ShredConfig {
  paths: string[];
  passes: number;
  renames: number;
  scrub_slack: boolean;
}

export interface ShredProgress {
  current_file: string;
  files_done: number;
  files_total: number;
  percent: number;
}

export interface ShredFileResult {
  path: string;
  bytes_wiped: number;
  passes_completed: number;
  success: boolean;
  is_cow?: boolean;
  error?: string;
}

export interface ShredResult {
  total_files: number;
  total_bytes: number;
  failed_files: number;
  results: ShredFileResult[];
}

export interface ScanConfig {
  source_path: string;
  output_dir: string;
  file_types: string[];
  min_confidence: number;
  recovery_modes?: string[];
  dry_run?: boolean;
}

export interface ExtractRequest {
  source_path: string;
  output_dir: string;
  files: CarvedFile[];
}

export interface ExtractResult {
  extracted_count: number;
  total_size_bytes: number;
  files: string[];
}

export interface CarvedFile {
  id: string;
  filename: string;
  file_type: string;
  size_bytes: number;
  confidence: number;
  offset_bytes: number;
  category: string;
  recovery_method?: string;
  original_metadata?: string;
}

export interface ScanProgress {
  sector_current: number;
  sector_total: number;
  percent: number;
  files_found: number;
  speed_mbps: number;
}

export interface ScanResult {
  total_files: number;
  total_size_bytes: number;
  duration_secs: number;
  entropy_heatmap: number[];
  files: CarvedFile[];
}

export interface FirmwareCapabilities {
  is_nvme: boolean;
  nvme_sanitize_supported: boolean;
  ata_frozen: boolean;
  hpa_enabled: boolean;
  dco_enabled: boolean;
  recommended_method: string;
  warnings: string[];
}

export interface FirmwareEraseConfig {
  device_path: string;
  method: string;
  ata_password?: string;
  expert_passphrase?: string;
}

export interface FirmwareEraseResult {
  method_used: string;
  success: boolean;
  command_output: string;
  duration_secs: number;
  warnings: string[];
  password_used?: string;
  case_id?: string;
}

export interface CaseRecord {
  id: string;
  created_at: string;
  operation_type: string;
  target: string;
  status: string;
}

export interface PluginItem {
  name: string;
  category: string;
  plugin_type: string;
  extension: string;
  has_validator: boolean;
  status: string;
  description: string;
}

export interface DiagnosticResult {
  name: string;
  category: string;
  passed: boolean;
  message: string;
  duration_us: number;
}

export interface DiagnosticSummary {
  total: number;
  passed: number;
  failed: number;
  total_duration_ms: number;
  results: DiagnosticResult[];
}

export type SectorHealth = 'healthy' | 'normal' | 'slow' | 'bad' | 'reallocated' | 'zeroed';
export type ClusterAllocation = 'unknown' | 'system_mft' | 'active_file' | 'fragmented' | 'carvable_deleted' | 'free_space' | 'wiped' | 'bad_sector';

export interface SectorCell {
  index: number;
  lba: number;
  offset_bytes: number;
  health: SectorHealth;
  latency_ms: number;
  entropy: number;
  allocation: ClusterAllocation;
  file_hint?: string;
}

export interface BadSectorDefect {
  lba: number;
  offset_bytes: number;
  error_code: string;
  message: string;
}

export interface SurfaceScanResult {
  device_path: string;
  total_size_bytes: number;
  total_blocks: number;
  sample_block_size: number;
  cells: SectorCell[];
  good_count: number;
  slow_count: number;
  bad_count: number;
  reallocated_count: number;
  zeroed_count: number;
  health_score: number;
  defects: BadSectorDefect[];
  scan_duration_ms: number;
}

