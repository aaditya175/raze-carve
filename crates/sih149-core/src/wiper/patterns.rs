use rand::rngs::OsRng;
use rand::RngCore;

/// Returns whether a method name is supported by the wipe engine.  Keep this
/// separate from `passes_for_method`: silently selecting a default pass for a
/// typo is unacceptable for a destructive operation.
pub fn is_known_method(method: &str) -> bool {
    matches!(
        method.to_lowercase().replace('_', "-").as_str(),
        "zero"
            | "nist"
            | "nist800-88"
            | "single-pass"
            | "random"
            | "dod"
            | "dod3"
            | "dod-5220-22-m"
            | "dod7"
            | "dod-7pass"
            | "dod-5220-22-m-ece"
            | "gutmann"
            | "gutmann-35"
    )
}

/// Canonical pass sequence generator for all frontends (CLI, TUI, Tauri GUI, REST API).
/// Centralized in sih149-core to guarantee uniform compliance across all interfaces.
pub fn passes_for_method(method: &str) -> Vec<u8> {
    match method.to_lowercase().replace('_', "-").as_str() {
        "zero" | "nist" | "nist800-88" | "single-pass" => vec![1],
        "random" => vec![3],
        "dod" | "dod3" | "dod-5220-22-m" => vec![1, 2, 3],
        "dod7" | "dod-7pass" | "dod-5220-22-m-ece" => vec![1, 2, 3, 1, 2, 3, 3],
        "gutmann" | "gutmann-35" => (1u8..=35).collect(),
        // Callers must validate a user supplied method with `is_known_method`
        // before reaching this function. Returning no passes is fail-closed
        // for any internal caller that forgets to do so.
        _ => Vec::new(),
    }
}

/// A reusable, in-place pattern writer. Callers allocate ONE buffer and
/// ONE generator per wipe operation and reuse both across every chunk,
/// instead of allocating a new Vec on every 1MB chunk.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum WipePattern {
    Zeros,
    Ones,
    Byte(u8),
    Periodic3 {
        bytes: [u8; 3],
        offset: u64,
    },
    /// A keyed deterministic stream. The key is obtained from the OS CSPRNG
    /// for every pass, while the byte stream can be regenerated during
    /// verification without storing the media contents or exposing the key in
    /// an audit record.
    Random(RandomPattern),
}

pub struct RandomPattern {
    key: [u8; 32],
    offset: u64,
}

impl Clone for RandomPattern {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            // A clone is a verifier for the complete stream, not a snapshot
            // of its current write position.
            offset: 0,
        }
    }
}

impl RandomPattern {
    fn new() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key, offset: 0 }
    }

    fn fill(&mut self, buf: &mut [u8]) {
        self.fill_at(buf, self.offset);
        self.offset = self.offset.saturating_add(buf.len() as u64);
    }

    fn fill_at(&self, buf: &mut [u8], byte_offset: u64) {
        let mut written = 0usize;
        while written < buf.len() {
            let absolute = byte_offset.saturating_add(written as u64);
            let block_index = absolute / 32;
            let block_start = (absolute % 32) as usize;
            let block = blake3::keyed_hash(&self.key, &block_index.to_le_bytes());
            let bytes = block.as_bytes();
            let take = (buf.len() - written).min(bytes.len() - block_start);
            buf[written..written + take].copy_from_slice(&bytes[block_start..block_start + take]);
            written += take;
        }
    }
}

impl WipePattern {
    /// Return the pattern for a pass under the specified sanitization standard.
    /// If method is "gutmann", produces the genuine Peter Gutmann (1996) 35-pass algorithm.
    pub fn for_method_pass(method: &str, pass: u8) -> Self {
        if method.to_lowercase().contains("gutmann") {
            Self::for_gutmann_pass(pass)
        } else {
            Self::for_pass(pass)
        }
    }

    pub fn for_pass(pass: u8) -> Self {
        match pass {
            0 | 1 => WipePattern::Zeros,
            2 => WipePattern::Ones,
            _ => WipePattern::Random(RandomPattern::new()),
        }
    }

