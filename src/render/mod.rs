//! Draws a frame: chrome around the play area, the canvas inside it.

mod canvas;
mod chrome;
mod echoes;
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
    chrome::header(buf, l.header, game.tick, signal(game));
    chrome::border(buf, l.frame);
    panel::receiver(buf, l.receiver, game);
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
    for p in &game.sector.portals {
        echoes::portal(&mut canvas, p, game.tick);
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
    echoes::draw(buf, play, &game.sector.echoes, game.tick);
    for p in &game.sector.portals {
        echoes::portal_label(buf, play, p, &game.sector.echoes);
    }

    if !input.confirm_quit {
        match game.state {
            State::Over => chrome::question(buf, play, &format!("SIGNAL LOST  ·  score {}  ·  enter to fly again", game.score)),
            State::Home => chrome::question(buf, play, &format!("HOME  ·  score {}  ·  enter to fly again", game.score)),
            State::Flight => {}
        }
    }
    if input.confirm_quit {
        chrome::question(buf, play, "Quit Kaiku?  y / n");
    }
}

/// What the receiver hears, 0..1, for the header's waveform: the locked
/// echo's clarity, or a murmur while echoes are in range.
fn signal(game: &Game) -> f32 {
    use crate::game::echo::EchoState;
    let drifting = game.sector.echoes.iter().filter(|e| e.state == EchoState::Drifting);
    match game.typing.lock.and_then(|id| game.sector.echo(id)) {
        Some(e) => e.clarity,
        None => drifting.map(|e| e.clarity * 0.4).fold(0.0, f32::max),
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

    #[test]
    fn an_echo_being_decoded_shows_in_the_field_and_the_receiver() {
        let mut game = Game::new(4);
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        for _ in 0..60 * 13 {
            game.update(&Controls::default());
        }
        let letters: String = game.sector.echoes[0].letters().take(6).collect();
        for c in letters.chars() {
            game.type_letter(c);
        }
        game.update(&Controls::default());
        let input = Input::with_mode(Mode::Timing);
        term.draw(|f| draw(f, &game, &input)).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..30).map(|y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n").collect();
        if std::env::var("KAIKU_PRINT").is_ok() {
            println!("{text}");
        }
        assert!(text.contains('▸'), "the locked echo is marked");
        assert!(text.contains(&format!("RX {}", &game.sector.echoes[0].text[..3])), "the receiver shows it being decoded");
    }
}
