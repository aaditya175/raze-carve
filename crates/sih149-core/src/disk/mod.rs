pub mod block_device;
pub mod ewf;
pub mod raw_image;
pub mod surface_scanner;

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Trait representing a disk source that can be read from, written to, and sought.
pub trait DiskSource: Read + Write + Seek + Send + Sync {
    /// Get the total size of the disk source in bytes.
    fn size(&self) -> Result<u64>;

    /// Get the sector size of the disk source in bytes.
    fn sector_size(&self) -> Result<u32>;
}

/// Unified evidence source that transparently wraps physical block devices,
/// Expert Witness Format (E01) forensic images, or raw (.dd/.img) disk images.
pub enum EvidenceSource {
    Block(block_device::BlockDevice),
    E01(ewf::E01Image),
    Raw(raw_image::RawImage),
}

impl EvidenceSource {
    /// Open an evidence path, automatically detecting if it is a block device,
    /// an E01 image, or a raw disk image, and opening with read-only protections.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        let s = p.to_string_lossy();
        if s.starts_with("/dev/") {
            block_device::BlockDevice::open_readonly(p).map(EvidenceSource::Block)
        } else if ewf::E01Image::is_e01(p) {
            ewf::E01Image::open(p).map(EvidenceSource::E01)
        } else {
            raw_image::RawImage::open(p).map(EvidenceSource::Raw)
        }
    }
}

impl Read for EvidenceSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Block(d) => d.read(buf),
            Self::E01(d) => d.read(buf),
            Self::Raw(d) => d.read(buf),
        }
    }
}

impl Write for EvidenceSource {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Block(d) => d.write(buf),
            Self::E01(d) => d.write(buf),
            Self::Raw(d) => d.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Block(d) => d.flush(),
            Self::E01(d) => d.flush(),
            Self::Raw(d) => d.flush(),
        }
    }
}

impl Seek for EvidenceSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Block(d) => d.seek(pos),
            Self::E01(d) => d.seek(pos),
            Self::Raw(d) => d.seek(pos),
        }
    }
}

impl DiskSource for EvidenceSource {
    fn size(&self) -> Result<u64> {
        match self {
            Self::Block(d) => d.size(),
            Self::E01(d) => d.size(),
            Self::Raw(d) => d.size(),
        }
    }

    fn sector_size(&self) -> Result<u32> {
        match self {
            Self::Block(d) => d.sector_size(),
            Self::E01(d) => d.sector_size(),
            Self::Raw(d) => d.sector_size(),
        }
    }
}
