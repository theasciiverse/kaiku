//! The frame around the play area, after Charm's Crush TUI (ideas only): a
//! gradient wordmark, `╱` hatch fields, rounded gradient borders and a HUD line.

use super::palette::{self, CHROME_BG, DIM, HATCH, RAMP, TEXT};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

pub const WORDMARK: &str = "K A I K U";
pub const SUBTITLE: &str = "echo home";

/// Splits the screen: header row, bordered play area, HUD row. Returns the
/// play area inside the border.
pub fn layout(area: Rect) -> (Rect, Rect, Rect) {
    let header = Rect { height: 1.min(area.height), ..area };
    let hud = Rect { y: area.bottom().saturating_sub(1), height: 1.min(area.height), ..area };
    let frame = Rect { y: area.y + 1, height: area.height.saturating_sub(2), ..area };
    (header, frame, hud)
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

/// Header: `╱╱╱ K A I K U ╱╱╱╱ echo home ╱╱╱…`, the wordmark in the ramp.
pub fn header(buf: &mut Buffer, area: Rect, tick: u64) {
    let bg = Style::default().bg(CHROME_BG);
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_char('╱').set_style(bg.fg(HATCH));
        }
    }
    let mut x = area.x + 3;
    x = put(buf, x, area.y, " ", bg);
    let n = WORDMARK.chars().count().max(2) - 1;
    // a slow shimmer runs along the wordmark
    let phase = (tick % 240) as f32 / 240.0;
    for (i, ch) in WORDMARK.chars().enumerate() {
        let t = i as f32 / n as f32;
        let shimmer = ((t - phase).abs() < 0.12) as u8 as f32 * 0.25;
        let style = bg.fg(palette::ramp(&RAMP, t * (1.0 - shimmer))).add_modifier(Modifier::BOLD);
        x = put(buf, x, area.y, &ch.to_string(), style);
    }
    x = put(buf, x, area.y, " ", bg);
    x = put(buf, x + 4, area.y, " ", bg);
    x = put(buf, x, area.y, SUBTITLE, bg.fg(DIM).add_modifier(Modifier::ITALIC));
    put(buf, x, area.y, " ", bg);
}

/// Rounded border whose colour runs along the ramp from left to right.
pub fn border(buf: &mut Buffer, frame: Rect) {
    if frame.width < 2 || frame.height < 2 {
        return;
    }
    let w = (frame.width - 1).max(1) as f32;
    let color = |x: u16| palette::ramp(&RAMP, (x - frame.x) as f32 / w);
    let style = |x: u16| Style::default().fg(color(x)).bg(CHROME_BG);
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

/// HUD line: shield bar, lives, and the input note on the right.
pub fn hud(buf: &mut Buffer, area: Rect, shield: f32, lives: u8, note: Option<&str>) {
    let bg = Style::default().bg(CHROME_BG);
    for x in area.left()..area.right() {
        put(buf, x, area.y, " ", bg);
    }
    let mut x = put(buf, area.x + 1, area.y, "SHIELD ", bg.fg(DIM));
    let segments = 10;
    let lit = (shield.clamp(0.0, 1.0) * segments as f32).round() as usize;
    for i in 0..segments {
        let fg = if i < lit { palette::ramp(&[(90, 200, 255), (154, 102, 255)], i as f32 / 9.0) } else { HATCH };
        x = put(buf, x, area.y, if i < lit { "▰" } else { "▱" }, bg.fg(fg));
    }
    x = put(buf, x + 3, area.y, "LIVES ", bg.fg(DIM));
    for _ in 0..lives {
        x = put(buf, x, area.y, "◆ ", bg.fg(Color::Rgb(255, 96, 168)));
    }
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
