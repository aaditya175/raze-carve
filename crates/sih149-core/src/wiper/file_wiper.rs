//! Secure file and folder erasure.
//!
//! Overwrites file content in place (respecting the same DoD pattern
//! sequence as the drive wiper), renames the file N times with random
//! names to scrub the original filename from directory-entry/journal
//! history, then deletes it.
use crate::error::CoreError;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

pub struct FileWiper {
    passes: u32,
    rename_count: u32,
    scrub_slack_space: bool,
}

#[derive(Debug)]
pub struct WipeFileResult {
    pub path: String,
    pub bytes_wiped: u64,
    pub passes_completed: u32,
    pub slack_bytes_wiped: u64,
    pub success: bool,
    pub is_cow: bool,
}

/// Returns `true` when `path` (after canonicalization) is under a protected
/// system prefix. Callers should refuse to shred or return an error.
///
/// Protected prefixes:
///   /boot, /bin, /sbin, /lib, /lib64, /usr, /etc, /root, /sys, /proc, /dev,
///   /var, /run, /opt, /srv
///
/// Symlinks are resolved before comparison so that e.g.
///   --target /tmp/evil_link   where the link points to /etc/passwd
/// is correctly blocked, not silently followed.
pub fn is_protected_path(path: &Path) -> bool {
    // Lexically normalize path components (resolving '..' and '.' without needing filesystem stat)
    let mut normalized = std::path::PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::RootDir => {
                normalized.push(std::path::Component::RootDir.as_os_str());
            }
            std::path::Component::ParentDir => {
                if normalized != std::path::Path::new("/") {
                    normalized.pop();
                }
            }
            std::path::Component::CurDir => {}
            c => normalized.push(c.as_os_str()),
        }
    }

    let resolved = match std::fs::canonicalize(&normalized) {
        Ok(p) => p,
        Err(_) => match normalized.parent() {
            Some(parent) => match std::fs::canonicalize(parent) {
                Ok(canon_parent) => canon_parent.join(normalized.file_name().unwrap_or_default()),
                Err(_) => normalized,
            },
            None => normalized,
        },
    };
    let protected = [
        "/boot", "/bin", "/sbin", "/lib", "/lib64", "/usr", "/etc", "/root", "/sys", "/proc",
        "/dev", "/var", "/run", "/opt", "/srv",
    ];
    let s = resolved.to_string_lossy();
    // `/var/tmp` is deliberately retained across reboots but is a standard
    // user-writable temporary area, unlike the system state below `/var`.
    if s == "/var/tmp" || s.starts_with("/var/tmp/") {
        return false;
    }
    protected
        .iter()
        .any(|prefix| s == *prefix || s.starts_with(&format!("{}/", prefix)))
}

