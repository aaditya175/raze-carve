//! SecureForge optional web server mode
//!
//! Exposes the same operations as Tauri IPC via REST API over HTTP.
//! Allows browser-based access on a local network (forensic workstation).
//! Also serves the compiled React web interface (`src-ui/dist`).
//!
//! Usage:
//!   secureforge-desktop --server --port 7878
//!   secureforge-desktop --server --port 7878 --api-token mysecrettoken

use axum::{
    extract::{ConnectInfo, Path, State},
    http::{HeaderValue, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

/// Simple in-memory token-bucket rate limiter keyed by client IP.
/// Prevents unauthenticated/brute-force/burst attacks against the API.
struct RateLimiter {
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }

    fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        if buckets.len() > 500 {
            buckets.retain(|_, (_, start)| now.duration_since(*start) <= self.window);
        }
        match buckets.get_mut(key) {
            Some((count, start)) => {
                if now.duration_since(*start) > self.window {
                    *count = 1;
                    *start = now;
                    true
                } else if *count < self.max_requests {
                    *count += 1;
                    true
                } else {
                    false
                }
            }
            None => {
                buckets.insert(key.to_string(), (1, now));
                true
            }
        }
    }
}

type SharedRateLimiter = Arc<RateLimiter>;

/// Shared server state injected into every handler via `State<Arc<AppState>>`
#[allow(dead_code)]
pub struct AppState {
    pub host: String,
    pub port: u16,
    pub require_auth: bool,
    pub api_token: String,
    limiter: SharedRateLimiter,
    pub trust_proxy: bool,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct WipeRequest {
    device_path: String,
    method: String,
    verify: bool,
    expert_passphrase: Option<String>,
}

#[derive(Serialize)]
struct WipeResponse {
    job_id: String,
    status: String,
    message: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScanRequest {
    source_path: String,
    output_dir: String,
    file_types: Vec<String>,
    min_confidence: u8,
    #[serde(default)]
    recovery_modes: Option<Vec<String>>,
    #[serde(default)]
    dry_run: Option<bool>,
}

#[derive(Deserialize)]
struct EntropyRequest {
    device_path: String,
    chunks: Option<usize>,
}

#[derive(Deserialize)]
struct SectorScanRequest {
    device_path: String,
    target_cells: Option<usize>,
}

#[derive(Deserialize)]
struct HexRequest {
    file_path: String,
    offset: u64,
    length: usize,
    allowed_root: Option<String>,
}

// Bounded-lifetime in-memory job store so web-mode operations can be polled
// without retaining completed results for the lifetime of the server.
struct StoredJob {
    value: serde_json::Value,
    updated_at: Instant,
}

const JOB_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

lazy_static::lazy_static! {
    static ref SCAN_JOBS: Mutex<std::collections::HashMap<String, StoredJob>> =
        Mutex::new(std::collections::HashMap::new());
}

fn get_scan_job(job_id: &str) -> Option<serde_json::Value> {
    let now = Instant::now();
    let mut jobs = SCAN_JOBS.lock().unwrap_or_else(|e| e.into_inner());
    jobs.retain(|_, job| now.duration_since(job.updated_at) <= JOB_RETENTION);
    jobs.get(job_id).map(|job| job.value.clone())
}

fn set_scan_job(job_id: &str, value: serde_json::Value) {
    let now = Instant::now();
    let mut jobs = SCAN_JOBS.lock().unwrap_or_else(|e| e.into_inner());
    jobs.retain(|_, job| now.duration_since(job.updated_at) <= JOB_RETENTION);
    jobs.insert(
        job_id.to_string(),
        StoredJob {
            value,
            updated_at: now,
        },
    );
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Constant-time bearer token comparison to prevent timing side-channel attacks.
fn verify_token(state: &AppState, token: &str) -> bool {
    let expected = &state.api_token;
    if expected.is_empty() {
        return !state.require_auth;
    }
    // Constant-time comparison without leaking length via early return.
    // XOR all bytes, padding the shorter side with 0xFF (guaranteed mismatch).
    let a = expected.as_bytes();
    let b = token.as_bytes();
    let max_len = a.len().max(b.len());
    let mut diff: u8 = if a.len() != b.len() { 1 } else { 0 };
    for i in 0..max_len {
        let ai = a.get(i).copied().unwrap_or(0xFF);
        let bi = b.get(i).copied().unwrap_or(0xFF);
        diff |= ai ^ bi;
    }
    diff == 0
}

/// Authentication middleware for all `/api/*` routes (except `/health`).
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: Next,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let path = req.uri().path();
    // Only /health, /api/auth/token (localhost), and non-api static asset routes do not require Bearer header
    if path == "/health" || path == "/api/auth/token" || !path.starts_with("/api") {
        return Ok(next.run(req).await);
    }

    // Rate limit by the direct peer IP to blunt brute-force and burst attacks.
    // Only trust X-Forwarded-For when fronted by an explicit trusted reverse proxy.
    let client_ip = if state.trust_proxy {
        req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim())
            .filter(|s| !s.is_empty() && s.parse::<std::net::IpAddr>().is_ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "proxy_client".to_string())
    } else {
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|peer| peer.0.ip().to_string())
            .unwrap_or_else(|| "unknown_direct_client".to_string())
    };

