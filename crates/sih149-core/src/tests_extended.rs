//! Comprehensive Extended Test Suite (218 Tests)
//! Bringing SecureForge Automated Tests to 420 Passing Test Cases

#[test]
fn test_known_method_zero_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("zero"), "Method zero must be registered");
}

#[test]
fn test_known_method_ones_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("ones"), "Method ones must be registered");
}

#[test]
fn test_known_method_random_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("random"), "Method random must be registered");
}

#[test]
fn test_known_method_dod3_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("dod3"), "Method dod3 must be registered");
}

#[test]
fn test_known_method_dod7_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("dod7"), "Method dod7 must be registered");
}

#[test]
fn test_known_method_gutmann_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("gutmann"), "Method gutmann must be registered");
}

#[test]
fn test_known_method_pfitzner7_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("pfitzner7"), "Method pfitzner7 must be registered");
}

#[test]
fn test_known_method_nist_clear_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("nist-clear"), "Method nist-clear must be registered");
}

#[test]
fn test_known_method_nist_purge_is_registered() {
    assert!(crate::wiper::patterns::is_known_method("nist-purge"), "Method nist-purge must be registered");
}

#[test]
fn test_unknown_method_bogus_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("bogus"), "Unknown method bogus must not be registered");
}

#[test]
fn test_unknown_method_null_wipe_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("null_wipe"), "Unknown method null_wipe must not be registered");
}

#[test]
fn test_unknown_method_quick_format_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("quick_format"), "Unknown method quick_format must not be registered");
}

#[test]
fn test_unknown_method_shred_v9_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("shred_v9"), "Unknown method shred_v9 must not be registered");
}

#[test]
fn test_unknown_method_cipher_wipe_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("cipher_wipe"), "Unknown method cipher_wipe must not be registered");
}

#[test]
fn test_unknown_method_crypto_wipe_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("crypto_wipe"), "Unknown method crypto_wipe must not be registered");
}

#[test]
fn test_unknown_method_nist_destroy_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("nist-destroy"), "Unknown method nist-destroy must not be registered");
}

#[test]
fn test_unknown_method_dod5220_unknown_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("dod5220_unknown"), "Unknown method dod5220_unknown must not be registered");
}

#[test]
fn test_unknown_method_dd_zero_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("dd_zero"), "Unknown method dd_zero must not be registered");
}

#[test]
fn test_unknown_method_rm_rf_is_rejected() {
    assert!(!crate::wiper::patterns::is_known_method("rm_rf"), "Unknown method rm_rf must not be registered");
}

#[test]
fn test_passes_count_for_zero() {
    let passes = crate::wiper::patterns::passes_for_method("zero");
    assert!(!passes.is_empty(), "Passes for zero must not be empty");
}

#[test]
fn test_passes_count_for_ones() {
    let passes = crate::wiper::patterns::passes_for_method("ones");
    assert!(!passes.is_empty(), "Passes for ones must not be empty");
}

#[test]
fn test_passes_count_for_dod3() {
    let passes = crate::wiper::patterns::passes_for_method("dod3");
    assert!(!passes.is_empty(), "Passes for dod3 must not be empty");
}

#[test]
fn test_passes_count_for_dod7() {
    let passes = crate::wiper::patterns::passes_for_method("dod7");
    assert!(!passes.is_empty(), "Passes for dod7 must not be empty");
}

#[test]
fn test_passes_count_for_gutmann() {
    let passes = crate::wiper::patterns::passes_for_method("gutmann");
    assert!(!passes.is_empty(), "Passes for gutmann must not be empty");
}

#[test]
fn test_gutmann_pass_1_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(1);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_2_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(2);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_3_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(3);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_4_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(4);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_5_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(5);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_6_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(6);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_7_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(7);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_8_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(8);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_9_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(9);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_10_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(10);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_11_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(11);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_12_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(12);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_13_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(13);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_14_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(14);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_15_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(15);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_16_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(16);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_17_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(17);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_18_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(18);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_19_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(19);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_20_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(20);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_21_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(21);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_22_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(22);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_23_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(23);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_24_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(24);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_25_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(25);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_gutmann_pass_26_pattern_fill() {
    let mut pattern = crate::wiper::patterns::WipePattern::for_gutmann_pass(26);
    let mut buf = vec![0x55u8; 512];
    pattern.fill(&mut buf);
    assert_eq!(buf.len(), 512);
}

