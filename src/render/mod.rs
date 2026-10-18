//! Draws a frame: chrome around the play area, the canvas inside it.

mod canvas;
mod chrome;
mod palette;
mod panel;
mod sprites;

use crate::game::enemy::Kind;
use crate::game::fx::Tint;
use crate::game::shot::Owner;
use crate::game::{Game, State};
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
    chrome::inner(chrome::layout(size).frame)
}

pub fn draw(f: &mut Frame, game: &Game, input: &Input) {
    let area = f.area();
    let buf = f.buffer_mut();
    let l = chrome::layout(area);
    let play = chrome::inner(l.frame);
    if play.width < MIN_COLS || play.height < MIN_ROWS {
        buf.set_string(area.x, area.y, "Kaiku needs a bigger terminal", Style::default().fg(palette::TEXT));
        return;
    }
    // no receiver signal yet: echoes arrive in M2
    chrome::header(buf, l.header, game.tick, 0.0);
    chrome::border(buf, l.frame);
    panel::receiver(buf, l.receiver, game.tick);
    panel::powerups(buf, l.powerups, &[], game.tick);
    chrome::hud(buf, l.hud, game.ship.shield, game.ship.lives, game.score, input.mode_note());

    let mut canvas = Canvas::new(play.width, play.height);
    for s in &game.stars.stars {
        canvas.dot(s.x as i32, s.y as i32, palette::STAR[s.layer as usize]);
    }
    for p in &game.particles {
        canvas.dot(p.x as i32, p.y as i32, particle_color(p.tint, p.life / p.max));
    }
    for s in game.shots.iter().filter(|s| s.owner == Owner::Player) {
        for i in 0..3 {
            canvas.dot(s.x as i32 - i, s.y as i32, palette::SHOT);
        }
    }
    // dots to half-block pixels: 2×2 dots each
    let px = |v: f32| (v / 2.0).round() as i32;
    for e in &game.enemies {
        let art: &[&str] = match e.kind {
            Kind::Scout => &sprites::SCOUT,
            Kind::Gunship => &sprites::GUNSHIP,
        };
        sprites::draw_enemy(&mut canvas, art, e.side, e.flash > 0.0, px(e.x), px(e.y));
    }
    // the ship blinks while invulnerable
    let blink = game.ship.grace > 0.0 && (game.tick / 4).is_multiple_of(2);
    if game.state == State::Flight && !blink {
        sprites::draw(&mut canvas, &sprites::SHIP, px(game.ship.x), px(game.ship.y), game.tick);
    }
    // enemy fire last, as half-block pixels in its own colour: nothing may hide it
    for s in game.shots.iter().filter(|s| s.owner != Owner::Player) {
        canvas.pixel(px(s.x - 1.0), px(s.y - 1.0), palette::ENEMY_SHOT);
    }
    canvas.blit(buf, play, palette::SPACE);

    if game.state == State::Over && !input.confirm_quit {
        chrome::question(buf, play, &format!("SIGNAL LOST  ·  score {}  ·  enter to fly again", game.score));
    }
    if input.confirm_quit {
        chrome::question(buf, play, "Quit Kaiku?  y / n");
    }
}

fn particle_color(tint: Tint, life: f32) -> ratatui::style::Color {
    let hot = match tint {
        Tint::Side(side) => sprites::side_colors(side)[1],
        Tint::Player => (140, 220, 255),
        Tint::Spark => (255, 236, 160),
    };
    // fade into the dark as the particle dies
    palette::rgb(palette::lerp((40, 30, 60), hot, life))
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
        // twelve seconds of flying up and down, firing, so the war has arrived
        for t in 0..720 {
            let dy = if (t / 60) % 2 == 0 { 1 } else { -1 };
            game.update(&Controls { dx: 0, dy, fire: true });
        }
        let input = Input::with_mode(Mode::Timing);
        term.draw(|f| draw(f, &game, &input)).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..24).map(|y| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n").collect();
        if std::env::var("KAIKU_PRINT").is_ok() {
            println!("{text}");
        }
        assert!(text.contains("KAIKU kaiku kaiku kaiku"), "the name and its echoes");
        assert!(text.contains('╭') && text.contains("SHIELD"));
        assert!(text.contains("RECEIVER") && text.contains("POWER-UPS"), "the bottom panel");
        assert!(text.contains('▀') || text.contains('▄'), "the ship is half-blocks");
        assert!(text.chars().any(|c| ('\u{2801}'..='\u{28FF}').contains(&c)), "stars and shots are Braille");
    }
}