    // Active job progress polling is lightweight and requires frequent polling without tripping rate limits
    let is_job_poll = path.starts_with("/api/jobs/");
    if !is_job_poll {
        let rate_key = format!("{}:{}", client_ip, path);
        if !state.limiter.allow(&rate_key) {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Rate limit exceeded. Try again shortly."
                })),
            ));
        }
    }

    if !state.require_auth {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    let header_token = auth_header
        .and_then(|h| {
            h.strip_prefix("Bearer ")
                .or_else(|| h.strip_prefix("bearer "))
        })
        .map(str::trim);

    let query_token = req.uri().query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            if parts.next()? == "token" {
                parts.next()
            } else {
                None
            }
        })
    });

    let token = header_token.or(query_token);

    match token {
        Some(t) if verify_token(&state, t) => Ok(next.run(req).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": "error",
                "message": "Unauthorized: Missing or invalid Bearer token."
            })),
        )),
    }
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /health — liveness probe
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "server"
    }))
}

/// GET /api/drives — list all block devices via lsblk
async fn get_drives(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::drives::list_drives() {
        Ok(drives) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": drives })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/wipe — enqueue a validated wipe job. Completion is available only
/// through the job endpoint; a 202 response never represents completed media.
async fn post_wipe(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WipeRequest>,
) -> impl IntoResponse {
    if !sih149_core::wiper::patterns::is_known_method(&req.method) {
        return (
            StatusCode::BAD_REQUEST,
            Json(WipeResponse {
                job_id: "".into(),
                status: "rejected".into(),
                message: format!("Unknown wipe method: {}", req.method),
            }),
        );
    }
    if let Err(error) =
        sih149_core::wiper::file_wiper::validate_wipe_target(FsPath::new(&req.device_path))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(WipeResponse {
                job_id: "".into(),
                status: "rejected".into(),
                message: error.to_string(),
            }),
        );
    }

    let job_id = format!("wipe-{}", uuid::Uuid::new_v4());

    tracing::info!(
        job_id = %job_id,
        device = %req.device_path,
        method = %req.method,
        verify = req.verify,
        "Wipe job enqueued"
    );

    set_scan_job(
        &job_id,
        serde_json::json!({
            "job_id": job_id,
            "status": "in_progress",
            "progress_percent": 0
        }),
    );

    let job_id_bg = job_id.clone();
    let wipe_config = crate::commands::wiper::WipeConfig {
        device_path: req.device_path.clone(),
        method: req.method.clone(),
        verify: req.verify,
        expert_passphrase: req.expert_passphrase,
    };
    tokio::task::spawn_blocking(move || {
        let job_id_progress = job_id_bg.clone();
        let result = crate::commands::wiper::run_wipe(&wipe_config, move |progress| {
            set_scan_job(
                &job_id_progress,
                serde_json::json!({
                    "job_id": job_id_progress,
                    "status": "in_progress",
                    "progress_percent": progress.percent,
                    "progress": progress,
                }),
            );
        });
        let payload = match result {
            Ok(res) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "completed",
                "progress_percent": 100,
                "result": res
            }),
            Err(e) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "failed",
                "progress_percent": 100,
                "message": e
            }),
        };
        set_scan_job(&job_id_bg, payload);
    });

    (
        StatusCode::ACCEPTED,
        Json(WipeResponse {
            job_id,
            status: "queued".into(),
            message: format!("Wipe of {} using {} enqueued", req.device_path, req.method),
        }),
    )
}