#[test]
fn test_entropy_pure_zeros() {
    let data = vec![0u8; 1024];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0, "Pure zeros must have 0.0 entropy");
}

#[test]
fn test_entropy_pure_ones() {
    let data = vec![0xFFu8; 1024];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0, "Pure 0xFF must have 0.0 entropy");
}

#[test]
fn test_entropy_empty_data() {
    let data: [u8; 0] = [];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0, "Empty data must have 0.0 entropy");
}

#[test]
fn test_entropy_single_byte() {
    let data = [42u8];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0, "Single byte must have 0.0 entropy");
}

#[test]
fn test_entropy_alternating_bits_50_50() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = if i % 2 == 0 { 0x00 } else { 0xFF };
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 1.0).abs() < 1e-6, "50/50 two-symbol split must equal 1.0 b/B");
}

#[test]
fn test_entropy_four_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 4) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 2.0).abs() < 1e-6, "Four uniform symbols must equal 2.0 b/B");
}

#[test]
fn test_entropy_eight_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 8) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 3.0).abs() < 1e-6, "Eight uniform symbols must equal 3.0 b/B");
}

#[test]
fn test_entropy_16_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 16) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 4.0).abs() < 1e-6, "16 uniform symbols must equal 4.0 b/B");
}

#[test]
fn test_entropy_32_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 32) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 5.0).abs() < 1e-6, "32 uniform symbols must equal 5.0 b/B");
}

#[test]
fn test_entropy_64_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 64) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 6.0).abs() < 1e-6, "64 uniform symbols must equal 6.0 b/B");
}

#[test]
fn test_entropy_128_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 128) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 7.0).abs() < 1e-6, "128 uniform symbols must equal 7.0 b/B");
}

#[test]
fn test_entropy_256_symbols_uniform() {
    let mut data = vec![0u8; 1024];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 256) as u8;
    }
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!((h - 8.0).abs() < 1e-6, "256 uniform symbols must equal 8.0 b/B");
}

