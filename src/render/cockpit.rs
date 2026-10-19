//! The cockpit scene: the window onto the stars, framed by struts; the
//! dashboard with its status lights and the console where repairs are typed;
//! the first echo, garbled, in the window.

use super::canvas::Canvas;
use super::echoes;
use super::palette::{self, Rgb, DIM, SIGNAL, TRACE};
use super::sprites;
use crate::game::enemy::Side;
use crate::game::cockpit::{Cockpit, Step, COMMANDS};
use crate::game::Game;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const DASH: Rgb = (20, 25, 38);
const DASH_EDGE: Rgb = (64, 76, 100);
const STRUT: Color = Color::Rgb(52, 62, 84);
const RING: Color = Color::Rgb(200, 186, 168);
const MISS_BG: Color = Color::Rgb(70, 22, 40);

/// The war, far off: tiny ships of both fleets crossing the window at their
/// own speeds, the two sides heading opposite ways, firing the odd tracer,
/// with faint flashes in the distance. Drawn before the planet, whose pixels
/// hide whatever passes behind it. A pure function of the clock.
fn distant_war(canvas: &mut Canvas, w: i32, dash_top: i32, tick: u64) {
    let (dw, dh) = ((w * 2) as f32, (dash_top * 4) as f32);
    let t = tick as f32 / 60.0;
    let far = |c: Rgb, k: f32| palette::rgb(palette::lerp((10, 14, 26), c, k));
    for i in 0..7u32 {
        let side = if i % 2 == 0 { Side::Amber } else { Side::Jade };
        let dir = if side == Side::Amber { 1.0 } else { -1.0 };
        let speed = 3.0 + (i * 7 % 5) as f32 * 1.6;
        let y = (dh * (0.12 + 0.72 * ((i as f32 * 0.618).fract()))).round();
        let span = dw + 40.0;
        let run = (i as f32 * 53.0 + speed * t) % span - 20.0;
        let x = if dir > 0.0 { run } else { dw - run };
        let [hull, high, engine] = sprites::side_colors(side);
        // a few dots: nose, hull, a fin, and a flicker of engine behind
        let dot = |canvas: &mut Canvas, dx: f32, dy: f32, c: Color| canvas.dot((x + dx * dir) as i32, (y + dy) as i32, c);
        dot(canvas, 0.0, 0.0, far(high, 0.75));
        dot(canvas, -1.0, 0.0, far(hull, 0.7));
        dot(canvas, -2.0, 0.0, far(hull, 0.7));
        dot(canvas, -2.0, -1.0, far(hull, 0.55));
        if (tick / 5 + i as u64).is_multiple_of(2) {
            dot(canvas, -3.0, 0.0, far(engine, 0.6));
        }
        // now and then a tracer, racing ahead
        let phase = (t * 0.35 + i as f32 * 0.29).fract();
        if phase < 0.25 {
            let tx = 3.0 + phase * 160.0;
            dot(canvas, tx, 0.0, far((255, 54, 64), 0.55));
        }
    }
    // far-off flashes: a ring of dots swelling and fading
    for k in 0..3u32 {
        let cycle = t / 3.5 + k as f32 * 0.37;
        let (n, phase) = (cycle.floor(), cycle.fract());
        if phase > 0.3 {
            continue;
        }
        let hash = |m: f32| ((n * 12.9898 + k as f32 * 78.233 + m).sin() * 43758.547).fract().abs();
        let (fx, fy) = (hash(1.0) * dw, hash(2.0) * dh * 0.9);
        let r = phase / 0.3 * 5.0 + 1.0;
        let c = far((255, 200, 120), 0.6 * (1.0 - phase / 0.3));
        for j in 0..12 {
            let a = j as f32 / 12.0 * std::f32::consts::TAU;
            canvas.dot((fx + a.cos() * r) as i32, (fy + a.sin() * r * 0.8) as i32, c);
        }
    }
}

