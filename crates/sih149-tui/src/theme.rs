use ratatui::style::Color;

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // Backgrounds & Surface (Obsidian Cyberpunk palette)
    pub const BG: Color = Color::Rgb(10, 12, 18);
    pub const SURFACE: Color = Color::Rgb(16, 20, 30);
    pub const SURFACE2: Color = Color::Rgb(24, 28, 42);
    pub const BORDER: Color = Color::Rgb(45, 52, 75);
    pub const BORDER_HI: Color = Color::Rgb(86, 110, 168);

    // Accent palette
    pub const CYAN: Color = Color::Rgb(56, 189, 248); // Ice Cyan Primary
    pub const TEAL: Color = Color::Rgb(45, 212, 191); // Neon Mint
    pub const PURPLE: Color = Color::Rgb(168, 85, 247); // Electric Violet
    pub const BLUE: Color = Color::Rgb(96, 165, 250); // Deep Sky
    pub const INDIGO: Color = Color::Rgb(129, 140, 248); // Soft Indigo

    // Status
    pub const SUCCESS: Color = Color::Rgb(52, 211, 153); // Emerald Pass
    pub const DANGER: Color = Color::Rgb(248, 113, 113); // Coral Red Fail
    pub const WARNING: Color = Color::Rgb(251, 191, 36); // Amber Warn
    pub const INFO: Color = Color::Rgb(147, 197, 253); // Soft Blue Info

    // Text hierarchy
    pub const TEXT: Color = Color::Rgb(241, 245, 249); // High contrast white
    pub const TEXT_DIM: Color = Color::Rgb(148, 163, 184); // Slate 400
    pub const MUTED: Color = Color::Rgb(100, 116, 139); // Slate 500
    pub const HIGHLIGHT: Color = Color::Rgb(255, 255, 255);

    // Micro-animation helpers
    pub fn spinner(tick: u64) -> &'static str {
        const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        FRAMES[(tick / 2) as usize % FRAMES.len()]
    }

    pub fn activity_bar(tick: u64) -> &'static str {
        const BARS: &[&str] = &[" ▃▅", "▃▅▇", "▅▇█", "▇██", "█▇▅", "▇▅▃", "▅▃ ", "▃  "];
        BARS[(tick / 2) as usize % BARS.len()]
    }

    pub fn pulse_color(tick: u64) -> Color {
        const PULSE: &[Color] = &[
            Color::Rgb(56, 189, 248),
            Color::Rgb(96, 165, 250),
            Color::Rgb(129, 140, 248),
            Color::Rgb(168, 85, 247),
            Color::Rgb(129, 140, 248),
            Color::Rgb(96, 165, 250),
        ];
        PULSE[(tick / 3) as usize % PULSE.len()]
    }

    pub fn active_border(tick: u64, is_focused: bool) -> Color {
        if is_focused {
            Self::pulse_color(tick)
        } else {
            Self::BORDER
        }
    }

    // Entropy heatmap colors (low→high entropy)
    pub const ENT_0: Color = Color::Rgb(15, 23, 42); // dead / zeroed
    pub const ENT_1: Color = Color::Rgb(20, 83, 45); // near-zero (dark green)
    pub const ENT_2: Color = Color::Rgb(34, 197, 94); // low (green)
    pub const ENT_3: Color = Color::Rgb(234, 179, 8); // moderate (yellow)
    pub const ENT_4: Color = Color::Rgb(249, 115, 22); // high (orange)
    pub const ENT_5: Color = Color::Rgb(239, 68, 68); // encrypted/compressed (red)

    pub fn entropy_color(e: f64) -> Color {
        match e as u32 {
            0 => Self::ENT_0,
            1 => Self::ENT_1,
            2..=3 => Self::ENT_2,
            4..=5 => Self::ENT_3,
            6 => Self::ENT_4,
            _ => Self::ENT_5,
        }
    }

    pub fn entropy_bar_char(e: f64) -> char {
        let level = ((e / 8.0) * 8.0).round() as usize;
        ['░', '░', '▒', '▒', '▓', '▓', '█', '█', '█'][level.min(8)]
    }
}
