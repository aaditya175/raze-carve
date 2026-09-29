use sih149_core::diagnostics::{run_diagnostics, DiagnosticSummary};

#[tauri::command]
pub fn run_system_diagnostics() -> Result<DiagnosticSummary, String> {
    Ok(run_diagnostics())
}
