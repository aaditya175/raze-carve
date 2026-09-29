use crate::error::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Represents a file signature (magic bytes) for carving.
#[derive(Debug, Deserialize, Clone)]
pub struct FileSignature {
    /// File extension (e.g., "jpg")
    pub extension: String,
    /// Human-readable description
    pub description: String,
    /// Magic bytes at the beginning of the file (hex encoded)
    pub magic_header: String,
    /// Magic bytes at the end of the file (hex encoded), if any
    pub magic_footer: Option<String>,
    /// Maximum expected file size in bytes
    pub max_size: u64,
    /// Per-byte mask: true = must match, false = wildcard (any byte).
    /// When None, all header bytes must match exactly (backward compatible).
    #[serde(skip)]
    pub header_mask: Option<Vec<bool>>,
}

/// Collection of file signatures.
#[derive(Debug, Deserialize, Default)]
pub struct SignatureDatabase {
    pub signatures: Vec<FileSignature>,
}

impl SignatureDatabase {
    /// Load signatures from a TOML file.
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let db: SignatureDatabase =
            toml::from_str(&content).map_err(|e| crate::error::CoreError::Parse(e.to_string()))?;
        Ok(db)
    }

    /// Convert magic hex string to bytes
    pub fn parse_hex(hex_str: &str) -> Result<Vec<u8>> {
        let hex_str = hex_str.replace(" ", "");
        if !hex_str.len().is_multiple_of(2) {
            return Err(crate::error::CoreError::Parse(format!(
                "Invalid magic hex string (odd length): '{}'",
                hex_str
            )));
        }
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16)
                    .map_err(|e| crate::error::CoreError::Parse(e.to_string()))
            })
            .collect()
    }

    /// Canonical builtin signature catalog for core forensic file types.
    pub fn builtin() -> Self {
        Self {
            signatures: vec![
                // ── Images ──────────────────────────────────────────────
                FileSignature {
                    extension: "jpg".into(),
                    description: "JPEG Image".into(),
                    magic_header: "ffd8ff".into(),
                    magic_footer: Some("ffd9".into()),
                    max_size: 50 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "png".into(),
                    description: "PNG Image".into(),
                    magic_header: "89504e470d0a1a0a".into(),
                    magic_footer: Some("49454e44ae426082".into()),
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "gif".into(),
                    description: "GIF Image (87a)".into(),
                    magic_header: "474946383761".into(), // GIF87a
                    magic_footer: Some("003b".into()),
                    max_size: 50 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "gif".into(),
                    description: "GIF Image (89a)".into(),
                    magic_header: "474946383961".into(), // GIF89a
                    magic_footer: Some("003b".into()),
                    max_size: 50 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "bmp".into(),
                    description: "BMP Image".into(),
                    magic_header: "424d".into(), // BM
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "webp".into(),
                    description: "WebP Image".into(),
                    // RIFF????WEBP — bytes 0-3 = RIFF, 4-7 = wildcard (size), 8-11 = WEBP
                    magic_header: "524946460000000057454250".into(),
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: Some(vec![
                        true, true, true, true,    // RIFF
                        false, false, false, false, // ???? (file size)
                        true, true, true, true,    // WEBP
                    ]),
                },
                // ── Documents ───────────────────────────────────────────
                FileSignature {
                    extension: "pdf".into(),
                    description: "PDF Document".into(),
                    magic_header: "25504446".into(), // %PDF
                    magic_footer: Some("2525454f46".into()), // %%EOF
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "doc".into(),
                    description: "Microsoft Office (OLE2)".into(),
                    magic_header: "d0cf11e0a1b11ae1".into(),
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "rtf".into(),
                    description: "Rich Text Format".into(),
                    magic_header: "7b5c727466".into(), // {\rtf
                    magic_footer: Some("7d".into()), // }
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "xml".into(),
                    description: "XML Document".into(),
                    magic_header: "3c3f786d6c".into(), // <?xml
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "html".into(),
                    description: "HTML Document".into(),
                    magic_header: "3c68746d6c".into(), // <html
                    magic_footer: None,
                    max_size: 50 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "html".into(),
                    description: "HTML Document (DOCTYPE)".into(),
                    magic_header: "3c21444f43545950".into(), // <!DOCTYP
                    magic_footer: None,
                    max_size: 50 * 1024 * 1024,
                    header_mask: None,
                },
                // ── Archives ────────────────────────────────────────────
                FileSignature {
                    extension: "zip".into(),
                    description: "ZIP Archive".into(),
                    magic_header: "504b0304".into(),
                    magic_footer: None,
                    max_size: 1024 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "gz".into(),
                    description: "Gzip Archive".into(),
                    magic_header: "1f8b08".into(),
                    magic_footer: None,
                    max_size: 1024 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "7z".into(),
                    description: "7-Zip Archive".into(),
                    magic_header: "377abcaf271c".into(),
                    magic_footer: None,
                    max_size: 1024 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "rar".into(),
                    description: "RAR Archive".into(),
                    magic_header: "526172211a0700".into(), // Rar!\x1a\x07\x00
                    magic_footer: None,
                    max_size: 1024 * 1024 * 1024,
                    header_mask: None,
                },
                // ── Audio ───────────────────────────────────────────────
                FileSignature {
                    extension: "mp3".into(),
                    description: "MP3 Audio (ID3v2)".into(),
                    magic_header: "494433".into(), // ID3
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "mp3".into(),
                    description: "MP3 Audio (Frame Sync)".into(),
                    magic_header: "fffb".into(), // MPEG frame sync + layer 3
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "flac".into(),
                    description: "FLAC Audio".into(),
                    magic_header: "664c6143".into(), // fLaC
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "ogg".into(),
                    description: "OGG Vorbis Audio".into(),
                    magic_header: "4f676753".into(), // OggS
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "wav".into(),
                    description: "WAV Audio".into(),
                    // RIFF????WAVE — 4 wildcard bytes for file size
                    magic_header: "524946460000000057415645".into(),
                    magic_footer: None,
                    max_size: 2 * 1024 * 1024 * 1024,
                    header_mask: Some(vec![
                        true, true, true, true,    // RIFF
                        false, false, false, false, // ???? (file size)
                        true, true, true, true,    // WAVE
                    ]),
                },
                FileSignature {
                    extension: "aac".into(),
                    description: "AAC Audio (ADTS)".into(),
                    magic_header: "fff1".into(), // ADTS sync
                    magic_footer: None,
                    max_size: 100 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "m4a".into(),
                    description: "M4A/AAC Audio".into(),
                    // ????ftypM4A — wildcard size + ftyp + M4A
                    magic_header: "00000000667479704d344120".into(),
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: Some(vec![
                        false, false, false, false, // ???? (atom size)
                        true, true, true, true,     // ftyp
                        true, true, true, true,     // M4A_  (with space)
                    ]),
                },
                FileSignature {
                    extension: "midi".into(),
                    description: "MIDI Audio".into(),
                    magic_header: "4d546864".into(), // MThd
                    magic_footer: None,
                    max_size: 10 * 1024 * 1024,
                    header_mask: None,
                },
                // ── Video ───────────────────────────────────────────────
                FileSignature {
                    extension: "mp4".into(),
                    description: "MP4 Video".into(),
                    // ????ftyp — wildcard atom size + ftyp marker
                    magic_header: "0000000066747970".into(),
                    magic_footer: None,
                    max_size: 10u64 * 1024 * 1024 * 1024,
                    header_mask: Some(vec![
                        false, false, false, false, // ???? (atom size)
                        true, true, true, true,     // ftyp
                    ]),
                },
                FileSignature {
                    extension: "avi".into(),
                    description: "AVI Video".into(),
                    // RIFF????AVI_
                    magic_header: "524946460000000041564920".into(),
                    magic_footer: None,
                    max_size: 10u64 * 1024 * 1024 * 1024,
                    header_mask: Some(vec![
                        true, true, true, true,    // RIFF
                        false, false, false, false, // ???? (file size)
                        true, true, true, true,    // AVI_
                    ]),
                },
                FileSignature {
                    extension: "mkv".into(),
                    description: "Matroska Video".into(),
                    magic_header: "1a45dfa3".into(), // EBML header
                    magic_footer: None,
                    max_size: 10u64 * 1024 * 1024 * 1024,
                    header_mask: None,
                },
                // ── Databases ───────────────────────────────────────────
                FileSignature {
                    extension: "sqlite".into(),
                    description: "SQLite Database".into(),
                    magic_header: "53514c69746520666f726d6174203300".into(),
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                // ── Executables ─────────────────────────────────────────
                FileSignature {
                    extension: "elf".into(),
                    description: "ELF Executable".into(),
                    magic_header: "7f454c46".into(), // \x7FELF
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
                FileSignature {
                    extension: "exe".into(),
                    description: "Windows PE Executable".into(),
                    magic_header: "4d5a".into(), // MZ
                    magic_footer: None,
                    max_size: 500 * 1024 * 1024,
                    header_mask: None,
                },
            ],
        }
    }
}
