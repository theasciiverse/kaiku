//! Half-block pixel art. One character is one pixel; `.` is transparent.

use super::canvas::Canvas;
use super::palette::{self, Rgb};
use crate::game::enemy::{Kind, Side};
use crate::game::powerup::{Loot, PowerUp};

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

/// A power-up pod: a capsule whose core shows what it holds. 4×3 pixels.
pub const POD: [&str; 3] = [".oo.", "oXXo", ".oo."];
/// A power-up a ship left behind: a small diamond, no word on it. 3×3 pixels.
pub const DROP: [&str; 3] = [".o.", "oXo", ".o."];

/// A shield cell: a small cross in the shield's cyan. 3×3 pixels.
pub const CELL: [&str; 3] = [".c.", "ccc", ".c."];

/// A drop pulses between its colour and white, so it catches the eye.
pub fn draw_drop(canvas: &mut Canvas, loot: Loot, x: i32, y: i32, tick: u64) {
    let kind = match loot {
        Loot::Power(kind) => kind,
        Loot::Cell => {
            let c = if (tick / 8).is_multiple_of(2) { (190, 236, 255) } else { (60, 210, 200) };
            draw_with(canvas, &CELL, x, y, |ch| (ch == 'c').then_some(c));
            return;
        }
    };
    let core = power_color(kind);
    let glint = if (tick / 8).is_multiple_of(2) { (255, 255, 255) } else { core };
    draw_with(canvas, &DROP, x, y, |c| match c {
        'o' => Some(glint),
        'X' => Some(core),
        _ => None,
    });
}

/// The escort drone, nose right. 4×2 pixels.
pub const DRONE: [&str; 2] = [".cw.", "cbwy"];

/// The colour a power-up shows in its pod and its word.
pub fn power_color(kind: PowerUp) -> Rgb {
    match kind {
        PowerUp::Spread | PowerUp::Rapid => (255, 230, 120),
        PowerUp::Shield | PowerUp::Slow => (90, 200, 255),
        PowerUp::Bomb => (255, 96, 168),
        PowerUp::Drone => (226, 232, 255),
    }
}

/// `hp` of `POD_HP` left: a hit shell darkens, so you see it cracking.
pub fn draw_pod(canvas: &mut Canvas, kind: PowerUp, hp: u8, flash: bool, x: i32, y: i32, tick: u64) {
    // the shell glints as it turns, and dims with every hit
    let bright = if (tick / 10).is_multiple_of(3) { (220, 236, 255) } else { (120, 150, 190) };
    let wear = 1.0 - hp as f32 / crate::game::powerup::POD_HP as f32;
    let shell = if flash { (255, 255, 255) } else { palette::lerp(bright, (60, 40, 50), wear * 0.8) };
    let core = power_color(kind);
    draw_with(canvas, &POD, x, y, |c| match c {
        'o' => Some(shell),
        'X' => Some(core),
        _ => None,
    });
}

pub fn draw_drone(canvas: &mut Canvas, x: i32, y: i32, tick: u64) {
    draw(canvas, &DRONE, x, y, tick);
}

/// Enemy ships face left. `a` hull, `A` highlight, `e` engine, `k` dark.
pub const SCOUT: [&str; 3] = ["..aa.", "kAAae", "..aa."];
pub const GUNSHIP: [&str; 5] = ["...aa..", ".aaAAae", "kAAAAae", ".aaAAae", "...aa.."];
pub const INTERCEPTOR: [&str; 3] = ["..aaa.", "kAAAAe", "..aaa."];
pub const MINE: [&str; 3] = ["a.a", ".A.", "a.a"];
pub const LANCER: [&str; 4] = ["...aaa.", "kkAAAae", "..AAAae", "...aaa."];
pub const JAMMER: [&str; 5] = [".A..A.", "..AA..", ".aaaa.", "aakkaa", ".aaaa."];
pub const CARRIER: [&str; 8] = [
    "....aaaaaaaa....",
    "..aaAAAAAAAAaa..",
    ".aAAAAkkkkAAAAae",
    "kAAAAAAAAAAAAAae",
    "kAAAAAAAAAAAAAae",
    ".aAAAAkkkkAAAAae",
    "..aaAAAAAAAAaa..",
    "....aaaaaaaa....",
];

/// The art for a kind of enemy.
pub fn enemy_art(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Scout => &SCOUT,
        Kind::Gunship => &GUNSHIP,
        Kind::Interceptor => &INTERCEPTOR,
        Kind::Mine => &MINE,
        Kind::Lancer => &LANCER,
        Kind::Jammer => &JAMMER,
        Kind::Carrier => &CARRIER,
    }
}

/// Hull, highlight and engine colours of a side.
pub fn side_colors(side: Side) -> [Rgb; 3] {
    match side {
        Side::Amber => [(214, 120, 40), (255, 196, 92), (255, 90, 40)],
        Side::Jade => [(40, 168, 120), (140, 250, 190), (90, 255, 230)],
    }
}

/// Draws an enemy; `flash` turns it white for the moment it is hit.
pub fn draw_enemy(canvas: &mut Canvas, art: &[&str], side: Side, flash: bool, x: i32, y: i32) {
    let [hull, high, engine] = side_colors(side);
    draw_with(canvas, art, x, y, |c| match c {
        _ if flash && c != '.' => Some((255, 255, 255)),
        'a' => Some(hull),
        'A' => Some(high),
        'e' => Some(engine),
        'k' => Some((60, 50, 74)),
        _ => None,
    });
}

fn draw_with(canvas: &mut Canvas, art: &[&str], x: i32, y: i32, color: impl Fn(char) -> Option<Rgb>) {
    for (row, line) in art.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if let Some(c) = color(ch) {
                canvas.pixel(x + col as i32, y + row as i32, palette::rgb(c));
            }
        }
    }
}

fn color(c: char, tick: u64) -> Option<Rgb> {
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
    draw_with(canvas, art, x, y, |ch| color(ch, tick).map(|c| palette::lerp(c, palette::CHROME_BG_RGB, fade)));
}
