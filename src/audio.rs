//! Sound, all synthesised here; Kaiku ships no audio files.
//!
//! Each cue is written as a short row of steps (a pitch, a waveform and a
//! volume) played at a fixed speed, in the manner of Ceres's effects. The
//! bank is rendered once at start. Without an audio device, without the
//! `sound` feature, or muted (`--mute`, F8), every call does nothing.

use crate::game::cue::Cue;

#[cfg(feature = "sound")]
mod synth {
    use super::Cue;

    pub const SR: u32 = 44_100;
    const SRF: f32 = SR as f32;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Wave {
        Tri,
        Square,
        Pulse,
        Saw,
        Noise,
    }
    use Wave::*;

    /// One step: MIDI pitch (fractions slide), waveform, volume 0-1.
    type Step = (f32, Wave, f32);

    /// Steps from pitch `a` to `b` in `n` even moves, volume `v0` to `v1`.
    fn slide(a: f32, b: f32, n: usize, w: Wave, v0: f32, v1: f32) -> Vec<Step> {
        (0..n)
            .map(|i| {
                let t = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
                (a + (b - a) * t, w, v0 + (v1 - v0) * t)
            })
            .collect()
    }

    fn notes(ps: &[f32], w: Wave, v: f32, each: usize) -> Vec<Step> {
        ps.iter().flat_map(|&p| std::iter::repeat_n((p, w, v), each)).collect()
    }

    fn fade(mut s: Vec<Step>, from: f32) -> Vec<Step> {
        let n = s.len() as f32;
        for (i, st) in s.iter_mut().enumerate() {
            let t = i as f32 / n;
            if t > from {
                st.2 *= 1.0 - (t - from) / (1.0 - from);
            }
        }
        s
    }

    /// A cue's sound: step length in seconds, and layers played together.
    fn score(cue: Cue) -> (f32, Vec<Vec<Step>>) {
        match cue {
            // a soft blip: it plays a lot
            Cue::Shot => (0.008, vec![slide(84.0, 76.0, 3, Pulse, 0.22, 0.0)]),
            Cue::Kill => (0.012, vec![slide(62.0, 36.0, 8, Noise, 0.6, 0.0), slide(70.0, 50.0, 6, Square, 0.25, 0.0)]),
            Cue::BigKill => (0.016, vec![slide(55.0, 26.0, 18, Noise, 0.8, 0.0), slide(52.0, 32.0, 14, Saw, 0.4, 0.0)]),
            Cue::Shrapnel => (0.01, vec![slide(84.0, 44.0, 12, Noise, 0.6, 0.0)]),
            Cue::ShieldHit => (0.012, vec![slide(50.0, 40.0, 7, Square, 0.55, 0.05), slide(70.0, 50.0, 5, Noise, 0.5, 0.0)]),
            Cue::LifeLost => (0.02, vec![slide(66.0, 30.0, 26, Saw, 0.6, 0.0), slide(54.0, 24.0, 26, Noise, 0.6, 0.0)]),
            Cue::GameOver => (0.12, vec![fade(notes(&[67.0, 63.0, 60.0, 55.0], Tri, 0.5, 2), 0.6)]),
            // typing: a tick for a right letter, a dull buzz for a wrong one
            Cue::Key => (0.007, vec![vec![(91.0, Tri, 0.18), (91.0, Tri, 0.05)]]),
            Cue::Miss => (0.02, vec![notes(&[40.0, 38.0], Pulse, 0.22, 2)]),
            Cue::Decoded => (0.04, vec![fade(notes(&[72.0, 76.0, 79.0, 84.0, 88.0], Tri, 0.45, 1), 0.5)]),
            Cue::Claimed => (0.03, vec![fade(notes(&[79.0, 84.0, 88.0, 91.0], Square, 0.35, 1), 0.5)]),
            Cue::PodBroken => (0.015, vec![slide(70.0, 40.0, 8, Noise, 0.5, 0.0), slide(60.0, 48.0, 8, Tri, 0.35, 0.0)]),
            Cue::Bomb => (0.02, vec![slide(30.0, 86.0, 30, Noise, 0.9, 0.0), slide(40.0, 22.0, 30, Saw, 0.5, 0.0)]),
            Cue::Dash => (0.012, vec![slide(58.0, 86.0, 8, Noise, 0.5, 0.1)]),
            // the lancer: a rising hum while it warns, then a buzz
            Cue::Warning => (0.05, vec![slide(48.0, 60.0, 20, Pulse, 0.12, 0.3)]),
            Cue::Beam => (0.05, vec![notes(&[43.0], Saw, 0.4, 12), notes(&[31.0], Noise, 0.25, 12)]),
            Cue::Jamming => (0.06, vec![fade(notes(&[70.0, 66.0, 70.0, 66.0, 70.0, 66.0], Pulse, 0.2, 1), 0.6)]),
            Cue::GatesOpen => (0.06, vec![fade(notes(&[60.0, 67.0, 72.0, 79.0, 84.0], Tri, 0.4, 1), 0.6)]),
            Cue::Jammed => (0.07, vec![notes(&[60.0, 54.0], Pulse, 0.3, 2)]),
            Cue::Steady => (0.07, vec![fade(notes(&[72.0, 79.0], Tri, 0.4, 2), 0.6)]),
            Cue::Jump => (0.03, vec![fade(slide(48.0, 96.0, 40, Tri, 0.25, 0.55), 0.7), slide(40.0, 70.0, 40, Noise, 0.2, 0.0)]),
            Cue::ExtraLife => (0.035, vec![fade(notes(&[72.0, 76.0, 79.0, 84.0, 88.0, 91.0], Square, 0.4, 1), 0.6)]),
            Cue::Home => (0.18, vec![fade(notes(&[60.0, 64.0, 67.0, 72.0, 76.0, 79.0, 84.0], Tri, 0.4, 1), 0.6)]),
            Cue::Typewriter => (0.004, vec![vec![(100.0, Noise, 0.25), (100.0, Noise, 0.08)]]),
            Cue::Repaired => (0.04, vec![fade(notes(&[76.0, 83.0], Square, 0.35, 2), 0.5)]),
        }
    }

