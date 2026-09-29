use crate::disk::DiskSource;
use crate::error::Result;
use crate::wiper::patterns::WipePattern;
use std::io::SeekFrom;
use tracing::{info, warn};

/// Verifies that a disk has been successfully wiped with a specific pattern.
/// For deterministic patterns (zeros/ones), does exact byte comparison.
/// Random output must be verified with `verify_random_wipe`, which compares
/// every sampled byte to the keyed stream that was actually written. Entropy
/// alone is not evidence of a successful overwrite.
pub fn verify_wipe<D: DiskSource>(
    disk: &mut D,
    expected_pattern: fn(usize) -> Vec<u8>,
    sample_rate_percent: u8,
    is_random_pass: bool,
) -> Result<bool> {
    let size = disk.size()?;
    if size == 0 {
        return Ok(true);
    }
    let chunk_size = 1024 * 1024; // 1 MB chunks

    let step = if sample_rate_percent >= 100 {
        chunk_size as u64
    } else {
        let skip = chunk_size as u64 * (100 - sample_rate_percent as u64)
            / (sample_rate_percent as u64).max(1);
        chunk_size as u64 + skip
    };

    let mut buffer = vec![0u8; chunk_size];
    let expected_buf = if !is_random_pass {
        Some(expected_pattern(chunk_size))
    } else {
        None
    };
    let mut offset = 0;
    let mut last_verified_end: u64 = 0;

    info!(
        "Starting wipe verification on {} bytes with step {}",
        size, step
    );

    while offset < size {
        disk.seek(SeekFrom::Start(offset))?;

        let to_read = std::cmp::min(chunk_size as u64, size - offset) as usize;
        disk.read_exact(&mut buffer[..to_read])
            .map_err(crate::error::CoreError::Io)?;

        if is_random_pass {
            return Err(crate::error::CoreError::Wiper(
                "random-pattern verification requires verify_random_wipe with the written pattern"
                    .to_string(),
            ));
        } else {
            let matches = if let Some(ref exp) = expected_buf {
                buffer[..to_read] == exp[..to_read]
            } else {
                let expected = expected_pattern(to_read);
                buffer[..to_read] == expected[..]
            };
            if !matches {
                warn!("Wipe verification failed at offset {}", offset);
                return Ok(false);
            }
        }

        last_verified_end = offset + to_read as u64;
        offset += step;
    }

    // NIST SP 800-88 Rev. 1 Section 4.7 requires verification of both media boundaries
    // (beginning and end) plus sampled interior areas. If sampling skipped the final
    // sectors/blocks (e.g. secondary GPT header or volume slack), verify the tail boundary.
    if last_verified_end < size {
        let tail_chunk_size = std::cmp::min(chunk_size as u64, size) as usize;
        let tail_offset = size - tail_chunk_size as u64;
        disk.seek(SeekFrom::Start(tail_offset))?;
        disk.read_exact(&mut buffer[..tail_chunk_size])
            .map_err(crate::error::CoreError::Io)?;

        if is_random_pass {
            return Err(crate::error::CoreError::Wiper(
                "random-pattern verification requires verify_random_wipe with the written pattern"
                    .to_string(),
            ));
        } else {
            let matches = if let Some(ref exp) = expected_buf {
                buffer[..tail_chunk_size] == exp[..tail_chunk_size]
            } else {
                let expected = expected_pattern(tail_chunk_size);
                buffer[..tail_chunk_size] == expected[..]
            };
            if !matches {
                warn!(
                    "Tail boundary wipe verification failed at offset {}",
                    tail_offset
                );
                return Ok(false);
            }
        }
    }

    info!("Wipe verification passed.");
    Ok(true)
}

/// Verify a sampled random pass against the exact keyed stream used to write
/// it. The key remains process-local and is never written to an audit report.
pub fn verify_random_wipe<D: DiskSource>(
    disk: &mut D,
    pattern: &WipePattern,
    sample_rate_percent: u8,
) -> Result<bool> {
    let size = disk.size()?;
    if size == 0 {
        return Ok(true);
    }
    let chunk_size = 1024 * 1024usize;
    let step = if sample_rate_percent >= 100 {
        chunk_size as u64
    } else {
        chunk_size as u64
            + chunk_size as u64 * (100 - sample_rate_percent as u64)
                / (sample_rate_percent as u64).max(1)
    };
    let mut actual = vec![0u8; chunk_size];
    let mut expected = vec![0u8; chunk_size];
    let mut offsets = Vec::new();
    let mut offset = 0u64;
    while offset < size {
        offsets.push(offset);
        offset = offset.saturating_add(step);
    }
    let tail_offset = size.saturating_sub((chunk_size as u64).min(size));
    if !offsets.contains(&tail_offset) {
        offsets.push(tail_offset);
    }

    for offset in offsets {
        let len = ((size - offset).min(chunk_size as u64)) as usize;
        disk.seek(SeekFrom::Start(offset))?;
        disk.read_exact(&mut actual[..len])
            .map_err(crate::error::CoreError::Io)?;
        if !pattern.is_random() {
            return Err(crate::error::CoreError::Wiper(
                "verify_random_wipe was called with a non-random pattern".to_string(),
            ));
        }
        pattern.fill_at(&mut expected[..len], offset);
        if actual[..len] != expected[..len] {
            warn!("Random wipe verification failed at offset {}", offset);
            return Ok(false);
        }
    }
    Ok(true)
}

/// Verify sampled media against the precise final pattern used during writing.
/// This is the preferred API for every wipe frontend: it verifies keyed random
/// passes and deterministic/Gutmann patterns by byte comparison, never entropy.
pub fn verify_pattern_wipe<D: DiskSource>(
    disk: &mut D,
    pattern: &WipePattern,
    sample_rate_percent: u8,
) -> Result<bool> {
    if pattern.is_random() {
        return verify_random_wipe(disk, pattern, sample_rate_percent);
    }

    let size = disk.size()?;
    if size == 0 {
        return Ok(true);
    }
    let chunk_size = 1024 * 1024usize;
    let step = if sample_rate_percent >= 100 {
        chunk_size as u64
    } else {
        chunk_size as u64
            + chunk_size as u64 * (100 - sample_rate_percent as u64)
                / (sample_rate_percent as u64).max(1)
    };
    let mut actual = vec![0u8; chunk_size];
    let mut expected = vec![0u8; chunk_size];
    let mut offsets = Vec::new();
    let mut offset = 0u64;
    while offset < size {
        offsets.push(offset);
        offset = offset.saturating_add(step);
    }
    let tail_offset = size.saturating_sub((chunk_size as u64).min(size));
    if !offsets.contains(&tail_offset) {
        offsets.push(tail_offset);
    }

    for offset in offsets {
        let len = ((size - offset).min(chunk_size as u64)) as usize;
        disk.seek(SeekFrom::Start(offset))?;
        disk.read_exact(&mut actual[..len])
            .map_err(crate::error::CoreError::Io)?;
        pattern.fill_at(&mut expected[..len], offset);
        if actual[..len] != expected[..len] {
            warn!("Wipe verification failed at offset {}", offset);
            return Ok(false);
        }
    }
    Ok(true)
}
