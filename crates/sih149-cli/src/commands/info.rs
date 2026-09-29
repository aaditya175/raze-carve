use crate::display;
use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct InfoArgs {
    #[arg(short, long)]
    pub device: Option<String>,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &InfoArgs) -> anyhow::Result<()> {
    let output = Command::new("lsblk")
        .args([
            "-J",
            "-b",
            "-o",
            "NAME,SIZE,TYPE,MOUNTPOINT,VENDOR,MODEL,SERIAL,ROTA,TRAN,RM",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)?;

    let blockdevices = parsed["blockdevices"].as_array().ok_or_else(|| {
        anyhow::anyhow!(
            "lsblk returned unexpected JSON — 'blockdevices' key missing or not an array"
        )
    })?;

    if args.json {
        if let Some(dev) = &args.device {
            let smart_val = match Command::new("smartctl").args(["-j", "-a", dev]).output() {
                Ok(o) => {
                    let s_out = String::from_utf8_lossy(&o.stdout);
                    serde_json::from_str(&s_out).unwrap_or(serde_json::json!({}))
                }
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            };
            let combined = serde_json::json!({
                "blockdevices": blockdevices,
                "smart": smart_val,
            });
            println!("{}", serde_json::to_string_pretty(&combined)?);
        } else {
            println!("{}", serde_json::to_string_pretty(blockdevices)?);
        }
        return Ok(());
    }

    display::print_section("System Drives");
    display::print_drive_table(blockdevices);

    if let Some(dev) = &args.device {
        display::print_section(&format!("SMART Info for {}", dev));
        let smart_out = Command::new("smartctl").args(["-j", "-a", dev]).output();
        match smart_out {
            Ok(o) => {
                let s_out = String::from_utf8_lossy(&o.stdout);
                let s_parsed: serde_json::Value =
                    serde_json::from_str(&s_out).unwrap_or(serde_json::json!({}));
                if let Some(passed) = s_parsed["smart_status"]["passed"].as_bool() {
                    if passed {
                        println!("Health Status: \x1b[1;32mPASSED (Healthy)\x1b[0m");
                    } else {
                        println!("Health Status: \x1b[1;31mFAILING / PRE-FAILURE DETECTED\x1b[0m");
                    }
                } else if let Some(msgs) = s_parsed["smartctl"]["messages"].as_array() {
                    let msg_str = msgs
                        .iter()
                        .filter_map(|m| m["string"].as_str())
                        .collect::<Vec<_>>()
                        .join("; ");
                    println!("Health Status: Unavailable ({})", msg_str);
                } else {
                    println!("Health Status: Unavailable (Requires elevated privileges or unsupported device)");
                }
            }
            Err(e) => {
                display::print_error(&format!("Failed to run smartctl: {}", e));
            }
        }
    }

    Ok(())
}
