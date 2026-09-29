use crate::display;
use clap::Args;
use sih149_core::wiper::file_wiper::{is_protected_path, FileWiper};
use std::io::{self, Write};
use std::path::Path;

#[derive(Args)]
pub struct ShredArgs {
    /// File or directory to securely shred
    #[arg(short, long)]
    pub target: String,

    /// Number of overwrite passes (default: 3 for DoD-style)
    #[arg(short, long, default_value = "3")]
    pub passes: u32,

    /// Number of random renames before deletion (scrubs directory-entry history)
    #[arg(long, default_value = "8")]
    pub renames: u32,

    /// Also attempt to scrub slack space (requires raw device access — will error if unsupported)
    #[arg(long)]
    pub scrub_slack: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: &ShredArgs) -> anyhow::Result<()> {
    let target = Path::new(&args.target);

    if !target.exists() {
        anyhow::bail!("Target does not exist: {}", args.target);
    }

    if is_protected_path(target) {
        display::print_warn(&format!(
            "Notice: {} is a system path — proceeding per operator request.",
            args.target
        ));
    }

    let is_cow = sih149_core::wiper::file_wiper::detect_cow_filesystem(target).unwrap_or(false);
    if is_cow {
        display::print_warn(
            "Target resides on a Copy-on-Write filesystem (Btrfs/ZFS). In-place overwriting may not eradicate previous snapshot blocks.",
        );
    }

    if !args.yes {
        println!(
            "\x1b[1;38;2;248;113;113mWARNING:\x1b[0m {} will be \x1b[1mPERMANENTLY ERADICATED\x1b[0m ({} passes, {} renames).",
            args.target, args.passes, args.renames
        );
        print!("Type \x1b[1;38;2;251;191;36mERASE\x1b[0m to continue: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "ERASE" {
            display::print_warn("Aborted by user.");
            return Ok(());
        }
    }

    display::print_section("Forensic Shredding Engine");
    println!("  Target  : \x1b[1m{}\x1b[0m", args.target);
    println!(
        "  Passes  : \x1b[38;2;56;189;248m{} in-place cryptographic overwrite passes\x1b[0m",
        args.passes
    );
    println!(
        "  Renames : \x1b[38;2;168;85;247m{} journal obfuscation cycles\x1b[0m",
        args.renames
    );
    println!(
        "  Slack   : {}\n",
        if args.scrub_slack {
            "\x1b[38;2;52;211;153mEnabled (raw block zeroing)\x1b[0m"
        } else {
            "Disabled"
        }
    );

    let spinner = display::create_spinner("Overwriting sectors with DoD & CSPRNG streams...");

    let wiper = FileWiper::new(args.passes, args.renames, args.scrub_slack);

    if target.is_dir() {
        let results = wiper
            .wipe_directory(target)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        spinner.finish_and_clear();

        let total_bytes: u64 = results.iter().map(|r| r.bytes_wiped).sum();
        let file_count = results.len();
        display::print_success(&format!(
            "Recursively destroyed {} file(s), {} total with verified inode scrubs.",
            file_count,
            display::format_bytes(total_bytes)
        ));
    } else {
        let result = wiper
            .wipe_file(target)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        spinner.finish_and_clear();

        display::print_success(&format!(
            "Eradicated {} ({} across {} verified passes with {} renames).",
            args.target,
            display::format_bytes(result.bytes_wiped),
            result.passes_completed,
            args.renames,
        ));
    }

    let db_path = dirs::data_local_dir()
        .unwrap_or_default()
        .join("secureforge")
        .join("cases.db");
    let case_id = format!(
        "shred-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
        let _ = db.create_case(
            &case_id,
            "CLI Operator",
            &args.target,
            &format!(
                "file_shred ({} passes, {} renames)",
                args.passes, args.renames
            ),
        );
        let audit_json = serde_json::json!({
            "event": "shred_completed",
            "target": &args.target,
            "passes": args.passes,
            "renames": args.renames,
            "scrub_slack": args.scrub_slack,
            "cow_filesystem": is_cow,
        })
        .to_string();
        let hash = sih149_core::audit::signing::hash_bytes(audit_json.as_bytes());
        let _ = db.insert_audit_entry(&case_id, &audit_json, &hash);
        let _ = db.update_case_status(&case_id, "completed");
    }

    crate::commands::report::generate_case_certificate_pdf(&case_id, "erasure", None);

    Ok(())
}
