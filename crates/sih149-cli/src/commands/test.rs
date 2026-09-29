use crate::display;
use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
}

pub fn run(args: &TestArgs) -> anyhow::Result<()> {
    if !args.json {
        display::print_section("Subsystem Diagnostics & Live Assertions");
        println!("\x1b[38;2;148;163;184mExecuting cryptographic, memory, safety, carver, and database assertions...\x1b[0m\n");
    }

    let spinner = if !args.json {
        Some(display::create_spinner(
            "Running in-situ diagnostic self-tests...",
        ))
    } else {
        None
    };

    let summary = sih149_core::diagnostics::run_diagnostics();

    if let Some(sp) = spinner {
        sp.finish_and_clear();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        if summary.failed > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("\x1b[38;2;60;66;96m┌──────────┬──────────────────────────────────────┬────────┬───────────┬────────────────────────────────────────────────────────┐\x1b[0m");
    println!("\x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<8}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<36}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<6}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<9}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<54}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m",
        "DOMAIN", "ASSERTION", "STATUS", "LATENCY", "DIAGNOSTIC EVIDENCE");
    println!("\x1b[38;2;60;66;96m├──────────┼──────────────────────────────────────┼────────┼───────────┼────────────────────────────────────────────────────────┤\x1b[0m");

    for r in &summary.results {
        let (cat_color, cat_badge) = match r.category.as_str() {
            "Wiper" => ("\x1b[38;2;56;189;248m", "WIPER"),
            "Safety" => ("\x1b[38;2;251;191;36m", "SAFETY"),
            "Carver" => ("\x1b[38;2;168;85;247m", "CARVER"),
            "Crypto" => ("\x1b[38;2;52;211;153m", "CRYPTO"),
            "Database" => ("\x1b[38;2;129;140;248m", "DB"),
            "Classifier" => ("\x1b[38;2;45;212;191m", "CLASS"),
            _ => ("\x1b[38;2;148;163;184m", "CORE"),
        };

        let status = if r.passed {
            "\x1b[1;38;2;52;211;153mPASS\x1b[0m"
        } else {
            "\x1b[1;38;2;248;113;113mFAIL\x1b[0m"
        };

        let latency = format!("{} µs", r.duration_us);
        let msg_truncated = if r.message.len() > 54 {
            format!("{}...", &r.message[..51])
        } else {
            r.message.clone()
        };

        println!(
            "\x1b[38;2;60;66;96m│\x1b[0m {}{:<8}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m {:<36} \x1b[38;2;60;66;96m│\x1b[0m {:<15} \x1b[38;2;60;66;96m│\x1b[0m {:<9} \x1b[38;2;60;66;96m│\x1b[0m \x1b[38;2;148;163;184m{:<54}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m",
            cat_color, cat_badge, r.name, status, latency, msg_truncated
        );
    }
    println!("\x1b[38;2;60;66;96m└──────────┴──────────────────────────────────────┴────────┴───────────┴────────────────────────────────────────────────────────┘\x1b[0m");

    println!("\n  Telemetry: {} assertions | \x1b[1;38;2;52;211;153m{} passed\x1b[0m | \x1b[1;38;2;248;113;113m{} failed\x1b[0m | total latency: \x1b[1m{:.2} ms\x1b[0m\n",
        summary.total, summary.passed, summary.failed, summary.total_duration_ms);

    if summary.failed == 0 {
        display::print_success("All forensic & cryptographic assertions verified 100% functional.");
        Ok(())
    } else {
        display::print_error(&format!(
            "{} test(s) FAILED — system integrity check failed",
            summary.failed
        ));
        std::process::exit(1);
    }
}