/// POST /api/scan — run a carving scan in the background, store the result,
/// and return a job id that can be polled via GET /api/jobs/:id
async fn post_scan(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let job_id = format!("scan-{}", uuid::Uuid::new_v4());

    tracing::info!(
        job_id = %job_id,
        source = %req.source_path,
        types = ?req.file_types,
        "Scan job enqueued"
    );

    set_scan_job(
        &job_id,
        serde_json::json!({
            "job_id": job_id,
            "status": "in_progress",
            "progress_percent": 0
        }),
    );

    // Run the same scan engine the Tauri IPC path uses, off the async thread.
    let job_id_bg = job_id.clone();
    let config = crate::commands::carver::ScanConfig {
        source_path: req.source_path,
        output_dir: req.output_dir,
        file_types: req.file_types,
        min_confidence: req.min_confidence,
        recovery_modes: req.recovery_modes,
        dry_run: req.dry_run,
    };
    let source_log = config.source_path.clone();
    tokio::task::spawn_blocking(move || {
        let job_id_progress = job_id_bg.clone();
        let result = crate::commands::carver::run_scan(&config, move |p| {
            set_scan_job(
                &job_id_progress,
                serde_json::json!({
                    "job_id": job_id_progress,
                    "status": "in_progress",
                    "progress_percent": p.percent,
                    "progress": p,
                }),
            );
        });
        let payload = match result {
            Ok(scan) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "completed",
                "progress_percent": 100,
                "result": scan
            }),
            Err(e) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "failed",
                "progress_percent": 100,
                "message": e
            }),
        };
        set_scan_job(&job_id_bg, payload);
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "job_id": job_id,
            "status": "queued",
            "message": format!("Scan of {} enqueued", source_log)
        })),
    )
}

