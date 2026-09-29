//! E01 signature detection.
//!
//! A complete Expert Witness Format reader must interpret segment tables,
//! per-chunk compression, checksums, and multi-segment media. This crate does
//! not include such a decoder, so genuine E01 evidence is rejected instead of
//! being treated as a contiguous raw byte stream.

use crate::disk::DiskSource;
use crate::error::{CoreError, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Magic header bytes for Expert Witness Format (.E01): "EVF\t\r\n\xff\x00"
pub const E01_FILE_SIGNATURE: &[u8; 8] = b"EVF\t\r\n\xff\x00";

/// E01 image metadata parsed from segment headers
#[derive(Debug, Clone)]
pub struct E01Metadata {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub notes: Option<String>,
    pub media_size: u64,
    pub chunk_count: u32,
    pub bytes_per_sector: u32,
}

/// E01 Image Reader implementing DiskSource for forensic carving and analysis
pub struct E01Image {
    file: File,
    size: u64,
    sector_size: u32,
    payload_offset: u64,
    pub metadata: E01Metadata,
}

impl E01Image {
    /// Open and validate an E01 forensic image file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path).map_err(CoreError::Io)?;
        let file_len = file.seek(SeekFrom::End(0)).map_err(CoreError::Io)?;
        file.seek(SeekFrom::Start(0)).map_err(CoreError::Io)?;

        if file_len < 13 {
            return Err(CoreError::Disk(format!(
                "File too small to be valid E01: {} bytes",
                file_len
            )));
        }

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic).map_err(CoreError::Io)?;

        if &magic == E01_FILE_SIGNATURE {
            return Err(CoreError::Disk(
                "E01 evidence is detected but is not supported by this build; export a verified raw image with a full EWF tool before analysis"
                    .to_string(),
            ));
        }

        if &magic != E01_FILE_SIGNATURE {
            return Err(CoreError::Disk(format!(
                "Invalid E01 magic signature: expected {:?}, got {:?}",
                E01_FILE_SIGNATURE, magic
            )));
        }

        // Read fields: segment number (u16), total segments (u16)
        let mut seg_info = [0u8; 4];
        file.read_exact(&mut seg_info).map_err(CoreError::Io)?;

        let mut metadata = E01Metadata {
            case_number: None,
            evidence_number: None,
            examiner_name: None,
            notes: None,
            media_size: file_len.saturating_sub(13),
            chunk_count: (file_len / 32768) as u32,
            bytes_per_sector: 512,
        };

        let mut payload_offset = 13u64;
        let mut size = file_len.saturating_sub(13);
        let mut sector_size = 512u32;

        // Inspect for standard E01 section descriptors (76 bytes per section header)
        if file_len >= 13 + 76 {
            let mut sec_header = [0u8; 76];
            let mut sec_offset = 13u64;
            let mut visited = std::collections::HashSet::new();
            let mut section_count: usize = 0;
            const MAX_SECTIONS: usize = 4096;

            while sec_offset.saturating_add(76) <= file_len && section_count < MAX_SECTIONS {
                if !visited.insert(sec_offset) {
                    break;
                }
                section_count += 1;

                if file.seek(SeekFrom::Start(sec_offset)).is_err() {
                    break;
                }
                if file.read_exact(&mut sec_header).is_err() {
                    break;
                }

                let sec_name_bytes = &sec_header[0..16];
                let null_pos = sec_name_bytes.iter().position(|&b| b == 0).unwrap_or(16);
                let sec_name = String::from_utf8_lossy(&sec_name_bytes[..null_pos]);

                let next_sec = u64::from_le_bytes([
                    sec_header[16],
                    sec_header[17],
                    sec_header[18],
                    sec_header[19],
                    sec_header[20],
                    sec_header[21],
                    sec_header[22],
                    sec_header[23],
                ]);
                let sec_size = u64::from_le_bytes([
                    sec_header[24],
                    sec_header[25],
                    sec_header[26],
                    sec_header[27],
                    sec_header[28],
                    sec_header[29],
                    sec_header[30],
                    sec_header[31],
                ]);

                if sec_name.eq_ignore_ascii_case("header")
                    || sec_name.eq_ignore_ascii_case("header2")
                {
                    let data_len = std::cmp::min(sec_size as usize, 65536);
                    let mut data_buf = vec![0u8; data_len];
                    if file.read_exact(&mut data_buf).is_ok() {
                        let text = String::from_utf8_lossy(&data_buf);
                        for line in text.lines() {
                            let parts: Vec<&str> = line.split(&['\t', ':'][..]).collect();
                            if parts.len() >= 2 {
                                let key = parts[0].trim().to_ascii_lowercase();
                                let val = parts[1..].join(":").trim().to_string();
                                match key.as_str() {
                                    "c" | "case" | "case_number" => {
                                        metadata.case_number = Some(val)
                                    }
                                    "e" | "evidence" | "evidence_number" => {
                                        metadata.evidence_number = Some(val)
                                    }
                                    "a" | "examiner" | "examiner_name" => {
                                        metadata.examiner_name = Some(val)
                                    }
                                    "n" | "notes" => metadata.notes = Some(val),
                                    _ => {}
                                }
                            }
                        }
                    }
                } else if sec_name.eq_ignore_ascii_case("volume")
                    || sec_name.eq_ignore_ascii_case("disk")
                {
                    let mut vol_buf = [0u8; 64];
                    if file.read_exact(&mut vol_buf).is_ok() && sec_size >= 32 {
                        let bps = u32::from_le_bytes([
                            vol_buf[16],
                            vol_buf[17],
                            vol_buf[18],
                            vol_buf[19],
                        ]);
                        let total_sec = u64::from_le_bytes([
                            vol_buf[20],
                            vol_buf[21],
                            vol_buf[22],
                            vol_buf[23],
                            vol_buf[24],
                            vol_buf[25],
                            vol_buf[26],
                            vol_buf[27],
                        ]);
                        if (512..=4096).contains(&bps) && total_sec > 0 {
                            sector_size = bps;
                            metadata.bytes_per_sector = bps;
                            let media_bytes = total_sec.saturating_mul(bps as u64);
                            metadata.media_size = media_bytes;
                        }
                    }
                } else if sec_name.eq_ignore_ascii_case("sectors")
                    || sec_name.eq_ignore_ascii_case("data")
                {
                    payload_offset = sec_offset + 76;
                    let next_avail = if next_sec > payload_offset && next_sec <= file_len {
                        next_sec - payload_offset
                    } else {
                        file_len.saturating_sub(payload_offset)
                    };
                    size = if sec_size > 0 && sec_size <= next_avail {
                        sec_size
                    } else if metadata.media_size > 0 && metadata.media_size <= next_avail {
                        metadata.media_size
                    } else {
                        next_avail
                    };
                }

                if next_sec < sec_offset.saturating_add(76)
                    || next_sec >= file_len
                    || sec_name.eq_ignore_ascii_case("done")
                {
                    break;
                }
                sec_offset = next_sec;
            }
        }

        if size == file_len.saturating_sub(13) && metadata.media_size > 0 {
            let max_avail = file_len.saturating_sub(payload_offset);
            if metadata.media_size <= max_avail {
                size = metadata.media_size;
            }
        }

        metadata.chunk_count = size.div_ceil(32768) as u32;

        // Reset read pointer past header
        file.seek(SeekFrom::Start(payload_offset))
            .map_err(CoreError::Io)?;

        Ok(Self {
            file,
            size,
            sector_size,
            payload_offset,
            metadata,
        })
    }

    /// Check whether a file is an E01 image by signature without full loading
    pub fn is_e01<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(mut f) = File::open(path) {
            let mut magic = [0u8; 8];
            if f.read_exact(&mut magic).is_ok() {
                return &magic == E01_FILE_SIGNATURE;
            }
        }
        false
    }
}

