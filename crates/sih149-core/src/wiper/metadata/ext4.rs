//! ext2/ext3/ext4 metadata scrubber.
//!
//! Targets:
//! - Inode fields (timestamps, size, block pointers, extended attributes)
//! - Directory entries (dirent records in parent directory blocks)
//! - Journal references (ext3/ext4 journal transaction records)

/// Scrubs an ext2/ext3/ext4 inode record (typically 128, 156, or 256 bytes) in-place:
/// Sanitizes mode, uid/gid, file size, timestamps (atime, mtime, ctime, crtime, dtime),
/// block pointers and extent headers (0xF30A), and extended attribute blocks.
/// Returns true if an inode containing metadata was recognized and sanitized.
pub fn scrub_ext4_inode(inode: &mut [u8]) -> bool {
    if inode.len() < 128 {
        return false;
    }

    // If completely zeroed, nothing to scrub
    if inode.iter().all(|&b| b == 0) {
        return false;
    }

    // Exclude NTFS MFT signatures (FILE / BAAD)
    if &inode[0..4] == b"FILE" || &inode[0..4] == b"BAAD" {
        return false;
    }

    let has_extent_magic = inode[40..42] == [0x0A, 0xF3];
    let mode = u16::from_le_bytes([inode[0], inode[1]]);
    let file_type = mode & 0xF000;
    let has_valid_mode = matches!(
        file_type,
        0x8000 | 0x4000 | 0xA000 | 0x2000 | 0x6000 | 0x1000
    );
    let size = u32::from_le_bytes([inode[4], inode[5], inode[6], inode[7]]);
    let has_size = size > 0;
    let has_timestamps = inode[8..24].iter().any(|&b| b != 0);

    if !has_extent_magic && !(has_valid_mode && (has_size || has_timestamps)) {
        return false;
    }

    // Zero out the entire inode structure to eliminate residual file data and block pointers
    inode.fill(0);
    true
}

/// Scrubs an ext2/ext3/ext4 directory entry (dirent / dirent2) in-place.
/// Sanitizes the target inode number and filename characters, preventing dirent reconstruction.
/// Returns true if a directory entry was sanitized.
pub fn scrub_ext4_dir_entry(dirent: &mut [u8]) -> bool {
    if dirent.len() < 8 {
        return false;
    }
    let inode_num = u32::from_le_bytes([dirent[0], dirent[1], dirent[2], dirent[3]]);
    let rec_len = u16::from_le_bytes([dirent[4], dirent[5]]) as usize;
    let name_len = dirent[6] as usize;

    if rec_len < 8 || rec_len > dirent.len() || name_len == 0 || 8 + name_len > rec_len {
        return false;
    }

    if inode_num == 0 && dirent[8..8 + name_len].iter().all(|&b| b == 0) {
        return false;
    }

    // Zero inode number to mark unlinked/unused
    dirent[0..4].fill(0);
    // Zero filename and name_len to wipe residual name leakage
    dirent[6] = 0;
    dirent[7] = 0; // file_type
    dirent[8..8 + name_len].fill(0);
    true
}