/// POST /api/scan/extract — extract only chosen carved files to output dir
async fn post_extract_files(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<crate::commands::carver::ExtractRequest>,
) -> impl IntoResponse {
    match crate::commands::carver::extract_selected_files(req) {
        Ok(res) => (
            StatusCode::OK,
            Json(serde_json::to_value(res).unwrap_or_default()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

/// GET /api/cases — list all forensic cases from SQLite DB
async fn get_cases(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::reports::list_cases() {
        Ok(cases) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": cases })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/cases/:id/audit — get audit log for a case
async fn get_case_audit_log(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(case_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match crate::commands::reports::get_audit_log(case_id) {
        Ok(data) => {
            let parsed: serde_json::Value =
                serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({ "raw": data }));
            (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "success", "data": parsed })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/plugins — list all loaded TOML & Lua signature plugins
async fn get_plugins(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::plugins::list_plugins() {
        Ok(plugins) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": plugins })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/entropy — calculate real Shannon entropy over blocks
async fn post_entropy(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<EntropyRequest>,
) -> impl IntoResponse {
    match crate::commands::plugins::get_drive_entropy(req.device_path, req.chunks) {
        Ok(entropies) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": entropies })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

async fn post_sectors_scan(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SectorScanRequest>,
) -> impl IntoResponse {
    let resolved = crate::commands::carver::resolve_evidence_path(&req.device_path);
    let cells = req.target_cells.unwrap_or(256);
    match sih149_core::disk::surface_scanner::scan_surface(&resolved, cells) {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": result })),
        ),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": error.to_string() })),
        ),
    }
}

/// GET /api/jobs/:id — poll a running carve job; returns the completed
/// ScanResult once `/api/scan`'s background task finishes.
async fn get_job_status(
    State(_state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match get_scan_job(&job_id) {
        Some(job) => (StatusCode::OK, Json(job)),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "job_id": job_id,
                "status": "unknown"
            })),
        ),
    }
}

/// POST /api/hex — read a hex preview from an evidence file (mirrors the
/// Tauri `get_file_hex_preview` command so web mode behaves identically).
async fn post_hex(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<HexRequest>,
) -> impl IntoResponse {
    match crate::commands::carver::get_file_hex_preview_str(
        &req.file_path,
        req.offset,
        req.length,
        req.allowed_root.as_deref(),
    ) {
        Ok(hex) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": hex })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/diagnostics — run all forensic and cryptographic self-tests
async fn get_diagnostics(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let summary = sih149_core::diagnostics::run_diagnostics();
    (
        StatusCode::OK,
        Json(serde_json::to_value(&summary).unwrap_or_default()),
    )
}

#[derive(Deserialize)]
struct PassphraseRequest {
    passphrase: String,
}

#[derive(Deserialize)]
struct DeviceRequest {
    device_path: String,
}

/// GET /api/auth/status — query expert mode configuration status
async fn get_auth_status(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::auth::is_expert_configured().await {
        Ok(configured) => (
            StatusCode::OK,
            Json(serde_json::json!({ "is_configured": configured })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

/// POST /api/auth/setup — initialize expert passphrase
async fn post_auth_setup(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PassphraseRequest>,
) -> impl IntoResponse {
    match crate::commands::auth::setup_expert_passphrase(req.passphrase).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success" })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/auth/verify — verify expert passphrase
async fn post_auth_verify(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PassphraseRequest>,
) -> impl IntoResponse {
    match crate::commands::auth::verify_expert_passphrase(req.passphrase).await {
        Ok(valid) => (StatusCode::OK, Json(serde_json::json!({ "valid": valid }))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/auth/token — automatic token discovery for local browser / GUI sessions
async fn get_local_auth_token(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // 1. Direct connection MUST be from local loopback (127.0.0.1 or ::1)
    if !peer.ip().is_loopback() {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "error",
                "message": "Token discovery is strictly restricted to local loopback connections"
            })),
        );
    }

    // 2. Browser anti-CSRF / anti-cross-site check
    if let Some(sec_fetch) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if sec_fetch == "cross-site" {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Cross-site access denied"
                })),
            );
        }
    }

    // 3. Allowed Origin check
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let expected_1 = format!("http://localhost:{}", state.port);
        let expected_2 = format!("http://127.0.0.1:{}", state.port);
        #[allow(unused_mut)]
        let mut allowed = origin == expected_1 || origin == expected_2;
        #[cfg(debug_assertions)]
        {
            if origin == "http://localhost:5173" || origin == "http://127.0.0.1:5173" {
                allowed = true;
            }
        }
        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Untrusted origin rejected"
                })),
            );
        }
    }

    if state.host == "127.0.0.1" || state.host == "localhost" {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "token": state.api_token
            })),
        )
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "error",
                "message": "Token auto-discovery is restricted to localhost instances"
            })),
        )
    }
}

/// POST /api/firmware/detect — probe controller for hardware sanitization features
async fn post_firmware_detect(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DeviceRequest>,
) -> impl IntoResponse {
    match crate::commands::firmware::detect_firmware_capabilities(req.device_path) {
        Ok(caps) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": caps })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/firmware/erase — execute hardware firmware erasure
async fn post_firmware_erase(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<crate::commands::firmware::FirmwareEraseConfig>,
) -> impl IntoResponse {
    match crate::commands::firmware::start_firmware_erase(req).await {
        Ok(res) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": res })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/scan/cancel — abort active carver scan
async fn post_scan_cancel(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = crate::commands::carver::cancel_scan();
    (
        StatusCode::OK,
        Json(
            serde_json::json!({ "status": "success", "message": "Carver cancellation signalled" }),
        ),
    )
}

/// POST /api/wipe/cancel — abort active wiper job
async fn post_wipe_cancel(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = crate::commands::wiper::cancel_wipe();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "success", "message": "Wiper cancellation signalled" })),
    )
}

