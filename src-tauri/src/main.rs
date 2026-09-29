//! SecureForge Tauri desktop application backend.
//!
//! Registers IPC command handlers that bridge the React frontend
//! to the sih149-core engine. Also provides an embedded HTTP background
//! server (via axum) for universal browser-based and scriptable access.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod server;

use commands::auth::*;
use commands::carver::*;
use commands::diagnostics::*;
use commands::drives::*;
use commands::firmware::*;
use commands::plugins::*;
use commands::reports::*;
use commands::shredder::*;
use commands::wiper::*;

fn main() -> anyhow::Result<()> {
    tracing::info!("SecureForge Desktop v{}", env!("CARGO_PKG_VERSION"));

    // If launched via sudo on Linux, preserve or re-bind XAUTHORITY and DISPLAY
    // so GTK/WebKit can communicate with the user's active desktop session.
    #[cfg(target_os = "linux")]
    {
        if std::env::var("XAUTHORITY").is_err() {
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let candidate =
                    std::path::PathBuf::from(format!("/home/{}/.Xauthority", sudo_user));
                if candidate.exists() {
                    std::env::set_var("XAUTHORITY", candidate);
                }
            }
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("SecureForge Desktop & Embedded Server\n\nUsage: secureforge-desktop [OPTIONS]\n\nOptions:\n  -s, --server            Run headless background server\n      --host <HOST>       Bind host [default: 127.0.0.1]\n      --port <PORT>       Bind port [default: 7878]\n      --api-token <TOKEN> Explicit API authentication token\n  -h, --help              Print help\n  -V, --version           Print version");
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("secureforge-desktop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let is_headless =
        std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err();
    let force_server = args
        .iter()
        .any(|a| a == "--server" || a == "--headless" || a == "-s");

    let mut port: u16 = 7878;
    let mut host = "127.0.0.1".to_string();
    let mut api_token: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse() {
                port = p;
            }
            i += 2;
            continue;
        } else if args[i].starts_with("--port=") {
            if let Some(val) = args[i].split('=').nth(1) {
                if let Ok(p) = val.parse() {
                    port = p;
                }
            }
        } else if args[i] == "--host" && i + 1 < args.len() {
            host = args[i + 1].clone();
            i += 2;
            continue;
        } else if args[i].starts_with("--host=") {
            if let Some(val) = args[i].split('=').nth(1) {
                host = val.to_string();
            }
        } else if args[i] == "--api-token" && i + 1 < args.len() {
            api_token = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if args[i].starts_with("--api-token=") {
            if let Some(val) = args[i].split('=').nth(1) {
                api_token = Some(val.to_string());
            }
        }
        i += 1;
    }

    // In explicit/headless server mode, run the listener in this process so a
    // bind failure is returned to the shell instead of being silently dropped.
    if force_server || is_headless {
        println!("\x1b[1;36m[SECUREFORGE]\x1b[0m Universal Forensic Engine active.");
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(crate::server::start_server(host, port, api_token));
    }

    // Desktop mode keeps the web API in a background runtime. Any failure is
    // logged explicitly; the desktop UI remains usable through native IPC.
    let bg_host = host.clone();
    let bg_token = api_token.clone();
    std::thread::spawn(move || match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            if let Err(error) = rt.block_on(crate::server::start_server(bg_host, port, bg_token)) {
                let is_addr_in_use = error.chain().any(|cause| {
                    if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
                        io_err.kind() == std::io::ErrorKind::AddrInUse
                    } else {
                        cause.to_string().contains("Address already in use")
                    }
                });
                if is_addr_in_use {
                    println!(
                        "[SECUREFORGE] Background web server: port {} is already in use (reusing active instance).",
                        port
                    );
                } else {
                    eprintln!("[SECUREFORGE] Background web server failed: {}", error);
                }
            }
        }
        Err(error) => {
            eprintln!(
                "[SECUREFORGE] Failed to create background server runtime: {}",
                error
            );
        }
    });

    // Otherwise launch native Tauri desktop window with graceful fallback
    let tauri_res = std::panic::catch_unwind(|| {
        tauri::Builder::default()
            .plugin(tauri_plugin_shell::init())
            .invoke_handler(tauri::generate_handler![
                list_drives,
                get_drive_info,
                start_wipe,
                cancel_wipe,
                estimate_wipe_time,
                start_scan,
                cancel_scan,
                extract_selected_files,
                get_file_hex_preview,
                shred_files,
                cancel_shred,
                detect_firmware_capabilities,
                start_firmware_erase,
                setup_expert_passphrase,
                verify_expert_passphrase,
                is_expert_configured,
                list_cases,
                export_report,
                get_audit_log,
                list_plugins,
                get_drive_entropy,
                run_system_diagnostics,
                scan_drive_sectors,
            ])
            .run(tauri::generate_context!())
    });

    if let Err(_panic_err) = tauri_res {
        eprintln!("\n\x1b[1;33m[SECUREFORGE NOTICE]\x1b[0m Graphical window could not attach to display server.");
        eprintln!("Check the background server startup message before opening http://localhost:{} in a browser.\n", port);

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            tokio::signal::ctrl_c().await.ok();
        });
    }

    Ok(())
}
