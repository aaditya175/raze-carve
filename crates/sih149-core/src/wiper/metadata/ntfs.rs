//! NTFS metadata scrubber.
//!
//! Targets:
//! - MFT (Master File Table) record for the file
//!   - $STANDARD_INFORMATION attribute
//!   - $FILE_NAME attribute(s)
//!   - $DATA attribute (resident data for small files)
//!   - Run lists for non-resident data
//! - $LogFile journal references
//! - $UsnJrnl (Update Sequence Number Journal) entries

/// Scrubs an NTFS MFT record (1024 bytes) in-place:
/// Sanitizes $FILE_NAME attributes (timestamps, original filename),
/// $DATA resident payload bytes, and cluster runlists to prevent forensic reconstruction.
/// Returns true if an MFT record was recognized and sanitized.
pub fn scrub_ntfs_mft_record(record: &mut [u8]) -> bool {
    if record.len() < 1024 || &record[0..4] != b"FILE" {
        return false;
    }

    let first_attr_offset = u16::from_le_bytes([record[20], record[21]]) as usize;
    if first_attr_offset >= 48 && first_attr_offset < record.len() {
        let mut attr_offset = first_attr_offset;
        while attr_offset + 16 <= record.len() {
            let attr_type = u32::from_le_bytes([
                record[attr_offset],
                record[attr_offset + 1],
                record[attr_offset + 2],
                record[attr_offset + 3],
            ]);
            if attr_type == 0xFFFFFFFF || attr_type == 0 {
                break;
            }
            let attr_len = u32::from_le_bytes([
                record[attr_offset + 4],
                record[attr_offset + 5],
                record[attr_offset + 6],
                record[attr_offset + 7],
            ]) as usize;
            if attr_len < 16 || attr_offset + attr_len > record.len() {
                break;
            }

            // Zero out attribute payload (from val_offset to attr_len)
            let non_resident = record[attr_offset + 8];
            if non_resident == 0 && attr_len >= 24 {
                let val_offset =
                    u16::from_le_bytes([record[attr_offset + 20], record[attr_offset + 21]])
                        as usize;
                if val_offset >= 24 && attr_offset + val_offset < attr_offset + attr_len {
                    let start = attr_offset + val_offset;
                    let end = attr_offset + attr_len;
                    record[start..end].fill(0);
                }
            } else if non_resident == 1 && attr_len >= 64 {
                // Non-resident runlist sanitization
                let run_offset =
                    u16::from_le_bytes([record[attr_offset + 32], record[attr_offset + 33]])
                        as usize;
                if run_offset >= 64 && attr_offset + run_offset < attr_offset + attr_len {
                    let start = attr_offset + run_offset;
                    let end = attr_offset + attr_len;
                    record[start..end].fill(0);
                }
            }

            attr_offset += attr_len;
        }
    }

    // Overwrite the magic signature to prevent future MFT carving
    record[0..4].copy_from_slice(b"BAAD");
    true
}