#[derive(Deserialize)]
struct ExportReportRequest {
    case_id: Option<String>,
    #[serde(rename = "caseId")]
    case_id_camel: Option<String>,
    format: String,
    output_path: Option<String>,
    #[serde(rename = "outputPath")]
    output_path_camel: Option<String>,
    investigator: Option<String>,
}

/// POST /api/reports/export — export case report or certificate (PDF/HTML/JSON)
async fn post_export_report(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ExportReportRequest>,
) -> impl IntoResponse {
    let case_id = req.case_id.or(req.case_id_camel).unwrap_or_default();
    let output_path = req.output_path.or(req.output_path_camel).unwrap_or_default();
    match crate::commands::reports::export_report(case_id, req.format, output_path, req.investigator) {
        Ok(path) => {
            let abs_path = if FsPath::new(&path).is_absolute() {
                PathBuf::from(&path)
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&path)
            };
            let canonical = abs_path.canonicalize().unwrap_or(abs_path);
            let abs_str = canonical.to_string_lossy().to_string();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "data": abs_str,
                    "path": abs_str,
                    "message": format!("Report exported successfully to {}", abs_str)
                })),
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err
            })),
        ),
    }
}

#[derive(Deserialize)]
struct PreviewQuery {
    file_path: Option<String>,
    source_path: Option<String>,
    offset: Option<u64>,
    size: Option<usize>,
    mime: Option<String>,
}

