//! Music, synthesised here like the effects: one endless voice on the mixer,
//! steered every frame by the game (as Ceres does its music).
//!
//! The music is game state, not a backdrop:
//! - **The journey darkens it.** It starts as chiptune (square arpeggios, a
//!   plucked bass, a beat) and, sector by sector, turns into darker space
//!   ambient: long detuned pads, a slower pulse, more echo on everything.
//! - **The war is the beat.** Drums come in with the number of ships on
//!   screen and drop away when it clears.
//! - **The echoes are the melody.** Kaiku has one theme. While echoes are in
//!   range you hear it in pieces: notes drop out or slip off pitch the fainter
//!   the echo, and a jammer makes it waver. Home is the first time it plays
//!   whole and clean.
//! - **Moments have their own colour:** an eerie drone under the crawl, a
//!   tense drone when the gates open, a riser through the turn, a calm theme
//!   on the map.

// without the `sound` feature nothing plays the band
#![cfg_attr(not(feature = "sound"), allow(dead_code))]

use std::sync::{Arc, Mutex};

pub const SR: u32 = 44_100;
const SRF: f32 = SR as f32;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mood {
    #[default]
    Crawl,
    Cockpit,
    Flight,
    Gates,
    Turn,
    Map,
    Home,
    Over,
}

/// What the game tells the music, every frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ctrl {
    pub mood: Mood,
    /// how thick the fighting is, 0..1
    pub intensity: f32,
    /// how clearly an echo comes through, 0 (none in range) .. 1
    pub clarity: f32,
    pub jam: bool,
    /// how far into the journey: 0 chiptune .. 1 dark space ambient
    pub dark: f32,
    /// semitones: each sector sits in its own key
    pub key: i32,
    pub muted: bool,
}

/// The theme, two bars of sixteenths: (step, MIDI pitch, length in steps).
/// It ends on the echo of its own opening: kaiku.
const THEME: [(u32, i32, u32); 11] = [
    (0, 69, 3),
    (4, 72, 2),
    (6, 76, 2),
    (8, 74, 4),
    (12, 72, 2),
    (14, 71, 2),
    (16, 69, 4),
    (20, 67, 2),
    (22, 69, 2),
    (24, 64, 4),
    (28, 69, 4),
];

/// Four bars of chords. Bright: Am F C G. Dark: Am F Dm Em.
const BRIGHT: [[i32; 3]; 4] = [[57, 60, 64], [53, 57, 60], [48, 52, 55], [55, 59, 62]];
const DARK: [[i32; 3]; 4] = [[57, 60, 64], [53, 57, 60], [50, 53, 57], [52, 55, 59]];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Wave {
    Tri,
    Square,
    Pulse,
    Saw,
}

#[derive(Clone, Copy, Debug)]
struct Voice {
    freq: f32,
    phase: f32,
    wave: Wave,
    vol: f32,
    /// envelope: linear attack, hold, exponential release, all in samples
    attack: f32,
    hold: f32,
    release: f32,
    age: f32,
    /// how much of it goes to the echo (delay)
    send: f32,
    /// vibrato depth in semitones (a jammer's waver)
    wobble: f32,
}

impl Voice {
    fn env(&self) -> f32 {
        let a = self.attack.max(1.0);
        if self.age < a {
            self.age / a
        } else if self.age < a + self.hold {
            1.0
        } else {
            (-(self.age - a - self.hold) / self.release.max(1.0) * 5.0).exp()
        }
    }

    fn done(&self) -> bool {
        self.age > self.attack + self.hold + self.release
    }

