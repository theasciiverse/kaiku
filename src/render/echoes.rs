//! Echo text and portals. Echoes are text on whole cells, drawn over the
//! canvas: garbled glyphs, flickering clear, typed letters bright for good.

use super::canvas::Canvas;
use super::palette::{self, SIGNAL};
use crate::game::echo::{Echo, EchoState, Vis};
use crate::game::sector::Portal;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const GARBLE: &[char] = &['#', '%', '&', '@', '$', '*', '+', '=', '?', '!', '<', '>', '/', '\\', '|', '~', '^'];
/// Behind an echo the receiver just rejected a key for.
const MISS_BG: Color = Color::Rgb(70, 22, 40);

/// One echo as styled characters.
pub fn cells(e: &Echo, tick: u64) -> Vec<(char, Style)> {
    let bg = if e.flash > 0.0 { Some(MISS_BG) } else { None };
    e.text
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            let (c, style) = match e.vis(i, tick) {
                Vis::Typed => (ch, Style::default().fg(palette::rgb(SIGNAL[2])).add_modifier(Modifier::BOLD)),
                Vis::Clear { blink: false } => (ch, Style::default().fg(palette::rgb(SIGNAL[1]))),
                Vis::Clear { blink: true } => (ch, Style::default().fg(palette::rgb(SIGNAL[0]))),
                Vis::Garbled(h) => {
                    let g = if h % 3 == 0 { char::from_u32(0x2800 + (h >> 8) % 256).unwrap_or('#') } else { GARBLE[(h >> 4) as usize % GARBLE.len()] };
                    (g, Style::default().fg(Color::Rgb(56, 104, 116)))
                }
            };
            (c, bg.map_or(style, |b| style.bg(b)))
        })
        .collect()
}

pub fn put_cells(buf: &mut Buffer, x: i32, y: u16, cells: &[(char, Style)], clip: Rect) {
    for (i, (c, style)) in cells.iter().enumerate() {
        let cx = x + i as i32;
        if cx < clip.left() as i32 || cx >= clip.right() as i32 || y < clip.top() || y >= clip.bottom() {
            continue;
        }
        if let Some(cell) = buf.cell_mut((cx as u16, y)) {
            cell.set_char(*c).set_style(*style);
        }
    }
}

/// The drifting echoes over the play area; the locked one gets a marker.
pub fn draw(buf: &mut Buffer, play: Rect, echoes: &[Echo], tick: u64) {
    for e in echoes.iter().filter(|e| e.state == EchoState::Drifting) {
        let x = play.x as i32 + (e.x / 2.0).floor() as i32;
        let y = play.y + (e.y / 4.0) as u16;
        if e.locked {
            let mark = [('▸', Style::default().fg(palette::rgb(SIGNAL[2])).add_modifier(Modifier::BOLD))];
            put_cells(buf, x - 2, y, &mark, play);
        }
        put_cells(buf, x, y, &cells(e, tick), play);
    }
}

/// A portal: a ring of half-block pixels whose colours swirl; brighter for a
/// clearer echo.
pub fn portal(canvas: &mut Canvas, p: &Portal, tick: u64) {
    const RING: [&str; 10] = [".####.", "##..##", "#....#", "#....#", "#....#", "#....#", "#....#", "#....#", "##..##", ".####."];
    let (px, py) = ((p.x / 2.0).round() as i32, (p.y / 2.0).round() as i32);
    let glow = 0.35 + 0.65 * p.clarity.clamp(0.0, 1.0);
    for (row, line) in RING.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != '#' {
                continue;
            }
            let t = ((row + col) as u64 * 7 + tick) % 48;
            let hue = palette::ramp(&SIGNAL, (t as f32 / 48.0 * std::f32::consts::TAU).sin() * 0.5 + 0.5);
            let Color::Rgb(r, g, b) = hue else { continue };
            canvas.pixel(px + col as i32, py + row as i32, palette::rgb(palette::lerp(palette::CHROME_BG_RGB, (r, g, b), glow)));
        }
    }
}

/// A portal's label above it: the start of its echo, or `???` for a blind one.
pub fn portal_label(buf: &mut Buffer, play: Rect, p: &Portal, echoes: &[Echo]) {
    let text = match p.echo.and_then(|id| echoes.iter().find(|e| e.id == id)) {
        Some(e) => e.text.chars().take(14).collect::<String>(),
        None => "???".to_string(),
    };
    let w = text.chars().count() as i32;
    let x = play.x as i32 + (p.x / 2.0) as i32 + 3 - w / 2;
    let y = (play.y + (p.y / 4.0) as u16).saturating_sub(1).max(play.y);
    let style = Style::default().fg(palette::rgb(SIGNAL[1])).add_modifier(Modifier::ITALIC);
    let cells: Vec<(char, Style)> = text.chars().map(|c| (c, style)).collect();
    put_cells(buf, x, y, &cells, play);
}