/// A banded gas giant with a tilted ring, out of the window, lit from the
/// left. Half-block pixels for the body, Braille dots for the ring. `w` and
/// `dash_top` are in cells.
fn planet(canvas: &mut Canvas, w: i32, dash_top: i32, tick: u64) {
    let (ph, pw) = (dash_top as f32 * 2.0, w as f32);
    let r = (ph * 0.36).max(4.0);
    let (cx, cy) = (pw * 0.68, ph * 0.46);
    // the ring's far side: behind the planet
    ring(canvas, cx, cy, r, true);
    let turn = tick as f32 / 400.0;
    for py in (cy - r) as i32..=(cy + r) as i32 {
        for px in (cx - r) as i32..=(cx + r) as i32 {
            let (dx, dy) = (px as f32 - cx, py as f32 - cy);
            if dx * dx + dy * dy > r * r || py >= dash_top * 2 {
                continue;
            }
            // bands across the latitude, gently wavy as it turns
            let lat = dy / r;
            let band = ((lat * 9.0 + (dx / r * 2.0 + turn).sin() * 0.3).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let base = palette::lerp((150, 96, 120), (214, 170, 140), band);
            // lit from the upper left, the night side falls to the dark
            let light = (0.95 - (dx / r) * 0.55 - (dy / r) * 0.25).clamp(0.08, 1.0);
            canvas.pixel(px, py, palette::rgb(palette::lerp((8, 8, 18), base, light)));
        }
    }
    // the near side of the ring: in front of it
    ring(canvas, cx, cy, r, false);
}

/// Half of the ring: the far (upper) half or the near (lower) one, in dots.
fn ring(canvas: &mut Canvas, cx: f32, cy: f32, r: f32, far: bool) {
    // in dots: a half-block pixel is 2×2 dots
    let (dcx, dcy, dr) = (cx * 2.0 + 1.0, cy * 2.0 + 1.0, r * 2.0);
    for i in 0..360 {
        let a = i as f32 / 360.0 * std::f32::consts::TAU;
        let (x, y) = (a.cos() * dr * 1.7, a.sin() * dr * 0.35);
        // tilt the ring
        let (tx, ty) = (x * 0.97 - y * 0.24, x * 0.24 + y * 0.97);
        let behind = a.sin() < 0.0;
        if behind != far {
            continue;
        }
        let (pdx, pdy) = (tx / 2.0, ty / 2.0);
        let over_disc = pdx * pdx + pdy * pdy < r * r;
        match (far, over_disc) {
            // behind the planet's disc it is hidden
            (true, true) => {}
            // in front of the disc a dot would vanish under its pixels: draw a pixel
            (false, true) => canvas.pixel((cx + pdx) as i32, (cy + pdy) as i32, RING),
            _ => canvas.dot((dcx + tx) as i32, (dcy + ty) as i32, RING),
        }
    }
}

pub fn draw(canvas: &mut Canvas, buf: &mut Buffer, play: Rect, game: &Game, c: &Cockpit) {
    let (w, h) = (play.width as i32, play.height as i32);
    // the dashboard takes the lower third, in cells
    let dash_top = h - (h / 3).max(6);
    // the ship is adrift: the stars hold still and twinkle, and a ringed
    // planet hangs in the window, slowly turning
    for (i, s) in game.stars.stars.iter().enumerate() {
        if (s.y as i32) < dash_top * 4 {
            let twinkle = (i as u64 * 7919 + game.tick / 12).is_multiple_of(23);
            let layer = if twinkle { 2 } else { s.layer as usize };
            canvas.dot(s.x as i32, s.y as i32, palette::STAR[layer]);
        }
    }
    distant_war(canvas, w, dash_top, game.tick);
    planet(canvas, w, dash_top, game.tick);
    // window struts, in dots: from the top corners down to the dashboard
    let (dw, dt) = (w * 2, dash_top * 4);
    for (x0, x1) in [(0, dw / 8), (dw - 1, dw - 1 - dw / 8)] {
        for y in 0..dt {
            let x = x0 + (x1 - x0) * y / dt.max(1);
            canvas.dot(x, y, STRUT);
            canvas.dot(x + if x1 > x0 { 1 } else { -1 }, y, STRUT);
        }
    }
    // the dashboard: a lit edge, then dark panel, in half-block pixels
    for py in dash_top * 2..h * 2 {
        let color = if py == dash_top * 2 { DASH_EDGE } else { DASH };
        for px in 0..w {
            canvas.pixel(px, py, palette::rgb(color));
        }
    }
    // status lights come alive as repairs go in
    let lit = match c.step {
        Step::Command(n) => n,
        Step::Echo => COMMANDS.len(),
        Step::Done => COMMANDS.len() + 1,
    };
    for i in 0..4 {
        let on = i < lit;
        let color: Rgb = match () {
            _ if on => SIGNAL[1 + (i == 3) as usize],
            // dead lights blink red
            _ if (game.tick / 20).is_multiple_of(2) => (200, 40, 50),
            _ => (60, 20, 26),
        };
        canvas.pixel(w - 14 + i as i32 * 3, dash_top * 2 + 3, palette::rgb(color));
    }
    canvas.blit(buf, play, palette::SPACE);

    // the echo, in the window
    if matches!(c.step, Step::Echo | Step::Done) {
        let cells = echoes::cells(&c.echo, game.tick);
        let x = play.x as i32 + (w - cells.len() as i32) / 2;
        echoes::put_cells(buf, x, play.y + (dash_top as u16) / 2, &cells, play);
    }

    // the console: the log, then the prompt with the command to type
    let bg = palette::rgb(DASH);
    let x = play.x + 3;
    let rows = (h - dash_top - 2).max(1) as usize;
    let mut lines: Vec<Vec<(char, Style)>> = c
        .log
        .iter()
        .map(|l| {
            let fg = if l.starts_with('>') { palette::rgb(SIGNAL[1]) } else { DIM };
            l.chars().map(|ch| (ch, Style::default().fg(fg).bg(bg))).collect()
        })
        .collect();
    if let Some((word, done)) = c.command() {
        let cursor_bg = if c.flash > 0.0 { MISS_BG } else { bg };
        let mut prompt: Vec<(char, Style)> = "> ".chars().map(|ch| (ch, Style::default().fg(palette::rgb(SIGNAL[1])).bg(bg))).collect();
        for (i, ch) in word.chars().enumerate() {
            let style = match i {
                _ if i < done => Style::default().fg(palette::rgb(SIGNAL[2])).add_modifier(Modifier::BOLD).bg(bg),
                _ if i == done => Style::default().fg(palette::TEXT).bg(cursor_bg).add_modifier(Modifier::UNDERLINED),
                _ => Style::default().fg(TRACE).bg(bg),
            };
            prompt.push((ch, style));
        }
        lines.push(prompt);
    }
    let skip = lines.len().saturating_sub(rows);
    for (i, line) in lines.iter().skip(skip).enumerate() {
        echoes::put_cells(buf, x as i32, play.y + dash_top as u16 + 1 + i as u16, line, play);
    }
    let hint: Vec<(char, Style)> = "enter or pad: skip".chars().map(|ch| (ch, Style::default().fg(TRACE).bg(bg))).collect();
    echoes::put_cells(buf, play.right() as i32 - hint.len() as i32 - 2, play.bottom() - 1, &hint, play);
}
