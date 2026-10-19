//! Draws a frame: chrome around the play area, the canvas inside it.

mod canvas;
mod chrome;
mod cockpit;
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
    panel::powerups(buf, l.powerups, &game.powers.owned(), game.tick);
    // with a pad in, two can play: say who does what
    let note = match (&input.pad_name, input.mode_note()) {
        (Some(_), Some(m)) => Some(format!("PILOT · pad   COMMS · keys  ·  {m}")),
        (Some(_), None) => Some("PILOT · pad   COMMS · keys".to_string()),
        (None, m) => m.map(str::to_string),
    };
    let hud = chrome::Hud {
        shield: game.ship.shield,
        lives: game.ship.lives,
        bombs: game.powers.bombs,
        score: game.score,
        multiplier: game.chain.multiplier(),
        dash: game.ship.dash_cool / crate::game::ship::DASH_COOLDOWN,
        note: note.as_deref(),
    };
    chrome::hud(buf, l.hud, &hud);

    let mut canvas = Canvas::new(play.width, play.height);
    let bg = palette::tunnel(game.sector.number);
    // dots to half-block pixels: 2×2 dots each
    let px = |v: f32| (v / 2.0).round() as i32;
    match game.state {
        State::Intro => journey::intro(&mut canvas, buf, play, game),
        State::Cockpit => {
            if let Some(c) = &game.cockpit {
                cockpit::draw(&mut canvas, buf, play, game, c);
            }
        }
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
            State::Over => summary(buf, play, game, "signal lost"),
            // let Earth rise a while before the words, and keep them off it
            State::Home if game.state_t > 4.0 => summary(buf, Rect { width: play.width * 3 / 5, ..play }, game, "home"),
            _ => {}
        }
    }
    if input.confirm_quit {
        chrome::question(buf, play, "PAUSED  ·  y: quit  ·  any other key: fly on");
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
    walls(canvas, game);
    if game.state == State::Home {
        journey::earth(canvas, game);
    }
    for e in &game.enemies {
        // an interceptor lining up flashes: its tell before the dash
        let flash = e.flash > 0.0 || (e.aiming() && (game.tick / 4).is_multiple_of(2));
        sprites::draw_enemy(canvas, sprites::enemy_art(e.kind), e.side, flash, px(e.x), px(e.y));
        if e.kind == Kind::Jammer {
            jammer_pulse(canvas, e, game.tick);
        }
    }
    for p in &game.sector.portals {
        echoes::portal(canvas, p, &game.sector, game.tick);
    }
    for p in &game.pods {
        sprites::draw_pod(canvas, p.kind, p.hp, p.flash > 0.0 && p.hp < crate::game::powerup::POD_HP, px(p.x), px(p.y), game.tick);
    }
    if let Some(d) = &game.powers.drone {
        sprites::draw_drone(canvas, px(d.x), px(d.y), game.tick);
    }
    // a smart bomb: a ring racing out from the ship
    if game.bomb_t < 0.6 {
        let (cx, cy) = game.ship.center();
        let r = game.bomb_t / 0.6 * game.w;
        for i in 0..360 {
            let a = i as f32 / 360.0 * std::f32::consts::TAU;
            canvas.dot((cx + a.cos() * r) as i32, (cy + a.sin() * r * 0.7) as i32, palette::rgb(sprites::power_color(crate::game::powerup::PowerUp::Bomb)));
        }
    }
    // the ship blinks while invulnerable
    let blink = game.ship.grace > 0.0 && (game.tick / 4).is_multiple_of(2);
    if game.state != State::Over && !blink {
        sprites::draw(canvas, &sprites::SHIP, px(game.ship.x), px(game.ship.y), game.tick);
    }
    // lancer beams: a dotted warning line, then the beam, in the enemy-fire red
    for b in game.enemies.iter().filter(|e| e.hp > 0).filter_map(|e| e.beam()) {
        if b.live {
            for x in 0..px(b.x_end) {
                canvas.pixel(x, px(b.y - 1.0), palette::ENEMY_SHOT);
            }
        } else {
            for x in (0..b.x_end as i32).step_by(3) {
                canvas.dot(x, b.y as i32, palette::BEAM_WARNING);
            }
        }
    }
    // enemy fire last, as half-block pixels in its own colour: nothing may hide it
    for s in game.shots.iter().filter(|s| s.owner != Owner::Player) {
        canvas.pixel(px(s.x - 1.0), px(s.y - 1.0), palette::ENEMY_SHOT);
    }
    canvas.blit(buf, play, bg);
    if game.state != State::Home {
        echoes::draw(buf, play, &game.sector.echoes, game.tick);
        echoes::pod_words(buf, play, &game.pods);
        for p in &game.sector.portals {
            echoes::portal_label(buf, play, p, &game.sector, game.tick);
        }
        fire_on_top(buf, play, bg, game);
    }
}

