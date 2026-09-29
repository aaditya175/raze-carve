use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PngCarveMetadata {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub exact_file_size: u64,
    pub text_tags: HashMap<String, String>,
}

/// Validates PNG structure from a buffer.
pub fn validate_png(data: &[u8]) -> Result<bool> {
    if data.len() < 8 {
        return Ok(false);
    }

    const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data[0..8] != PNG_HEADER {
        return Ok(false);
    }

    const IEND: [u8; 4] = [0x49, 0x45, 0x4E, 0x44];

    let mut offset = 8;
    while offset + 8 <= data.len() {
        let length = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let chunk_type = &data[offset + 4..offset + 8];

        if chunk_type == IEND {
            return Ok(offset + 12 <= data.len());
        }

        match offset.checked_add(4 + 4 + length + 4) {
            Some(next) => offset = next,
            None => return Ok(false),
        }
    }

    Ok(false)
}

/// Deep structural parser for PNG images.
/// Extracts image dimensions, bit depth, and embedded tEXt metadata comments.
pub fn parse_png_metadata(data: &[u8]) -> Option<PngCarveMetadata> {
    if data.len() < 33 {
        return None;
    }

    const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data[0..8] != PNG_HEADER {
        return None;
    }

    // IHDR must be the first chunk: offset 8
    // Length (4) + "IHDR" (4) + Data (13) + CRC (4)
    if &data[12..16] != b"IHDR" {
        return None;
    }

    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    let bit_depth = data[24];
    let color_type = data[25];

    let mut text_tags = HashMap::new();
    let mut exact_file_size = 0u64;

    let mut offset = 8;
    while offset + 8 <= data.len() {
        let length = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let chunk_type = &data[offset + 4..offset + 8];

        if chunk_type == b"tEXt" && offset + 8 + length <= data.len() {
            let chunk_data = &data[offset + 8..offset + 8 + length];
            if let Some(null_pos) = chunk_data.iter().position(|&b| b == 0) {
                let key = String::from_utf8_lossy(&chunk_data[..null_pos]).to_string();
                let val = String::from_utf8_lossy(&chunk_data[null_pos + 1..]).to_string();
                text_tags.insert(key, val);
            }
        }

        if chunk_type == b"IEND" {
            if offset + 12 <= data.len() {
                exact_file_size = (offset + 12) as u64;
            }
            break;
        }

        match offset.checked_add(12 + length) {
            Some(next) => offset = next,
            None => break,
        }
    }

    Some(PngCarveMetadata {
        width,
        height,
        bit_depth,
        color_type,
        exact_file_size: if exact_file_size > 0 {
            exact_file_size
        } else {
            data.len() as u64
        },
        text_tags,
    })
}
