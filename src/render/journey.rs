//! Between sectors: the turn (the field swings round as you come out of a
//! portal), the route map, and the arrival home.

use super::canvas::Canvas;
use super::palette::{self, Rgb, SIGNAL};
use crate::game::route::{Route, MAX_DISTANCE};
use crate::game::{intro, Game, TURN_TIME};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The turn: the starfield rotates a quarter turn, streaking, while the
/// portal's ring opens out from the ship past the edges of the screen.
pub fn turn(canvas: &mut Canvas, game: &Game) {
    let t = game.state_t / TURN_TIME;
    // the whole field swings about its centre
    let (cx, cy) = (game.w / 2.0, game.h / 2.0);
    let a = ease(t) * std::f32::consts::FRAC_PI_2 * game.turn_dir;
    let (sin, cos) = a.sin_cos();
    // streak length peaks mid-turn
    let streak = (t * std::f32::consts::PI).sin() * 10.0 + 1.0;
    for s in &game.stars.stars {
        let color = palette::STAR[s.layer as usize];
        // the field is wider than tall: tile it so a quarter turn still fills it
        for tile in [-1.0, 0.0, 1.0] {
            for k in 0..streak as i32 {
                let (x, y) = (s.x + k as f32 - cx, s.y + tile * game.h - cy);
                canvas.dot((cx + x * cos - y * sin) as i32, (cy + x * sin + y * cos) as i32, color);
            }
        }
    }
    let (cx, cy) = game.ship.center();
    // the ring, in dots: an ellipse widening from the ship
    let r = ease(t) * game.w.max(game.h) * 1.2;
    for i in 0..240 {
        let th = i as f32 / 240.0 * std::f32::consts::TAU;
        let color = palette::ramp(&SIGNAL, (th * 3.0 + t * 8.0).sin() * 0.5 + 0.5);
        canvas.dot((cx + th.cos() * r) as i32, (cy + th.sin() * r * 0.8) as i32, color);
    }
}

/// Where a sector sits on the map, in cells within `area`: home at the right,
/// the far reaches at the left.
fn place(area: Rect, route: &Route, layer: usize, i: usize) -> (u16, u16) {
    let span = area.width.saturating_sub(16) as f32;
    let x = area.right() as f32 - 8.0 - span * layer as f32 / MAX_DISTANCE as f32;
    let n = route.layers[layer].len() as f32;
    let y = area.y as f32 + 2.0 + (area.height as f32 - 5.0) * (i as f32 + 1.0) / (n + 1.0);
    (x as u16, y as u16)
}

/// The route map: every sector by its distance from home, the path flown so
/// far, where you are, and how far is left.
pub fn map(canvas: &mut Canvas, buf: &mut Buffer, play: Rect, game: &Game) {
    let route = &game.route;
    let area = Rect { x: 0, y: 0, ..play };
    // the path flown, as Braille lines between cell centres
    for pair in route.visited.windows(2) {
        let (a, b) = (place(area, route, pair[0].0, pair[0].1), place(area, route, pair[1].0, pair[1].1));
        line(canvas, (a.0 as f32 * 2.0 + 1.0, a.1 as f32 * 4.0 + 2.0), (b.0 as f32 * 2.0 + 1.0, b.1 as f32 * 4.0 + 2.0), palette::rgb(SIGNAL[0]));
    }
    canvas.blit(buf, play, palette::SPACE);
    let title = Style::default().fg(palette::TEXT).add_modifier(Modifier::BOLD);
    put(buf, play.x + 2, play.y + 1, "ROUTE HOME", title);
    put(buf, play.x + 14, play.y + 1, &format!("sector {} · {}", game.sector.number, route.here().name), Style::default().fg(palette::DIM));
    for (d, layer) in route.layers.iter().enumerate() {
        for (i, node) in layer.iter().enumerate() {
            let (x, y) = place(area, route, d, i);
            let here = route.at == (d, i);
            let seen = route.visited.contains(&(d, i));
            let (mark, color): (&str, Rgb) = match () {
                _ if d == 0 => ("★", SIGNAL[2]),
                // where you are pulses
                _ if here => (if (game.tick / 20).is_multiple_of(2) { "◉" } else { "○" }, SIGNAL[2]),
                _ if seen => ("◆", SIGNAL[1]),
                _ => ("◇", (40, 70, 84)),
            };
            let style = Style::default().fg(palette::rgb(color));
            put(buf, play.x + x, play.y + y, mark, style.add_modifier(Modifier::BOLD));
            let name_style = if here || d == 0 { style.add_modifier(Modifier::BOLD) } else { style };
            put(buf, (play.x + x).saturating_sub(1), play.y + y + 1, &node.name, name_style);
        }
    }
    let d = route.distance();
    let foot = format!("{d} jump{} to home  ·  enter: fly on", if d == 1 { "" } else { "s" });
    put(buf, play.x + 2, play.bottom().saturating_sub(2), &foot, Style::default().fg(palette::rgb(SIGNAL[1])));
}

