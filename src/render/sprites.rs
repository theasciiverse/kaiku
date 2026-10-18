//! Half-block pixel art. One character is one pixel; `.` is transparent.

use super::canvas::Canvas;
use super::palette;

/// The player's ship, nose to the right: 9×5 pixels (`SHIP_W`×`SHIP_H` dots).
pub const SHIP: [&str; 5] = [
    "cc.......", //
    ".bbww....",
    "fbwwwwwwy",
    ".bbww....",
    "cc.......",
];

/// Power-up illustrations for the bottom panel, 5×4 pixels (5×2 cells) each.
pub const POWERUPS: [(&str, [&str; 4]); 6] = [
    ("spread", ["...yy", "yy.y.", "yy.y.", "...yy"]),
    ("rapid", [".....", "yy.yy", "yy.yy", "....."]),
    ("shield", [".ccc.", "c...c", "c...c", ".ccc."]),
    ("bomb", ["...f.", ".rrr.", "rrrrr", ".rrr."]),
    ("drone", [".cc..", "cwwc.", "cwwc.", ".cc.."]),
    ("slow", ["ccccc", ".c.c.", ".c.c.", "ccccc"]),
];

fn color(c: char, tick: u64) -> Option<(u8, u8, u8)> {
    Some(match c {
        'c' => (90, 200, 255),
        'b' => (70, 96, 190),
        'w' => (226, 232, 255),
        'y' => (255, 230, 120),
        'r' => (255, 96, 168),
        // engine flame flickers
        'f' => [(255, 140, 60), (255, 90, 40), (255, 200, 90)][(tick / 4 % 3) as usize],
        _ => return None,
    })
}

/// Draws a sprite with its top-left at half-block pixel (x, y).
pub fn draw(canvas: &mut Canvas, art: &[&str], x: i32, y: i32, tick: u64) {
    draw_faded(canvas, art, x, y, tick, 0.0);
}

/// Draws a sprite faded towards the chrome background by `fade` (0 = full
/// colour): power-ups the ship doesn't have show as dim silhouettes.
pub fn draw_faded(canvas: &mut Canvas, art: &[&str], x: i32, y: i32, tick: u64, fade: f32) {
    for (row, line) in art.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if let Some(c) = color(ch, tick) {
                canvas.pixel(x + col as i32, y + row as i32, palette::rgb(palette::lerp(c, palette::CHROME_BG_RGB, fade)));
            }
        }
    }
}
