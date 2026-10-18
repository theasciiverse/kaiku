//! The simulation: no terminal in here, so it runs headless in tests.

pub mod rng;
pub mod ship;
pub mod shot;
pub mod stars;

use crate::input::Controls;
use rng::Rng;
use ship::Ship;
use shot::Shot;
use stars::Starfield;

/// Fixed simulation step: 60 Hz, as in Entity.
pub const DT: f32 = 1.0 / 60.0;
const SHOT_SPEED: f32 = 260.0;

pub struct Game {
    /// field size in dots
    pub w: f32,
    pub h: f32,
    pub ship: Ship,
    pub shots: Vec<Shot>,
    pub stars: Starfield,
    pub tick: u64,
    rng: Rng,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game { w: 0.0, h: 0.0, ship: Ship::new(0.0), shots: Vec::new(), stars: Starfield::default(), tick: 0, rng: Rng::new(seed) }
    }

    /// Fits the field to the play area, in cells. The first fit places the ship.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let (w, h) = (cols as f32 * 2.0, rows as f32 * 4.0);
        if (w, h) == (self.w, self.h) {
            return;
        }
        let first = self.w == 0.0;
        self.w = w;
        self.h = h;
        self.stars = Starfield::new(w, h, &mut self.rng);
        if first {
            self.ship = Ship::new(h);
        }
        self.ship.clamp(w, h);
        log::info!("field {cols}x{rows} cells");
    }

    pub fn update(&mut self, c: &Controls) {
        self.tick += 1;
        if self.ship.update(c, DT, self.w, self.h) {
            let (x, y) = self.ship.nose();
            self.shots.push(Shot { x, y, vx: SHOT_SPEED });
        }
        shot::update(&mut self.shots, DT, self.w);
        self.stars.update(DT, 1.0, self.w, self.h, &mut self.rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ship_stays_in_the_field_and_fires() {
        let mut g = Game::new(7);
        g.resize(80, 20);
        let c = Controls { dx: 1, dy: -1, fire: true };
        for _ in 0..30 {
            g.update(&c);
        }
        assert!(!g.shots.is_empty(), "held fire shoots");
        for _ in 0..600 {
            g.update(&c);
        }
        assert!(g.ship.x + ship::SHIP_W <= g.w && g.ship.y >= 0.0);
        assert!(g.shots.iter().all(|s| s.x <= g.w + 4.0));
    }
}
