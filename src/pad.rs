//! The gamepad: the pilot's controls, so one player can fly while another
//! types. Gamepads are first-class in Asciiverse (uXo's rule); this follows
//! Ceres: gilrs, only SDL-mapped devices (`KAIKU_ANY_PAD=1` accepts any), pads
//! hot-plug. Buttons go by position (gilrs' South is Xbox A but Switch B).
//! Left stick or d-pad flies, the stick in proportion; South fires; East or
//! the left bumper drops a smart bomb; West or the right bumper dashes; Start
//! pauses. South also confirms: on the map, and to fly again after a run.

/// Stick deadzone, as in uXo.
#[cfg_attr(not(feature = "gamepad"), allow(dead_code))]
const DEADZONE: f32 = 0.3;

/// What the pad wants this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadState {
    /// stick or d-pad, each axis -1..1 (up is negative, as on screen)
    pub x: f32,
    pub y: f32,
    pub fire: bool,
}

/// One-shot presses since the game last read them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadPresses {
    pub bomb: bool,
    pub dash: bool,
    pub confirm: bool,
    pub pause: bool,
    pub help: bool,
}

/// A stick axis with the deadzone taken out and the rest rescaled, so a
/// small push past the deadzone is a slow drift and full tilt full speed.
#[cfg_attr(not(feature = "gamepad"), allow(dead_code))]
pub fn shape(v: f32) -> f32 {
    let a = v.abs();
    if a < DEADZONE {
        0.0
    } else {
        v.signum() * ((a - DEADZONE) / (1.0 - DEADZONE)).min(1.0)
    }
}

pub struct Pad {
    #[cfg(feature = "gamepad")]
    gilrs: Option<gilrs::Gilrs>,
    /// the pad in use, for rumble
    #[cfg(feature = "gamepad")]
    id: Option<gilrs::GamepadId>,
    /// effects still playing: dropping one stops it
    #[cfg(feature = "gamepad")]
    rumbles: Vec<(gilrs::ff::Effect, std::time::Instant)>,
    /// the pad in use, by name, while one is connected
    pub name: Option<String>,
    pub state: PadState,
    presses: PadPresses,
}

#[cfg(feature = "gamepad")]
fn usable(p: &gilrs::Gamepad) -> bool {
    p.mapping_source() == gilrs::MappingSource::SdlMappings || std::env::var("KAIKU_ANY_PAD").is_ok_and(|v| v == "1")
}

impl Pad {
    pub fn open() -> Pad {
        #[cfg_attr(not(feature = "gamepad"), allow(unused_mut))]
        let mut pad = Pad {
            #[cfg(feature = "gamepad")]
            gilrs: None,
            #[cfg(feature = "gamepad")]
            id: None,
            #[cfg(feature = "gamepad")]
            rumbles: Vec::new(),
            name: None,
            state: PadState::default(),
            presses: PadPresses::default(),
        };
        #[cfg(feature = "gamepad")]
        match gilrs::Gilrs::new() {
            Ok(g) => {
                let all: Vec<_> = g.gamepads().map(|(id, p)| (id, p.name().to_string(), usable(&p), p.is_ff_supported())).collect();
                log::info!("gamepads (id, name, used, rumble): {all:?}");
                if let Some((id, name, _, _)) = all.into_iter().find(|p| p.2) {
                    pad.id = Some(id);
                    pad.name = Some(name);
                }
                pad.gilrs = Some(g);
            }
            Err(e) => log::warn!("gamepad support unavailable: {e}"),
        }
        pad
    }

