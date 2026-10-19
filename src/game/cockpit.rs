//! The cockpit scene after the crawl, and the tutorial: the receiver is broken.
//! The player types three repair commands, then the first echo comes through,
//! garbled; typing it out locks it clear, and it points the way out.

use super::echo::{Echo, EchoState};
use super::rng::Rng;

/// The repair commands, in order, and what the console answers.
pub const COMMANDS: [(&str, &str); 3] = [
    ("reboot", "receiver rebooted · antenna damaged"),
    ("tune", "tuned to 1420 MHz · the hydrogen line"),
    ("scan", "scanning · a faint signal"),
];
/// The first echo: always true, always clear enough to read.
const FIRST_ECHO: (&str, &str) = ("the eagle has landed", "Apollo 11 · 1969");
/// Seconds between the echo locking and the first turn.
const LINGER: f32 = 1.8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// typing repair command `n`
    Command(usize),
    /// decoding the first echo
    Echo,
    /// the echo is locked; the way out lights up
    Done,
}

pub struct Cockpit {
    pub step: Step,
    /// letters of the current command typed
    pub progress: usize,
    /// console lines so far
    pub log: Vec<String>,
    pub echo: Echo,
    /// seconds of miss flash left
    pub flash: f32,
    /// seconds since the echo locked
    pub done_t: f32,
}

impl Cockpit {
    pub fn new(rng: &mut Rng) -> Cockpit {
        let echo = Echo::new(1, FIRST_ECHO.0.to_string(), FIRST_ECHO.1, true, 0.85, (0.0, 0.0), rng);
        Cockpit { step: Step::Command(0), progress: 0, log: vec!["receiver offline".to_string()], echo, flash: 0.0, done_t: 0.0 }
    }

    /// The command being typed and how much of it is done.
    pub fn command(&self) -> Option<(&'static str, usize)> {
        match self.step {
            Step::Command(n) => Some((COMMANDS[n].0, self.progress)),
            _ => None,
        }
    }

    pub fn key(&mut self, c: char) {
        match self.step {
            Step::Command(n) => {
                let word = COMMANDS[n].0;
                if word.chars().nth(self.progress) == Some(c) {
                    self.progress += 1;
                    if self.progress == word.len() {
                        self.log.push(format!("> {word}"));
                        self.log.push(COMMANDS[n].1.to_string());
                        self.progress = 0;
                        self.step = if n + 1 < COMMANDS.len() { Step::Command(n + 1) } else { Step::Echo };
                        if self.step == Step::Echo {
                            // the only thing to type: locked from the start, first letter legible
                            self.echo.locked = true;
                            self.log.push("an echo · type what you read".to_string());
                        }
                    }
                } else {
                    self.flash = 0.15;
                }
            }
            Step::Echo => {
                if !self.echo.key(c) {
                    self.flash = 0.15;
                } else if self.echo.state == EchoState::Decoded {
                    self.log.push("locked · the echo points this way".to_string());
                    self.step = Step::Done;
                }
            }
            Step::Done => {}
        }
    }

    /// Returns true when the scene is over and the first turn should begin.
    pub fn update(&mut self, dt: f32) -> bool {
        self.flash = (self.flash - dt).max(0.0);
        self.echo.flash = (self.echo.flash - dt).max(0.0);
        if self.step == Step::Done {
            self.done_t += dt;
        }
        self.done_t >= LINGER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(c: &mut Cockpit, s: &str) {
        for ch in s.chars() {
            c.key(ch);
        }
    }

    #[test]
    fn three_commands_then_the_echo_then_out() {
        let mut c = Cockpit::new(&mut Rng::new(1));
        typed(&mut c, "rebooX");
        assert_eq!((c.step, c.progress), (Step::Command(0), 5), "a wrong key doesn't advance");
        typed(&mut c, "t");
        assert_eq!(c.step, Step::Command(1));
        typed(&mut c, "tunescan");
        assert_eq!(c.step, Step::Echo);
        typed(&mut c, "theeaglehaslanded");
        assert_eq!(c.step, Step::Done);
        assert!(!c.update(1.0));
        assert!(c.update(1.0), "then the first turn");
    }
}
