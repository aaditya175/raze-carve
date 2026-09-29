use crate::carver::entropy::calculate_shannon_entropy;

#[derive(Debug, Clone, PartialEq)]
pub enum EntropyType {
    CompressedOrEncrypted, // 7.20 - 8.00
    StructuredText,        // 3.20 - 5.80
    BinaryExecutable,      // 5.80 - 7.20
    SparsePadding,         // 0.00 - 1.50
    Unknown,
}

#[derive(Debug, Clone)]
pub struct EntropyCluster {
    pub start_offset: u64,
    pub end_offset: u64,
    pub size_bytes: u64,
    pub avg_entropy: f64,
    pub variance: f64,
    pub cluster_type: EntropyType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZeroBlockRun {
    pub start_offset: u64,
    pub end_offset: u64,
    pub size_bytes: u64,
    pub block_count: usize,
}

/// Identifies contiguous runs of zero-filled blocks (sparse holes or unallocated slack).
pub fn find_zero_block_runs(
    buffer: &[u8],
    base_offset: u64,
    block_size: usize,
    min_consecutive_blocks: usize,
) -> Vec<ZeroBlockRun> {
    let mut runs = Vec::new();
    if block_size == 0 || buffer.len() < block_size || min_consecutive_blocks == 0 {
        return runs;
    }

    let num_blocks = buffer.len() / block_size;
    let mut run_start: Option<usize> = None;
    let mut run_len = 0;

    for i in 0..num_blocks {
        let start = i * block_size;
        let end = start + block_size;
        let is_zero = buffer[start..end].iter().all(|&b| b == 0);

        if is_zero {
            if run_start.is_none() {
                run_start = Some(i);
            }
            run_len += 1;
        } else if let Some(start_block) = run_start {
            if run_len >= min_consecutive_blocks {
                let s_off = base_offset + (start_block * block_size) as u64;
                let e_off = base_offset + (i * block_size) as u64;
                runs.push(ZeroBlockRun {
                    start_offset: s_off,
                    end_offset: e_off,
                    size_bytes: e_off - s_off,
                    block_count: run_len,
                });
            }
            run_start = None;
            run_len = 0;
        }
    }

    if let Some(start_block) = run_start {
        if run_len >= min_consecutive_blocks {
            let s_off = base_offset + (start_block * block_size) as u64;
            let e_off = base_offset + (num_blocks * block_size) as u64;
            runs.push(ZeroBlockRun {
                start_offset: s_off,
                end_offset: e_off,
                size_bytes: e_off - s_off,
                block_count: run_len,
            });
        }
    }

    runs
}

/// Carves continuous stream boundaries based on statistical Shannon entropy profiles
/// with configurable centered moving-window variance smoothing.
///
/// Moving-window smoothing prevents transient micro-dips (e.g. metadata tables or symbol headers
/// in compressed streams) from artificially fragmenting contiguous streams into disjoint clusters.
pub fn cluster_entropy_streams_smoothed(
    buffer: &[u8],
    base_offset: u64,
    block_size: usize,
    window_size: usize,
) -> Vec<EntropyCluster> {
    let mut clusters = Vec::new();
    if block_size == 0 || buffer.len() < block_size {
        return clusters;
    }

    let num_blocks = buffer.len() / block_size;
    let mut raw_entropies = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks {
        let start = i * block_size;
        let end = start + block_size;
        let entropy = calculate_shannon_entropy(&buffer[start..end]);
        raw_entropies.push(entropy);
    }

    if raw_entropies.is_empty() {
        return clusters;
    }

    // Centered window smoothing
    let w = if window_size == 0 { 1 } else { window_size };
    let half_w = w / 2;
    let mut smoothed_profiles = Vec::with_capacity(num_blocks);

    for (i, &raw_e) in raw_entropies.iter().enumerate() {
        let smoothed_e = if w > 1 {
            let s = i.saturating_sub(half_w);
            let e = std::cmp::min(raw_entropies.len(), i + half_w + 1);
            let sum: f64 = raw_entropies[s..e].iter().sum();
            sum / (e - s) as f64
        } else {
            raw_e
        };

        let etype = match smoothed_e {
            e if e >= 7.20 => EntropyType::CompressedOrEncrypted,
            e if e >= 5.80 => EntropyType::BinaryExecutable,
            e if e >= 3.20 => EntropyType::StructuredText,
            e if e <= 1.50 => EntropyType::SparsePadding,
            _ => EntropyType::Unknown,
        };
        smoothed_profiles.push((i, raw_e, etype));
    }

    let mut current_type = smoothed_profiles[0].2.clone();
    let mut current_start = 0;
    let mut cluster_entropies = vec![smoothed_profiles[0].1];

    for (idx, raw_e, etype) in smoothed_profiles.into_iter().skip(1) {
        if etype == current_type {
            cluster_entropies.push(raw_e);
        } else {
            if current_type != EntropyType::SparsePadding && cluster_entropies.len() >= 2 {
                let start_off = base_offset + (current_start * block_size) as u64;
                let end_off = base_offset + (idx * block_size) as u64;
                let n = cluster_entropies.len() as f64;
                let avg = cluster_entropies.iter().sum::<f64>() / n;
                let var = cluster_entropies
                    .iter()
                    .map(|&x| (x - avg).powi(2))
                    .sum::<f64>()
                    / n;
                clusters.push(EntropyCluster {
                    start_offset: start_off,
                    end_offset: end_off,
                    size_bytes: end_off - start_off,
                    avg_entropy: avg,
                    variance: var,
                    cluster_type: current_type.clone(),
                });
            }
            current_type = etype;
            current_start = idx;
            cluster_entropies.clear();
            cluster_entropies.push(raw_e);
        }
    }

    if current_type != EntropyType::SparsePadding && cluster_entropies.len() >= 2 {
        let start_off = base_offset + (current_start * block_size) as u64;
        let end_off = base_offset + (num_blocks * block_size) as u64;
        let n = cluster_entropies.len() as f64;
        let avg = cluster_entropies.iter().sum::<f64>() / n;
        let var = cluster_entropies
            .iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>()
            / n;
        clusters.push(EntropyCluster {
            start_offset: start_off,
            end_offset: end_off,
            size_bytes: end_off - start_off,
            avg_entropy: avg,
            variance: var,
            cluster_type: current_type,
        });
    }

    clusters
}

/// Carves continuous stream boundaries based on statistical Shannon entropy profiles.
/// Uses a 3-block moving-window smoothing by default to prevent micro-dip fragmentation.
pub fn cluster_entropy_streams(
    buffer: &[u8],
    base_offset: u64,
    block_size: usize,
) -> Vec<EntropyCluster> {
    cluster_entropy_streams_smoothed(buffer, base_offset, block_size, 3)
}