    fn sample(&mut self) -> f32 {
        let wob = if self.wobble > 0.0 { 2f32.powf(self.wobble * (self.age / SRF * 7.0).sin() / 12.0) } else { 1.0 };
        self.phase = (self.phase + self.freq * wob / SRF).fract();
        let p = self.phase;
        let x = match self.wave {
            Wave::Tri => 1.0 - 4.0 * (p - 0.5).abs(),
            Wave::Square => 0.6 * if p < 0.5 { 1.0 } else { -1.0 },
            Wave::Pulse => 0.6 * if p < 0.25 { 1.0 } else { -1.0 },
            Wave::Saw => (p * 2.0 - 1.0) * 0.6,
        };
        let v = x * self.vol * self.env();
        self.age += 1.0;
        v
    }
}

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// The endless music source.
pub struct Band {
    ctrl: Arc<Mutex<Ctrl>>,
    poll: u32,
    /// the latest instructions, and the slow-moving values smoothed from them
    want: Ctrl,
    intensity: f32,
    dark: f32,
    clarity: f32,
    level: f32,
    voices: Vec<Voice>,
    /// position within the current sixteenth, in samples, and steps so far
    in_step: f32,
    step: u64,
    /// drums: envelopes and the kick's falling pitch
    kick: f32,
    kick_phase: f32,
    hat: f32,
    snare: f32,
    noise: u32,
    lp: f32,
    /// the echo: a delay line
    delay: Vec<f32>,
    at: usize,
    /// samples and sixteenths spent in the current mood (the riser, the drones)
    mood_age: f32,
    mood_step: u64,
}

impl Band {
    pub fn new(ctrl: Arc<Mutex<Ctrl>>) -> Band {
        Band {
            ctrl,
            poll: 0,
            want: Ctrl::default(),
            intensity: 0.0,
            dark: 0.0,
            clarity: 0.0,
            level: 0.0,
            voices: Vec::with_capacity(24),
            in_step: 0.0,
            step: 0,
            kick: 0.0,
            kick_phase: 0.0,
            hat: 0.0,
            snare: 0.0,
            noise: 0x2468_ace1,
            lp: 0.0,
            delay: vec![0.0; SR as usize],
            at: 0,
            mood_age: 0.0,
            mood_step: 0,
        }
    }

    fn rand(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        self.noise as f32 / u32::MAX as f32
    }

    /// A sixteenth's length in samples: the tempo eases from 104 to 84 bpm
    /// as the music darkens.
    fn step_len(&self) -> f32 {
        let bpm = 104.0 - 20.0 * self.dark;
        SRF * 60.0 / bpm / 4.0
    }

    #[allow(clippy::too_many_arguments)]
    fn play(&mut self, midi: f32, wave: Wave, vol: f32, attack: f32, hold: f32, release: f32, send: f32, wobble: f32) {
        if self.voices.len() >= 24 {
            self.voices.remove(0);
        }
        self.voices.push(Voice {
            freq: hz(midi),
            phase: 0.0,
            wave,
            vol,
            attack: attack * SRF,
            hold: hold * SRF,
            release: release * SRF,
            age: 0.0,
            send,
            wobble,
        });
    }

