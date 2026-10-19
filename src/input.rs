//! Keyboard: held flight keys in three tiers, plus typed letters.
//!
//! The held-key tiers are adapted from Entity's `src/input.rs`:
//!
//! 1. **Exact**: the terminal speaks the kitty keyboard protocol (or the platform
//!    reports key-up, as on Windows), so presses and releases are real.
//! 2. **Timing**: any other terminal only sends presses plus OS key repeat. A key
//!    counts as held while its events keep arriving; the repeat delay and interval
//!    are learned from the events themselves. Before the first repeat a tap and a
//!    hold look identical, so that window is capped.
//! 3. **Latched**: a direction stays on until it is pressed again or reversed, and
//!    Space toggles fire. Selected with `KAIKU_INPUT=latch`.
//!
//! Flight is arrows or Ctrl+h/j/k/l plus Space; plain letters a–z are text for the
//! typing layer. In raw mode crossterm decodes Ctrl+H (0x08) and Ctrl+J (0x0A) as
//! Ctrl+letter, since Backspace sends 0x7F, so vim keys work on legacy terminals too.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};
use std::time::{Duration, Instant};

/// Longest a key is assumed held before its first repeat arrives (tap versus hold is
/// unknowable until then). A slow OS repeat delay therefore shows as a short stall.
const PRE_REPEAT_CAP: Duration = Duration::from_millis(220);
const REPEAT_DELAY_DEFAULT: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL_DEFAULT: Duration = Duration::from_millis(33);
const RELEASE_MIN: Duration = Duration::from_millis(60);
const RELEASE_MAX: Duration = Duration::from_millis(160);
/// Timing mode: fire keeps going this long after the last Space event. A terminal
/// repeats only the newest key, so holding Space and then steering stops the Space
/// repeats; the linger bridges that so a quick dodge doesn't silence the gun.
const FIRE_LINGER: Duration = Duration::from_millis(450);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Exact,
    Timing,
    Latched,
}

/// Learned OS key repeat timing, shared by all keys.
#[derive(Clone, Copy, Debug)]
struct Repeat {
    delay: Duration,
    interval: Duration,
}

/// One key as seen through presses only (timing mode).
#[derive(Clone, Copy, Default, Debug)]
struct Hold {
    last: Option<Instant>,
    events: u32,
}

impl Hold {
    /// Feed one press/repeat event; returns true when it is a fresh press.
    fn event(&mut self, now: Instant, rep: &mut Repeat) -> bool {
        if let Some(last) = self.last {
            let gap = now.duration_since(last);
            let tight = gap <= rep.interval * 3;
            let continues = if self.events == 1 {
                // first repeat arrives about one repeat delay after the press
                gap >= rep.delay.mul_f32(0.5) && gap <= rep.delay.mul_f32(1.6)
            } else {
                gap <= RELEASE_MAX.max(rep.interval * 4)
            };
            if continues {
                self.events += 1;
                let blend = |old: Duration, new: Duration| (old.mul_f32(0.5) + new.mul_f32(0.5)).max(Duration::from_millis(10));
                if self.events == 2 {
                    rep.delay = blend(rep.delay, gap.clamp(Duration::from_millis(100), Duration::from_millis(1000)));
                } else if gap < Duration::from_millis(200) {
                    rep.interval = blend(rep.interval, gap);
                }
                self.last = Some(now);
                return !tight;
            }
        }
        self.last = Some(now);
        self.events = 1;
        true
    }

    fn held(&self, now: Instant, rep: &Repeat) -> bool {
        let Some(last) = self.last else { return false };
        let timeout = if self.events >= 2 {
            (rep.interval * 5 / 2).clamp(RELEASE_MIN, RELEASE_MAX)
        } else {
            PRE_REPEAT_CAP.min(rep.delay + rep.interval * 2)
        };
        now.duration_since(last) <= timeout
    }

    fn clear(&mut self) {
        *self = Hold::default();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

impl Dir {
    fn index(self) -> usize {
        self as usize
    }

    fn opposite(self) -> Dir {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
        }
    }
}

/// Which key is driving a direction: the arrow or its Ctrl+vim twin.
const ARROW: usize = 0;
const VIM: usize = 1;

/// What the simulation reads each tick.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Controls {
    /// -1 left .. 1 right: the keys give whole steps, a pad's stick anything between
    pub dx: f32,
    /// -1 up .. 1 down
    pub dy: f32,
    pub fire: bool,
}