    pub const ALL: [Cue; 25] = [
        Cue::Shot,
        Cue::Kill,
        Cue::BigKill,
        Cue::Shrapnel,
        Cue::ShieldHit,
        Cue::LifeLost,
        Cue::GameOver,
        Cue::Key,
        Cue::Miss,
        Cue::Decoded,
        Cue::Claimed,
        Cue::PodBroken,
        Cue::Bomb,
        Cue::Dash,
        Cue::Warning,
        Cue::Beam,
        Cue::Jamming,
        Cue::GatesOpen,
        Cue::Jammed,
        Cue::Steady,
        Cue::Jump,
        Cue::ExtraLife,
        Cue::Home,
        Cue::Typewriter,
        Cue::Repaired,
    ];

    /// A cue's samples, mono, at `SR`.
    pub fn render(cue: Cue) -> Vec<f32> {
        let (len, layers) = score(cue);
        let per = (len * SRF) as usize;
        let mut out = vec![0.0f32; per * layers.iter().map(Vec::len).max().unwrap_or(0)];
        for steps in &layers {
            let (mut ph, mut noise, mut lp, mut hold, mut prev_vol) = (0.0f32, 0x1234_5678u32, 0.0f32, 0.0f32, 0.0f32);
            for (k, &(pitch, w, vol)) in steps.iter().enumerate() {
                let next = steps.get(k + 1).map_or(pitch, |s| s.0);
                for s in 0..per {
                    let t = s as f32 / per as f32;
                    let p = pitch + (next - pitch) * t * 0.5;
                    let f = 440.0 * 2f32.powf((p - 69.0) / 12.0);
                    ph = (ph + f / SRF).fract();
                    // ramp volume changes over 64 samples against clicks
                    let v = prev_vol + (vol - prev_vol) * (s as f32 / 64.0).min(1.0);
                    let x = match w {
                        Tri => 1.0 - 4.0 * (ph - 0.5).abs(),
                        Square => 0.6 * if ph < 0.5 { 1.0 } else { -1.0 },
                        Pulse => 0.6 * if ph < 0.25 { 1.0 } else { -1.0 },
                        Saw => (ph * 2.0 - 1.0) * 0.6,
                        Noise => {
                            // sample-and-hold noise at the pitch rate, softened
                            if ph < f / SRF {
                                noise ^= noise << 13;
                                noise ^= noise >> 17;
                                noise ^= noise << 5;
                                hold = (noise as f32 / u32::MAX as f32) * 2.0 - 1.0;
                            }
                            lp += (hold - lp) * 0.5;
                            lp
                        }
                    };
                    out[k * per + s] += x * v;
                }
                prev_vol = vol;
            }
        }
        // short fade out against clicks
        let n = out.len();
        for (i, s) in out.iter_mut().enumerate().skip(n.saturating_sub(200)) {
            *s *= (n - i) as f32 / 200.0;
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn every_cue_renders_to_a_short_clean_sound() {
            for cue in ALL {
                let s = render(cue);
                assert!(!s.is_empty(), "{cue:?}");
                assert!(s.len() < SR as usize * 3, "{cue:?} is under three seconds");
                assert!(s.iter().all(|x| x.is_finite() && x.abs() < 2.0), "{cue:?} stays in range");
            }
        }
    }
}

pub struct Audio {
    /// what the music is told each frame; it plays on the same mixer
    music: std::sync::Arc<std::sync::Mutex<crate::music::Ctrl>>,
    /// `--no-music`: effects only
    pub no_music: bool,
    #[cfg(feature = "sound")]
    sink: Option<rodio::MixerDeviceSink>,
    #[cfg(feature = "sound")]
    bank: std::collections::HashMap<Cue, rodio::buffer::SamplesBuffer>,
    /// when the last shot sounded: the gun fires faster than it should be heard
    #[cfg(feature = "sound")]
    last_shot: Option<std::time::Instant>,
    pub muted: bool,
}

impl Audio {
    pub fn new(muted: bool, no_music: bool) -> Audio {
        let music = crate::music::shared();
        #[cfg(feature = "sound")]
        {
            use std::num::NonZero;
            let sink = match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(mut s) => {
                    s.log_on_drop(false);
                    Some(s)
                }
                Err(e) => {
                    log::warn!("no audio: {e}");
                    None
                }
            };
            let bank = if sink.is_some() {
                synth::ALL
                    .iter()
                    .map(|&c| (c, rodio::buffer::SamplesBuffer::new(NonZero::new(1).unwrap(), NonZero::new(synth::SR).unwrap(), synth::render(c))))
                    .collect()
            } else {
                Default::default()
            };
            if let Some(s) = &sink {
                s.mixer().add(rodio::Source::amplify(crate::music::Band::new(music.clone()), 0.5));
            }
            Audio { music, no_music, sink, bank, last_shot: None, muted }
        }
        #[cfg(not(feature = "sound"))]
        Audio { music, no_music, muted }
    }

    /// Steers the music for this frame.
    pub fn music(&mut self, mut ctrl: crate::music::Ctrl) {
        ctrl.muted = self.muted || self.no_music;
        if let Ok(mut c) = self.music.lock() {
            *c = ctrl;
        }
    }

    #[cfg_attr(not(feature = "sound"), allow(unused_variables))]
    pub fn play(&mut self, cue: Cue) {
        #[cfg(feature = "sound")]
        {
            use rodio::Source;
            if self.muted {
                return;
            }
            if cue == Cue::Shot {
                let now = std::time::Instant::now();
                if self.last_shot.is_some_and(|t| now.duration_since(t).as_millis() < 90) {
                    return;
                }
                self.last_shot = Some(now);
            }
            let (Some(sink), Some(b)) = (&self.sink, self.bank.get(&cue)) else { return };
            sink.mixer().add(b.clone().amplify(0.35));
        }
    }
}
