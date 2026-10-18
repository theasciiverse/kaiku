//! The frame around the play area. Its look is a receiver listening for echoes:
//! the name echoing in the header over a Braille waveform, rounded borders in
//! the signal ramp, signal meters in the titles, a HUD line.

use super::palette::{self, CHROME_BG, DIM, TEXT, TRACE};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

pub const WORDMARK: &str = "KAIKU";
/// The name's echoes in the header, quieter each time.
pub const ECHO: &str = "kaiku";
pub const SUBTITLE: &str = "echoes home";

/// Rows of the bottom panel, borders included: two lines of content.
pub const PANEL_ROWS: u16 = 4;
/// Width of the power-up box, borders included: six 5-cell icons, spaced.
pub const POWERUP_COLS: u16 = 6 * 6 + 3;

/// The screen, top to bottom: header row, bordered play area, the bottom panel
/// (receiver for messages, power-up illustrations), HUD row.
pub struct Layout {
    pub header: Rect,
    pub frame: Rect,
    pub receiver: Rect,
    pub powerups: Rect,
    pub hud: Rect,
}

pub fn layout(area: Rect) -> Layout {
    let header = Rect { height: 1.min(area.height), ..area };
    let hud = Rect { y: area.bottom().saturating_sub(1), height: 1.min(area.height), ..area };
    let panel_y = area.bottom().saturating_sub(1 + PANEL_ROWS);
    let frame = Rect { y: area.y + 1, height: panel_y.saturating_sub(area.y + 1), ..area };
    let pw = POWERUP_COLS.min(area.width / 2);
    let receiver = Rect { y: panel_y, width: area.width - pw, height: PANEL_ROWS, ..area };
    let powerups = Rect { x: area.x + area.width - pw, y: panel_y, width: pw, height: PANEL_ROWS };
    Layout { header, frame, receiver, powerups, hud }
}

/// The play area inside a frame.
pub fn inner(frame: Rect) -> Rect {
    Rect { x: frame.x + 1, y: frame.y + 1, width: frame.width.saturating_sub(2), height: frame.height.saturating_sub(2) }
}

fn put(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) -> u16 {
    let mut cx = x;
    for ch in s.chars() {
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_char(ch).set_style(style);
        }
        cx += 1;
    }
    cx
}

/// Header: the name and its echoes, `KAIKU kaiku kaiku kaiku`, each fainter,
/// over a Braille waveform: the receiver's trace. `signal` (0..1) is the
/// strength of what the receiver hears; it sets the waveform's amplitude.
pub fn header(buf: &mut Buffer, area: Rect, tick: u64, signal: f32) {
    let bg = Style::default().bg(CHROME_BG);
    wave(buf, area, tick, signal);
    let mut x = put(buf, area.x + 1, area.y, " ", bg);
    // a slow glint runs along the name
    let glint = (tick / 6 % 40) as usize;
    for (i, ch) in WORDMARK.chars().enumerate() {
        let fg = if i == glint { palette::rgb(palette::SIGNAL[2]) } else { palette::ramp(&palette::SIGNAL, 0.55 + i as f32 * 0.08) };
        x = put(buf, x, area.y, &ch.to_string(), bg.fg(fg).add_modifier(Modifier::BOLD));
    }
    for (k, fade) in [0.55, 0.32, 0.16].into_iter().enumerate() {
        x = put(buf, x, area.y, " ", bg);
        let fg = palette::rgb(palette::lerp(palette::CHROME_BG_RGB, palette::SIGNAL[1], fade));
        let style = if k == 0 { bg.fg(fg).add_modifier(Modifier::BOLD) } else { bg.fg(fg) };
        x = put(buf, x, area.y, ECHO, style);
    }
    put(buf, x, area.y, " ", bg);
    let sub = format!(" {SUBTITLE} ");
    let sx = area.right().saturating_sub(sub.chars().count() as u16 + 2);
    put(buf, sx, area.y, &sub, bg.fg(DIM).add_modifier(Modifier::ITALIC));
}

/// One row of Braille waveform across `area`.
fn wave(buf: &mut Buffer, area: Rect, tick: u64, signal: f32) {
    let amp = 0.35 + signal.clamp(0.0, 1.0) * 1.6;
    for cx in area.left()..area.right() {
        let mut bits = 0u32;
        for sub in 0..2u16 {
            let t = (cx * 2 + sub) as f32 * 0.31 - tick as f32 * 0.07;
            // a carrier plus a slower swell; with no signal it is a low hum
            let v = (t.sin() * 0.7 + (t * 0.23).sin() * 0.3) * amp;
            let row = (1.5 + v).round().clamp(0.0, 3.0) as usize;
            bits |= [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]][sub as usize][row];
        }
        let ch = char::from_u32(0x2800 + bits).unwrap_or(' ');
        put(buf, cx, area.y, &ch.to_string(), Style::default().fg(TRACE).bg(CHROME_BG));
    }
}

