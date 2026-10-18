//! Colours and gradients.

use ratatui::style::Color;

pub type Rgb = (u8, u8, u8);

pub const SPACE: Color = Color::Rgb(6, 10, 18);

/// Each tunnel has its own darkness, so a new sector looks like a new place.
pub fn tunnel(sector: u32) -> Color {
    const DARK: [Color; 5] = [SPACE, Color::Rgb(14, 8, 22), Color::Rgb(4, 16, 18), Color::Rgb(16, 12, 6), Color::Rgb(6, 8, 26)];
    DARK[(sector.saturating_sub(1) as usize) % DARK.len()]
}
pub const CHROME_BG_RGB: Rgb = (10, 16, 26);
pub const CHROME_BG: Color = Color::Rgb(CHROME_BG_RGB.0, CHROME_BG_RGB.1, CHROME_BG_RGB.2);
/// The receiver's quiet trace: waveform, empty meter segments.
pub const TRACE: Color = Color::Rgb(30, 64, 74);
pub const DIM: Color = Color::Rgb(100, 128, 148);
pub const TEXT: Color = Color::Rgb(214, 230, 240);

/// The receiver ramp, the chrome's own colours: deep teal to signal cyan to ice.
pub const SIGNAL: [Rgb; 3] = [(22, 120, 122), (60, 210, 200), (190, 236, 255)];

pub const STAR: [Color; 3] = [Color::Rgb(70, 74, 110), Color::Rgb(130, 136, 180), Color::Rgb(220, 224, 255)];
pub const SHOT: Color = Color::Rgb(255, 226, 120);
/// Reserved for enemy fire: no sprite, word or decoration uses it.
pub const ENEMY_SHOT: Color = Color::Rgb(255, 54, 64);
/// The same red, dimmed: a lancer's warning line before the beam.
pub const BEAM_WARNING: Color = Color::Rgb(150, 36, 46);

pub fn rgb(c: Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

/// A point along a multi-stop ramp, t in [0, 1].
pub fn ramp(stops: &[Rgb], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t.floor() as usize).min(stops.len() - 2);
    rgb(lerp(stops[i], stops[i + 1], t - i as f32))
}
