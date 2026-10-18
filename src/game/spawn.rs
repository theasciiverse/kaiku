//! When and what the war sends through the tunnel. For now a paced random
//! script; sectors will author their own later.

use super::enemy::{Enemy, Side};
use super::rng::Rng;

/// Never more ships than this at once.
const MAX_ENEMIES: usize = 14;
/// Seconds until the pace reaches its fastest.
const RAMP: f32 = 60.0;

#[derive(Clone, Debug)]
pub struct Spawner {
    next: f32,
    pub elapsed: f32,
}

impl Default for Spawner {
    fn default() -> Spawner {
        Spawner { next: 1.5, elapsed: 0.0 }
    }
}

impl Spawner {
    pub fn update(&mut self, dt: f32, w: f32, h: f32, rng: &mut Rng, enemies: &mut Vec<Enemy>) {
        self.elapsed += dt;
        self.next -= dt;
        if self.next > 0.0 || enemies.len() >= MAX_ENEMIES {
            return;
        }
        let pace = (self.elapsed / RAMP).min(1.0);
        self.next = rng.range(2.4, 4.0) * (1.0 - 0.45 * pace);
        let side = if rng.unit() < 0.5 { Side::Amber } else { Side::Jade };
        let roll = rng.unit();
        if roll < 0.55 {
            // a snaking line of scouts
            let n = 4 + (rng.unit() * 3.0) as usize;
            let y = rng.range(8.0, (h - 14.0).max(9.0));
            for i in 0..n {
                enemies.push(Enemy::scout(side, w + 2.0 + i as f32 * 14.0, y, i as f32 * 0.6, rng));
            }
        } else if roll < 0.8 || self.elapsed < 20.0 {
            let y = rng.range(0.0, (h - 10.0).max(1.0));
            let hold = rng.range(w * 0.55, w * 0.85);
            enemies.push(Enemy::gunship(side, w + 2.0, y, hold, rng));
        } else {
            // a skirmish: one gunship of each side, high and low, firing at each other
            let (top, low) = (rng.range(0.0, h * 0.3), rng.range(h * 0.6, (h - 10.0).max(h * 0.6)));
            enemies.push(Enemy::gunship(side, w + 2.0, top, rng.range(w * 0.6, w * 0.8), rng));
            enemies.push(Enemy::gunship(side.other(), w + 12.0, low, rng.range(w * 0.6, w * 0.8), rng));
        }
    }
}
