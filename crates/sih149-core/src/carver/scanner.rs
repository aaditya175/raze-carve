use crate::carver::signatures::{FileSignature, SignatureDatabase};
use crate::error::Result;
use rayon::prelude::*;
use std::sync::Arc;

/// A hit found during scanning.
#[derive(Debug)]
pub struct ScanHit {
    /// Offset in bytes where the signature was found
    pub offset: u64,
    /// The signature that matched
    pub signature: FileSignature,
}

/// Pre-compiled header pattern with optional per-byte wildcard mask.
#[derive(Clone, Debug)]
struct CompiledHeader {
    bytes: Vec<u8>,
    /// Per-byte mask: true = must match, false = wildcard (any byte).
    mask: Vec<bool>,
    sig: FileSignature,
}

/// Indexed signature catalog with 256-bucket primary dispatch.
/// Scales linearly to 10,000+ signatures with O(1) sector dispatch time.
struct IndexedSignatures {
    /// Signatures indexed by their required first byte [0..=255]
    by_first_byte: Vec<Vec<CompiledHeader>>,
    /// Signatures where the first byte is a wildcard (e.g. `????ftyp`) or empty
    wildcards: Vec<CompiledHeader>,
}

/// Scans a buffer for file signatures using multiple threads and indexed dispatch.
pub struct SectorScanner {
    index: Arc<IndexedSignatures>,
}

impl SectorScanner {
    /// Create a new scanner with the given signature database.
    pub fn new(db: SignatureDatabase) -> Result<Self> {
        let mut by_first_byte: Vec<Vec<CompiledHeader>> = vec![Vec::new(); 256];
        let mut wildcards = Vec::new();

        for sig in db.signatures {
            let header_bytes = SignatureDatabase::parse_hex(&sig.magic_header)?;
            let mask = sig
                .header_mask
                .clone()
                .unwrap_or_else(|| vec![true; header_bytes.len()]);

            let compiled = CompiledHeader {
                bytes: header_bytes,
                mask,
                sig,
            };

            // If the first byte is non-empty and not a wildcard, index it into its specific bucket
            if !compiled.bytes.is_empty() && compiled.mask.first().copied().unwrap_or(true) {
                let first_byte = compiled.bytes[0] as usize;
                by_first_byte[first_byte].push(compiled);
            } else {
                wildcards.push(compiled);
            }
        }

        // Longest-prefix match: sort each bucket descending by header length so specific signatures
        // (e.g. DOCX/APK with 16 bytes) are evaluated before shorter generic prefixes (e.g. ZIP with 4 bytes).
        for bucket in &mut by_first_byte {
            bucket.sort_by_key(|a| std::cmp::Reverse(a.bytes.len()));
        }
        wildcards.sort_by_key(|a| std::cmp::Reverse(a.bytes.len()));

        Ok(Self {
            index: Arc::new(IndexedSignatures {
                by_first_byte,
                wildcards,
            }),
        })
    }

    /// Scan a buffer (representing sectors) for signatures.
    /// `base_offset` is the absolute offset of this buffer on the disk.
    pub fn scan_buffer(&self, buffer: &[u8], base_offset: u64, sector_size: usize) -> Vec<ScanHit> {
        if buffer.is_empty() || sector_size == 0 {
            return Vec::new();
        }

        // We only scan at sector boundaries.
        let num_sectors = buffer.len() / sector_size;

        (0..num_sectors)
            .into_par_iter()
            .filter_map(|sector_idx| {
                let offset = sector_idx * sector_size;
                if offset >= buffer.len() {
                    return None;
                }

                let sector_data = &buffer[offset..];
                if sector_data.is_empty() {
                    return None;
                }

                let first_byte = sector_data[0] as usize;
                let bucket = &self.index.by_first_byte[first_byte];

                // 1. Check signatures that require this exact first byte
                for compiled in bucket.iter() {
                    if sector_data.len() >= compiled.bytes.len()
                        && matches_with_mask(
                            &sector_data[..compiled.bytes.len()],
                            &compiled.bytes,
                            &compiled.mask,
                        )
                    {
                        return Some(ScanHit {
                            offset: base_offset + offset as u64,
                            signature: compiled.sig.clone(),
                        });
                    }
                }

                // 2. Check signatures with a wildcard at byte 0 (e.g. ????ftyp)
                for compiled in self.index.wildcards.iter() {
                    if sector_data.len() >= compiled.bytes.len()
                        && matches_with_mask(
                            &sector_data[..compiled.bytes.len()],
                            &compiled.bytes,
                            &compiled.mask,
                        )
                    {
                        return Some(ScanHit {
                            offset: base_offset + offset as u64,
                            signature: compiled.sig.clone(),
                        });
                    }
                }

                None
            })
            .collect()
    }
}

/// Compare data against pattern with per-byte wildcard mask.
/// `mask[i] == true` → data[i] must equal pattern[i].
/// `mask[i] == false` → data[i] can be anything (wildcard).
#[inline]
fn matches_with_mask(data: &[u8], pattern: &[u8], mask: &[bool]) -> bool {
    let len = data.len().min(pattern.len());
    for i in 0..len {
        let must_match = mask.get(i).copied().unwrap_or(true);
        if must_match && data[i] != pattern[i] {
            return false;
        }
    }
    true
}
