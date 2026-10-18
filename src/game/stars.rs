//! The starfield: Braille dots drifting left in three parallax layers. The scroll
//! speed is the ship's speed through the tunnel, so it is game state, not decoration.

use super::rng::Rng;

/// Dots per second for each layer, far to near.
const LAYER_SPEED: [f32; 3] = [10.0, 24.0, 55.0];

#[derive(Clone, Debug)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    /// 0 far .. 2 near
    pub layer: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Starfield {
    pub stars: Vec<Star>,
}

impl Starfield {
    pub fn new(w: f32, h: f32, rng: &mut Rng) -> Starfield {
        // about one star per 40 cells
        let count = ((w * h) / (8.0 * 40.0)).max(8.0) as usize;
        let stars = (0..count)
            .map(|i| Star { x: rng.range(0.0, w), y: rng.range(0.0, h), layer: [0, 0, 0, 1, 1, 2][i % 6] })
            .collect();
        Starfield { stars }
    }

    pub fn update(&mut self, dt: f32, speed: f32, w: f32, h: f32, rng: &mut Rng) {
        for s in &mut self.stars {
            s.x -= LAYER_SPEED[s.layer as usize] * speed * dt;
            if s.x < 0.0 {
                s.x += w;
                s.y = rng.range(0.0, h);
            }
        }
    }
}