/// Enemy fire and live beams once more, over any text: words must never hide
/// what can hurt you.
fn fire_on_top(buf: &mut ratatui::buffer::Buffer, play: Rect, bg: ratatui::style::Color, game: &Game) {
    let mut half = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= play.width as i32 || y >= play.height as i32 * 2 {
            return;
        }
        if let Some(cell) = buf.cell_mut((play.x + x as u16, play.y + (y / 2) as u16)) {
            // over text the other half goes back to the tunnel; over pixels it keeps theirs
            let glyph = if y % 2 == 0 { '▀' } else { '▄' };
            if cell.symbol() != "▀" && cell.symbol() != "▄" && cell.symbol() != " " {
                cell.set_bg(bg);
            }
            cell.set_char(glyph).set_fg(palette::ENEMY_SHOT);
        }
    };
    let px = |v: f32| (v / 2.0).round() as i32;
    for s in game.shots.iter().filter(|s| s.owner != Owner::Player) {
        half(px(s.x - 1.0), px(s.y - 1.0));
    }
    for b in game.enemies.iter().filter(|e| e.hp > 0).filter_map(|e| e.beam()).filter(|b| b.live) {
        for x in 0..px(b.x_end) {
            half(x, px(b.y - 1.0));
        }
    }
}

/// The tunnel walls, in half-block pixels: a lit edge and a darker body, in
/// the sector's style.
fn walls(canvas: &mut Canvas, game: &Game) {
    use crate::game::terrain::Style;
    let (body, edge): (palette::Rgb, palette::Rgb) = match game.terrain.style {
        Style::Open => return,
        Style::Rock => ((44, 36, 34), (100, 84, 72)),
        Style::Ice => ((26, 52, 70), (150, 210, 240)),
        Style::Wreck => ((40, 40, 50), (130, 116, 92)),
    };
    let rows = (game.h / 2.0) as i32;
    for x in 0..(game.w / 2.0) as i32 {
        let (top, bottom) = game.terrain.depth(x as f32 * 2.0 + 1.0);
        let (t, b) = ((top / 2.0).ceil() as i32, (bottom / 2.0).ceil() as i32);
        for y in 0..t {
            canvas.pixel(x, y, palette::rgb(if y == t - 1 { edge } else { body }));
        }
        for y in 0..b {
            canvas.pixel(x, rows - 1 - y, palette::rgb(if y == b - 1 { edge } else { body }));
        }
    }
}

/// The end of a run: how it went.
fn summary(buf: &mut ratatui::buffer::Buffer, area: Rect, game: &Game, title: &str) {
    let s = &game.stats;
    let best = if game.record { format!("{:07}  new best", game.best) } else { format!("{:07}", game.best) };
    let time = format!("{}:{:02}", s.seconds as u32 / 60, s.seconds as u32 % 60);
    let lines = [
        ("score".to_string(), format!("{:07}", game.score)),
        ("best".to_string(), best),
        ("sectors".to_string(), format!("{}  ·  {} true, {} detours", s.sectors, s.true_jumps, s.false_jumps)),
        ("kills".to_string(), format!("{}  ·  best chain {}", s.kills, game.chain.best)),
        ("decoded".to_string(), format!("{} echoes  ·  {} pods", s.echoes, s.pods)),
        ("typing".to_string(), format!("{}% of {} keys", s.accuracy(), s.keys)),
        ("time".to_string(), time),
    ];
    chrome::summary(buf, area, title, &lines, "enter: fly again");
}