    /// Peter Gutmann's 35-pass data sanitization algorithm (1996):
    /// - Passes 1-4: Random (CSPRNG)
    /// - Passes 5-6: 0x55, 0xAA (alternating bit sequences)
    /// - Passes 7-9: 0x924924, 0x492492, 0x249249 (1,7 MFM encoded patterns)
    /// - Passes 10-25: 0x00 to 0xFF in increments of 0x11
    /// - Passes 26-28: 0x924924, 0x492492, 0x249249 (repeat MFM patterns)
    /// - Passes 29-31: 0x6DB6DB, 0xB6DB6D, 0xDB6DB6 (2,7 RLL encoded patterns)
    /// - Passes 32-35: Random (CSPRNG)
    pub fn for_gutmann_pass(pass: u8) -> Self {
        match pass {
            1..=4 | 32..=35 => WipePattern::Random(RandomPattern::new()),
            5 => WipePattern::Byte(0x55),
            6 => WipePattern::Byte(0xAA),
            7 => WipePattern::periodic3([0x92, 0x49, 0x24]),
            8 => WipePattern::periodic3([0x49, 0x24, 0x92]),
            9 => WipePattern::periodic3([0x24, 0x92, 0x49]),
            10..=25 => WipePattern::Byte((pass - 10) * 0x11),
            26 => WipePattern::periodic3([0x92, 0x49, 0x24]),
            27 => WipePattern::periodic3([0x49, 0x24, 0x92]),
            28 => WipePattern::periodic3([0x24, 0x92, 0x49]),
            29 => WipePattern::periodic3([0x6D, 0xB6, 0xDB]),
            30 => WipePattern::periodic3([0xB6, 0xDB, 0x6D]),
            31 => WipePattern::periodic3([0xDB, 0x6D, 0xB6]),
            _ => WipePattern::Random(RandomPattern::new()),
        }
    }

    /// Fill `buf` in place with this pass's pattern. No allocation.
    pub fn fill(&mut self, buf: &mut [u8]) {
        match self {
            WipePattern::Zeros => buf.fill(0),
            WipePattern::Ones => buf.fill(0xFF),
            WipePattern::Byte(b) => buf.fill(*b),
            WipePattern::Periodic3 { bytes, offset } => {
                for (index, byte) in buf.iter_mut().enumerate() {
                    *byte = bytes[((*offset + index as u64) % 3) as usize];
                }
                *offset = offset.saturating_add(buf.len() as u64);
            }
            WipePattern::Random(pattern) => pattern.fill(buf),
        }
    }

    fn periodic3(bytes: [u8; 3]) -> Self {
        Self::Periodic3 { bytes, offset: 0 }
    }

    /// Recreate this pattern at an absolute byte offset without changing the
    /// state of the writer. This lets verification compare what was actually
    /// written, including Gutmann's three-byte sequences, across chunk edges.
    pub fn fill_at(&self, buf: &mut [u8], byte_offset: u64) {
        match self {
            WipePattern::Zeros => buf.fill(0),
            WipePattern::Ones => buf.fill(0xFF),
            WipePattern::Byte(byte) => buf.fill(*byte),
            WipePattern::Periodic3 { bytes, .. } => {
                for (index, byte) in buf.iter_mut().enumerate() {
                    *byte = bytes[((byte_offset + index as u64) % 3) as usize];
                }
            }
            WipePattern::Random(pattern) => pattern.fill_at(buf, byte_offset),
        }
    }

    /// Returns true if this pattern generates random data (CSPRNG)
    pub fn is_random(&self) -> bool {
        matches!(self, WipePattern::Random(_))
    }
}

// Compatibility API kept for callers and tests that request a one-off pattern
pub fn generate_zeros(size: usize) -> Vec<u8> {
    vec![0; size]
}

pub fn generate_ones(size: usize) -> Vec<u8> {
    vec![0xFF; size]
}

pub fn generate_random(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    OsRng.fill_bytes(&mut buf);
    buf
}

pub fn get_dod_pattern(pass: u8) -> fn(usize) -> Vec<u8> {
    match pass {
        1 => generate_zeros,
        2 => generate_ones,
        _ => generate_random,
    }
}

/// Determine if a given sanitization method and pass produces random bytes
pub fn is_random_pass(method: &str, pass: u8) -> bool {
    match method.to_lowercase().replace('_', "-").as_str() {
        "random" => true,
        "dod" | "dod3" | "dod-5220-22-m" | "dod7" | "dod-7pass" | "dod-5220-22-m-ece" => pass == 3,
        "gutmann" | "gutmann-35" => matches!(pass, 1..=4 | 32..=35),
        _ => false,
    }
}

/// Retrieve the appropriate deterministic or random generator function for verifying a pass
pub fn get_pattern_fn_for_pass(method: &str, pass: u8) -> fn(usize) -> Vec<u8> {
    if is_random_pass(method, pass) {
        generate_random
    } else {
        match pass {
            2 => generate_ones,
            _ => generate_zeros,
        }
    }
}
