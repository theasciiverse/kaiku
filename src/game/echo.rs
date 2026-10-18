//! Echoes: snippets of Earth's transmissions drifting through the tunnel,
//! garbled, flickering into clear now and then. Typing one decodes it. A true
//! echo comes from the direction of home; false ones are jammed fakes. The
//! clearer an echo, the more it points home.

use super::rng::Rng;

/// A transmission as the data file has it.
#[derive(Clone, Debug)]
pub struct Transmission {
    pub text: &'static str,
    pub source: &'static str,
}

/// All transmissions, from `data/echoes.txt`.
pub fn transmissions() -> Vec<Transmission> {
    include_str!("../../data/echoes.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('|'))
        .map(|(text, source)| Transmission { text: text.trim(), source: source.trim() })
        .collect()
}

/// Words the war's jammers splice into false echoes.
const JAM: [&str; 10] = ["war", "fire", "never", "dark", "lost", "falling", "silence", "burning", "alone", "empty"];

/// A false echo of `t`: one word swapped for a jammer's word, or its tail
/// spliced onto the start of another transmission. The text is the tell.
pub fn jammed(t: &Transmission, all: &[Transmission], rng: &mut Rng) -> String {
    let words: Vec<&str> = t.text.split(' ').collect();
    if words.len() >= 4 && rng.unit() < 0.4 {
        let other = &all[(rng.unit() * all.len() as f32) as usize % all.len()];
        let tail: Vec<&str> = other.text.split(' ').collect();
        if other.text != t.text && tail.len() >= 2 {
            let half = words.len() / 2;
            return words[..half].iter().chain(&tail[tail.len() / 2..]).copied().collect::<Vec<_>>().join(" ");
        }
    }
    let long: Vec<usize> = (0..words.len()).filter(|&i| words[i].len() > 2).collect();
    let pick = long[(rng.unit() * long.len() as f32) as usize % long.len().max(1)];
    let jam = JAM[(rng.unit() * JAM.len() as f32) as usize % JAM.len()];
    words.iter().enumerate().map(|(i, w)| if i == pick { jam } else { w }).collect::<Vec<_>>().join(" ")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EchoState {
    Drifting,
    /// typed out: it now names a portal
    Decoded,
    /// drifted off the field before it was decoded
    Lost,
}

/// What the receiver makes of a decoded echo once it has analysed it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// not decoded yet, or still being analysed
    Pending,
    /// a false echo: its gate is ruled out
    Jammed,
    /// a true echo holds steady (a strong hint, never a promise on screen)
    Steady,
}

/// How one character of an echo shows right now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vis {
    /// already typed: clear for good
    Typed,
    /// readable this moment; `blink` is the dim half of a flicker
    Clear { blink: bool },
    /// noise; the value picks the glyph
    Garbled(u32),
}

/// Drift speed in dots per second; slower while being decoded.
const DRIFT: f32 = 7.0;
const DRIFT_LOCKED: f32 = 3.0;
/// Seconds the receiver takes to analyse a decoded echo.
pub const ANALYSIS_TIME: f32 = 4.0;
/// The longer an echo is in range, the clearer it reads: this much more of it
/// shows per second, up to `LISTEN_MAX`.
const LISTEN_RATE: f32 = 0.015;
const LISTEN_MAX: f32 = 0.3;
/// A wrong key costs this much clarity.
const MISS_COST: f32 = 0.03;

#[derive(Clone, Debug)]
pub struct Echo {
    pub id: u32,
    pub text: String,
    pub source: &'static str,
    /// true: from the direction of home
    pub truth: bool,
    /// 0..1: how clearly it comes through, and how strongly it points home
    pub clarity: f32,
    /// letters typed so far
    pub progress: usize,
    pub state: EchoState,
    pub locked: bool,
    /// seconds of miss flash left
    pub flash: f32,
    /// clarity a jammer is taking off it right now
    pub jam: f32,
    /// seconds in range, and seconds of analysis since it was decoded
    age: f32,
    pub analysed: f32,
    /// position of the first character, in dots (y on whole cells)
    pub x: f32,
    pub y: f32,
    seed: u64,
}

impl Echo {
    pub fn new(id: u32, text: String, source: &'static str, truth: bool, clarity: f32, (x, y): (f32, f32), rng: &mut Rng) -> Echo {
        let y = (y / 4.0).floor() * 4.0;
        Echo { id, text, source, truth, clarity, progress: 0, state: EchoState::Drifting, locked: false, flash: 0.0, jam: 0.0, age: 0.0, analysed: 0.0, x, y, seed: rng.next_u64() }
    }

