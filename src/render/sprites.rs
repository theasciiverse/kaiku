//! Half-block pixel art. One character is one pixel; `.` is transparent.

use super::canvas::Canvas;
use ratatui::style::Color;

/// The player's ship, nose to the right: 9×5 pixels (`SHIP_W`×`SHIP_H` dots).
pub const SHIP: [&str; 5] = [
    "cc.......", //
    ".bbww....",
    "fbwwwwwwy",
    ".bbww....",
    "cc.......",
];

fn color(c: char, tick: u64) -> Option<Color> {
    Some(match c {
        'c' => Color::Rgb(90, 200, 255),
        'b' => Color::Rgb(70, 96, 190),
        'w' => Color::Rgb(226, 232, 255),
        'y' => Color::Rgb(255, 230, 120),
        // engine flame flickers
        'f' => [Color::Rgb(255, 140, 60), Color::Rgb(255, 90, 40), Color::Rgb(255, 200, 90)][(tick / 4 % 3) as usize],
        _ => return None,
    })
}

/// Draws a sprite with its top-left at half-block pixel (x, y).
pub fn draw(canvas: &mut Canvas, art: &[&str], x: i32, y: i32, tick: u64) {
    for (row, line) in art.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if let Some(c) = color(ch, tick) {
                canvas.pixel(x + col as i32, y + row as i32, c);
            }
        }
    }
}