    /// The start of a sixteenth: which notes and drums it brings.
    fn on_step(&mut self) {
        let s = (self.step % 16) as u32;
        let bar = (self.step / 16 % 4) as usize;
        let d = self.dark;
        let key = self.want.key;
        let chords = if d > 0.5 { DARK } else { BRIGHT };
        let chord = chords[bar].map(|n| (n + key) as f32);
        let mood = self.want.mood;
        let step_s = self.step_len() / SRF;
        let lead_wave = if d < 0.5 { Wave::Square } else { Wave::Tri };

        // the pad: long chords, more of them the darker it gets
        let pad = match mood {
            Mood::Crawl | Mood::Over => 0.0,
            Mood::Map | Mood::Home | Mood::Cockpit => 0.5,
            _ => d,
        };
        if s == 0 && pad > 0.02 {
            for (i, &n) in chord.iter().enumerate() {
                // a slight detune between the voices: the ambient shimmer
                let detune = [0.0, 0.07, -0.06][i];
                self.play(n + detune, Wave::Tri, 0.10 * pad, 0.6 + 0.8 * d, step_s * 12.0, 1.6, 0.5, 0.0);
            }
        }

        // the bass: plucked eighths in chiptune, one long note a bar in the dark
        if matches!(mood, Mood::Flight | Mood::Gates | Mood::Map | Mood::Home) {
            let root = chord[0] - 12.0;
            if d < 0.5 && s.is_multiple_of(2) && mood != Mood::Map {
                let wave = if d < 0.25 { Wave::Square } else { Wave::Tri };
                self.play(root, wave, 0.22 * (1.0 - d), 0.003, step_s * 0.8, 0.08, 0.0, 0.0);
            } else if s == 0 {
                self.play(root, Wave::Tri, 0.26, 0.05, step_s * 14.0, 0.8, 0.2, 0.0);
            }
        }

        // the arpeggio: the chiptune voice, fading out as the music darkens
        if matches!(mood, Mood::Flight | Mood::Map) && d < 0.85 {
            let n = chord[(s as usize) % 3] + if s % 6 >= 3 { 12.0 } else { 0.0 } + 12.0;
            let vol = 0.07 * (1.0 - d) * if mood == Mood::Map { 0.5 } else { 1.0 };
            self.play(n, Wave::Pulse, vol, 0.002, step_s * 0.4, 0.05, 0.3, 0.0);
        }

        // the war: drums with the fighting, softer and sparser in the dark
        let drums = match mood {
            Mood::Flight => (0.15 + 0.85 * self.intensity) * (1.0 - 0.65 * d),
            Mood::Gates => 0.35,
            _ => 0.0,
        };
        if drums > 0.02 {
            if s.is_multiple_of(4) {
                self.kick = drums;
                self.kick_phase = 0.0;
            }
            if s % 2 == 1 && (d < 0.6 || s % 4 == 3) {
                self.hat = 0.35 * drums;
            }
            if (s == 4 || s == 12) && d < 0.7 && mood == Mood::Flight {
                self.snare = 0.6 * drums;
            }
        }

        // the theme: in pieces while echoes are in range, whole at home
        let clarity = match mood {
            Mood::Home => 1.0,
            Mood::Map => 0.55,
            Mood::Flight | Mood::Gates | Mood::Cockpit => self.clarity,
            _ => 0.0,
        };
        if clarity > 0.05 {
            let pos = (self.step % 32) as u32;
            if let Some(&(_, pitch, len)) = THEME.iter().find(|(at, _, _)| *at == pos) {
                // a faint echo drops notes and slips off pitch; a clear one doesn't
                let broken = 1.0 - clarity;
                let dropped = self.rand() < broken * 0.6;
                if !dropped {
                    let slip = if self.rand() < broken * 0.5 { [-2.0, -1.0, 1.0, 2.0][(self.rand() * 4.0) as usize % 4] } else { 0.0 };
                    let wobble = if self.want.jam { 0.6 } else { 0.0 };
                    let vol = (0.10 + 0.08 * clarity) * if mood == Mood::Map { 0.6 } else { 1.0 };
                    self.play(pitch as f32 + key as f32 + slip, lead_wave, vol, 0.01 + 0.05 * d, step_s * len as f32 * 0.8, 0.3 + 0.6 * d, 0.6, wobble);
                }
            }
        }

        // the crawl and the end: a low drone and a few far-off notes
        if matches!(mood, Mood::Crawl | Mood::Over) {
            // from the first beat of the mood, then every two bars
            if self.mood_step % 32 == 1 {
                let root = 45.0 + key as f32;
                self.play(root, Wave::Tri, 0.16, 1.5, step_s * 28.0, 2.5, 0.4, 0.0);
                self.play(root + 7.0, Wave::Tri, 0.08, 2.0, step_s * 26.0, 2.5, 0.4, 0.0);
            }
            if mood == Mood::Crawl && s == 8 && self.rand() < 0.6 {
                let pent = [69.0, 72.0, 74.0, 76.0, 79.0, 81.0];
                let n = pent[(self.rand() * pent.len() as f32) as usize % pent.len()];
                self.play(n + key as f32, Wave::Tri, 0.06, 0.02, step_s * 2.0, 1.2, 0.9, 0.0);
            }
        }

        // the gates: a tense drone under it all
        if mood == Mood::Gates && self.mood_step % 16 == 1 {
            let root = chord[0] - 24.0;
            self.play(root, Wave::Saw, 0.06, 0.4, step_s * 15.0, 0.4, 0.2, 0.15);
            self.play(root + 7.0, Wave::Tri, 0.08, 0.4, step_s * 15.0, 0.4, 0.2, 0.0);
        }
    }

