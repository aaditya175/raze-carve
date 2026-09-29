//! Difference hash (dHash) for perceptual image deduplication.
//!
//! Computes a 64-bit hash by:
//! 1. Resizing image to 9x8 grayscale
//! 2. Comparing adjacent pixel brightness (left vs right)
//! 3. Encoding differences as bits
//!
//! Hamming distance between hashes indicates visual similarity.
//! Distance < 10 = likely duplicates.

/// Computes a 64-bit difference hash from a 9x8 grayscale grid (72 pixels).
/// Matches standard dHash algorithm row-by-row.
pub fn compute_dhash_from_grid(grid_9x8: &[u8; 72]) -> u64 {
    let mut hash: u64 = 0;
    for row in 0..8 {
        let mut row_byte: u8 = 0;
        for col in 0..8 {
            let left = grid_9x8[row * 9 + col];
            let right = grid_9x8[row * 9 + col + 1];
            if left > right {
                row_byte |= 1 << col;
            }
        }
        hash |= (row_byte as u64) << (row * 8);
    }
    hash
}

/// Formats a 64-bit dHash into a 16-character hexadecimal string.
pub fn dhash_to_hex(hash: u64) -> String {
    let mut s = String::with_capacity(16);
    for row in 0..8 {
        let byte = ((hash >> (row * 8)) & 0xFF) as u8;
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

/// Parses a 16-character hexadecimal string into a 64-bit dHash.
pub fn dhash_from_hex(hex_str: &str) -> Option<u64> {
    if hex_str.len() != 16 {
        return None;
    }
    let mut hash: u64 = 0;
    for row in 0..8 {
        let byte_str = &hex_str[row * 2..row * 2 + 2];
        let byte = u8::from_str_radix(byte_str, 16).ok()?;
        hash |= (byte as u64) << (row * 8);
    }
    Some(hash)
}

/// Computes the Hamming distance (number of differing bits) between two 64-bit hashes.
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Computes the Hamming distance between two 16-character hexadecimal dHash strings.
pub fn hamming_distance_hex(a: &str, b: &str) -> Option<u32> {
    let hash_a = dhash_from_hex(a)?;
    let hash_b = dhash_from_hex(b)?;
    Some(hamming_distance(hash_a, hash_b))
}

/// Computes the visual similarity percentage (0.0% to 100.0%) based on bit overlap.
pub fn similarity_percentage(a: u64, b: u64) -> f64 {
    let distance = hamming_distance(a, b);
    ((64 - distance) as f64 / 64.0) * 100.0
}

/// Returns true if two hashes are within the specified Hamming distance threshold.
/// Distance <= 10 indicates high visual similarity / near-duplicates.
pub fn is_visual_duplicate(a: u64, b: u64, threshold: u32) -> bool {
    hamming_distance(a, b) <= threshold
}

/// Downsamples an arbitrary width x height grayscale buffer into a 9x8 grid.
pub fn downsample_grayscale_to_9x8(width: usize, height: usize, pixels: &[u8]) -> Option<[u8; 72]> {
    if width == 0 || height == 0 || pixels.len() < width * height {
        return None;
    }

    let mut grid = [0u8; 72];
    for out_y in 0..8 {
        let y_start = (out_y * height) / 8;
        let y_end = (((out_y + 1) * height) / 8).max(y_start + 1).min(height);
        let y_span = y_end.saturating_sub(y_start).max(1);

        for out_x in 0..9 {
            let x_start = (out_x * width) / 9;
            let x_end = (((out_x + 1) * width) / 9).max(x_start + 1).min(width);
            let x_span = x_end.saturating_sub(x_start).max(1);

            let mut sum: u32 = 0;
            for y in y_start..y_end {
                let row_offset = y * width;
                for x in x_start..x_end {
                    sum += pixels[row_offset + x] as u32;
                }
            }
            grid[out_y * 9 + out_x] = (sum / (y_span * x_span) as u32) as u8;
        }
    }
    Some(grid)
}

/// Converts an RGB interleaved pixel slice into grayscale using standard Rec. 601 luminance weights.
pub fn rgb_to_grayscale(rgb: &[u8]) -> Vec<u8> {
    let num_pixels = rgb.len() / 3;
    let mut gray = Vec::with_capacity(num_pixels);
    for i in 0..num_pixels {
        let r = rgb[i * 3] as u32;
        let g = rgb[i * 3 + 1] as u32;
        let b = rgb[i * 3 + 2] as u32;
        let y = (r * 299 + g * 587 + b * 114) / 1000;
        gray.push(y as u8);
    }
    gray
}

/// Converts an RGBA interleaved pixel slice into grayscale, blending with a white background based on alpha.
pub fn rgba_to_grayscale(rgba: &[u8]) -> Vec<u8> {
    let num_pixels = rgba.len() / 4;
    let mut gray = Vec::with_capacity(num_pixels);
    for i in 0..num_pixels {
        let r = rgba[i * 4] as u32;
        let g = rgba[i * 4 + 1] as u32;
        let b = rgba[i * 4 + 2] as u32;
        let a = rgba[i * 4 + 3] as u32;
        let r_comp = (r * a + 255 * (255 - a)) / 255;
        let g_comp = (g * a + 255 * (255 - a)) / 255;
        let b_comp = (b * a + 255 * (255 - a)) / 255;
        let y = (r_comp * 299 + g_comp * 587 + b_comp * 114) / 1000;
        gray.push(y.min(255) as u8);
    }
    gray
}