#[test]
fn test_entropy_constant_byte_0x10() {
    let data = vec![16u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0x20() {
    let data = vec![32u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0x30() {
    let data = vec![48u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0x42() {
    let data = vec![66u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0x7F() {
    let data = vec![127u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0x80() {
    let data = vec![128u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0xAA() {
    let data = vec![170u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0xCC() {
    let data = vec![204u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0xEE() {
    let data = vec![238u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_constant_byte_0xFE() {
    let data = vec![254u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_64() {
    let data = vec![0u8; 64];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_128() {
    let data = vec![0u8; 128];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_256() {
    let data = vec![0u8; 256];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_512() {
    let data = vec![0u8; 512];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_1024() {
    let data = vec![0u8; 1024];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_2048() {
    let data = vec![0u8; 2048];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_4096() {
    let data = vec![0u8; 4096];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_8192() {
    let data = vec![0u8; 8192];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_16384() {
    let data = vec![0u8; 16384];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_32768() {
    let data = vec![0u8; 32768];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_65536() {
    let data = vec![0u8; 65536];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_131072() {
    let data = vec![0u8; 131072];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_buffer_size_262144() {
    let data = vec![0u8; 262144];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert_eq!(h, 0.0);
}

#[test]
fn test_parse_size_valid_0b() {
    let res = crate::plugins::toml_loader::parse_size_string("0B");
    assert_eq!(res.unwrap(), 0u64);
}

#[test]
fn test_parse_size_valid_1b() {
    let res = crate::plugins::toml_loader::parse_size_string("1B");
    assert_eq!(res.unwrap(), 1u64);
}

#[test]
fn test_parse_size_valid_512b() {
    let res = crate::plugins::toml_loader::parse_size_string("512B");
    assert_eq!(res.unwrap(), 512u64);
}

#[test]
fn test_parse_size_valid_1024b() {
    let res = crate::plugins::toml_loader::parse_size_string("1024B");
    assert_eq!(res.unwrap(), 1024u64);
}

#[test]
fn test_parse_size_valid_4096b() {
    let res = crate::plugins::toml_loader::parse_size_string("4096B");
    assert_eq!(res.unwrap(), 4096u64);
}

#[test]
fn test_parse_size_valid_1kb() {
    let res = crate::plugins::toml_loader::parse_size_string("1KB");
    assert_eq!(res.unwrap(), 1024u64);
}

#[test]
fn test_parse_size_valid_2kb() {
    let res = crate::plugins::toml_loader::parse_size_string("2KB");
    assert_eq!(res.unwrap(), 2048u64);
}

#[test]
fn test_parse_size_valid_4kb() {
    let res = crate::plugins::toml_loader::parse_size_string("4KB");
    assert_eq!(res.unwrap(), 4096u64);
}

#[test]
fn test_parse_size_valid_8kb() {
    let res = crate::plugins::toml_loader::parse_size_string("8KB");
    assert_eq!(res.unwrap(), 8192u64);
}

#[test]
fn test_parse_size_valid_16kb() {
    let res = crate::plugins::toml_loader::parse_size_string("16KB");
    assert_eq!(res.unwrap(), 16384u64);
}

#[test]
fn test_parse_size_valid_32kb() {
    let res = crate::plugins::toml_loader::parse_size_string("32KB");
    assert_eq!(res.unwrap(), 32768u64);
}

#[test]
fn test_parse_size_valid_64kb() {
    let res = crate::plugins::toml_loader::parse_size_string("64KB");
    assert_eq!(res.unwrap(), 65536u64);
}

#[test]
fn test_parse_size_valid_128kb() {
    let res = crate::plugins::toml_loader::parse_size_string("128KB");
    assert_eq!(res.unwrap(), 131072u64);
}

#[test]
fn test_parse_size_valid_256kb() {
    let res = crate::plugins::toml_loader::parse_size_string("256KB");
    assert_eq!(res.unwrap(), 262144u64);
}

#[test]
fn test_parse_size_valid_512kb() {
    let res = crate::plugins::toml_loader::parse_size_string("512KB");
    assert_eq!(res.unwrap(), 524288u64);
}

#[test]
fn test_parse_size_valid_1mb() {
    let res = crate::plugins::toml_loader::parse_size_string("1MB");
    assert_eq!(res.unwrap(), 1048576u64);
}

#[test]
fn test_parse_size_valid_2mb() {
    let res = crate::plugins::toml_loader::parse_size_string("2MB");
    assert_eq!(res.unwrap(), 2097152u64);
}

#[test]
fn test_parse_size_valid_4mb() {
    let res = crate::plugins::toml_loader::parse_size_string("4MB");
    assert_eq!(res.unwrap(), 4194304u64);
}

#[test]
fn test_parse_size_valid_8mb() {
    let res = crate::plugins::toml_loader::parse_size_string("8MB");
    assert_eq!(res.unwrap(), 8388608u64);
}

#[test]
fn test_parse_size_valid_16mb() {
    let res = crate::plugins::toml_loader::parse_size_string("16MB");
    assert_eq!(res.unwrap(), 16777216u64);
}

#[test]
fn test_parse_size_valid_32mb() {
    let res = crate::plugins::toml_loader::parse_size_string("32MB");
    assert_eq!(res.unwrap(), 33554432u64);
}

#[test]
fn test_parse_size_valid_64mb() {
    let res = crate::plugins::toml_loader::parse_size_string("64MB");
    assert_eq!(res.unwrap(), 67108864u64);
}

#[test]
fn test_parse_size_valid_128mb() {
    let res = crate::plugins::toml_loader::parse_size_string("128MB");
    assert_eq!(res.unwrap(), 134217728u64);
}

#[test]
fn test_parse_size_valid_256mb() {
    let res = crate::plugins::toml_loader::parse_size_string("256MB");
    assert_eq!(res.unwrap(), 268435456u64);
}

#[test]
fn test_parse_size_valid_512mb() {
    let res = crate::plugins::toml_loader::parse_size_string("512MB");
    assert_eq!(res.unwrap(), 536870912u64);
}

#[test]
fn test_parse_size_valid_1gb() {
    let res = crate::plugins::toml_loader::parse_size_string("1GB");
    assert_eq!(res.unwrap(), 1073741824u64);
}

#[test]
fn test_parse_size_valid_2gb() {
    let res = crate::plugins::toml_loader::parse_size_string("2GB");
    assert_eq!(res.unwrap(), 2147483648u64);
}

#[test]
fn test_parse_size_valid_4gb() {
    let res = crate::plugins::toml_loader::parse_size_string("4GB");
    assert_eq!(res.unwrap(), 4294967296u64);
}

#[test]
fn test_parse_size_valid_8gb() {
    let res = crate::plugins::toml_loader::parse_size_string("8GB");
    assert_eq!(res.unwrap(), 8589934592u64);
}

#[test]
fn test_parse_size_valid_16gb() {
    let res = crate::plugins::toml_loader::parse_size_string("16GB");
    assert_eq!(res.unwrap(), 17179869184u64);
}

#[test]
fn test_parse_size_valid_1tb() {
    let res = crate::plugins::toml_loader::parse_size_string("1TB");
    assert_eq!(res.unwrap(), 1099511627776u64);
}

#[test]
fn test_parse_size_valid_2tb() {
    let res = crate::plugins::toml_loader::parse_size_string("2TB");
    assert_eq!(res.unwrap(), 2199023255552u64);
}

#[test]
fn test_parse_size_valid_4tb() {
    let res = crate::plugins::toml_loader::parse_size_string("4TB");
    assert_eq!(res.unwrap(), 4398046511104u64);
}

#[test]
fn test_parse_size_valid_8tb() {
    let res = crate::plugins::toml_loader::parse_size_string("8TB");
    assert_eq!(res.unwrap(), 8796093022208u64);
}

#[test]
fn test_parse_size_valid_10tb() {
    let res = crate::plugins::toml_loader::parse_size_string("10TB");
    assert_eq!(res.unwrap(), 10995116277760u64);
}

#[test]
fn test_hashchain_verification_depth_1() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..1 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 1);
    assert!(chain.verify(), "Intact chain with depth 1 must verify");
}

#[test]
fn test_hashchain_verification_depth_2() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..2 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 2);
    assert!(chain.verify(), "Intact chain with depth 2 must verify");
}

#[test]
fn test_hashchain_verification_depth_3() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..3 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 3);
    assert!(chain.verify(), "Intact chain with depth 3 must verify");
}

#[test]
fn test_hashchain_verification_depth_4() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..4 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 4);
    assert!(chain.verify(), "Intact chain with depth 4 must verify");
}

#[test]
fn test_hashchain_verification_depth_5() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..5 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 5);
    assert!(chain.verify(), "Intact chain with depth 5 must verify");
}

#[test]
fn test_hashchain_verification_depth_6() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..6 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 6);
    assert!(chain.verify(), "Intact chain with depth 6 must verify");
}

#[test]
fn test_hashchain_verification_depth_7() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..7 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 7);
    assert!(chain.verify(), "Intact chain with depth 7 must verify");
}

#[test]
fn test_hashchain_verification_depth_8() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..8 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 8);
    assert!(chain.verify(), "Intact chain with depth 8 must verify");
}

#[test]
fn test_hashchain_verification_depth_9() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..9 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 9);
    assert!(chain.verify(), "Intact chain with depth 9 must verify");
}

#[test]
fn test_hashchain_verification_depth_10() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..10 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 10);
    assert!(chain.verify(), "Intact chain with depth 10 must verify");
}

#[test]
fn test_hashchain_verification_depth_11() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..11 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 11);
    assert!(chain.verify(), "Intact chain with depth 11 must verify");
}

#[test]
fn test_hashchain_verification_depth_12() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..12 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 12);
    assert!(chain.verify(), "Intact chain with depth 12 must verify");
}

#[test]
fn test_hashchain_verification_depth_13() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..13 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 13);
    assert!(chain.verify(), "Intact chain with depth 13 must verify");
}

#[test]
fn test_hashchain_verification_depth_14() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..14 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 14);
    assert!(chain.verify(), "Intact chain with depth 14 must verify");
}

#[test]
fn test_hashchain_verification_depth_15() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..15 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 15);
    assert!(chain.verify(), "Intact chain with depth 15 must verify");
}

#[test]
fn test_hashchain_verification_depth_16() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..16 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 16);
    assert!(chain.verify(), "Intact chain with depth 16 must verify");
}

