//! The cockpit scene: the window onto the stars, framed by struts; the
//! dashboard with its status lights and the console where repairs are typed;
//! the first echo, garbled, in the window.

use super::canvas::Canvas;
use super::echoes;
use super::palette::{self, Rgb, DIM, SIGNAL, TRACE};
use crate::game::cockpit::{Cockpit, Step, COMMANDS};
use crate::game::Game;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const DASH: Rgb = (20, 25, 38);
const DASH_EDGE: Rgb = (64, 76, 100);
const STRUT: Color = Color::Rgb(52, 62, 84);
const MISS_BG: Color = Color::Rgb(70, 22, 40);

pub fn draw(canvas: &mut Canvas, buf: &mut Buffer, play: Rect, game: &Game, c: &Cockpit) {
    let (w, h) = (play.width as i32, play.height as i32);
    // the dashboard takes the lower third, in cells
    let dash_top = h - (h / 3).max(6);
    for s in &game.stars.stars {
        if (s.y as i32) < dash_top * 4 {
            canvas.dot(s.x as i32, s.y as i32, palette::STAR[s.layer as usize]);
        }
    }
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
    let hint: Vec<(char, Style)> = "enter: skip".chars().map(|ch| (ch, Style::default().fg(TRACE).bg(bg))).collect();
    echoes::put_cells(buf, play.right() as i32 - 13, play.bottom() - 1, &hint, play);
}
