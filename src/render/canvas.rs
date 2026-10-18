//! One pixel canvas over the play area that mixes two resolutions:
//! Braille dots (2×4 per cell) for fine detail, and half-block pixels (1×2 per
//! cell, i.e. 2×2 dots) for chunky sprites. A cell shows half-blocks if any of
//! its half-block pixels is set, otherwise its Braille dots. Text is drawn on top
//! of the canvas, on whole cells, by the caller.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Braille bit for the dot at (column 0..2, row 0..4) of a cell.
const BRAILLE_BIT: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

pub struct Canvas {
    cols: u16,
    rows: u16,
    dots: Vec<u8>,
    dot_color: Vec<Color>,
    /// half-block pixels per cell: [top, bottom]
    px: Vec<[Option<Color>; 2]>,
}

impl Canvas {
    pub fn new(cols: u16, rows: u16) -> Canvas {
        let n = cols as usize * rows as usize;
        Canvas { cols, rows, dots: vec![0; n], dot_color: vec![Color::Reset; n], px: vec![[None; 2]; n] }
    }

    fn cell(&self, cx: i32, cy: i32) -> Option<usize> {
        (cx >= 0 && cy >= 0 && cx < self.cols as i32 && cy < self.rows as i32).then(|| cy as usize * self.cols as usize + cx as usize)
    }

    /// Sets one Braille dot, in dot coordinates.
    pub fn dot(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }
        if let Some(i) = self.cell(x / 2, y / 4) {
            self.dots[i] |= BRAILLE_BIT[(x % 2) as usize][(y % 4) as usize];
            self.dot_color[i] = color;
        }
    }

    /// Sets one half-block pixel, in half-block coordinates (x = cell column,
    /// y = half rows).
    pub fn pixel(&mut self, x: i32, y: i32, color: Color) {
        if y < 0 {
            return;
        }
        if let Some(i) = self.cell(x, y / 2) {
            self.px[i][(y % 2) as usize] = Some(color);
        }
    }

    /// Writes the canvas into the buffer at `area` (same size as the canvas),
    /// over the background `bg`.
    pub fn blit(&self, buf: &mut Buffer, area: Rect, bg: Color) {
        for cy in 0..self.rows.min(area.height) {
            for cx in 0..self.cols.min(area.width) {
                let i = cy as usize * self.cols as usize + cx as usize;
                let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) else { continue };
                match self.px[i] {
                    [Some(t), Some(b)] => {
                        cell.set_char('▀').set_fg(t).set_bg(b);
                    }
                    [Some(t), None] => {
                        cell.set_char('▀').set_fg(t).set_bg(bg);
                    }
                    [None, Some(b)] => {
                        cell.set_char('▄').set_fg(b).set_bg(bg);
                    }
                    [None, None] if self.dots[i] != 0 => {
                        let ch = char::from_u32(0x2800 + self.dots[i] as u32).unwrap_or(' ');
                        cell.set_char(ch).set_fg(self.dot_color[i]).set_bg(bg);
                    }
                    _ => {
                        cell.set_char(' ').set_bg(bg);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_and_half_blocks_share_one_grid() {
        let mut c = Canvas::new(2, 1);
        c.dot(0, 0, Color::White);
        c.dot(1, 3, Color::White);
        c.pixel(1, 1, Color::Red);
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        c.blit(&mut buf, area, Color::Black);
        assert_eq!(buf[(0, 0)].symbol(), "⢁");
        assert_eq!(buf[(1, 0)].symbol(), "▄", "a half-block pixel wins its cell");
    }
}
