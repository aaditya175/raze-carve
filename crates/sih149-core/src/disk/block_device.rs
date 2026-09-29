use crate::disk::DiskSource;
use crate::error::Result;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

// BLKGETSIZE64 ioctl: returns device size in bytes (u64).
// Defined by the Linux kernel; not exposed by std, so we call it via nix.
nix::ioctl_read!(ioctl_blkgetsize64, 0x12, 114, u64);

// BLKSSZGET ioctl: returns logical sector size in bytes (i32).
// In Linux <linux/fs.h>, BLKSSZGET is _IO(0x12, 104) = 0x1268.
nix::ioctl_read_bad!(ioctl_blksszget, 0x1268, i32);

/// A Linux block device reader/writer.
pub struct BlockDevice {
    file: File,
    size: u64,
    sector_size: u32,
}

impl BlockDevice {
    /// Open a block device at the given path with read/write permissions.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_internal(path, true)
    }

    /// Open a block device in strict read-only mode (software write-blocking)
    /// to preserve forensic integrity during recovery or analysis.
    pub fn open_readonly<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_internal(path, false)
    }

    /// Open a real block device for destructive work. Raw image files remain
    /// valid evidence sources through `open_readonly`, but are never accepted
    /// as a wipe target by mistake.
    pub fn open_for_wipe<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !is_block_device(path) {
            return Err(crate::error::CoreError::Disk(format!(
                "Refusing destructive operation on non-block-device path: {}",
                path.display()
            )));
        }
        Self::open_internal(path, true)
    }

    fn open_internal<P: AsRef<Path>>(path: P, writable: bool) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(writable).open(&path)?;

        let fd = file.as_raw_fd();
        let mut size: u64 = 0;
        let mut sector_size: i32 = 512;

        // Try the block-device ioctls first (this is the real device size).
        // If they fail (e.g. path is a regular file, not a block device),
        // fall back to file metadata so raw disk-image files still work.
        let is_block_device = unsafe { ioctl_blkgetsize64(fd, &mut size) }.is_ok();
        if !is_block_device {
            size = file.metadata()?.len();
        } else {
            // Sector size ioctl is best-effort; 512 is a safe default if it fails.
            let _ = unsafe { ioctl_blksszget(fd, &mut sector_size) };
        }

        if size == 0 {
            return Err(crate::error::CoreError::Disk(format!(
                "Could not determine size of {:?} (ioctl and metadata both returned 0)",
                path.as_ref()
            )));
        }

        Ok(Self {
            file,
            size,
            sector_size: if sector_size > 0 {
                sector_size as u32
            } else {
                512
            },
        })
    }
}

/// Whether a path is a real block device. This is intentionally public so all
/// UI, CLI, TUI and server entry points apply the same destructive-operation
/// guard before an open/write is attempted.
pub fn is_block_device(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        std::fs::metadata(path)
            .map(|metadata| metadata.file_type().is_block_device())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

impl BlockDevice {
    /// Flush userspace data and force the kernel to commit it before a pass is
    /// considered complete. `Write::flush` alone does not provide this
    /// durability boundary for a block device.
    pub fn sync_all(&self) -> Result<()> {
        self.file.sync_all().map_err(crate::error::CoreError::Io)
    }
}

impl Read for BlockDevice {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for BlockDevice {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for BlockDevice {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl DiskSource for BlockDevice {
    fn size(&self) -> Result<u64> {
        Ok(self.size)
    }

    fn sector_size(&self) -> Result<u32> {
        Ok(self.sector_size)
    }
}