    /// Drains pad events; call once per frame, before reading the state.
    pub fn poll(&mut self) {
        #[cfg(feature = "gamepad")]
        {
            use gilrs::{Axis, Button, EventType};
            let Some(g) = &mut self.gilrs else { return };
            while let Some(ev) = g.next_event() {
                let pad = g.gamepad(ev.id);
                if !usable(&pad) {
                    continue;
                }
                match ev.event {
                    EventType::Connected => {
                        self.id = Some(ev.id);
                        self.name = Some(pad.name().to_string());
                        log::info!("gamepad connected: {:?}", self.name);
                    }
                    EventType::Disconnected => {
                        log::info!("gamepad disconnected");
                        self.id = None;
                        self.name = None;
                        self.state = PadState::default();
                        continue;
                    }
                    EventType::ButtonPressed(b, _) => {
                        log::debug!("pad press {b:?}");
                        match b {
                            Button::South => self.presses.confirm = true,
                            Button::East | Button::LeftTrigger => self.presses.bomb = true,
                            Button::West | Button::RightTrigger => self.presses.dash = true,
                            Button::Start => self.presses.pause = true,
                            Button::Select => self.presses.help = true,
                            _ => {}
                        }
                    }
                    _ => {}
                }
                let on = |b: Button| pad.is_pressed(b);
                let axis = |a: Axis| pad.axis_data(a).map_or(0.0, |d| d.value());
                let dpad = |neg: Button, pos: Button| on(pos) as i8 as f32 - on(neg) as i8 as f32;
                // gilrs' stick y is up-positive; the screen's is down-positive
                let (sx, sy) = (shape(axis(Axis::LeftStickX)), -shape(axis(Axis::LeftStickY)));
                let (dx, dy) = (dpad(Button::DPadLeft, Button::DPadRight), dpad(Button::DPadUp, Button::DPadDown));
                self.state = PadState {
                    x: if dx != 0.0 { dx } else { sx },
                    y: if dy != 0.0 { dy } else { sy },
                    fire: on(Button::South),
                };
            }
        }
    }

    /// Rumbles the pad: `strong` and `weak` motors 0 to 1, for `ms`, as uXo
    /// does. Nothing happens without a pad that can.
    #[cfg_attr(not(feature = "gamepad"), allow(unused_variables))]
    pub fn rumble(&mut self, strong: f32, weak: f32, ms: u32) {
        #[cfg(feature = "gamepad")]
        {
            use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
            let now = std::time::Instant::now();
            self.rumbles.retain(|(_, until)| *until > now);
            let (Some(id), Some(g)) = (self.id, &mut self.gilrs) else { return };
            if !g.gamepad(id).is_ff_supported() {
                return;
            }
            let dur = Ticks::from_ms(ms);
            let mag = |v: f32| (v.clamp(0.0, 1.0) * u16::MAX as f32) as u16;
            let built = EffectBuilder::new()
                .add_effect(BaseEffect { kind: BaseEffectType::Strong { magnitude: mag(strong) }, scheduling: Replay { play_for: dur, ..Default::default() }, envelope: Default::default() })
                .add_effect(BaseEffect { kind: BaseEffectType::Weak { magnitude: mag(weak) }, scheduling: Replay { play_for: dur, ..Default::default() }, envelope: Default::default() })
                .repeat(gilrs::ff::Repeat::For(dur))
                .gamepads(&[id])
                .finish(g);
            match built.and_then(|e| e.play().map(|_| e)) {
                Ok(e) => self.rumbles.push((e, now + std::time::Duration::from_millis(ms as u64 + 50))),
                Err(e) => log::debug!("rumble failed: {e}"),
            }
        }
    }

    /// What the pilot feels: hits and losses hard, the bomb and the jump big,
    /// and a small pulse when comms decodes an echo or claims a power-up, so
    /// the pilot feels the other player's work.
    pub fn feel(&mut self, cue: crate::game::cue::Cue) {
        use crate::game::cue::Cue;
        let (strong, weak, ms) = match cue {
            Cue::ShieldHit => (0.45, 0.3, 120),
            Cue::LifeLost => (1.0, 0.6, 380),
            Cue::GameOver => (1.0, 1.0, 700),
            Cue::Bomb => (0.8, 0.8, 350),
            Cue::Jump => (0.35, 0.5, 300),
            Cue::BigKill => (0.3, 0.2, 110),
            Cue::Beam => (0.2, 0.0, 200),
            Cue::Dash => (0.0, 0.4, 80),
            Cue::Decoded | Cue::Claimed => (0.15, 0.5, 90),
            Cue::ExtraLife => (0.0, 0.6, 200),
            Cue::Cell => (0.0, 0.25, 50),
            _ => return,
        };
        self.rumble(strong, weak, ms);
    }

    /// The one-shot presses since the last call.
    pub fn take_presses(&mut self) -> PadPresses {
        std::mem::take(&mut self.presses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stick_has_a_deadzone_then_scales_to_full() {
        assert_eq!(shape(0.2), 0.0);
        assert_eq!(shape(-0.29), 0.0);
        assert!((shape(0.65) - 0.5).abs() < 1e-6, "halfway past the deadzone is half speed");
        assert_eq!(shape(1.0), 1.0);
        assert_eq!(shape(-1.0), -1.0);
    }
}
