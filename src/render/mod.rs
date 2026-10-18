//! Draws a frame: chrome around the play area, the canvas inside it.

mod canvas;
mod chrome;
mod echoes;
mod journey;
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
    let bg = palette::tunnel(game.sector.number);
    // dots to half-block pixels: 2×2 dots each
    let px = |v: f32| (v / 2.0).round() as i32;
    match game.state {
        State::Intro => journey::intro(&mut canvas, buf, play, game),
        State::Map => journey::map(&mut canvas, buf, play, game),
        State::Turn => {
            journey::turn(&mut canvas, game);
            sprites::draw(&mut canvas, &sprites::SHIP, px(game.ship.x), px(game.ship.y), game.tick);
            canvas.blit(buf, play, bg);
        }
        State::Flight | State::Over | State::Home => field(&mut canvas, buf, play, bg, game),
    }

    if !input.confirm_quit {
        match game.state {
            State::Over => chrome::question(buf, play, &format!("SIGNAL LOST  ·  score {}  ·  enter to fly again", game.score)),
            // let Earth rise a while before the words
            State::Home if game.state_t > 4.0 => {
                // left of Earth, not over it
                let left = Rect { width: play.width * 3 / 5, ..play };
                chrome::question(buf, left, &format!("HOME  ·  score {}  ·  enter to fly again", game.score))
            }
            _ => {}
        }
    }
    if input.confirm_quit {
        chrome::question(buf, play, "Quit Kaiku?  y / n");
    }
}

/// The tunnel: stars, war, echoes, portals, the ship; Earth when home.
fn field(canvas: &mut Canvas, buf: &mut ratatui::buffer::Buffer, play: Rect, bg: ratatui::style::Color, game: &Game) {
    let px = |v: f32| (v / 2.0).round() as i32;
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
    if game.state == State::Home {
        journey::earth(canvas, game);
    }
    for e in &game.enemies {
        let art: &[&str] = match e.kind {
            Kind::Scout => &sprites::SCOUT,
            Kind::Gunship => &sprites::GUNSHIP,
        };
        sprites::draw_enemy(canvas, art, e.side, e.flash > 0.0, px(e.x), px(e.y));
    }
    for p in &game.sector.portals {
        echoes::portal(canvas, p, game.tick);
    }
    // the ship blinks while invulnerable
    let blink = game.ship.grace > 0.0 && (game.tick / 4).is_multiple_of(2);
    if game.state != State::Over && !blink {
        sprites::draw(canvas, &sprites::SHIP, px(game.ship.x), px(game.ship.y), game.tick);
    }
    // enemy fire last, as half-block pixels in its own colour: nothing may hide it
    for s in game.shots.iter().filter(|s| s.owner != Owner::Player) {
        canvas.pixel(px(s.x - 1.0), px(s.y - 1.0), palette::ENEMY_SHOT);
    }
    canvas.blit(buf, play, bg);
    if game.state != State::Home {
        echoes::draw(buf, play, &game.sector.echoes, game.tick);
        for p in &game.sector.portals {
            echoes::portal_label(buf, play, p, &game.sector.echoes);
        }
    }
}

/// What the receiver hears, 0..1, for the header's waveform: the locked
/// echo's clarity, or a murmur while echoes are in range.
fn signal(game: &Game) -> f32 {
    use crate::game::echo::EchoState;
    if game.state == State::Home {
        return 1.0;
    }
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

    fn frame(game: &Game, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let input = Input::with_mode(Mode::Timing);
        term.draw(|f| draw(f, game, &input)).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n").collect();
        if std::env::var("KAIKU_PRINT").is_ok() {
            println!("{text}");
        }
        text
    }

    #[test]
    fn the_map_shows_the_route_home() {
        let mut game = Game::new(5);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.route.jump(true, &mut crate::game::rng::Rng::new(1));
        game.route.jump(false, &mut crate::game::rng::Rng::new(2));
        game.state = State::Map;
        let text = frame(&game, 100, 30);
        assert!(text.contains("ROUTE HOME") && text.contains("HOME") && text.contains('★'));
        assert!(text.contains(&game.route.here().name));
        assert!(text.contains("6 jumps to home"));
    }

    #[test]
    fn home_shows_earth_and_the_last_echo() {
        let mut game = Game::new(5);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.state = State::Home;
        game.state_t = 5.0;
        let text = frame(&game, 100, 30);
        assert!(text.contains("hello from the children of planet earth"));
        assert!(text.contains("HOME  ·  score"));
    }

    #[test]
    fn the_turn_draws_streaking_stars_and_the_ship() {
        let mut game = Game::new(6);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.state = State::Turn;
        game.state_t = crate::game::TURN_TIME / 2.0;
        let text = frame(&game, 100, 30);
        assert!(text.chars().filter(|c| ('\u{2801}'..='\u{28FF}').contains(c)).count() > 100, "streaks and the ring");
        assert!(text.contains('▀'), "the ship rides through");
    }

    #[test]
    fn the_intro_types_out_its_story() {
        let mut game = Game::new(6);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.start_intro();
        game.state_t = 2.0;
        let early = frame(&game, 100, 30);
        assert!(early.contains("AN INTERPLANETARY WAR") && !early.contains("FOLLOW THEM"));
        game.intro_key();
        let done = frame(&game, 100, 30);
        assert!(done.contains("FOLLOW THEM"), "the first key finishes the text");
        game.intro_key();
        assert_eq!(game.state, State::Flight, "the next one flies");
    }
}