fn detect_mime_type(path_or_ext: &str, mime_override: Option<&str>) -> &'static str {
    if let Some(m) = mime_override {
        if !m.is_empty() && m != "auto" {
            let m_lower = m.to_lowercase();
            return match m_lower.as_str() {
                "png" | "image/png" => "image/png",
                "jpg" | "jpeg" | "image/jpeg" => "image/jpeg",
                "gif" | "image/gif" => "image/gif",
                "webp" | "image/webp" => "image/webp",
                "bmp" | "image/bmp" => "image/bmp",
                "svg" | "image/svg+xml" => "image/svg+xml",
                "mp4" | "video/mp4" => "video/mp4",
                "webm" | "video/webm" => "video/webm",
                "mp3" | "audio/mpeg" => "audio/mpeg",
                "wav" | "audio/wav" => "audio/wav",
                "pdf" | "application/pdf" => "application/pdf",
                "txt" | "text/plain" => "text/plain; charset=utf-8",
                "json" | "application/json" => "application/json",
                "xml" | "application/xml" => "application/xml",
                _ => "application/octet-stream",
            };
        }
    }

    let lower = path_or_ext.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".mp4") || lower.ends_with(".m4v") {
        "video/mp4"
    } else if lower.ends_with(".webm") {
        "video/webm"
    } else if lower.ends_with(".mov") {
        "video/quicktime"
    } else if lower.ends_with(".mkv") {
        "video/x-matroska"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else if lower.ends_with(".wav") {
        "audio/wav"
    } else if lower.ends_with(".ogg") {
        "audio/ogg"
    } else if lower.ends_with(".pdf") {
        "application/pdf"
    } else if lower.ends_with(".txt") || lower.ends_with(".log") || lower.ends_with(".csv") {
        "text/plain; charset=utf-8"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".xml") {
        "application/xml"
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

/// GET /api/preview — serve media preview for carved file on disk or in-memory from source
async fn get_preview(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<PreviewQuery>,
) -> impl IntoResponse {
    use std::io::{Read, Seek, SeekFrom};

    // 1. If file_path on disk is supplied and exists
    if let Some(ref file_path) = query.file_path {
        let path = FsPath::new(file_path);
        if let Ok(resolved) = path.canonicalize() {
            if sih149_core::wiper::file_wiper::is_protected_path(&resolved) {
                return axum::response::Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(
                        "{\"status\":\"error\",\"message\":\"Access denied: protected system path\"}",
                    ))
                    .unwrap();
            }

            let resolved_str = resolved.to_string_lossy();
            if resolved_str.contains("/.ssh")
                || resolved_str.contains("/.gnupg")
                || resolved_str.contains("/.aws")
                || resolved_str.ends_with("/id_rsa")
                || resolved_str.ends_with("/id_ed25519")
                || resolved_str.contains("/.bash_history")
                || resolved_str.contains("/.zsh_history")
            {
                return axum::response::Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(
                        "{\"status\":\"error\",\"message\":\"Access denied: sensitive security credential path\"}",
                    ))
                    .unwrap();
            }

            if resolved.is_file() {
                match fs::read(&resolved) {
                    Ok(bytes) => {
                        let mime = detect_mime_type(file_path, query.mime.as_deref());
                        let filename = resolved.file_name().and_then(|n| n.to_str()).unwrap_or("preview");
                        return axum::response::Response::builder()
                            .status(StatusCode::OK)
                            .header(axum::http::header::CONTENT_TYPE, mime)
                            .header(
                                axum::http::header::CONTENT_DISPOSITION,
                                format!("inline; filename=\"{}\"", filename),
                            )
                            .body(axum::body::Body::from(bytes))
                            .unwrap();
                    }
                    Err(e) => {
                        return axum::response::Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(axum::http::header::CONTENT_TYPE, "application/json")
                            .body(axum::body::Body::from(format!(
                                "{{\"status\":\"error\",\"message\":\"Failed to read file: {}\"}}",
                                e
                            )))
                            .unwrap();
                    }
                }
            }
        }
    }

    // 2. Direct read from source image / block device at offset
    if let (Some(ref source_path), Some(offset), Some(size)) =
        (&query.source_path, query.offset, query.size)
    {
        let max_size = 50 * 1024 * 1024; // 50MB max for live preview
        let read_size = size.min(max_size);
        let resolved = crate::commands::carver::resolve_evidence_path(source_path);

        match fs::File::open(&resolved) {
            Ok(mut file) => {
                if let Err(e) = file.seek(SeekFrom::Start(offset)) {
                    return axum::response::Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header(axum::http::header::CONTENT_TYPE, "application/json")
                        .body(axum::body::Body::from(format!(
                            "{{\"status\":\"error\",\"message\":\"Failed to seek offset: {}\"}}",
                            e
                        )))
                        .unwrap();
                }

                let mut buffer = vec![0u8; read_size];
                match file.read_exact(&mut buffer) {
                    Ok(_) => {
                        let hint = query.file_path.as_deref().unwrap_or("");
                        let mime = detect_mime_type(hint, query.mime.as_deref());
                        return axum::response::Response::builder()
                            .status(StatusCode::OK)
                            .header(axum::http::header::CONTENT_TYPE, mime)
                            .header(
                                axum::http::header::CONTENT_DISPOSITION,
                                "inline; filename=\"carved_preview\"",
                            )
                            .body(axum::body::Body::from(buffer))
                            .unwrap();
                    }
                    Err(e) => {
                        return axum::response::Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(axum::http::header::CONTENT_TYPE, "application/json")
                            .body(axum::body::Body::from(format!(
                                "{{\"status\":\"error\",\"message\":\"Failed to read bytes: {}\"}}",
                                e
                            )))
                            .unwrap();
                    }
                }
            }
            Err(e) => {
                return axum::response::Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(format!(
                        "{{\"status\":\"error\",\"message\":\"Failed to open evidence source: {}\"}}",
                        e
                    )))
                    .unwrap();
            }
        }
    }

    axum::response::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            "{\"status\":\"error\",\"message\":\"Missing file_path or source_path+offset+size\"}",
        ))
        .unwrap()
}