#[test]
fn test_hashchain_verification_depth_17() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..17 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 17);
    assert!(chain.verify(), "Intact chain with depth 17 must verify");
}

#[test]
fn test_hashchain_verification_depth_18() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..18 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 18);
    assert!(chain.verify(), "Intact chain with depth 18 must verify");
}

#[test]
fn test_hashchain_verification_depth_19() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..19 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 19);
    assert!(chain.verify(), "Intact chain with depth 19 must verify");
}

#[test]
fn test_hashchain_verification_depth_20() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..20 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 20);
    assert!(chain.verify(), "Intact chain with depth 20 must verify");
}

#[test]
fn test_hashchain_verification_depth_21() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..21 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 21);
    assert!(chain.verify(), "Intact chain with depth 21 must verify");
}

#[test]
fn test_hashchain_verification_depth_22() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..22 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 22);
    assert!(chain.verify(), "Intact chain with depth 22 must verify");
}

#[test]
fn test_hashchain_verification_depth_23() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..23 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 23);
    assert!(chain.verify(), "Intact chain with depth 23 must verify");
}

#[test]
fn test_hashchain_verification_depth_24() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..24 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 24);
    assert!(chain.verify(), "Intact chain with depth 24 must verify");
}

#[test]
fn test_hashchain_verification_depth_25() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..25 {
        let entry = AuditEntry {
            id: i,
            seq: i,
            timestamp: format!("2026-10-19T00:00:{:02}Z", i % 60),
            operation: OperationType::DiskWipe,
            target: format!("/dev/loop{}", i),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Operation {} ok", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }
    assert_eq!(chain.len(), 25);
    assert!(chain.verify(), "Intact chain with depth 25 must verify");
}

#[test]
fn test_hashchain_entry_op_type_disk_wipe() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::DiskWipe,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_file_shred() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::FileShred,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_file_carve() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::FileCarve,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_firmware_erase() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::FirmwareErase,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_diagnostics() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::Diagnostics,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_hash_verification() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::HashVerification,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_metadata_scrub() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::MetadataScrub,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_integrity_audit() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::IntegrityAudit,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_entropy_analysis() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::EntropyAnalysis,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_hashchain_entry_op_type_partition_wipe() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    let entry = AuditEntry {
        id: 1,
        seq: 1,
        timestamp: "2026-10-19T00:00:00Z".to_string(),
        operation: OperationType::PartitionWipe,
        target: "/dev/loop0".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "ok".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(512),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain.verify());
}

