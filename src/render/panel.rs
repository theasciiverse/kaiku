//! The bottom panel: the receiver, where long echoes and ship-to-ship messages
//! are decoded, and the power-up illustrations.

use super::canvas::Canvas;
use super::chrome;
use super::echoes;
use crate::game::echo::EchoState;
use crate::game::sector::Phase;
use crate::game::{Game, State};

/// What the receiver hears on arrival.
const HOME_ECHO: (&str, &str) = ("hello from the children of planet earth", "Voyager Golden Record · 1977");
use super::palette::{self, CHROME_BG, DIM, TRACE};
use super::sprites;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

/// Cheap per-cell hash for static noise, so it shimmers but replays.
fn noise(x: u16, tick: u64) -> u32 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (tick / 3).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    h as u32
}

/// The receiver box. Line one is what the receiver hears: the echo being
/// decoded, a jump result, or static. Line two lists what has been decoded.
pub fn receiver(buf: &mut Buffer, area: Rect, game: &Game) {
    chrome::clear(buf, area);
    chrome::titled(buf, area, "receiver");
    if area.height < 4 || area.width < 20 {
        return;
    }
    let inner = chrome::inner(area);
    let clip = Rect { x: inner.x + 1, width: inner.width.saturating_sub(2), ..inner };
    let label = Style::default().fg(DIM).add_modifier(Modifier::BOLD);
    let x = chrome::text(buf, clip.x, inner.y, "RX ", label);
    let locked = game.typing.lock.and_then(|id| game.sector.echo(id)).filter(|e| e.state == EchoState::Drifting);
    let drifting = game.sector.echoes.iter().any(|e| e.state == EchoState::Drifting);
    if game.state == State::Home {
        // the last echo: perfectly clear and steady
        let x = chrome::text(buf, x, inner.y, HOME_ECHO.0, Style::default().fg(palette::rgb(palette::SIGNAL[2])).add_modifier(Modifier::BOLD));
        chrome::text(buf, x + 2, inner.y, HOME_ECHO.1, Style::default().fg(DIM));
        chrome::text(buf, clip.x, inner.y + 1, "◉ home", Style::default().fg(palette::rgb(palette::SIGNAL[2])));
        return;
    }
    if let Some(e) = locked {
        let cells = echoes::cells(e, game.tick);
        echoes::put_cells(buf, x as i32, inner.y, &cells, clip);
        meter(buf, x + cells.len() as u16 + 2, inner.y, e.clarity, clip);
    } else if let Some((note, _)) = &game.note {
        chrome::text(buf, x, inner.y, note, Style::default().fg(palette::rgb(palette::SIGNAL[2])));
    } else if drifting {
        chrome::text(buf, x, inner.y, "echoes in range · type to lock on", Style::default().fg(palette::rgb(palette::SIGNAL[1])).add_modifier(Modifier::ITALIC));
    } else {
        statics(buf, x, inner.y, clip.right(), game.tick);
    }
    // line two: decoded echoes, newest first, with their sources
    let mut x = clip.x;
    let decoded: Vec<_> = game.sector.echoes.iter().rev().filter(|e| e.state == EchoState::Decoded).collect();
    if decoded.is_empty() {
        let hint = if game.sector.phase == Phase::Portals { "portals open · fly into one" } else { "··· listening ···" };
        chrome::text(buf, x, inner.y + 1, hint, Style::default().fg(TRACE));
    }
    for e in decoded {
        if x >= clip.right() {
            break;
        }
        let item = [
            ('◉', Style::default().fg(palette::ramp(&palette::SIGNAL, e.clarity))),
            (' ', Style::default()),
        ];
        echoes::put_cells(buf, x as i32, inner.y + 1, &item, clip);
        let text: Vec<(char, Style)> = format!("{}  {}", e.text, e.source)
            .chars()
            .enumerate()
            .map(|(i, c)| (c, Style::default().fg(if i < e.text.chars().count() { palette::rgb(palette::SIGNAL[2]) } else { DIM })))
            .collect();
        echoes::put_cells(buf, x as i32 + 2, inner.y + 1, &text, clip);
        x += text.len() as u16 + 5;
    }
}

/// A small clarity meter: how strongly the echo points home.
fn meter(buf: &mut Buffer, x: u16, y: u16, clarity: f32, clip: Rect) {
    let lit = (clarity.clamp(0.0, 1.0) * 5.0).round() as usize;
    let cells: Vec<(char, Style)> = (0..5).map(|i| ('▰', Style::default().fg(if i < lit { palette::ramp(&palette::SIGNAL, i as f32 / 4.0) } else { TRACE }))).collect();
    echoes::put_cells(buf, x as i32, y, &cells, clip);
}

/// Static: "no signal" and sparse Braille noise, fading out.
fn statics(buf: &mut Buffer, x: u16, y: u16, end: u16, tick: u64) {
    let x = chrome::text(buf, x, y, "no signal ", Style::default().fg(DIM).add_modifier(Modifier::ITALIC));
    for cx in x..end {
        let n = noise(cx, tick);
        let ch = if n.is_multiple_of(3) { char::from_u32(0x2800 + (n >> 8) % 256).unwrap_or(' ') } else { ' ' };
        let t = (cx - x) as f32 / (end - x).max(1) as f32;
        let fg = palette::ramp(&[(100, 128, 148), (30, 64, 74)], t);
        chrome::text(buf, cx, y, &ch.to_string(), Style::default().fg(fg));
    }
}

/// The power-up box: every power-up's illustration, dim unless the ship has it.
pub fn powerups(buf: &mut Buffer, area: Rect, owned: &[bool], tick: u64) {
    chrome::clear(buf, area);
    chrome::titled(buf, area, "power-ups");
    let inner = chrome::inner(area);
    if inner.height < 2 {
        return;
    }
    let mut canvas = Canvas::new(inner.width, inner.height);
    for (i, (_, art)) in sprites::POWERUPS.iter().enumerate() {
        let x = 1 + i as i32 * 6;
        if x + 5 > inner.width as i32 {
            break;
        }
        let fade = if owned.get(i).copied().unwrap_or(false) { 0.0 } else { 0.72 };
        sprites::draw_faded(&mut canvas, art, x, 0, tick, fade);
    }
    canvas.blit(buf, inner, CHROME_BG);
}