/// Home: Earth rising on the right, growing as you close in.
pub fn earth(canvas: &mut Canvas, game: &Game) {
    // the field in half-block pixels: one per cell across, two per cell down
    let (pw, ph) = (game.w / 2.0, game.h / 2.0);
    let grow = ease(game.state_t / 6.0);
    let r = ph * (0.2 + 0.16 * grow);
    let (cx, cy) = (pw - r - 4.0, ph / 2.0);
    for py in (cy - r) as i32..=(cy + r) as i32 {
        for px in (cx - r) as i32..=(cx + r) as i32 {
            let (dx, dy) = (px as f32 - cx, py as f32 - cy);
            if dx * dx + dy * dy > r * r {
                continue;
            }
            // land and cloud from a coarse hash, drifting slowly as it turns
            let cell = ((px + (game.tick / 30) as i32) / 3, py / 2);
            let h = (cell.0.wrapping_mul(73_856_093) ^ cell.1.wrapping_mul(19_349_663)) as u32 % 100;
            let base: Rgb = match h {
                0..=27 => (64, 160, 86),
                28..=36 => (232, 238, 246),
                _ => (36, 104, 196),
            };
            // the night side, to the right
            let light = (1.0 - (dx / r).max(0.0) * 0.8).clamp(0.15, 1.0);
            canvas.pixel(px, py, palette::rgb(palette::lerp((4, 8, 16), base, light)));
        }
    }
}

/// The opening crawl over the drifting starfield: centred lines typing out,
/// a blinking cursor, then the call to fly. If the text is taller than the
/// play area it scrolls up as it types.
pub fn intro(canvas: &mut Canvas, buf: &mut Buffer, play: Rect, game: &Game) {
    for s in &game.stars.stars {
        canvas.dot(s.x as i32, s.y as i32, palette::STAR[s.layer as usize]);
    }
    canvas.blit(buf, play, palette::SPACE);
    if game.intro_card {
        super::chrome::controls(buf, play, "PRESS ENTER");
        return;
    }
    let full: Vec<&str> = intro::text().lines().collect();
    let shown = intro::shown(game.state_t);
    let lines: Vec<&str> = shown.split('\n').collect();
    let width = full.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let x = play.x + play.width.saturating_sub(width) / 2;
    let room = play.height.saturating_sub(4);
    let top = play.y + play.height.saturating_sub(full.len() as u16 + 2) / 2;
    let scroll = (lines.len() as u16).saturating_sub(room);
    let text = Style::default().fg(palette::TEXT).add_modifier(Modifier::BOLD);
    let mut y = top.max(play.y + 1);
    for line in lines.iter().skip(scroll as usize) {
        put(buf, x, y, line, text);
        y += 1;
    }
    let blink = (game.tick / 15).is_multiple_of(2);
    let done = game.state_t >= intro::duration();
    if !done && blink {
        let last = lines.last().map_or(0, |l| l.chars().count()) as u16;
        put(buf, x + last, y - 1, "▌", Style::default().fg(palette::rgb(SIGNAL[1])));
    }
    if done && blink {
        let call = "PRESS ENTER";
        let cx = play.x + play.width.saturating_sub(call.len() as u16) / 2;
        put(buf, cx, (y + 1).min(play.bottom().saturating_sub(1)), call, Style::default().fg(palette::rgb(SIGNAL[1])).add_modifier(Modifier::BOLD));
    }
}

/// The credits: centred lines rolling up from the bottom over the stars,
/// headings bold, coming to rest with the last line above the middle and the
/// call to fly again below it.
pub fn credits(canvas: &mut Canvas, buf: &mut Buffer, play: Rect, game: &Game) {
    for s in &game.stars.stars {
        canvas.dot(s.x as i32, s.y as i32, palette::STAR[s.layer as usize]);
    }
    canvas.blit(buf, play, palette::SPACE);
    const ROLL: f32 = 2.2; // rows a second
    let lines: Vec<&str> = include_str!("../../data/credits.txt").trim_end().lines().collect();
    // they roll up until the last line rests just above the middle, and stay
    let (h, n) = (play.height as i32, lines.len() as i32);
    let rest = h - (h / 2 - 1) + n - 1;
    let offset = ((game.state_t * ROLL) as i32).min(rest);
    let top = play.y as i32 + h - offset;
    for (i, line) in lines.iter().enumerate() {
        let y = top + i as i32;
        if y < play.y as i32 || y >= play.bottom() as i32 {
            continue;
        }
        // headings are the all-capital lines
        let heading = line.chars().any(char::is_alphabetic) && !line.chars().any(char::is_lowercase);
        let style = if heading { Style::default().fg(palette::rgb(SIGNAL[2])).add_modifier(Modifier::BOLD) } else { Style::default().fg(palette::TEXT) };
        let x = play.x + play.width.saturating_sub(line.chars().count() as u16) / 2;
        put(buf, x, y as u16, line, style);
    }
    // once they've come to rest
    if offset == rest && (game.tick / 15).is_multiple_of(2) {
        let call = "enter: fly again";
        let cx = play.x + play.width.saturating_sub(call.len() as u16) / 2;
        put(buf, cx, (play.y + play.height / 2 + 2).min(play.bottom().saturating_sub(1)), call, Style::default().fg(palette::rgb(SIGNAL[1])).add_modifier(Modifier::BOLD));
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) {
    buf.set_string(x, y, s, style);
}

/// A straight Braille line between two points in dots.
fn line(canvas: &mut Canvas, a: (f32, f32), b: (f32, f32), color: ratatui::style::Color) {
    let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) as usize).max(1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        canvas.dot((a.0 + (b.0 - a.0) * t) as i32, (a.1 + (b.1 - a.1) * t) as i32, color);
    }
}