/// GET /api/jobs/:id/events — Server-Sent Events endpoint for real-time progress streaming
async fn get_job_events(
    State(_state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match get_scan_job(&job_id) {
        Some(job) => (
            StatusCode::OK,
            [
                (axum::http::header::CONTENT_TYPE, "text/event-stream"),
                (axum::http::header::CACHE_CONTROL, "no-cache"),
                (axum::http::header::CONNECTION, "keep-alive"),
            ],
            format!("event: progress\ndata: {}\n\n", job),
        ),
        None => (
            StatusCode::NOT_FOUND,
            [
                (axum::http::header::CONTENT_TYPE, "text/event-stream"),
                (axum::http::header::CACHE_CONTROL, "no-cache"),
                (axum::http::header::CONNECTION, "keep-alive"),
            ],
            format!(
                "event: error\ndata: {}\n\n",
                serde_json::json!({ "job_id": job_id, "status": "unknown" })
            ),
        ),
    }
}

/// Locate compiled static frontend directory
fn find_frontend_dist() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("src-ui/dist"),
        PathBuf::from("../src-ui/dist"),
        PathBuf::from("../../src-ui/dist"),
        PathBuf::from("/usr/local/share/secureforge/dist"),
        PathBuf::from("/usr/share/secureforge/dist"),
    ];
    for c in &candidates {
        if c.join("index.html").exists() {
            return Some(c.clone());
        }
    }
    None
}

/// Serve index.html without token injection (token must be provided out-of-band by operator)
async fn serve_index_html(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(dist_dir) = find_frontend_dist() {
        let index_path = dist_dir.join("index.html");
        if let Ok(html) = fs::read_to_string(index_path) {
            return (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            )
                .into_response();
        }
    }
    fallback_html_handler().await.into_response()
}

