//! Draws a frame: chrome around the play area, the canvas inside it.

mod canvas;
mod chrome;
mod palette;
mod sprites;

use crate::game::Game;
use crate::input::Input;
use canvas::Canvas;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

/// Smallest play area worth playing in, in cells.
const MIN_COLS: u16 = 40;
const MIN_ROWS: u16 = 12;

/// The play area for a terminal of this size, in cells.
pub fn play_area(size: Rect) -> Rect {
    let (_, frame, _) = chrome::layout(size);
    chrome::inner(frame)
}

pub fn draw(f: &mut Frame, game: &Game, input: &Input) {
    let area = f.area();
    let buf = f.buffer_mut();
    let (header, frame, hud) = chrome::layout(area);
    let play = chrome::inner(frame);
    if play.width < MIN_COLS || play.height < MIN_ROWS {
        buf.set_string(area.x, area.y, "Kaiku needs a bigger terminal", Style::default().fg(palette::TEXT));
        return;
    }
    chrome::header(buf, header, game.tick);
    chrome::border(buf, frame);
    chrome::hud(buf, hud, game.ship.shield, game.ship.lives, input.mode_note());

    let mut canvas = Canvas::new(play.width, play.height);
    for s in &game.stars.stars {
        canvas.dot(s.x as i32, s.y as i32, palette::STAR[s.layer as usize]);
    }
    for s in &game.shots {
        for i in 0..3 {
            canvas.dot(s.x as i32 - i, s.y as i32, palette::SHOT);
        }
    }
    // dots to half-block pixels: 2×2 dots each
    let (sx, sy) = ((game.ship.x / 2.0).round() as i32, (game.ship.y / 2.0).round() as i32);
    sprites::draw(&mut canvas, &sprites::SHIP, sx, sy, game.tick);
    canvas.blit(buf, play, palette::SPACE);

    if input.confirm_quit {
        chrome::question(buf, play, "Quit Kaiku?  y / n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Controls, Mode};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn a_frame_draws_chrome_ship_and_stars() {
        let mut game = Game::new(3);
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let play = play_area(Rect::new(0, 0, 80, 24));
        game.resize(play.width, play.height);
        for _ in 0..40 {
            game.update(&Controls { dx: 1, dy: 0, fire: true });
        }
        let input = Input::with_mode(Mode::Timing);
        term.draw(|f| draw(f, &game, &input)).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..24).map(|y| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n").collect();
        if std::env::var("KAIKU_PRINT").is_ok() {
            println!("{text}");
        }
        assert!(text.contains("K A I K U"));
        assert!(text.contains('╭') && text.contains("SHIELD"));
        assert!(text.contains('▀') || text.contains('▄'), "the ship is half-blocks");
        assert!(text.chars().any(|c| ('\u{2801}'..='\u{28FF}').contains(&c)), "stars and shots are Braille");
    }
}