pub struct Input {
    mode: Mode,
    rep: Repeat,
    // exact mode: each direction per source (arrow, Ctrl+vim)
    down: [[bool; 2]; 4],
    fire_down: bool,
    // timing mode
    hold: [Hold; 4],
    hold_fire: Hold,
    // latched mode
    latch_x: i8,
    latch_y: i8,
    fire_latched: bool,
    /// Letters typed since the typing layer last drained them, lowercase a–z.
    pub typed: Vec<char>,
    /// Backspace: drop the typing lock.
    pub drop_lock: bool,
    /// Enter: smart bomb.
    pub bomb: bool,
    /// Tab: dash.
    pub dash: bool,
    pub quit: bool,
    /// Esc was pressed: show the quit question and wait for y.
    pub confirm_quit: bool,
    /// the gamepad in use, if one is connected: two can play, pilot and comms
    pub pad_name: Option<String>,
}

impl Input {
    pub fn new(kitty: bool) -> Input {
        let mode = if kitty {
            Mode::Exact
        } else if std::env::var("KAIKU_INPUT").is_ok_and(|v| v == "latch") {
            Mode::Latched
        } else {
            Mode::Timing
        };
        Input::with_mode(mode)
    }

    pub fn with_mode(mode: Mode) -> Input {
        Input {
            mode,
            rep: Repeat { delay: REPEAT_DELAY_DEFAULT, interval: REPEAT_INTERVAL_DEFAULT },
            down: [[false; 2]; 4],
            fire_down: false,
            hold: [Hold::default(); 4],
            hold_fire: Hold::default(),
            latch_x: 0,
            latch_y: 0,
            fire_latched: false,
            typed: Vec::new(),
            drop_lock: false,
            bomb: false,
            dash: false,
            quit: false,
            confirm_quit: false,
            pad_name: None,
        }
    }