/// Rounded border whose colour runs along the signal ramp from left to right.
pub fn border(buf: &mut Buffer, frame: Rect) {
    if frame.width < 2 || frame.height < 2 {
        return;
    }
    let w = (frame.width - 1).max(1) as f32;
    let style = |x: u16| Style::default().fg(palette::ramp(&palette::SIGNAL, (x - frame.x) as f32 / w)).bg(CHROME_BG);
    let (l, r, t, b) = (frame.left(), frame.right() - 1, frame.top(), frame.bottom() - 1);
    for x in l..=r {
        let (top, bottom) = match x {
            _ if x == l => ('╭', '╰'),
            _ if x == r => ('╮', '╯'),
            _ => ('─', '─'),
        };
        put(buf, x, t, &top.to_string(), style(x));
        put(buf, x, b, &bottom.to_string(), style(x));
    }
    for y in t + 1..b {
        put(buf, l, y, "│", style(l));
        put(buf, r, y, "│", style(r));
    }
}

/// A border with a title in its top edge: `╭─ RECEIVER ▂▄▆ ───`.
pub fn titled(buf: &mut Buffer, frame: Rect, title: &str) {
    border(buf, frame);
    if frame.width < title.chars().count() as u16 + 12 {
        return;
    }
    let bg = Style::default().bg(CHROME_BG);
    let x = put(buf, frame.x + 2, frame.y, &format!(" {} ", title.to_uppercase()), bg.fg(TEXT).add_modifier(Modifier::BOLD));
    let mut x = x;
    for (i, ch) in "▂▄▆".chars().enumerate() {
        x = put(buf, x, frame.y, &ch.to_string(), bg.fg(palette::ramp(&palette::SIGNAL, 0.3 + i as f32 * 0.3)));
    }
    put(buf, x, frame.y, " ", bg);
}

/// Fills a rect with the chrome background.
pub fn clear(buf: &mut Buffer, area: Rect) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            put(buf, x, y, " ", Style::default().bg(CHROME_BG));
        }
    }
}

pub fn text(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) -> u16 {
    put(buf, x, y, s, style.bg(CHROME_BG))
}

/// HUD line: shield bar, lives, score, and the input note on the right.
pub fn hud(buf: &mut Buffer, area: Rect, shield: f32, lives: u8, score: u32, note: Option<&str>) {
    let bg = Style::default().bg(CHROME_BG);
    for x in area.left()..area.right() {
        put(buf, x, area.y, " ", bg);
    }
    let mut x = put(buf, area.x + 1, area.y, "SHIELD ", bg.fg(DIM));
    let segments = 10;
    let lit = (shield.clamp(0.0, 1.0) * segments as f32).round() as usize;
    for i in 0..segments {
        let fg = if i < lit { palette::ramp(&palette::SIGNAL[1..], i as f32 / 9.0) } else { TRACE };
        x = put(buf, x, area.y, if i < lit { "▰" } else { "▱" }, bg.fg(fg));
    }
    x = put(buf, x + 3, area.y, "LIVES ", bg.fg(DIM));
    for _ in 0..lives {
        x = put(buf, x, area.y, "▶ ", bg.fg(palette::rgb(palette::SIGNAL[2])));
    }
    x = put(buf, x + 2, area.y, "SCORE ", bg.fg(DIM));
    put(buf, x, area.y, &format!("{score:07}"), bg.fg(TEXT).add_modifier(Modifier::BOLD));
    let right = match note {
        Some(n) => format!("{n}  ·  esc quit "),
        None => "esc quit ".to_string(),
    };
    let rx = area.right().saturating_sub(right.chars().count() as u16);
    put(buf, rx, area.y, &right, bg.fg(DIM));
}

/// A small centred question box over the play area.
pub fn question(buf: &mut Buffer, area: Rect, text: &str) {
    let w = text.chars().count() as u16 + 4;
    if area.width < w || area.height < 3 {
        return;
    }
    let x = area.x + (area.width - w) / 2;
    let y = area.y + area.height / 2 - 1;
    let frame = Rect::new(x, y, w, 3);
    border(buf, frame);
    let style = Style::default().fg(TEXT).bg(CHROME_BG).add_modifier(Modifier::BOLD);
    put(buf, x + 1, y + 1, &format!(" {text} "), style);
}