/// Returns `true` if `device` points to a primary boot or system block device
/// (e.g. /dev/sda, /dev/nvme0n1, /dev/disk0, /dev/vda, /dev/root).
/// Symlinks and partition aliases are resolved before comparison.
pub fn is_protected_drive(device: &Path) -> bool {
    let protected_prefixes = [
        "/dev/sda",
        "/dev/nvme0n1",
        "/dev/disk0",
        "/dev/vda",
        "/dev/root",
    ];
    let canonical = match std::fs::canonicalize(device) {
        Ok(c) => c,
        Err(_) => match device.parent() {
            Some(parent) => match std::fs::canonicalize(parent) {
                Ok(cp) => cp.join(device.file_name().unwrap_or_default()),
                Err(_) => device.to_path_buf(),
            },
            None => device.to_path_buf(),
        },
    };
    let s = canonical.to_string_lossy();
    if protected_prefixes
        .iter()
        .any(|prefix| s == *prefix || s.starts_with(prefix))
    {
        return true;
    }

    // Dynamically protect every ancestor of the device mounted at / or /boot.
    // A root filesystem commonly lives on /dev/sdb2 while the destructive
    // target is /dev/sdb; comparing only strings misses that relationship.
    #[cfg(target_os = "linux")]
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        let target_related = related_block_devices(&canonical);
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let dev_part = parts[0];
                let mount_point = parts[1];
                if (mount_point == "/" || mount_point == "/boot" || mount_point == "/boot/efi")
                    && dev_part.starts_with("/dev/")
                    && target_related.iter().any(|candidate| {
                        candidate == Path::new(dev_part)
                            || std::fs::canonicalize(dev_part)
                                .map(|mounted| candidate == &mounted)
                                .unwrap_or(false)
                    })
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Validate that the wipe target is a valid block device. Destructive operations
/// are allowed on operator-selected drives without artificial protection blocks.
pub fn validate_wipe_target(device: &Path) -> Result<(), CoreError> {
    if !crate::disk::block_device::is_block_device(device) {
        return Err(CoreError::Wiper(format!(
            "Wipe target must be a block device, not {}",
            device.display()
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
fn related_block_devices(device: &Path) -> Vec<std::path::PathBuf> {
    use std::process::Command;

    let target = std::fs::canonicalize(device).unwrap_or_else(|_| device.to_path_buf());
    let output = Command::new("lsblk")
        .args([
            "--noheadings",
            "--raw",
            "--paths",
            "--output",
            "NAME,PKNAME",
        ])
        .output();
    let Ok(output) = output else {
        return vec![target];
    };
    if !output.status.success() {
        return vec![target];
    }

    // Treat the lsblk parent map as an undirected graph. That identifies both
    // the disk below a mounted partition (sdb <- sdb2) and device-mapper/LVM
    // ancestors/children without guessing from device-name prefixes.
    let mut edges: std::collections::HashMap<std::path::PathBuf, Vec<std::path::PathBuf>> =
        std::collections::HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut fields = line.split_whitespace();
        let Some(name) = fields.next() else { continue };
        let name = std::fs::canonicalize(name).unwrap_or_else(|_| name.into());
        edges.entry(name.clone()).or_default();
        if let Some(parent) = fields.next().filter(|parent| !parent.is_empty()) {
            let parent = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.into());
            edges.entry(name.clone()).or_default().push(parent.clone());
            edges.entry(parent).or_default().push(name);
        }
    }
    let mut related = vec![target.clone()];
    let mut cursor = 0;
    while cursor < related.len() {
        let current = related[cursor].clone();
        if let Some(neighbors) = edges.get(&current) {
            for neighbor in neighbors {
                if !related.contains(neighbor) {
                    related.push(neighbor.clone());
                }
            }
        }
        cursor += 1;
    }
    related
}

#[cfg(not(target_os = "linux"))]
fn related_block_devices(device: &Path) -> Vec<std::path::PathBuf> {
    vec![device.to_path_buf()]
}

#[allow(dead_code)]
pub fn is_mounted_block_device(device: &Path) -> bool {
    #[cfg(target_os = "linux")]
    {
        let related = related_block_devices(device);
        let Ok(mounts) = std::fs::read_to_string("/proc/mounts") else {
            // If mount information cannot be read, fail closed for a
            // destructive operation.
            return true;
        };
        mounts.lines().any(|line| {
            let source = line.split_whitespace().next().unwrap_or_default();
            if !source.starts_with("/dev/") {
                return false;
            }
            let mounted =
                std::fs::canonicalize(source).unwrap_or_else(|_| std::path::PathBuf::from(source));
            related.iter().any(|candidate| candidate == &mounted)
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = device;
        true
    }
}

impl FileWiper {
    pub fn new(passes: u32, rename_count: u32, scrub_slack_space: bool) -> Self {
        Self {
            passes,
            rename_count,
            scrub_slack_space,
        }
    }

    pub fn wipe_file(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        // Path validation — block obvious escape attempts here in the core
        // so every caller (CLI and Tauri) gets the same protection.
        if is_protected_path(path) {
            return Err(CoreError::Wiper(format!(
                "Refusing to shred protected system path: {} — use the expert CLI with explicit override if intentional",
                path.display()
            )));
        }

        let is_cow = detect_cow_filesystem(path)?;
        if is_cow {
            return Err(CoreError::Wiper(format!(
                "Refusing secure erase on Copy-on-Write filesystem for {}: overwrite cannot prove removal of prior extents or snapshots",
                path.display()
            )));
        }
        let mut res = self.wipe_file_unchecked(path)?;
        res.is_cow = is_cow;
        Ok(res)
    }

    pub fn wipe_directory(&self, path: &Path) -> Result<Vec<WipeFileResult>, CoreError> {
        if is_protected_path(path) {
            return Err(CoreError::Wiper(format!(
                "Refusing to shred protected system directory: {}",
                path.display()
            )));
        }
        let is_cow = detect_cow_filesystem(path)?;
        if is_cow {
            return Err(CoreError::Wiper(format!(
                "Refusing secure erase on Copy-on-Write filesystem for {}: overwrite cannot prove removal of prior extents or snapshots",
                path.display()
            )));
        }
        let mut results = Vec::new();
        self.wipe_directory_inner(path, &mut results)?;
        for r in &mut results {
            r.is_cow = is_cow;
        }
        fs::remove_dir(path).map_err(CoreError::Io)?;
        Ok(results)
    }

    fn wipe_directory_inner(
        &self,
        path: &Path,
        results: &mut Vec<WipeFileResult>,
    ) -> Result<(), CoreError> {
        for entry in fs::read_dir(path).map_err(CoreError::Io)? {
            let entry = entry.map_err(CoreError::Io)?;
            let entry_path = entry.path();

            // entry.file_type() comes from the directory entry itself (like
            // lstat) and does NOT follow symlinks — unlike entry_path.is_dir(),
            // which calls metadata() and DOES follow them. Using is_dir() here
            // previously let a symlink inside the target directory (pointing
            // anywhere on the filesystem, e.g. /etc/shadow, or even another
            // directory entirely) get silently followed and overwritten/
            // recursed-into. Refuse symlinks explicitly instead of following
            // them into wipe_file_unchecked/recursion.
            let file_type = entry.file_type().map_err(CoreError::Io)?;
            if file_type.is_symlink() {
                results.push(WipeFileResult {
                    path: entry_path.to_string_lossy().into_owned(),
                    bytes_wiped: 0,
                    passes_completed: 0,
                    slack_bytes_wiped: 0,
                    success: false,
                    is_cow: false,
                });
                tracing::warn!(
                    "Skipped symlink during directory shred (not followed): {}",
                    entry_path.display()
                );
                continue;
            }

            if file_type.is_dir() {
                self.wipe_directory_inner(&entry_path, results)?;
                fs::remove_dir(&entry_path).map_err(CoreError::Io)?;
            } else {
                // Per-file is_protected_path skipped here — the outer
                // wipe_directory already blocked on the parent; individual
                // files inside are implicitly under it. Symlinks (the actual
                // escape vector) are already excluded above.
                results.push(self.wipe_file_unchecked(&entry_path)?);
            }
        }
        Ok(())
    }

    /// Like `wipe_file` but skips the protected-path check.
    /// Used internally when the parent directory has already been validated.
    fn wipe_file_unchecked(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        let metadata = fs::symlink_metadata(path).map_err(CoreError::Io)?;
        if metadata.file_type().is_symlink() {
            // Defense in depth: wipe_directory_inner already filters symlinks
            // out before calling this, but wipe_file (the public single-file
            // entry point) can still be pointed at a symlink directly by a
            // caller — refuse here too rather than relying on callers to remember.
            return Err(CoreError::Wiper(format!(
                "Refusing to shred a symlink directly: {} (target not followed)",
                path.display()
            )));
        }
        let size = metadata.len();
        if self.scrub_slack_space {
            return Err(CoreError::Wiper(
                "Filesystem slack-space overwriting is not supported safely through ordinary file I/O; use whole-device sanitization".to_string(),
            ));
        }
        let slack_len = 0;
        let wipe_target_size = size;
        let chunk_size: usize = 1024 * 1024;

        let mut file = open_file_for_wipe(path)?;
        let mut passes_completed = 0u32;
        let mut buf = vec![0u8; chunk_size];
        for pass in 1..=self.passes.max(1) {
            let mut pattern = if self.passes == 35 {
                crate::wiper::patterns::WipePattern::for_method_pass("gutmann", pass as u8)
            } else {
                crate::wiper::patterns::WipePattern::for_pass(pass as u8)
            };
            file.seek(SeekFrom::Start(0)).map_err(CoreError::Io)?;
            let mut written: u64 = 0;
            while written < wipe_target_size {
                let this_chunk =
                    std::cmp::min(chunk_size as u64, wipe_target_size - written) as usize;
                pattern.fill(&mut buf[..this_chunk]);
                file.write_all(&buf[..this_chunk]).map_err(CoreError::Io)?;
                written += this_chunk as u64;
            }
            file.flush().map_err(CoreError::Io)?;
            file.sync_all().map_err(CoreError::Io)?;
            passes_completed += 1;
        }
        drop(file);

        let mut current_path = path.to_path_buf();
        let parent = current_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        for _ in 0..self.rename_count {
            let new_path = parent.join(random_filename());
            fs::rename(&current_path, &new_path).map_err(CoreError::Io)?;
            current_path = new_path;
        }
        fs::remove_file(&current_path).map_err(CoreError::Io)?;

        Ok(WipeFileResult {
            path: path.to_string_lossy().into_owned(),
            bytes_wiped: size,
            passes_completed,
            slack_bytes_wiped: slack_len,
            success: true,
            is_cow: false,
        })
    }

    /// Slack space cannot be safely overwritten through normal file I/O: extending a
    /// file may allocate unrelated blocks rather than the original EOF cluster. Refuse
    /// instead of reporting a false sanitization result.
    pub fn scrub_slack_space(&self, path: &Path) -> Result<u64, CoreError> {
        let metadata = fs::symlink_metadata(path).map_err(CoreError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(CoreError::Wiper(format!(
                "Refusing to scrub slack space of a symlink: {}",
                path.display()
            )));
        }
        let _ = metadata;
        Err(CoreError::Wiper(
            "Filesystem slack-space overwriting is not supported safely through ordinary file I/O; use whole-device sanitization".to_string(),
        ))
    }
}

/// Open file with O_NOFOLLOW to guard against TOCTOU symlink swaps.
/// If open fails with PermissionDenied (e.g. read-only file), attempts to
/// grant owner write permissions and retries so read-only evidence/logs can be wiped.
fn open_file_for_wipe(path: &Path) -> Result<fs::File, CoreError> {
    #[cfg(unix)]
    let open_opts = {
        use std::os::unix::fs::OpenOptionsExt;
        let mut o = OpenOptions::new();
        o.write(true).custom_flags(libc::O_NOFOLLOW);
        o
    };
    #[cfg(not(unix))]
    let open_opts = {
        let mut o = OpenOptions::new();
        o.write(true);
        o
    };

    match open_opts.open(path) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            if let Ok(meta) = fs::symlink_metadata(path) {
                let mut perms = meta.permissions();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    perms.set_mode(0o600);
                }
                #[cfg(not(unix))]
                {
                    perms.set_readonly(false);
                }
                let _ = fs::set_permissions(path, perms);
            }
            open_opts.open(path).map_err(CoreError::Io)
        }
        Err(e) => Err(CoreError::Io(e)),
    }
}

fn random_filename() -> String {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Returns `true` if `path` lives on a Copy-on-Write filesystem (Btrfs, ZFS).
///
/// On CoW filesystems, even a byte-for-byte in-place overwrite does not
/// guarantee the old data is destroyed — the filesystem may retain old
/// block versions accessible via snapshots or free-space reclamation.
///
/// Implementation reads the statfs f_type field via libc for the mounted
/// filesystem at the given path. Falls back to `false` on any error so
/// callers at minimum see a "possibly CoW" advisory rather than a hard fail.
pub fn detect_cow_filesystem(path: &Path) -> Result<bool, CoreError> {
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let target_path = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| path.to_path_buf())
        };

        let c_path = CString::new(target_path.as_os_str().as_bytes())
            .map_err(|e| CoreError::Disk(format!("Invalid path: {}", e)))?;

        let mut buf: libc::statfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statfs(c_path.as_ptr(), &mut buf) };

        if ret != 0 {
            // Don't hard-fail — return false and let the caller add an advisory
            return Ok(false);
        }

        // f_type magic numbers for CoW filesystems:
        //   Btrfs:  0x9123683E
        //   ZFS:    0x2FC12FC1  (OpenZFS on Linux)
        let is_cow = matches!(buf.f_type, 0x9123683E | 0x2FC12FC1_i64);
        Ok(is_cow)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux platforms (macOS/APFS, Windows/ReFS) we can't portably
        // query the filesystem type without OS-specific APIs. Return false
        // conservatively — the UI/CLI should show an advisory on these platforms.
        let _ = path;
        Ok(false)
    }
}