    /// The letters to type, in order.
    pub fn letters(&self) -> impl Iterator<Item = char> + '_ {
        self.text.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_lowercase())
    }

    pub fn next_letter(&self) -> Option<char> {
        self.letters().nth(self.progress)
    }

    pub fn width(&self) -> f32 {
        self.text.chars().count() as f32 * 2.0
    }

    /// Feeds one typed letter; returns true if it was the right one.
    pub fn key(&mut self, c: char) -> bool {
        if self.next_letter() == Some(c) {
            self.progress += 1;
            if self.next_letter().is_none() {
                self.state = EchoState::Decoded;
                self.locked = false;
            }
            true
        } else {
            self.clarity = (self.clarity - MISS_COST).max(0.05);
            self.flash = 0.15;
            false
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.flash = (self.flash - dt).max(0.0);
        self.age += dt;
        if self.state == EchoState::Decoded {
            self.analysed += dt;
        }
        if self.state == EchoState::Drifting {
            self.x -= if self.locked { DRIFT_LOCKED } else { DRIFT } * dt;
            if self.x + self.width() < 0.0 {
                self.state = EchoState::Lost;
                self.locked = false;
            }
        }
    }

    pub fn verdict(&self) -> Verdict {
        match () {
            _ if self.state != EchoState::Decoded || self.analysed < ANALYSIS_TIME => Verdict::Pending,
            _ if self.truth => Verdict::Steady,
            _ => Verdict::Jammed,
        }
    }

    /// How character `i` of the text shows at `tick`. Clear echoes flicker into
    /// full view more often and for longer, and stay partly readable between;
    /// every echo reads a little better the longer it has been in range.
    pub fn vis(&self, i: usize, tick: u64) -> Vis {
        let mut letter = 0;
        let mut is_letter = false;
        for (k, c) in self.text.chars().enumerate() {
            if k == i {
                is_letter = c.is_ascii_alphabetic();
                break;
            }
            if c.is_ascii_alphabetic() {
                letter += 1;
            }
        }
        if !is_letter {
            // spaces and punctuation always show: the shape of the words helps
            return Vis::Clear { blink: false };
        }
        if letter < self.progress || self.state == EchoState::Decoded {
            return Vis::Typed;
        }
        let c = (self.clarity - self.jam).clamp(0.0, 1.0);
        let listened = (self.age * LISTEN_RATE).min(LISTEN_MAX);
        let period = ((2.4 - 1.2 * c) * 60.0) as u64;
        let window = ((0.7 + 0.9 * c) * 60.0) as u64;
        let t = tick + self.seed % period;
        let cycle = t / period;
        let roll = unit(hash(self.seed ^ i as u64, cycle));
        if t % period < window {
            // the flicker: most letters come clear, blinking
            if roll < 0.75 + 0.25 * c {
                // about 3 Hz: slow enough to read through, easy on the eyes
                return Vis::Clear { blink: (t / 10).is_multiple_of(2) };
            }
        } else if roll < 0.2 + 0.35 * c + listened {
            return Vis::Clear { blink: false };
        }
        Vis::Garbled(hash(self.seed.wrapping_add(i as u64), tick / 4) as u32)
    }
}

fn hash(a: u64, b: u64) -> u64 {
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 29)
}

fn unit(h: u64) -> f32 {
    (h >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo(text: &str, clarity: f32) -> Echo {
        Echo::new(1, text.to_string(), "test", true, clarity, (100.0, 20.0), &mut Rng::new(5))
    }

    #[test]
    fn the_data_file_parses() {
        let all = transmissions();
        assert!(all.len() >= 10);
        assert!(all.iter().all(|t| !t.text.is_empty() && !t.source.is_empty()));
    }

    #[test]
    fn typing_skips_spaces_and_punctuation() {
        let mut e = echo("we've had", 0.5);
        for c in "wevehad".chars() {
            assert!(e.key(c), "{c}");
        }
        assert_eq!(e.state, EchoState::Decoded);
    }

    #[test]
    fn a_wrong_key_costs_clarity_but_not_progress() {
        let mut e = echo("eagle", 0.5);
        assert!(e.key('e'));
        assert!(!e.key('x'));
        assert_eq!(e.progress, 1);
        assert!(e.clarity < 0.5);
    }

    #[test]
    fn typed_letters_stay_clear_and_spaces_always_show() {
        let mut e = echo("the eagle", 0.0);
        e.key('t');
        for tick in 0..600 {
            assert_eq!(e.vis(0, tick), Vis::Typed);
            assert_eq!(e.vis(3, tick), Vis::Clear { blink: false });
        }
    }

    #[test]
    fn clearer_echoes_are_readable_more_often() {
        let readable = |c: f32| {
            let e = echo("the eagle has landed", c);
            (0..1200u64).flat_map(|t| (0..20).map(move |i| (t, i))).filter(|&(t, i)| matches!(e.vis(i, t), Vis::Clear { .. })).count()
        };
        assert!(readable(0.9) as f32 > readable(0.3) as f32 * 1.4, "{} vs {}", readable(0.9), readable(0.3));
    }

    #[test]
    fn jammed_echoes_differ_from_the_truth() {
        let all = transmissions();
        let mut rng = Rng::new(9);
        for t in &all {
            for _ in 0..5 {
                assert_ne!(jammed(t, &all, &mut rng), t.text);
            }
        }
    }

    #[test]
    fn a_decoded_echo_is_analysed_and_a_false_one_is_ruled_out() {
        let mut e = Echo::new(1, "lost".to_string(), "test", false, 0.3, (100.0, 20.0), &mut Rng::new(5));
        for c in "lost".chars() {
            e.key(c);
        }
        assert_eq!(e.verdict(), Verdict::Pending, "analysis takes a moment");
        e.update(ANALYSIS_TIME);
        assert_eq!(e.verdict(), Verdict::Jammed);
    }

    #[test]
    fn echoes_read_better_the_longer_they_are_in_range() {
        let readable = |e: &Echo| (0..600u64).flat_map(|t| (0..20).map(move |i| (t, i))).filter(|&(t, i)| matches!(e.vis(i, t), Vis::Clear { .. })).count();
        let mut e = echo("the eagle has landed", 0.3);
        let fresh = readable(&e);
        e.update(20.0);
        assert!(readable(&e) > fresh, "{} vs {fresh}", readable(&e));
    }
}