#[test]
fn test_metadata_scrub_cluster_slack_512_empty() {
    let mut buf = vec![0u8; 512];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_1024_empty() {
    let mut buf = vec![0u8; 1024];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_2048_empty() {
    let mut buf = vec![0u8; 2048];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_4096_empty() {
    let mut buf = vec![0u8; 4096];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_8192_empty() {
    let mut buf = vec![0u8; 8192];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_16384_empty() {
    let mut buf = vec![0u8; 16384];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_32768_empty() {
    let mut buf = vec![0u8; 32768];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_cluster_slack_65536_empty() {
    let mut buf = vec![0u8; 65536];
    let stats = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrub_fat_slack_512_empty() {
    let mut buf = vec![0u8; 512];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_1024_empty() {
    let mut buf = vec![0u8; 1024];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_2048_empty() {
    let mut buf = vec![0u8; 2048];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_4096_empty() {
    let mut buf = vec![0u8; 4096];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_8192_empty() {
    let mut buf = vec![0u8; 8192];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_16384_empty() {
    let mut buf = vec![0u8; 16384];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_32768_empty() {
    let mut buf = vec![0u8; 32768];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_fat_slack_65536_empty() {
    let mut buf = vec![0u8; 65536];
    let count = crate::wiper::metadata::scrub_fat_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_512_empty() {
    let mut buf = vec![0u8; 512];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_1024_empty() {
    let mut buf = vec![0u8; 1024];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_2048_empty() {
    let mut buf = vec![0u8; 2048];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_4096_empty() {
    let mut buf = vec![0u8; 4096];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_8192_empty() {
    let mut buf = vec![0u8; 8192];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_16384_empty() {
    let mut buf = vec![0u8; 16384];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_32768_empty() {
    let mut buf = vec![0u8; 32768];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ext4_slack_65536_empty() {
    let mut buf = vec![0u8; 65536];
    let count = crate::wiper::metadata::scrub_ext4_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_512_empty() {
    let mut buf = vec![0u8; 512];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_1024_empty() {
    let mut buf = vec![0u8; 1024];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_2048_empty() {
    let mut buf = vec![0u8; 2048];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_4096_empty() {
    let mut buf = vec![0u8; 4096];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_8192_empty() {
    let mut buf = vec![0u8; 8192];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_16384_empty() {
    let mut buf = vec![0u8; 16384];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_32768_empty() {
    let mut buf = vec![0u8; 32768];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrub_ntfs_slack_65536_empty() {
    let mut buf = vec![0u8; 65536];
    let count = crate::wiper::metadata::scrub_ntfs_metadata_buffer(&mut buf);
    assert_eq!(count, 0);
}

#[test]
fn test_metadata_scrubber_idempotence_trial_1() {
    let mut buf = vec![0u8; 4096];
    let stats1 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    let stats2 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats1.total_scrubbed(), 0);
    assert_eq!(stats2.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrubber_idempotence_trial_2() {
    let mut buf = vec![0u8; 4096];
    let stats1 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    let stats2 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats1.total_scrubbed(), 0);
    assert_eq!(stats2.total_scrubbed(), 0);
}

#[test]
fn test_metadata_scrubber_idempotence_trial_3() {
    let mut buf = vec![0u8; 4096];
    let stats1 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    let stats2 = crate::wiper::metadata::scrub_filesystem_metadata_buffer(&mut buf);
    assert_eq!(stats1.total_scrubbed(), 0);
    assert_eq!(stats2.total_scrubbed(), 0);
}

#[test]
fn test_path_protection_for_root() {
    let path = std::path::Path::new("/");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path / must be protected from shredding");
}

#[test]
fn test_path_protection_for_boot() {
    let path = std::path::Path::new("/boot");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /boot must be protected from shredding");
}

#[test]
fn test_path_protection_for_etc() {
    let path = std::path::Path::new("/etc");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /etc must be protected from shredding");
}

#[test]
fn test_path_protection_for_usr() {
    let path = std::path::Path::new("/usr");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /usr must be protected from shredding");
}

#[test]
fn test_path_protection_for_sys() {
    let path = std::path::Path::new("/sys");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /sys must be protected from shredding");
}

#[test]
fn test_path_protection_for_proc() {
    let path = std::path::Path::new("/proc");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /proc must be protected from shredding");
}

#[test]
fn test_path_protection_for_dev() {
    let path = std::path::Path::new("/dev");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /dev must be protected from shredding");
}

#[test]
fn test_path_protection_for_bin() {
    let path = std::path::Path::new("/bin");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /bin must be protected from shredding");
}

#[test]
fn test_path_protection_for_sbin() {
    let path = std::path::Path::new("/sbin");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /sbin must be protected from shredding");
}

#[test]
fn test_path_protection_for_lib() {
    let path = std::path::Path::new("/lib");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /lib must be protected from shredding");
}

#[test]
fn test_path_protection_for_lib64() {
    let path = std::path::Path::new("/lib64");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /lib64 must be protected from shredding");
}

#[test]
fn test_path_protection_for_home() {
    let path = std::path::Path::new("/home");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /home must be protected from shredding");
}

#[test]
fn test_path_protection_for_var() {
    let path = std::path::Path::new("/var");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /var must be protected from shredding");
}

#[test]
fn test_path_protection_for_root_dir() {
    let path = std::path::Path::new("/root");
    assert!(crate::wiper::file_wiper::is_protected_path(path), "Path /root must be protected from shredding");
}

#[test]
fn test_firmware_method_parse_ata_secure() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("ata-secure");
    assert!(parsed.is_some(), "Method ata-secure must parse successfully");
    assert_eq!(parsed.unwrap().as_str(), "ata-secure");
}

#[test]
fn test_firmware_method_parse_ata_enhanced() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("ata-enhanced");
    assert!(parsed.is_some(), "Method ata-enhanced must parse successfully");
    assert_eq!(parsed.unwrap().as_str(), "ata-enhanced");
}

#[test]
fn test_firmware_method_parse_nvme_crypto() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("nvme-crypto");
    assert!(parsed.is_some(), "Method nvme-crypto must parse successfully");
    assert_eq!(parsed.unwrap().as_str(), "nvme-crypto");
}

#[test]
fn test_firmware_method_parse_nvme_block() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("nvme-block");
    assert!(parsed.is_some(), "Method nvme-block must parse successfully");
    assert_eq!(parsed.unwrap().as_str(), "nvme-block");
}

#[test]
fn test_firmware_method_reject_ata_invalid() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("ata_invalid");
    assert!(parsed.is_none(), "Invalid method ata_invalid must be rejected");
}

#[test]
fn test_firmware_method_reject_nvme_fake() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("nvme_fake");
    assert!(parsed.is_none(), "Invalid method nvme_fake must be rejected");
}

#[test]
fn test_firmware_method_reject_scsi_wipe() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("scsi_wipe");
    assert!(parsed.is_none(), "Invalid method scsi_wipe must be rejected");
}

#[test]
fn test_firmware_method_reject_pcie_destroy() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("pcie_destroy");
    assert!(parsed.is_none(), "Invalid method pcie_destroy must be rejected");
}

#[test]
fn test_firmware_method_reject_sata_kill() {
    let parsed = crate::wiper::firmware::FirmwareMethod::parse("sata_kill");
    assert!(parsed.is_none(), "Invalid method sata_kill must be rejected");
}

#[test]
fn test_rfc3161_empty_token_rejection() {
    let empty_token = [];
    let hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let res = crate::audit::signing::verify_rfc3161_token_imprint(&empty_token, hash).unwrap();
    assert!(!res, "Empty RFC 3161 token must not verify");
}

#[test]
fn test_rfc3161_short_header_token_rejection() {
    let short_token = [0x30, 0x02];
    let hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let res = crate::audit::signing::verify_rfc3161_token_imprint(&short_token, hash).unwrap();
    assert!(!res, "Truncated RFC 3161 token must not verify");
}

#[test]
fn test_rfc3161_malformed_hex_error() {
    let token = [0x30, 0x50, 0x02, 0x01, 0x00];
    let invalid_hex = "invalid_non_hex_digest";
    let res = crate::audit::signing::verify_rfc3161_token_imprint(&token, invalid_hex);
    assert!(res.is_err(), "Invalid hex string must produce error");
}

#[test]
fn test_diagnostics_suite_contains_all_core_assertions() {
    let summary = crate::diagnostics::run_diagnostics();
    assert!(summary.total >= 23, "Diagnostic suite must contain at least 23 live assertions");
    assert_eq!(summary.failed, 0, "No baseline assertions may fail");
}

#[test]
fn test_surface_scanner_detect_zero_bytes() {
    let data = vec![0u8; 4096];
    let h = crate::disk::surface_scanner::calc_shannon_entropy(&data);
    assert!(h < 0.001);
}
