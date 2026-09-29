use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum DriveType {
    HDD,
    SSD,
    NVMe,
    USB,
    ZRAM,
    LOOP,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    pub serial: String,
    pub drive_type: DriveType,
    pub is_mounted: bool,
    pub mount_points: Vec<String>,
    pub is_system_drive: bool,
    pub smart_status: SmartStatus,
}

#[derive(Deserialize)]
struct LsblkOutput {
    #[serde(default)]
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Deserialize)]
struct LsblkDevice {
    name: String,
    #[serde(default)]
    size: Option<serde_json::Value>,
    #[serde(rename = "type", default)]
    dev_type: Option<String>,
    #[serde(default)]
    tran: Option<String>,
    #[serde(default)]
    rm: Option<serde_json::Value>,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    serial: Option<String>,
    #[serde(default)]
    rota: Option<serde_json::Value>,
    #[serde(default)]
    children: Option<Vec<LsblkDevice>>,
}

fn determine_drive_type(
    name: &str,
    tran: Option<&str>,
    rm: Option<&serde_json::Value>,
    rota: Option<&serde_json::Value>,
) -> DriveType {
    if name.starts_with("loop") {
        return DriveType::LOOP;
    }
    if name.starts_with("zram") || name.starts_with("ram") {
        return DriveType::ZRAM;
    }
    if name.starts_with("nvme") || tran == Some("nvme") {
        return DriveType::NVMe;
    }
    let is_removable = match rm {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_u64() == Some(1),
        Some(serde_json::Value::String(s)) => s == "1" || s == "true",
        _ => false,
    };
    if tran == Some("usb") || is_removable {
        return DriveType::USB;
    }
    if let Some(r) = rota {
        if r.as_bool() == Some(true)
            || r.as_str() == Some("1")
            || r.as_str() == Some("true")
            || r.as_i64() == Some(1)
            || r.as_u64() == Some(1)
        {
            return DriveType::HDD;
        } else if r.as_bool() == Some(false)
            || r.as_str() == Some("0")
            || r.as_str() == Some("false")
            || r.as_i64() == Some(0)
            || r.as_u64() == Some(0)
        {
            return DriveType::SSD;
        }
    }
    DriveType::Unknown
}

fn extract_size_bytes(val: Option<&serde_json::Value>) -> u64 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn collect_mount_points(dev: &LsblkDevice, out: &mut Vec<String>) {
    if let Some(ref mp) = dev.mountpoint {
        if !mp.is_empty() && !out.contains(mp) {
            out.push(mp.clone());
        }
    }
    if let Some(ref mps) = dev.mountpoints {
        for mp in mps.iter().flatten() {
            if !mp.is_empty() && !out.contains(mp) {
                out.push(mp.clone());
            }
        }
    }
    if let Some(ref children) = dev.children {
        for child in children {
            collect_mount_points(child, out);
        }
    }
}

