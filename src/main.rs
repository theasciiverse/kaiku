mod game;
mod highscore;
mod input;
mod pad;
mod render;

use game::Game;
use input::Input;
use ratatui::crossterm::event::{self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;
use ratatui::DefaultTerminal;
use std::io;
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_micros(16_667);

fn main() -> io::Result<()> {
    // Session logging: RUST_LOG=debug kaiku 2>kaiku.log
    env_logger::init();
    let mut terminal = ratatui::init();
    // probed once, before any input is read: the query answers on stdin
    let kitty = supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        // typing is plain a–z, so reporting every key as an escape code costs nothing
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
        execute!(io::stdout(), PushKeyboardEnhancementFlags(flags))?;
        // ratatui's panic hook restores the screen; pop the flags before it runs
        let restore = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
            restore(info);
        }));
    }
    log::info!("start kitty_keyboard={kitty}");
    let result = run(&mut terminal, kitty);
    if kitty {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
    }
    ratatui::restore();
    result
}

/// Command line: `--sector N` starts straight in sector N (no crawl, no
/// cockpit), for trying later sectors; `--skip-intro` starts in sector 1.
fn start(game: &mut Game) {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name);
    if let Some(i) = flag("--sector") {
        let n = args.get(i + 1).and_then(|n| n.parse().ok()).unwrap_or(1);
        game.start_at(n);
    } else if flag("--skip-intro").is_some() {
        game.start_at(1);
    } else {
        game.start_intro();
    }
}

fn run(terminal: &mut DefaultTerminal, kitty: bool) -> io::Result<()> {
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
    let mut game = Game::new(seed);
    game.best = highscore::load();
    start(&mut game);
    let mut pad = pad::Pad::open();
    // the best score is saved once per run, when the run ends on a record
    let mut saved = false;
    let mut input = Input::new(kitty);
    let mut next = Instant::now() + TICK;
    while !input.quit {
        let play = render::play_area(terminal.size()?.into());
        game.resize(play.width, play.height);
        // the pad: Start pauses (and resumes); B or a bumper is the smart bomb
        // in flight; A confirms everywhere else (the map, flying again, the crawl)
        pad.poll();
        input.pad_name = pad.name.clone();
        let presses = pad.take_presses();
        if presses.pause {
            input.confirm_quit = !input.confirm_quit;
        }
        match game.state {
            game::State::Flight => input.bomb |= presses.bomb,
            game::State::Cockpit => {}
            _ => input.bomb |= presses.confirm,
        }
        terminal.draw(|f| render::draw(f, &game, &input))?;
        if event::poll(next.saturating_duration_since(Instant::now()))? {
            loop {
                if let Event::Key(k) = event::read()? {
                    log::debug!("key {:?}", k);
                    input.on_key(k, Instant::now());
                }
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        let now = Instant::now();
        let mut steps = 0;
        while now >= next && steps < 5 {
            if !input.confirm_quit {
                if game.state == game::State::Intro && (input.bomb || !input.typed.is_empty()) {
                    game.intro_key();
                    input.typed.clear();
                    input.bomb = false;
                }
                // Enter on the game-over screen flies again
                if input.bomb {
                    match game.state {
                        game::State::Over | game::State::Home => game.restart(),
                        game::State::Map => game.fly_on(),
                        game::State::Cockpit => game.skip_cockpit(),
                        game::State::Flight => game.bomb(),
                        _ => {}
                    }
                }
                for c in input.typed.drain(..) {
                    game.type_letter(c);
                }
                if std::mem::take(&mut input.drop_lock) {
                    game.drop_lock();
                }
                game.update(&input::Input::with_pad(input.controls(now), &pad.state));
            }
            input.typed.clear();
            input.bomb = false;
            next += TICK;
            steps += 1;
        }
        if steps == 5 {
            next = now + TICK;
        }
        if game.finished() && game.record && !saved {
            highscore::save(game.best);
            saved = true;
        }
        if !game.finished() {
            saved = false;
        }
    }
    Ok(())
}
