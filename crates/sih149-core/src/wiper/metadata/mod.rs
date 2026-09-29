//! Filesystem-aware metadata scrubbing.
//!
//! After overwriting file content, residual metadata (filenames,
//! timestamps, allocation records) can still reveal information.
//! This module provides per-filesystem scrubbers.

pub mod ext4;
pub mod fat;
pub mod ntfs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MetadataScrubStats {
    pub ntfs_mft_scrubbed: usize,
    pub fat_entries_scrubbed: usize,
    pub ext4_inodes_scrubbed: usize,
}

impl MetadataScrubStats {
    pub fn total_scrubbed(&self) -> usize {
        self.ntfs_mft_scrubbed + self.fat_entries_scrubbed + self.ext4_inodes_scrubbed
    }
}

/// Scans a buffer for NTFS `$MFT` records on 512-byte sector boundaries
/// and sanitizes all identified records in-place.
pub fn scrub_ntfs_metadata_buffer(buffer: &mut [u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos + 1024 <= buffer.len() {
        if &buffer[pos..pos + 4] == b"FILE"
            && ntfs::scrub_ntfs_mft_record(&mut buffer[pos..pos + 1024])
        {
            count += 1;
            pos += 1024;
        } else {
            pos += 512;
        }
    }
    count
}

/// Scans a buffer for FAT/VFAT directory entries on 32-byte boundaries
/// and sanitizes all identified records in-place.
pub fn scrub_fat_metadata_buffer(buffer: &mut [u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos + 32 <= buffer.len() {
        if fat::scrub_fat_directory_entry(&mut buffer[pos..pos + 32]) {
            count += 1;
        }
        pos += 32;
    }
    count
}

/// Scans a buffer for ext4 inode structures on 256-byte or 128-byte boundaries
/// and sanitizes all identified inodes in-place.
pub fn scrub_ext4_metadata_buffer(buffer: &mut [u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos + 128 <= buffer.len() {
        if pos + 256 <= buffer.len() && ext4::scrub_ext4_inode(&mut buffer[pos..pos + 256]) {
            count += 1;
            pos += 256;
        } else if ext4::scrub_ext4_inode(&mut buffer[pos..pos + 128]) {
            count += 1;
            pos += 128;
        } else {
            pos += 128;
        }
    }
    count
}

/// Scans and sanitizes filesystem metadata remnants across a buffer
/// (NTFS MFT records, ext4 inodes, and FAT directory entries).
pub fn scrub_filesystem_metadata_buffer(buffer: &mut [u8]) -> MetadataScrubStats {
    let ntfs_mft_scrubbed = scrub_ntfs_metadata_buffer(buffer);
    let ext4_inodes_scrubbed = scrub_ext4_metadata_buffer(buffer);
    let fat_entries_scrubbed = scrub_fat_metadata_buffer(buffer);

    MetadataScrubStats {
        ntfs_mft_scrubbed,
        fat_entries_scrubbed,
        ext4_inodes_scrubbed,
    }
}
