use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CarvedFsMetadata {
    pub filesystem_type: String,
    pub filename: String,
    pub file_extension: String,
    pub file_size_bytes: u64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub is_resident: bool,
    pub resident_bytes: Option<Vec<u8>>,
    pub cluster_pointers: Vec<(u64, u64)>, // (count, start_cluster)
    pub attributes: HashMap<String, String>,
}

/// Scans a block buffer for NTFS `$MFT` record remnants (standard 1024-byte records).
/// When unallocated space is scanned, deleted MFT records retain complete metadata:
/// original Unicode filename, exact size in bytes, timestamps, and data runs.
pub fn scan_ntfs_mft_records(buffer: &[u8], base_offset: u64) -> Vec<(u64, CarvedFsMetadata)> {
    let mut results = Vec::new();
    let record_size = 1024;

    if buffer.len() < record_size {
        return results;
    }

    let mut pos = 0;
    while pos + record_size <= buffer.len() {
        // MFT record starts with "FILE" (0x454C4946) or "BAAD" (corrupt)
        if &buffer[pos..pos + 4] == b"FILE" {
            let record = &buffer[pos..pos + record_size];
            if let Some(meta) = parse_ntfs_mft_record(record) {
                let hit_offset = base_offset + pos as u64;
                results.push((hit_offset, meta));
            }
        }
        pos += 512; // MFT records are 512-byte sector aligned
    }

    results
}

/// Unapplies NTFS Update Sequence Array (fixups) to prevent corrupted attribute or data boundary bytes.
fn unapply_ntfs_fixups(record: &mut [u8]) {
    if record.len() < 48 {
        return;
    }
    let usa_offset = u16::from_le_bytes([record[4], record[5]]) as usize;
    let usa_count = u16::from_le_bytes([record[6], record[7]]) as usize;

    if usa_offset < 42 || usa_count < 2 || usa_offset + usa_count * 2 > record.len() {
        return;
    }

    let usn = [record[usa_offset], record[usa_offset + 1]];
    for sector_idx in 0..(usa_count - 1) {
        let sector_end = (sector_idx + 1) * 512 - 2;
        if sector_end + 2 <= record.len() && record[sector_end..sector_end + 2] == usn {
            let fixup_off = usa_offset + (sector_idx + 1) * 2;
            record[sector_end] = record[fixup_off];
            record[sector_end + 1] = record[fixup_off + 1];
        }
    }
}

/// Parses NTFS non-resident data runs into a vector of (cluster_count, start_lcn).
fn parse_ntfs_data_runs(record: &[u8], run_start: usize, run_limit: usize) -> Vec<(u64, u64)> {
    let mut runs = Vec::new();
    let mut pos = run_start;
    let mut prev_lcn: i64 = 0;

    while pos < run_limit && pos < record.len() {
        let header = record[pos];
        if header == 0 {
            break;
        }
        pos += 1;

        let len_size = (header & 0x0F) as usize;
        let off_size = ((header >> 4) & 0x0F) as usize;

        if len_size == 0 || len_size > 8 || off_size > 8 {
            break;
        }
        if pos + len_size + off_size > run_limit || pos + len_size + off_size > record.len() {
            break;
        }

        let mut len_buf = [0u8; 8];
        len_buf[..len_size].copy_from_slice(&record[pos..pos + len_size]);
        let cluster_count = u64::from_le_bytes(len_buf);
        pos += len_size;

        if off_size > 0 {
            let mut off_buf = [0u8; 8];
            off_buf[..off_size].copy_from_slice(&record[pos..pos + off_size]);
            let is_negative = (record[pos + off_size - 1] & 0x80) != 0;
            if is_negative {
                off_buf[off_size..8].fill(0xFF);
            }
            let lcn_delta = i64::from_le_bytes(off_buf);
            pos += off_size;
            prev_lcn = prev_lcn.wrapping_add(lcn_delta);
            if prev_lcn >= 0 && cluster_count > 0 {
                runs.push((cluster_count, prev_lcn as u64));
            }
        } else if cluster_count > 0 {
            // Sparse cluster run
            runs.push((cluster_count, 0));
        }
    }

    runs
}

