//! Colours and gradients.

use ratatui::style::Color;

pub type Rgb = (u8, u8, u8);

pub const SPACE: Color = Color::Rgb(8, 9, 22);
pub const CHROME_BG: Color = Color::Rgb(14, 13, 30);
pub const HATCH: Color = Color::Rgb(58, 46, 104);
pub const DIM: Color = Color::Rgb(110, 104, 150);
pub const TEXT: Color = Color::Rgb(214, 210, 240);

/// The wordmark and border ramp: echo pink to signal violet to deep blue.
pub const RAMP: [Rgb; 3] = [(255, 96, 168), (154, 102, 255), (72, 140, 255)];

pub const STAR: [Color; 3] = [Color::Rgb(70, 74, 110), Color::Rgb(130, 136, 180), Color::Rgb(220, 224, 255)];
pub const SHOT: Color = Color::Rgb(255, 226, 120);

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