/// A jammer's interference: Braille rings pulsing out from its dish.
fn jammer_pulse(canvas: &mut Canvas, e: &crate::game::enemy::Enemy, tick: u64) {
    let (cx, cy) = (e.x + 6.0, e.y + 1.0);
    let color = palette::rgb(sprites::side_colors(e.side)[1]);
    for k in 0..2u64 {
        let r = ((tick + k * 30) % 60) as f32 / 60.0 * 18.0 + 3.0;
        for i in 0..48 {
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            canvas.dot((cx + a.cos() * r) as i32, (cy + a.sin() * r * 0.8) as i32, color);
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
    if let Some(c) = game.cockpit.as_ref().filter(|c| c.step != crate::game::cockpit::Step::Command(0)) {
        // the receiver hums once rebooted, and sings once the echo is in
        return match c.step {
            crate::game::cockpit::Step::Command(_) => 0.1,
            _ => c.echo.clarity,
        };
    }
    let drifting = game.sector.echoes.iter().filter(|e| e.state == EchoState::Drifting && e.x < game.w);
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
            let dy = if (t / 60) % 2 == 0 { 1.0 } else { -1.0 };
            game.update(&Controls { dx: 0.0, dy, fire: true });
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
        assert!(text.contains("HOME") && text.contains("enter: fly again"));
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
        assert_eq!(game.state, State::Cockpit, "the next one goes to the cockpit");
    }

    #[test]
    fn the_cockpit_shows_the_console_and_then_the_echo() {
        let mut game = Game::new(6);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.start_intro();
        game.intro_key();
        game.intro_key();
        for c in "reb".chars() {
            game.type_letter(c);
        }
        let first = frame(&game, 100, 30);
        assert!(first.contains("receiver offline") && first.contains("> reboot"), "the prompt shows what to type");
        for c in "oottunescan".chars() {
            game.type_letter(c);
        }
        let echo = frame(&game, 100, 30);
        assert!(echo.contains("tuned to 1420 MHz") && echo.contains("type what you read"));
        assert!(echo.contains("RX "), "the receiver has the echo too");
    }

    #[test]
    fn a_jammed_echo_rules_out_its_gate() {
        let mut game = Game::new(9);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        for _ in 0..60 * 11 {
            game.update(&Controls::default());
            game.ship.grace = 1.0;
        }
        game.sector.echoes[0].truth = false;
        let letters: String = game.sector.echoes[0].letters().collect();
        for c in letters.chars() {
            game.type_letter(c);
        }
        for _ in 0..60 * 50 {
            game.update(&Controls::default());
            game.ship.grace = 1.0;
        }
        let text = frame(&game, 100, 30);
        assert!(text.contains("✕ jammed"), "the gate and the receiver say so");
        assert_eq!(text.matches("???").count(), 2, "the other two gates are blind");
    }

    #[test]
    fn every_kind_of_ship_draws() {
        use crate::game::enemy::{Enemy, Kind, Side};
        use crate::game::rng::Rng;
        let mut game = Game::new(3);
        let play = play_area(Rect::new(0, 0, 120, 34));
        game.resize(play.width, play.height);
        let kinds = [Kind::Scout, Kind::Gunship, Kind::Interceptor, Kind::Mine, Kind::Lancer, Kind::Jammer, Kind::Carrier];
        for (i, k) in kinds.into_iter().enumerate() {
            let side = if i % 2 == 0 { Side::Amber } else { Side::Jade };
            game.enemies.push(Enemy::of(k, side, 40.0 + i as f32 * 26.0, 6.0 + (i % 3) as f32 * 30.0, 0.0, &mut Rng::new(1)));
        }
        let text = frame(&game, 120, 34);
        assert!(text.matches('▀').count() + text.matches('▄').count() > 60);
    }

    #[test]
    fn enemy_fire_shows_through_echo_text() {
        use crate::game::shot::Shot;
        let mut game = Game::new(4);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        for _ in 0..60 * 13 {
            game.update(&Controls::default());
            game.ship.grace = 1.0;
        }
        game.shots.clear();
        let e = &game.sector.echoes[0];
        // a bullet right on the echo's third letter
        let (x, y) = (e.x + 5.0, e.y + 1.0);
        game.shots.push(Shot::new(x, y, 0.0, 0.0, Owner::Enemy(crate::game::enemy::Side::Amber)));
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let input = Input::with_mode(Mode::Timing);
        term.draw(|f| draw(f, &game, &input)).unwrap();
        let buf = term.backend().buffer();
        let cell = &buf[(play.x + ((x - 1.0) / 2.0).round() as u16, play.y + (((y - 1.0) / 2.0).round() as u16) / 2)];
        assert_eq!(cell.fg, palette::ENEMY_SHOT, "the bullet, not the letter");
    }

    #[test]
    fn later_tunnels_have_walls() {
        let mut game = Game::new(3);
        let play = play_area(Rect::new(0, 0, 100, 30));
        game.resize(play.width, play.height);
        game.start_at(2);
        for _ in 0..60 * 4 {
            game.update(&Controls::default());
        }
        let text = frame(&game, 100, 30);
        let top_row: String = text.lines().nth(2).unwrap().chars().collect();
        assert!(top_row.contains('▀') || top_row.contains('▄') || top_row.contains(' '), "{top_row}");
        assert_ne!(game.terrain.style, crate::game::terrain::Style::Open);
    }

    /// Not a check: how long a busy frame takes to draw on a big terminal.
    /// `cargo test --release frame_cost -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn frame_cost() {
        let mut game = Game::new(3);
        let play = play_area(Rect::new(0, 0, 200, 60));
        game.resize(play.width, play.height);
        game.start_at(4);
        for t in 0..60 * 40 {
            let dy = if (t / 60) % 2 == 0 { 1.0 } else { -1.0 };
            game.update(&Controls { dx: 0.0, dy, fire: true });
            game.ship.grace = 1.0;
        }
        let mut term = Terminal::new(TestBackend::new(200, 60)).unwrap();
        let input = Input::with_mode(Mode::Timing);
        let n = 300;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            game.update(&Controls { dx: 0.0, dy: 0.0, fire: true });
            term.draw(|f| draw(f, &game, &input)).unwrap();
        }
        let per = t0.elapsed() / n;
        println!("{} enemies, {} shots: {per:?} per update+frame", game.enemies.len(), game.shots.len());
    }

    /// Every state draws at every size without panicking, from too small to
    /// play to very large.
    #[test]
    fn every_state_draws_at_every_size() {
        let sizes = [(10, 4), (30, 8), (41, 14), (60, 16), (80, 24), (101, 31), (240, 70)];
        for (w, h) in sizes {
            let mut game = Game::new(w as u64 * 31 + h as u64);
            let play = play_area(Rect::new(0, 0, w, h));
            game.resize(play.width, play.height);
            game.start_intro();
            frame(&game, w, h);
            game.intro_key();
            frame(&game, w, h);
            game.intro_key();
            for c in "rebootab".chars() {
                game.type_letter(c);
            }
            frame(&game, w, h);
            game.skip_cockpit();
            for _ in 0..60 {
                game.update(&Controls::default());
            }
            frame(&game, w, h);
            game.start_at(4);
            for t in 0..60 * 62 {
                game.ship.grace = 1.0;
                game.update(&Controls { dx: 0.0, dy: if (t / 50) % 2 == 0 { 1.0 } else { -1.0 }, fire: true });
                if t % 600 == 0 {
                    frame(&game, w, h);
                }
            }
            frame(&game, w, h);
            for state in [State::Turn, State::Map, State::Home, State::Over] {
                game.state = state;
                game.state_t = 5.0;
                frame(&game, w, h);
            }
        }
    }
}
