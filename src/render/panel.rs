//! The bottom panel: the receiver, where long echoes and ship-to-ship messages
//! are decoded, and the power-up illustrations.

use super::canvas::Canvas;
use super::chrome;
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

/// The receiver box. Until the receiver has a signal it shows static.
pub fn receiver(buf: &mut Buffer, area: Rect, tick: u64) {
    chrome::clear(buf, area);
    chrome::titled(buf, area, "receiver");
    if area.height < 4 || area.width < 20 {
        return;
    }
    let inner = chrome::inner(area);
    let x = chrome::text(buf, inner.x + 1, inner.y, "RX ", Style::default().fg(DIM).add_modifier(Modifier::BOLD));
    let x = chrome::text(buf, x, inner.y, "no signal ", Style::default().fg(DIM).add_modifier(Modifier::ITALIC));
    // static: sparse Braille noise across the rest of the line, fading out
    let end = inner.right().saturating_sub(1);
    for cx in x..end {
        let n = noise(cx, tick);
        let ch = if n.is_multiple_of(3) { char::from_u32(0x2800 + (n >> 8) % 256).unwrap_or(' ') } else { ' ' };
        let t = (cx - x) as f32 / (end - x).max(1) as f32;
        let fg = palette::ramp(&[(100, 128, 148), (30, 64, 74)], t);
        chrome::text(buf, cx, inner.y, &ch.to_string(), Style::default().fg(fg));
    }
    chrome::text(buf, inner.x + 1, inner.y + 1, "··· listening ···", Style::default().fg(TRACE));
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