    /// Short note for the frame when input is not exact.
    pub fn mode_note(&self) -> Option<&'static str> {
        match self.mode {
            Mode::Exact => None,
            Mode::Timing => Some("keys: timing"),
            Mode::Latched => Some("keys: latched"),
        }
    }

    /// The flight controls with a pad's folded in: whichever asks for more on
    /// an axis wins, and either can fire. With a pad in, two can play: one
    /// flies on the pad while the other types.
    pub fn with_pad(c: Controls, pad: &crate::pad::PadState) -> Controls {
        let pick = |key: f32, stick: f32| if stick.abs() > key.abs() { stick } else { key };
        Controls { dx: pick(c.dx, pad.x), dy: pick(c.dy, pad.y), fire: c.fire || pad.fire }
    }

    pub fn on_key(&mut self, k: KeyEvent, now: Instant) {
        let press = k.kind == KeyEventKind::Press;
        let release = k.kind == KeyEventKind::Release;
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && k.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.confirm_quit {
            // modal: y quits, any other key press goes back to the game
            if press {
                if k.code == KeyCode::Char('y') {
                    self.quit = true;
                }
                self.confirm_quit = false;
            }
            return;
        }
        // letting go of Ctrl ends every Ctrl+vim hold (exact mode only sees this)
        if release && matches!(k.code, KeyCode::Modifier(ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl)) {
            for d in &mut self.down {
                d[VIM] = false;
            }
            return;
        }
        if let Some((dir, source)) = flight_dir(&k, release) {
            self.direction(dir, source, !release, press, now);
            return;
        }
        match k.code {
            KeyCode::Char(' ') => self.fire(!release, press, now),
            KeyCode::Char(c) if press && !ctrl && !k.modifiers.contains(KeyModifiers::ALT) && c.is_ascii_alphabetic() => {
                self.typed.push(c.to_ascii_lowercase())
            }
            KeyCode::Backspace if press => self.drop_lock = true,
            KeyCode::Enter if press => self.bomb = true,
            KeyCode::Tab if press => self.dash = true,
            KeyCode::Esc if press => self.confirm_quit = true,
            _ => {}
        }
    }

    fn direction(&mut self, dir: Dir, source: usize, down: bool, press: bool, now: Instant) {
        match self.mode {
            Mode::Exact => self.down[dir.index()][source] = down,
            Mode::Timing => {
                // a fresh press of one direction ends its opposite (turning around)
                if down && self.hold[dir.index()].event(now, &mut self.rep) {
                    self.hold[dir.opposite().index()].clear();
                }
            }
            Mode::Latched => {
                if press {
                    let (axis, sign) = match dir {
                        Dir::Left => (&mut self.latch_x, -1),
                        Dir::Right => (&mut self.latch_x, 1),
                        Dir::Up => (&mut self.latch_y, -1),
                        Dir::Down => (&mut self.latch_y, 1),
                    };
                    // the same direction again stops that axis
                    *axis = if *axis == sign { 0 } else { sign };
                }
            }
        }
    }

    fn fire(&mut self, down: bool, press: bool, now: Instant) {
        match self.mode {
            Mode::Exact => self.fire_down = down,
            Mode::Timing => {
                if down {
                    self.hold_fire.event(now, &mut self.rep);
                }
            }
            Mode::Latched => {
                if press {
                    self.fire_latched = !self.fire_latched;
                }
            }
        }
    }

    pub fn controls(&self, now: Instant) -> Controls {
        match self.mode {
            Mode::Exact => {
                let on = |d: Dir| self.down[d.index()].iter().any(|&b| b);
                Controls {
                    dx: on(Dir::Right) as i8 as f32 - on(Dir::Left) as i8 as f32,
                    dy: on(Dir::Down) as i8 as f32 - on(Dir::Up) as i8 as f32,
                    fire: self.fire_down,
                }
            }
            Mode::Timing => {
                let axis = |neg: Dir, pos: Dir| -> i8 {
                    let (n, p) = (&self.hold[neg.index()], &self.hold[pos.index()]);
                    match (n.held(now, &self.rep), p.held(now, &self.rep)) {
                        (true, false) => -1,
                        (false, true) => 1,
                        // both still counted: the most recent event is the one being held
                        (true, true) => {
                            if n.last >= p.last {
                                -1
                            } else {
                                1
                            }
                        }
                        _ => 0,
                    }
                };
                let fire = self.hold_fire.held(now, &self.rep) || self.hold_fire.last.is_some_and(|t| now.duration_since(t) <= FIRE_LINGER);
                Controls { dx: axis(Dir::Left, Dir::Right) as f32, dy: axis(Dir::Up, Dir::Down) as f32, fire }
            }
            Mode::Latched => Controls { dx: self.latch_x as f32, dy: self.latch_y as f32, fire: self.fire_latched },
        }
    }
}