    /// One sample of the drums.
    fn drums(&mut self) -> f32 {
        let mut x = 0.0;
        if self.kick > 0.001 {
            // a falling sine: the kick
            let f = 45.0 + 110.0 * self.kick.powi(3);
            self.kick_phase = (self.kick_phase + f / SRF).fract();
            x += (self.kick_phase * std::f32::consts::TAU).sin() * self.kick * 0.8;
            self.kick *= 0.9993;
        }
        if self.hat > 0.001 || self.snare > 0.001 {
            let n = self.rand() * 2.0 - 1.0;
            // the hat is the noise's bright edge, the snare its body
            let hp = n - self.lp;
            self.lp += (n - self.lp) * 0.2;
            x += hp * self.hat + self.lp * self.snare * 1.5;
            self.hat *= 0.996;
            self.snare *= 0.9985;
        }
        x
    }

    fn sample(&mut self) -> f32 {
        if self.poll == 0 {
            if let Ok(c) = self.ctrl.lock() {
                if c.mood != self.want.mood {
                    self.mood_age = 0.0;
                    self.mood_step = 0;
                }
                self.want = *c;
            }
            // smooth the slow values so nothing jumps
            self.intensity += (self.want.intensity - self.intensity) * 0.02;
            self.dark += (self.want.dark - self.dark) * 0.005;
            self.clarity += (self.want.clarity - self.clarity) * 0.05;
            let level = if self.want.muted { 0.0 } else { 1.0 };
            self.level += (level - self.level) * 0.05;
            self.poll = 512;
        }
        self.poll -= 1;
        self.mood_age += 1.0;

        self.in_step += 1.0;
        if self.in_step >= self.step_len() {
            self.in_step = 0.0;
            self.step += 1;
            self.mood_step += 1;
            self.on_step();
        }

        let (mut dry, mut send) = (0.0f32, 0.0f32);
        for v in &mut self.voices {
            let x = v.sample();
            dry += x;
            send += x * v.send;
        }
        self.voices.retain(|v| !v.done());
        dry += self.drums();

        // the turn: a rising sweep, as the field swings round
        if self.want.mood == Mood::Turn {
            let t = (self.mood_age / (SRF * 1.6)).min(1.0);
            let f = hz(48.0 + 48.0 * t);
            let p = (self.mood_age * f / SRF).fract();
            dry += (1.0 - 4.0 * (p - 0.5).abs()) * 0.12 * t;
            send += (self.rand() * 2.0 - 1.0) * 0.05 * t;
        }

        // the echo: a delay of three sixteenths, longer and fuller in the dark
        let len = ((self.step_len() * 3.0) as usize).min(self.delay.len() - 1);
        let read = (self.at + self.delay.len() - len) % self.delay.len();
        let echoed = self.delay[read];
        let feedback = 0.25 + 0.4 * self.dark;
        self.delay[self.at] = send + echoed * feedback;
        self.at = (self.at + 1) % self.delay.len();

        let x = (dry + echoed * 0.5) * 0.6 * self.level;
        // soft clip
        x / (1.0 + x.abs())
    }
}

impl Iterator for Band {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        Some(self.sample())
    }
}

