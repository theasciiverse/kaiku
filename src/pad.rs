//! The gamepad: the pilot's controls, so one player can fly while another
//! types. Gamepads are first-class in Asciiverse (uXo's rule); this follows
//! Ceres: gilrs, only SDL-mapped devices (`KAIKU_ANY_PAD=1` accepts any), pads
//! hot-plug. Left stick or d-pad flies, the stick in proportion; South (A)
//! fires; East (B) and the bumpers drop a smart bomb; Start pauses. A also
//! confirms: on the map, and to fly again after a run.

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
    pub confirm: bool,
    pub pause: bool,
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
            name: None,
            state: PadState::default(),
            presses: PadPresses::default(),
        };
        #[cfg(feature = "gamepad")]
        match gilrs::Gilrs::new() {
            Ok(g) => {
                let all: Vec<_> = g.gamepads().map(|(_, p)| (p.name().to_string(), usable(&p))).collect();
                log::info!("gamepads (name, used): {all:?}");
                pad.name = all.into_iter().find(|(_, used)| *used).map(|(n, _)| n);
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
                        self.name = Some(pad.name().to_string());
                        log::info!("gamepad connected: {:?}", self.name);
                    }
                    EventType::Disconnected => {
                        log::info!("gamepad disconnected");
                        self.name = None;
                        self.state = PadState::default();
                        continue;
                    }
                    EventType::ButtonPressed(b, _) => {
                        log::debug!("pad press {b:?}");
                        match b {
                            Button::South => self.presses.confirm = true,
                            Button::East | Button::LeftTrigger | Button::RightTrigger => self.presses.bomb = true,
                            Button::Start => self.presses.pause = true,
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
