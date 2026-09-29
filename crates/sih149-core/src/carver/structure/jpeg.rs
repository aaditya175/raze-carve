use crate::error::Result;

#[derive(Debug, Clone)]
pub struct JpegCarveMetadata {
    pub exact_file_size: u64,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_time: Option<String>,
    pub comments: Vec<String>,
}

/// Validates JPEG structure from a buffer.
/// Looks for SOI (Start of Image) and EOI (End of Image) markers.
pub fn validate_jpeg(data: &[u8]) -> Result<bool> {
    if data.len() < 4 {
        return Ok(false);
    }

    if data[0] != 0xFF || data[1] != 0xD8 {
        return Ok(false);
    }

    for i in 2..data.len() - 1 {
        if data[i] == 0xFF && data[i + 1] == 0xD9 {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Deep structural parser for JPEG images.
/// Scans markers (APP1 EXIF, COM) and locates true EOI boundary.
pub fn parse_jpeg_metadata(data: &[u8]) -> Option<JpegCarveMetadata> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }

    let mut exact_size = 0u64;
    let mut camera_make = None;
    let camera_model = None;
    let mut date_time = None;
    let mut comments = Vec::new();

    let mut i = 2;
    while i + 4 <= data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }

        let marker = data[i + 1];

        // EOI marker
        if marker == 0xD9 {
            exact_size = (i + 2) as u64;
            break;
        }

        // Standalone markers without length bytes
        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) || marker == 0x01 || marker == 0x00 {
            i += 2;
            continue;
        }

        // Variable length markers (SOF0..SOF15, DHT, DQT, DRI, SOS, APP0..APP15, COM)
        let has_length =
            (0xC0..=0xCF).contains(&marker) || (0xDA..=0xEF).contains(&marker) || marker == 0xFE;

        if has_length {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            if seg_len >= 2 && i + 2 + seg_len <= data.len() {
                // If SOS (Start of Scan, 0xDA), skip the SOS header and scan the entropy data for true EOI
                if marker == 0xDA {
                    let mut scan_idx = i + 2 + seg_len;
                    while scan_idx + 1 < data.len() {
                        if data[scan_idx] == 0xFF {
                            let next = data[scan_idx + 1];
                            if next == 0xD9 {
                                exact_size = (scan_idx + 2) as u64;
                                break;
                            } else if next != 0x00 && !(0xD0..=0xD7).contains(&next) {
                                // Another marker encountered (e.g. restart or next scan)
                                scan_idx += 2;
                                continue;
                            }
                        }
                        scan_idx += 1;
                    }
                    if exact_size > 0 {
                        break;
                    }
                }

                let payload = &data[i + 4..i + 2 + seg_len];
                // APP1 EXIF segment
                if marker == 0xE1 && payload.len() >= 6 && &payload[0..4] == b"Exif" {
                    let exif_str = String::from_utf8_lossy(payload);
                    for token in exif_str.split('\0') {
                        let t = token.trim();
                        if t.len() >= 10
                            && t.contains(':')
                            && t.chars().filter(|&c| c == ':').count() >= 2
                        {
                            if date_time.is_none() && t.len() >= 19 {
                                date_time = Some(t.to_string());
                            }
                        } else if camera_make.is_none()
                            && (t.eq_ignore_ascii_case("Canon")
                                || t.eq_ignore_ascii_case("Nikon")
                                || t.eq_ignore_ascii_case("Sony")
                                || t.eq_ignore_ascii_case("Apple")
                                || t.eq_ignore_ascii_case("Samsung"))
                        {
                            camera_make = Some(t.to_string());
                        }
                    }
                } else if marker == 0xFE {
                    // Comment marker
                    let comment_text = String::from_utf8_lossy(payload).trim().to_string();
                    if !comment_text.is_empty() {
                        comments.push(comment_text);
                    }
                }
                i += 2 + seg_len;
                continue;
            }
        }

        i += 1;
    }

    // Fallback search for EOI if marker walk hit entropy stream (SOS 0xDA)
    if exact_size == 0 {
        for j in (2..data.len() - 1).rev() {
            if data[j] == 0xFF && data[j + 1] == 0xD9 {
                exact_size = (j + 2) as u64;
                break;
            }
        }
    }

    Some(JpegCarveMetadata {
        exact_file_size: if exact_size > 0 {
            exact_size
        } else {
            data.len() as u64
        },
        camera_make,
        camera_model,
        date_time,
        comments,
    })
}