#[cfg(feature = "sound")]
impl rodio::Source for Band {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        std::num::NonZero::new(1).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        std::num::NonZero::new(SR).unwrap()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

/// What the music should do for the game as it is now.
pub fn ctrl_for(game: &crate::game::Game) -> Ctrl {
    use crate::game::echo::EchoState;
    use crate::game::sector::Phase;
    use crate::game::State;
    let mood = match game.state {
        State::Intro => Mood::Crawl,
        State::Cockpit => Mood::Cockpit,
        State::Flight if game.sector.phase == Phase::Portals => Mood::Gates,
        State::Flight => Mood::Flight,
        State::Turn => Mood::Turn,
        State::Map => Mood::Map,
        State::Home => Mood::Home,
        State::Over => Mood::Over,
    };
    let on_screen = |x: f32| x < game.w;
    let intensity = (game.enemies.iter().filter(|e| on_screen(e.x)).count() as f32 / 10.0).min(1.0);
    let locked = game.typing.lock.and_then(|id| game.sector.echo(id)).filter(|e| e.state == EchoState::Drifting);
    let drifting = game.sector.echoes.iter().filter(|e| e.state == EchoState::Drifting && on_screen(e.x));
    let clarity = match &game.cockpit {
        Some(c) if c.step != crate::game::cockpit::Step::Command(0) && game.state == State::Cockpit => c.echo.clarity,
        _ => locked.map(|e| e.clarity - e.jam).unwrap_or_else(|| drifting.map(|e| (e.clarity - e.jam) * 0.6).fold(0.0, f32::max)),
    };
    // the journey: about eight jumps from chiptune to the dark
    let dark = (game.stats.sectors as f32 / 8.0).min(1.0);
    const KEYS: [i32; 5] = [0, 2, -3, 5, -2];
    Ctrl {
        mood,
        intensity,
        clarity: clarity.clamp(0.0, 1.0),
        jam: game.sector.echoes.iter().any(|e| e.jam > 0.0),
        dark,
        key: KEYS[(game.sector.number.saturating_sub(1) as usize) % KEYS.len()],
        muted: false,
    }
}

pub fn shared() -> Arc<Mutex<Ctrl>> {
    Arc::new(Mutex::new(Ctrl::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(ctrl: Ctrl, secs: f32) -> Vec<f32> {
        let shared = Arc::new(Mutex::new(ctrl));
        let mut band = Band::new(shared);
        // settle the smoothed values first
        band.dark = ctrl.dark;
        band.intensity = ctrl.intensity;
        band.clarity = ctrl.clarity;
        (0..(secs * SRF) as usize).map(|_| band.sample()).collect()
    }

    fn rms(s: &[f32]) -> f32 {
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    }

    #[test]
    fn every_mood_plays_and_stays_in_range() {
        for mood in [Mood::Crawl, Mood::Cockpit, Mood::Flight, Mood::Gates, Mood::Turn, Mood::Map, Mood::Home, Mood::Over] {
            for dark in [0.0, 1.0] {
                let s = render(Ctrl { mood, dark, intensity: 0.5, clarity: 0.5, ..Ctrl::default() }, 4.0);
                assert!(s.iter().all(|x| x.is_finite() && x.abs() <= 1.0), "{mood:?}");
                assert!(rms(&s) > 0.005, "{mood:?} at dark {dark} is audible: {}", rms(&s));
            }
        }
    }

    #[test]
    fn muted_is_silent_and_the_war_is_louder_than_quiet() {
        let muted = render(Ctrl { mood: Mood::Flight, muted: true, ..Ctrl::default() }, 2.0);
        assert!(rms(&muted[SR as usize..]) < 0.001);
        let quiet = rms(&render(Ctrl { mood: Mood::Flight, intensity: 0.0, ..Ctrl::default() }, 4.0));
        let war = rms(&render(Ctrl { mood: Mood::Flight, intensity: 1.0, ..Ctrl::default() }, 4.0));
        assert!(war > quiet, "{war} vs {quiet}");
    }

    /// Not a check: how much of the audio thread the music needs.
    /// `cargo test --release music_cost -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn music_cost() {
        let t0 = std::time::Instant::now();
        let s = render(Ctrl { mood: Mood::Flight, intensity: 1.0, clarity: 0.8, dark: 0.5, ..Ctrl::default() }, 20.0);
        let spent = t0.elapsed();
        println!("20 s of music in {spent:?} ({:.2}% of real time), rms {:.3}", spent.as_secs_f32() / 20.0 * 100.0, rms(&s));
    }
}