/// Flight direction of a key, if it is one: arrows, or Ctrl+h/j/k/l. A release of
/// h/j/k/l counts whatever the modifiers, since Ctrl may already be up by then.
fn flight_dir(k: &KeyEvent, release: bool) -> Option<(Dir, usize)> {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    match k.code {
        KeyCode::Left => Some((Dir::Left, ARROW)),
        KeyCode::Right => Some((Dir::Right, ARROW)),
        KeyCode::Up => Some((Dir::Up, ARROW)),
        KeyCode::Down => Some((Dir::Down, ARROW)),
        KeyCode::Char(c) if ctrl || release => match c.to_ascii_lowercase() {
            'h' => Some((Dir::Left, VIM)),
            'l' => Some((Dir::Right, VIM)),
            'k' => Some((Dir::Up, VIM)),
            'j' => Some((Dir::Down, VIM)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(base: Instant, t: u64) -> Instant {
        base + Duration::from_millis(t)
    }

    fn key(i: &mut Input, code: KeyCode, mods: KeyModifiers, kind: KeyEventKind, at: Instant) {
        i.on_key(KeyEvent::new_with_kind(code, mods, kind), at);
    }

    fn press(i: &mut Input, code: KeyCode, at: Instant) {
        key(i, code, KeyModifiers::NONE, KeyEventKind::Press, at);
    }

    fn ctrl(i: &mut Input, c: char, at: Instant) {
        key(i, KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press, at);
    }

    #[test]
    fn plain_letters_type_and_ctrl_letters_fly() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Char('h'), t0);
        press(&mut i, KeyCode::Char('E'), t0);
        assert_eq!(i.typed, vec!['h', 'e']);
        assert_eq!(i.controls(ms(t0, 10)).dx, 0.0, "a plain h is text, not flight");
        ctrl(&mut i, 'h', ms(t0, 20));
        assert_eq!(i.controls(ms(t0, 30)).dx, -1.0);
        ctrl(&mut i, 'j', ms(t0, 40));
        assert_eq!(i.controls(ms(t0, 50)).dy, 1.0);
        assert_eq!(i.typed.len(), 2, "Ctrl+letters never reach the typing layer");
    }

    #[test]
    fn space_fires_and_is_not_text() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Char(' '), t0);
        assert!(i.controls(ms(t0, 10)).fire);
        assert!(i.typed.is_empty());
    }

    #[test]
    fn fire_lingers_through_a_dodge() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Char(' '), t0);
        for k in 0..10 {
            press(&mut i, KeyCode::Char(' '), ms(t0, 500 + 30 * k));
        }
        // the player starts steering: the terminal now repeats only the arrow
        press(&mut i, KeyCode::Up, ms(t0, 800));
        assert!(i.controls(ms(t0, 1000)).fire, "a short dodge keeps the gun going");
        assert!(!i.controls(ms(t0, 1400)).fire, "but it does stop eventually");
    }

    #[test]
    fn a_tap_is_a_short_nudge() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Right, t0);
        assert_eq!(i.controls(ms(t0, 100)).dx, 1.0);
        assert_eq!(i.controls(ms(t0, 400)).dx, 0.0, "no repeat arrived: it was a tap");
    }

    #[test]
    fn turning_around_switches_direction_at_once() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Right, t0);
        for k in 0..10 {
            press(&mut i, KeyCode::Right, ms(t0, 500 + 30 * k));
        }
        assert_eq!(i.controls(ms(t0, 790)).dx, 1.0);
        press(&mut i, KeyCode::Left, ms(t0, 800));
        assert_eq!(i.controls(ms(t0, 810)).dx, -1.0);
    }

    #[test]
    fn exact_mode_follows_press_and_release_even_if_ctrl_lets_go_first() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Exact);
        ctrl(&mut i, 'l', t0);
        assert_eq!(i.controls(ms(t0, 900)).dx, 1.0);
        // Ctrl comes up before l: the release has no modifier
        key(&mut i, KeyCode::Char('l'), KeyModifiers::NONE, KeyEventKind::Release, ms(t0, 950));
        assert_eq!(i.controls(ms(t0, 951)).dx, 0.0);
        assert!(i.typed.is_empty());
    }

    #[test]
    fn letting_go_of_ctrl_stops_vim_flight() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Exact);
        ctrl(&mut i, 'k', t0);
        key(&mut i, KeyCode::Modifier(ModifierKeyCode::LeftControl), KeyModifiers::NONE, KeyEventKind::Release, ms(t0, 100));
        assert_eq!(i.controls(ms(t0, 110)).dy, 0.0);
    }

    #[test]
    fn latched_flight_toggles() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Latched);
        press(&mut i, KeyCode::Down, t0);
        assert_eq!(i.controls(ms(t0, 5000)).dy, 1.0);
        press(&mut i, KeyCode::Down, t0);
        assert_eq!(i.controls(t0).dy, 0.0, "the same key again stops");
        press(&mut i, KeyCode::Char(' '), t0);
        assert!(i.controls(t0).fire);
    }

    #[test]
    fn esc_asks_before_quitting() {
        let t0 = Instant::now();
        let mut i = Input::with_mode(Mode::Timing);
        press(&mut i, KeyCode::Esc, t0);
        assert!(i.confirm_quit && !i.quit);
        press(&mut i, KeyCode::Char('n'), t0);
        assert!(!i.confirm_quit && !i.quit, "any other key cancels");
        assert!(i.typed.is_empty(), "the answer is not typed");
        press(&mut i, KeyCode::Esc, t0);
        press(&mut i, KeyCode::Char('y'), t0);
        assert!(i.quit);
    }

    #[test]
    fn a_pad_and_the_keys_fly_together() {
        let keys = Controls { dx: 1.0, dy: 0.0, fire: false };
        let pad = crate::pad::PadState { x: -0.4, y: 0.6, fire: true };
        assert_eq!(Input::with_pad(keys, &pad), Controls { dx: 1.0, dy: 0.6, fire: true });
    }
}
