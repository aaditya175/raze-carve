//! FAT12/FAT16/FAT32/exFAT metadata scrubber.
//!
//! Targets:
//! - Directory entry (32-byte record: filename, timestamps, first cluster)
//! - Long Filename (LFN) directory entries
//! - FAT chain entries (cluster allocation table)
//! - exFAT allocation bitmap

fn is_valid_sfn_char(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'_' | b'-'
                | b' '
                | b'~'
                | b'$'
                | b'#'
                | b'&'
                | b'@'
                | b'%'
                | b'!'
                | b'\''
                | b'('
                | b')'
                | b'^'
                | b'`'
                | b'{'
                | b'}'
        )
}

/// Scrubs a 32-byte FAT/VFAT directory entry in-place:
/// Sanitizes filename characters, timestamps, cluster pointers, and file size,
/// marking the entry with 0xE5 (deleted marker) and zeroing all payload bytes.
/// Returns true if an active or deleted FAT directory entry was sanitized.
pub fn scrub_fat_directory_entry(entry: &mut [u8]) -> bool {
    if entry.len() < 32 {
        return false;
    }

    // If completely zeroed or already scrubbed (0xE5 with all zeroes), nothing to do
    if entry[..32].iter().all(|&b| b == 0) {
        return false;
    }
    if entry[0] == 0xE5 && entry[1..32].iter().all(|&b| b == 0) {
        return false;
    }

    let attr = entry[11];
    let is_lfn = attr == 0x0F;

    if is_lfn {
        // LFN entries have type == 0 at byte 12 and cluster low == 0 at byte 26..28
        let is_valid_lfn = entry[12] == 0 && entry[26] == 0 && entry[27] == 0;
        if !is_valid_lfn {
            return false;
        }
    } else {
        // Check SFN directory entry validity
        let valid_attr = (attr & !0x3F) == 0;
        if !valid_attr {
            return false;
        }

        let valid_first = entry[0] == 0xE5 || is_valid_sfn_char(entry[0]);
        if !valid_first {
            return false;
        }

        let valid_name = entry[1..8].iter().all(|&b| is_valid_sfn_char(b));
        let valid_ext = entry[8..11].iter().all(|&b| is_valid_sfn_char(b));
        let not_all_spaces = !entry[1..11].iter().all(|&b| b == b' ');

        if !valid_name || !valid_ext || !not_all_spaces {
            return false;
        }
    }

    // Overwrite with deleted marker and zero out remaining fields
    entry[0] = 0xE5;
    entry[1..32].fill(0);
    true
}

/// Scrubs an entry in a FAT32 table buffer.
/// Replaces active or non-empty cluster links with 0 (unallocated cluster).
pub fn scrub_fat32_chain_entry(fat_buffer: &mut [u8], cluster_index: usize) -> bool {
    let offset = cluster_index * 4;
    if offset + 4 <= fat_buffer.len() {
        let val = u32::from_le_bytes([
            fat_buffer[offset],
            fat_buffer[offset + 1],
            fat_buffer[offset + 2],
            fat_buffer[offset + 3],
        ]) & 0x0FFFFFFF;
        if val != 0 {
            fat_buffer[offset..offset + 4].fill(0);
            return true;
        }
    }
    false
}