/// Parses an NTFS 1024-byte MFT record structure.
fn parse_ntfs_mft_record(record_slice: &[u8]) -> Option<CarvedFsMetadata> {
    if record_slice.len() < 48 {
        return None;
    }

    let mut record = record_slice.to_vec();
    unapply_ntfs_fixups(&mut record);

    // Offset 20: 2-byte first attribute offset
    let first_attr_offset = u16::from_le_bytes([record[20], record[21]]) as usize;
    if first_attr_offset < 48 || first_attr_offset >= record.len() {
        return None;
    }

    // Offset 22: 2-byte flags (0x01: in-use, 0x02: directory)
    let flags = u16::from_le_bytes([record[22], record[23]]);
    let is_directory = (flags & 0x02) != 0;
    if is_directory {
        return None; // Focus on carving files
    }

    let mut attr_offset = first_attr_offset;
    let mut filename = String::new();
    let mut ext = String::new();
    let mut file_size: u64 = 0;
    let mut created_at: Option<String> = None;
    let mut modified_at: Option<String> = None;
    let mut is_resident = false;
    let mut resident_bytes: Option<Vec<u8>> = None;
    let mut cluster_runs: Vec<(u64, u64)> = Vec::new();
    let mut attributes = HashMap::new();

    while attr_offset + 16 <= record.len() {
        let attr_type = u32::from_le_bytes([
            record[attr_offset],
            record[attr_offset + 1],
            record[attr_offset + 2],
            record[attr_offset + 3],
        ]);

        if attr_type == 0xFFFFFFFF || attr_type == 0 {
            break; // End of attributes marker
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

        let non_resident_flag = record[attr_offset + 8];

        match attr_type {
            // $STANDARD_INFORMATION (0x10)
            0x10 => {
                if non_resident_flag == 0 && attr_len >= 24 {
                    let val_offset =
                        u16::from_le_bytes([record[attr_offset + 20], record[attr_offset + 21]])
                            as usize;
                    let payload = attr_offset + val_offset;
                    if val_offset >= 24
                        && payload + 32 <= attr_offset + attr_len
                        && payload + 32 <= record.len()
                    {
                        let ctime = u64::from_le_bytes(
                            record[payload..payload + 8].try_into().unwrap_or([0; 8]),
                        );
                        let mtime = u64::from_le_bytes(
                            record[payload + 8..payload + 16]
                                .try_into()
                                .unwrap_or([0; 8]),
                        );
                        created_at = filetime_to_iso8601(ctime);
                        modified_at = filetime_to_iso8601(mtime);
                    }
                }
            }
            // $FILE_NAME (0x30)
            0x30 => {
                if non_resident_flag == 0 && attr_len >= 24 {
                    let val_offset =
                        u16::from_le_bytes([record[attr_offset + 20], record[attr_offset + 21]])
                            as usize;
                    let payload = attr_offset + val_offset;
                    if val_offset >= 24
                        && payload + 66 <= attr_offset + attr_len
                        && payload + 66 <= record.len()
                    {
                        let real_size = u64::from_le_bytes(
                            record[payload + 48..payload + 56]
                                .try_into()
                                .unwrap_or([0; 8]),
                        );
                        if file_size == 0 && real_size > 0 {
                            file_size = real_size;
                        }
                        let name_len = record[payload + 64] as usize;
                        let name_type = record[payload + 65]; // 0: POSIX, 1: Win32, 2: DOS, 3: Win32 & DOS
                        let name_start = payload + 66;
                        if name_len > 0
                            && name_start + name_len * 2 <= attr_offset + attr_len
                            && name_start + name_len * 2 <= record.len()
                        {
                            let mut utf16_units = Vec::with_capacity(name_len);
                            for i in 0..name_len {
                                let b0 = record[name_start + i * 2];
                                let b1 = record[name_start + i * 2 + 1];
                                utf16_units.push(u16::from_le_bytes([b0, b1]));
                            }
                            if let Ok(decoded) = String::from_utf16(&utf16_units) {
                                let clean_name = decoded.trim().trim_matches('\0');
                                if !clean_name.is_empty() {
                                    // Prioritize Win32/Win32+DOS (1 or 3) over DOS 8.3 (2)
                                    let should_replace = match name_type {
                                        1 | 3 => true,
                                        2 => filename.is_empty(),
                                        _ => filename.is_empty() || filename.contains('~'),
                                    };
                                    if should_replace
                                        && (!clean_name.starts_with('$') || filename.is_empty())
                                    {
                                        filename = clean_name.to_string();
                                        if let Some(pos) = filename.rfind('.') {
                                            ext = filename[pos + 1..].to_lowercase();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // $DATA (0x80)
            0x80 => {
                if non_resident_flag == 0 {
                    if attr_len >= 24 {
                        is_resident = true;
                        let val_offset = u16::from_le_bytes([
                            record[attr_offset + 20],
                            record[attr_offset + 21],
                        ]) as usize;
                        let val_len = u32::from_le_bytes([
                            record[attr_offset + 16],
                            record[attr_offset + 17],
                            record[attr_offset + 18],
                            record[attr_offset + 19],
                        ]) as usize;
                        let payload_start = attr_offset + val_offset;
                        if val_offset >= 24
                            && payload_start + val_len <= attr_offset + attr_len
                            && payload_start + val_len <= record.len()
                        {
                            file_size = val_len as u64;
                            resident_bytes =
                                Some(record[payload_start..payload_start + val_len].to_vec());
                        }
                    }
                } else if attr_len >= 64 && attr_offset + 64 <= record.len() {
                    // Non-resident $DATA: reads real size from non-resident header
                    let real_size = u64::from_le_bytes(
                        record[attr_offset + 48..attr_offset + 56]
                            .try_into()
                            .unwrap_or([0; 8]),
                    );
                    if real_size > 0 {
                        file_size = real_size;
                    }
                    let run_offset =
                        u16::from_le_bytes([record[attr_offset + 32], record[attr_offset + 33]])
                            as usize;
                    if run_offset >= 64 && attr_offset + run_offset < attr_offset + attr_len {
                        cluster_runs = parse_ntfs_data_runs(
                            &record,
                            attr_offset + run_offset,
                            attr_offset + attr_len,
                        );
                    }
                }
            }
            _ => {}
        }

        attr_offset += attr_len;
    }

    if filename.is_empty() {
        return None;
    }

    attributes.insert("NTFS_Record".to_string(), "MFT Inode Remnant".to_string());
    if is_resident {
        attributes.insert(
            "Data_Storage".to_string(),
            "Resident (in-record payload)".to_string(),
        );
    } else {
        attributes.insert(
            "Data_Storage".to_string(),
            format!("Non-resident cluster run ({} segments)", cluster_runs.len()),
        );
    }

    Some(CarvedFsMetadata {
        filesystem_type: "NTFS".to_string(),
        filename,
        file_extension: if ext.is_empty() {
            "bin".to_string()
        } else {
            ext
        },
        file_size_bytes: file_size,
        created_at,
        modified_at,
        is_resident,
        resident_bytes,
        cluster_pointers: cluster_runs,
        attributes,
    })
}

/// Converts Windows 64-bit FILETIME (100-nanosecond intervals since Jan 1 1601) to ISO-8601 string.
fn filetime_to_iso8601(filetime: u64) -> Option<String> {
    if filetime == 0 {
        return None;
    }
    // 116444736000000000 is 100ns intervals between 1601 and 1970
    if filetime < 116444736000000000 {
        return None;
    }
    let unix_100ns = filetime - 116444736000000000;
    let unix_secs = (unix_100ns / 10_000_000) as i64;
    let nsecs = ((unix_100ns % 10_000_000) * 100) as u32;

    Utc.timestamp_opt(unix_secs, nsecs)
        .single()
        .map(|dt| dt.to_rfc3339())
}

/// Computes the 8-bit checksum for an 11-byte 8.3 short filename as defined in Microsoft FAT specification.
pub fn fat_lfn_checksum(sfn: &[u8; 11]) -> u8 {
    let mut sum: u8 = 0;
    for &b in sfn {
        sum = ((sum & 1) << 7).wrapping_add(sum >> 1).wrapping_add(b);
    }
    sum
}

/// Extracts up to 13 UTF-16LE units from a 32-byte VFAT LFN directory entry.
fn extract_lfn_units(entry: &[u8]) -> Vec<u16> {
    let mut units = Vec::with_capacity(13);
    for &i in &[1, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30] {
        if i + 1 < entry.len() {
            units.push(u16::from_le_bytes([entry[i], entry[i + 1]]));
        }
    }
    units
}

/// Reconstructs a full long filename from collected VFAT LFN entries preceding an 8.3 entry.
fn reconstruct_lfn(parts: &[Vec<u16>], lfn_chk: Option<u8>, sfn_entry: &[u8]) -> Option<String> {
    if parts.is_empty() || sfn_entry.len() < 11 {
        return None;
    }

    // Verify that checksum matches the 8.3 directory entry
    if let Some(target_chk) = lfn_chk {
        let mut sfn_raw = [0u8; 11];
        sfn_raw.copy_from_slice(&sfn_entry[0..11]);
        // For deleted entries where sfn_entry[0] is 0xE5, test plausible ASCII first characters
        let matches = (32u8..=126u8).any(|c| {
            let mut test_sfn = sfn_raw;
            test_sfn[0] = c;
            fat_lfn_checksum(&test_sfn) == target_chk
        });
        if !matches && fat_lfn_checksum(&sfn_raw) != target_chk {
            return None;
        }
    }

    // LFN parts are stored in reverse physical order (part N ... part 1)
    let mut all_units = Vec::new();
    for part in parts.iter().rev() {
        for &u in part {
            if u == 0x0000 || u == 0xFFFF {
                break;
            }
            all_units.push(u);
        }
    }

    if all_units.is_empty() {
        return None;
    }

    let decoded = String::from_utf16_lossy(&all_units);
    let sanitized: String = decoded
        .chars()
        .filter(|&c| c != '\0' && c != '/' && c != '\\')
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Scans a block buffer for FAT/FAT32 deleted directory entries (32-byte records).
/// When files are deleted in FAT, the first byte is replaced with 0xE5, but the rest
/// of the 32-byte record remains intact (filename, extension, size, cluster, timestamps).
pub fn scan_fat_directory_entries(buffer: &[u8], base_offset: u64) -> Vec<(u64, CarvedFsMetadata)> {
    let mut results = Vec::new();
    let entry_size = 32;

    if buffer.len() < entry_size {
        return results;
    }

    let mut pending_lfn_parts: Vec<Vec<u16>> = Vec::new();
    let mut pending_lfn_chk: Option<u8> = None;

    let mut pos = 0;
    while pos + entry_size <= buffer.len() {
        let entry = &buffer[pos..pos + entry_size];
        let attr = entry[11];

        if attr == 0x0F {
            // VFAT LFN entry
            let chk = entry[13];
            if pending_lfn_chk.is_some() && pending_lfn_chk != Some(chk) {
                pending_lfn_parts.clear();
            }
            pending_lfn_chk = Some(chk);
            pending_lfn_parts.push(extract_lfn_units(entry));
        } else {
            // 0xE5 indicates deleted entry; valid entries also processed if looking for residual records
            if entry[0] == 0xE5 {
                if let Some(mut meta) = parse_fat_directory_entry(entry) {
                    if !pending_lfn_parts.is_empty() {
                        if let Some(lfn) =
                            reconstruct_lfn(&pending_lfn_parts, pending_lfn_chk, entry)
                        {
                            meta.filename = lfn.clone();
                            if let Some(p) = lfn.rfind('.') {
                                meta.file_extension = lfn[p + 1..].to_lowercase();
                            }
                            meta.attributes.insert(
                                "FAT_Name_Type".to_string(),
                                "VFAT Long File Name (LFN)".to_string(),
                            );
                        }
                    }
                    let hit_offset = base_offset + pos as u64;
                    results.push((hit_offset, meta));
                }
            }
            pending_lfn_parts.clear();
            pending_lfn_chk = None;
        }

        pos += entry_size;
    }

    results
}

/// Parses a 32-byte FAT directory entry.
fn parse_fat_directory_entry(entry: &[u8]) -> Option<CarvedFsMetadata> {
    let attr = entry[11];
    // 0x0F is VFAT LFN entry, 0x10 is subdirectory, 0x08 is volume label
    if attr == 0x0F || (attr & 0x10) != 0 || (attr & 0x08) != 0 {
        return None;
    }

    // Name bytes 1..8 (skip deleted marker at byte 0)
    let raw_name = &entry[1..8];
    let raw_ext = &entry[8..11];

    let is_valid_char = |b: u8| {
        b.is_ascii_alphanumeric()
            || b == b'_'
            || b == b'-'
            || b == b' '
            || b == b'~'
            || b == b'$'
            || b == b'#'
            || b == b'&'
            || b == b'@'
            || b == b'%'
            || b == b'!'
            || b == b'\''
            || b == b'('
            || b == b')'
            || b == b'^'
            || b == b'`'
            || b == b'{'
            || b == b'}'
    };
    if !raw_name.iter().all(|&b| is_valid_char(b)) || !raw_ext.iter().all(|&b| is_valid_char(b)) {
        return None;
    }

    let name_str: String = raw_name
        .iter()
        .filter(|&&b| b != b' ')
        .map(|&b| b as char)
        .collect();
    let ext_str: String = raw_ext
        .iter()
        .filter(|&&b| b != b' ')
        .map(|&b| (b as char).to_ascii_lowercase())
        .collect();

    if name_str.is_empty() {
        return None;
    }

    let full_name = if ext_str.is_empty() {
        format!("_{}", name_str)
    } else {
        format!("_{}.{}", name_str, ext_str)
    };

    // File size at offset 28 (4 bytes, little-endian)
    let file_size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]) as u64;
    if file_size == 0 || file_size > 1024 * 1024 * 1024 {
        return None; // Filter unreasonable sizes
    }

    // Starting cluster
    let cluster_high = u16::from_le_bytes([entry[20], entry[21]]) as u64;
    let cluster_low = u16::from_le_bytes([entry[26], entry[27]]) as u64;
    let start_cluster = (cluster_high << 16) | cluster_low;

    // Creation date/time at offset 14..18
    let ctime = u16::from_le_bytes([entry[14], entry[15]]);
    let cdate = u16::from_le_bytes([entry[16], entry[17]]);
    let created_at = parse_fat_datetime(cdate, ctime);

    // Modification date/time at offset 22..26
    let mtime = u16::from_le_bytes([entry[22], entry[23]]);
    let mdate = u16::from_le_bytes([entry[24], entry[25]]);
    let modified_at = parse_fat_datetime(mdate, mtime);

    let mut attributes = HashMap::new();
    attributes.insert(
        "FAT_Entry".to_string(),
        "Deleted 32-byte DirRecord".to_string(),
    );
    attributes.insert("Start_Cluster".to_string(), format!("{}", start_cluster));

    Some(CarvedFsMetadata {
        filesystem_type: "FAT32".to_string(),
        filename: full_name,
        file_extension: if ext_str.is_empty() {
            "bin".to_string()
        } else {
            ext_str
        },
        file_size_bytes: file_size,
        created_at,
        modified_at,
        is_resident: false,
        resident_bytes: None,
        cluster_pointers: vec![(file_size.div_ceil(4096), start_cluster)],
        attributes,
    })
}

fn parse_fat_datetime(date: u16, time: u16) -> Option<String> {
    let year = ((date >> 9) + 1980) as i32;
    let month = ((date >> 5) & 0x0F) as u32;
    let day = (date & 0x1F) as u32;

    let hour = (time >> 11) as u32;
    let minute = ((time >> 5) & 0x3F) as u32;
    let second = std::cmp::min(((time & 0x1F) as u32) * 2, 59);

    if (1..=12).contains(&month) && (1..=31).contains(&day) && hour < 24 && minute < 60 {
        Some(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hour, minute, second
        ))
    } else {
        None
    }
}