/// Fallback HTML when frontend bundle has not yet been built
async fn fallback_html_handler() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>SecureForge — Web Server Online</title>
    <style>
        body { background: #070c18; color: #f8fafc; font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { background: #0f172a; border: 1px solid #1e293b; padding: 2.5rem; border-radius: 1rem; max-width: 500px; text-align: center; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); }
        h1 { color: #38bdf8; margin-top: 0; }
        code { background: #1e293b; color: #10b981; padding: 0.2rem 0.5rem; border-radius: 0.25rem; font-family: monospace; }
        p { color: #94a3b8; font-size: 0.9rem; line-height: 1.5; }
    </style>
</head>
<body>
    <div class="card">
        <h1>SecureForge Server Online</h1>
        <p>API is active on <code>/health</code> and <code>/api/*</code>.</p>
        <p>To serve the full desktop web UI, build the frontend with:</p>
        <p><code>npm --prefix src-ui run build</code></p>
    </div>
</body>
</html>"#,
    )
}

// ── Router builder ────────────────────────────────────────────────────────────

/// Build the full axum router with all API routes, restricted CORS layer, and frontend file server.
pub fn build_router(state: Arc<AppState>) -> Router {
    #[allow(unused_mut)]
    let mut allowed_origins = vec![
        format!("http://localhost:{}", state.port)
            .parse()
            .unwrap_or(HeaderValue::from_static("http://localhost:7878")),
        format!("http://127.0.0.1:{}", state.port)
            .parse()
            .unwrap_or(HeaderValue::from_static("http://127.0.0.1:7878")),
    ];
    #[cfg(debug_assertions)]
    {
        if let Ok(v) = "http://localhost:5173".parse() {
            allowed_origins.push(v);
        }
        if let Ok(v) = "http://127.0.0.1:5173".parse() {
            allowed_origins.push(v);
        }
    }

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]);

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/api/drives", get(get_drives))
        .route("/api/wipe", post(post_wipe))
        .route("/api/scan", post(post_scan))
        .route("/api/scan/extract", post(post_extract_files))
        .route("/api/cases", get(get_cases))
        .route("/api/cases/:id/audit", get(get_case_audit_log))
        .route("/api/plugins", get(get_plugins))
        .route("/api/entropy", post(post_entropy))
        .route("/api/sectors/scan", post(post_sectors_scan))
        .route("/api/hex", post(post_hex))
        .route("/api/scan/cancel", post(post_scan_cancel))
        .route("/api/wipe/cancel", post(post_wipe_cancel))
        .route("/api/auth/token", get(get_local_auth_token))
        .route("/api/auth/status", get(get_auth_status))
        .route("/api/auth/setup", post(post_auth_setup))
        .route("/api/auth/verify", post(post_auth_verify))
        .route("/api/firmware/detect", post(post_firmware_detect))
        .route("/api/firmware/erase", post(post_firmware_erase))
        .route("/api/jobs/:id", get(get_job_status))
        .route("/api/jobs/:id/events", get(get_job_events))
        .route("/api/diagnostics", get(get_diagnostics))
        .route("/api/reports/export", post(post_export_report))
        .route("/api/preview", get(get_preview))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .layer(cors)
        .with_state(state.clone());

    if let Some(dist_dir) = find_frontend_dist() {
        let assets_dir = dist_dir.join("assets");
        let static_router = Router::new()
            .route("/", get(serve_index_html))
            .route("/index.html", get(serve_index_html))
            .fallback(serve_index_html)
            .with_state(state)
            .nest_service("/assets", ServeDir::new(assets_dir));
        static_router.merge(api_router)
    } else {
        api_router.fallback(fallback_html_handler)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the SecureForge HTTP API & Web UI server.
pub async fn start_server(
    host: String,
    port: u16,
    api_token: Option<String>,
) -> anyhow::Result<()> {
    let (token, was_generated) = match api_token {
        Some(t) if !t.is_empty() => (t, false),
        _ => {
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                .join("secureforge");
            let token_path = config_dir.join("api_token");
            if let Ok(saved) = std::fs::read_to_string(&token_path) {
                let trimmed = saved.trim().to_string();
                if !trimmed.is_empty() {
                    (trimmed, false)
                } else {
                    use rand::RngCore;
                    let mut bytes = [0u8; 16];
                    rand::rngs::OsRng.fill_bytes(&mut bytes);
                    let gen = bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    let _ = std::fs::create_dir_all(&config_dir);
                    let _ = std::fs::write(&token_path, &gen);
                    (gen, true)
                }
            } else {
                use rand::RngCore;
                let mut bytes = [0u8; 16];
                rand::rngs::OsRng.fill_bytes(&mut bytes);
                let generated_token = bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                let _ = std::fs::create_dir_all(&config_dir);
                let _ = std::fs::write(&token_path, &generated_token);
                eprintln!(
                    "[SecureForge] Generated persistent API token saved to {}",
                    token_path.display()
                );
                (generated_token, true)
            }
        }
    };

    let trust_proxy = std::env::var("SECUREFORGE_TRUST_PROXY").is_ok();

    let state = Arc::new(AppState {
        host: host.clone(),
        port,
        require_auth: true,
        api_token: token.clone(),
        limiter: Arc::new(RateLimiter::new(120, Duration::from_secs(60))),
        trust_proxy,
    });

    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Bind before announcing availability. A bind failure must be returned to
    // the caller rather than leaving a misleading "listening" message.
    let listener = TcpListener::bind(addr).await?;

    println!("============================================================");
    println!("  SecureForge Forensic Station — Web Server Active");
    println!("  Web UI URL : http://{}", addr);
    if was_generated {
        println!("  Auth Token : {} (Generated — copy this now)", token);
    } else {
        println!("  Auth Token : [Configured from CLI / environment]");
    }
    if host != "127.0.0.1" && host != "localhost" {
        eprintln!(
            "  WARNING: Binding to non-localhost address '{}'. Ensure firewall rules are in place.",
            host
        );
    }
    println!("============================================================");

    tracing::info!("SecureForge web server listening on http://{}", addr);

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