#[tauri::command]
pub fn list_drives() -> Result<Vec<DriveInfo>, String> {
    let output = Command::new("lsblk")
        .args([
            "-J",
            "-b",
            "-o",
            "NAME,SIZE,TYPE,TRAN,RM,MOUNTPOINT,MOUNTPOINTS,VENDOR,MODEL,SERIAL,ROTA",
        ])
        .output()
        .map_err(|e| format!("Failed to run lsblk: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk output: {}", e))?;

    let mut drives = Vec::new();

    for dev in parsed.blockdevices {
        let is_disk = dev.dev_type.as_deref() == Some("disk");
        let is_loop = dev.dev_type.as_deref() == Some("loop") || dev.name.starts_with("loop");
        if !is_disk && !is_loop {
            continue;
        }

        let drive_type = determine_drive_type(
            &dev.name,
            dev.tran.as_deref(),
            dev.rm.as_ref(),
            dev.rota.as_ref(),
        );
        let path = format!("/dev/{}", dev.name);

        let mut mount_points = Vec::new();
        collect_mount_points(&dev, &mut mount_points);

        let is_mounted = !mount_points.is_empty();
        let is_system_drive = mount_points.contains(&"/".to_string())
            || mount_points.contains(&"/boot".to_string())
            || mount_points.contains(&"/boot/efi".to_string())
            || mount_points.contains(&"/home".to_string());

        let mut size_bytes = extract_size_bytes(dev.size.as_ref());
        if size_bytes == 0 {
            if let Some(sys_size) = sih149_core::disk::surface_scanner::detect_block_device_size(
                std::path::Path::new(&path),
            ) {
                size_bytes = sys_size;
            }
        }

        // Skip unallocated 0-byte loop devices
        if is_loop && size_bytes == 0 {
            continue;
        }

        let mut model = dev.model.unwrap_or_else(|| "Unknown".to_string());
        if model == "Unknown" && (dev.name.starts_with("zram") || dev.name.starts_with("ram")) {
            model = "ZRAM Compressed Swap".to_string();
        } else if model == "Unknown" && is_loop {
            model = "Virtual Loop Disk".to_string();
        }
        let serial = dev.serial.unwrap_or_else(|| "Unknown".to_string());

        let mut smart_status = SmartStatus::Unknown;
        if is_loop || dev.name.starts_with("zram") || dev.name.starts_with("ram") {
            smart_status = SmartStatus::Healthy;
        } else if let Ok(output) = Command::new("smartctl").args(["-j", "-H", &path]).output() {
            if let Ok(parsed) = serde_json::from_slice::<SmartctlOutput>(&output.stdout) {
                if let Some(status) = parsed.smart_status {
                    smart_status = if status.passed {
                        SmartStatus::Healthy
                    } else {
                        SmartStatus::Critical
                    };
                }
            }
        }

        drives.push(DriveInfo {
            name: dev.name,
            path,
            size_bytes,
            model: model.trim().to_string(),
            serial: serial.trim().to_string(),
            drive_type,
            is_mounted,
            mount_points,
            is_system_drive,
            smart_status,
        });
    }

    Ok(drives)
}

#[derive(Deserialize)]
struct SmartctlOutput {
    smart_status: Option<SmartctlStatus>,
}

#[derive(Deserialize)]
struct SmartctlStatus {
    passed: bool,
}

#[tauri::command]
pub fn get_drive_info(device_path: String) -> Result<DriveInfo, String> {
    let drives = list_drives()?;
    let mut drive = drives
        .into_iter()
        .find(|d| d.path == device_path)
        .ok_or_else(|| format!("Drive not found: {}", device_path))?;

    let output = Command::new("smartctl")
        .args(["-j", "-a", &device_path])
        .output();

    if let Ok(output) = output {
        if let Ok(parsed) = serde_json::from_slice::<SmartctlOutput>(&output.stdout) {
            if let Some(status) = parsed.smart_status {
                drive.smart_status = if status.passed {
                    SmartStatus::Healthy
                } else {
                    SmartStatus::Critical
                };
            } else {
                drive.smart_status = SmartStatus::Unknown;
            }
        }
    }

    Ok(drive)
}

#[tauri::command]
pub fn scan_drive_sectors(
    device_path: String,
    target_cells: Option<usize>,
) -> Result<sih149_core::disk::surface_scanner::SurfaceScanResult, String> {
    let resolved = super::carver::resolve_evidence_path(&device_path);
    let cells = target_cells.unwrap_or(256);
    sih149_core::disk::surface_scanner::scan_surface(&resolved, cells)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_drives_includes_loop() {
        let drives = list_drives().expect("list_drives should succeed");
        println!("Found {} drives:", drives.len());
        for d in &drives {
            println!("  drive: {} {:?} {} bytes model={}", d.name, d.drive_type, d.size_bytes, d.model);
        }
        // If an active loop device exists on the host with non-zero size, it must be included
        let has_active_loop = std::process::Command::new("losetup")
            .arg("-a")
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        if has_active_loop {
            assert!(drives.iter().any(|d| d.name.starts_with("loop")));
        } else {
            // When no loop devices are mounted, verify regular drives were enumerated successfully
            assert!(!drives.is_empty());
        }
    }
}