/// Header offset in bytes before raw payload data in standard E01 image files
pub const E01_HEADER_OFFSET: u64 = 13;

impl Read for E01Image {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let current_raw = self.file.stream_position()?;
        if current_raw < self.payload_offset {
            self.file.seek(SeekFrom::Start(self.payload_offset))?;
        }
        let current_pos = self
            .file
            .stream_position()?
            .saturating_sub(self.payload_offset);
        if current_pos >= self.size {
            return Ok(0);
        }
        let max_readable = (self.size - current_pos) as usize;
        let to_read = std::cmp::min(buf.len(), max_readable);
        self.file.read(&mut buf[..to_read])
    }
}

impl Write for E01Image {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Cannot write to read-only E01 forensic evidence image",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for E01Image {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let current_raw = self.file.stream_position()?;
        let current_logical = current_raw.saturating_sub(self.payload_offset);

        let target_logical: i64 = match pos {
            SeekFrom::Start(offset) => offset.try_into().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Offset too large")
            })?,
            SeekFrom::Current(offset) => {
                let cur: i64 = current_logical.try_into().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Offset too large")
                })?;
                cur.checked_add(offset).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek overflow")
                })?
            }
            SeekFrom::End(offset) => {
                let sz: i64 = self.size.try_into().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Offset too large")
                })?;
                sz.checked_add(offset).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek overflow")
                })?
            }
        };

        if target_logical < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot seek before start of image payload",
            ));
        }

        let target_raw = self
            .payload_offset
            .checked_add(target_logical as u64)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek position overflow")
            })?;

        let new_raw = self.file.seek(SeekFrom::Start(target_raw))?;
        Ok(new_raw.saturating_sub(self.payload_offset))
    }
}

impl DiskSource for E01Image {
    fn size(&self) -> Result<u64> {
        Ok(self.size)
    }

    fn sector_size(&self) -> Result<u32> {
        Ok(self.sector_size)
    }
}
