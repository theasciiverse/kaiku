//! When and what the war sends through the tunnel: a paced random script whose
//! menu grows sector by sector. Sector 1 is scouts, gunships, skirmishes and
//! interceptors; from sector 2 mines, lancers and jammers join; from sector 3
//! carriers. Each sector also runs a little faster than the last.

use super::enemy::{Enemy, Kind, Side};
use super::rng::Rng;

/// Never more ships than this at once (a carrier's launches aside).
const MAX_ENEMIES: usize = 14;
/// Seconds until the pace within a sector reaches its fastest.
const RAMP: f32 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Wave {
    Scouts,
    Gunship,
    Skirmish,
    Interceptors,
    Mines,
    Lancer,
    Jammer,
    Carrier,
}

impl Wave {
    /// How often it comes, and from which sector on.
    fn weight(self, level: u32, elapsed: f32) -> f32 {
        let (from, weight) = match self {
            Wave::Scouts => (1, 3.0),
            Wave::Gunship => (1, 1.6),
            Wave::Skirmish if elapsed >= 20.0 => (1, 0.9),
            Wave::Interceptors if elapsed >= 15.0 => (1, 1.0),
            Wave::Mines => (2, 1.0),
            Wave::Lancer => (2, 0.8),
            // jammers come while the echoes are about
            Wave::Jammer if (8.0..45.0).contains(&elapsed) => (2, 0.7),
            Wave::Carrier if elapsed >= 10.0 => (3, 0.5),
            _ => return 0.0,
        };
        if level >= from {
            weight
        } else {
            0.0
        }
    }

    /// Waves that come at most once a sector.
    fn once(self) -> bool {
        matches!(self, Wave::Jammer | Wave::Carrier)
    }
}

const WAVES: [Wave; 8] = [Wave::Scouts, Wave::Gunship, Wave::Skirmish, Wave::Interceptors, Wave::Mines, Wave::Lancer, Wave::Jammer, Wave::Carrier];

#[derive(Clone, Debug)]
pub struct Spawner {
    next: f32,
    pub elapsed: f32,
    sent: Vec<Wave>,
}

impl Default for Spawner {
    fn default() -> Spawner {
        Spawner { next: 1.5, elapsed: 0.0, sent: Vec::new() }
    }
}

impl Spawner {
    /// `level` is the sector number: it decides the menu and the pace.
    pub fn update(&mut self, dt: f32, w: f32, h: f32, level: u32, rng: &mut Rng, enemies: &mut Vec<Enemy>) {
        self.elapsed += dt;
        self.next -= dt;
        if self.next > 0.0 || enemies.len() >= MAX_ENEMIES {
            return;
        }
        let pace = (self.elapsed / RAMP).min(1.0);
        let faster = (1.0 - 0.06 * level.saturating_sub(1) as f32).max(0.7);
        self.next = rng.range(2.4, 4.0) * (1.0 - 0.45 * pace) * faster;
        let menu: Vec<(Wave, f32)> = WAVES
            .iter()
            .filter(|w| !(w.once() && self.sent.contains(w)))
            .map(|&w| (w, w.weight(level, self.elapsed)))
            .filter(|(_, wt)| *wt > 0.0)
            .collect();
        let total: f32 = menu.iter().map(|m| m.1).sum();
        let mut roll = rng.unit() * total;
        let Some(&(wave, _)) = menu.iter().find(|(_, wt)| {
            roll -= wt;
            roll <= 0.0
        }) else {
            return;
        };
        self.sent.push(wave);
        log::debug!("wave {wave:?} at {:.0}s", self.elapsed);
        send(wave, w, h, rng, enemies);
    }
}

fn send(wave: Wave, w: f32, h: f32, rng: &mut Rng, enemies: &mut Vec<Enemy>) {
    let side = if rng.unit() < 0.5 { Side::Amber } else { Side::Jade };
    let row = |rng: &mut Rng, size: f32| rng.range(2.0, (h - size - 2.0).max(3.0));
    match wave {
        Wave::Scouts => {
            // a snaking line
            let n = 4 + (rng.unit() * 3.0) as usize;
            let y = rng.range(8.0, (h - 14.0).max(9.0));
            for i in 0..n {
                enemies.push(Enemy::scout(side, w + 2.0 + i as f32 * 14.0, y, i as f32 * 0.6, rng));
            }
        }
        Wave::Gunship => {
            let (y, hold) = (row(rng, 10.0), rng.range(w * 0.55, w * 0.85));
            enemies.push(Enemy::gunship(side, w + 2.0, y, hold, rng));
        }
        Wave::Skirmish => {
            // one gunship of each side, high and low, firing at each other
            let (top, low) = (rng.range(0.0, h * 0.3), rng.range(h * 0.6, (h - 10.0).max(h * 0.6)));
            enemies.push(Enemy::gunship(side, w + 2.0, top, rng.range(w * 0.6, w * 0.8), rng));
            enemies.push(Enemy::gunship(side.other(), w + 12.0, low, rng.range(w * 0.6, w * 0.8), rng));
        }
        Wave::Interceptors => {
            for i in 0..2 + (rng.unit() * 2.0) as usize {
                let (y, hold) = (row(rng, 6.0), rng.range(w * 0.55, w * 0.8));
                enemies.push(Enemy::of(Kind::Interceptor, side, w + 2.0 + i as f32 * 30.0, y, hold, rng));
            }
        }
        Wave::Mines => {
            // a loose field across the tunnel
            for i in 0..4 + (rng.unit() * 3.0) as usize {
                let y = row(rng, 6.0);
                enemies.push(Enemy::of(Kind::Mine, side, w + 2.0 + i as f32 * 18.0 + rng.range(0.0, 10.0), y, 0.0, rng));
            }
        }
        Wave::Lancer => {
            let (y, hold) = (row(rng, 8.0), rng.range(w * 0.78, w * 0.9));
            enemies.push(Enemy::of(Kind::Lancer, side, w + 2.0, y, hold, rng));
        }
        Wave::Jammer => {
            // high or low, out of the main stream
            let y = if rng.unit() < 0.5 { 2.0 } else { (h - 12.0).max(2.0) };
            enemies.push(Enemy::of(Kind::Jammer, side, w + 2.0, y, rng.range(w * 0.65, w * 0.85), rng));
        }
        Wave::Carrier => {
            let y = rng.range(h * 0.2, (h * 0.8 - 16.0).max(h * 0.2));
            enemies.push(Enemy::of(Kind::Carrier, side, w + 2.0, y, 0.0, rng));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(level: u32, seed: u64) -> Vec<Kind> {
        let mut s = Spawner::default();
        let mut rng = Rng::new(seed);
        let mut seen = Vec::new();
        for _ in 0..60 * 58 {
            let mut enemies = Vec::new();
            s.update(1.0 / 60.0, 200.0, 80.0, level, &mut rng, &mut enemies);
            seen.extend(enemies.iter().map(|e| e.kind));
        }
        seen
    }

    #[test]
    fn the_menu_grows_sector_by_sector() {
        let one: Vec<Kind> = (0..6).flat_map(|seed| kinds(1, seed)).collect();
        assert!(!one.iter().any(|k| matches!(k, Kind::Mine | Kind::Lancer | Kind::Jammer | Kind::Carrier)), "sector 1 stays simple");
        let three: Vec<Kind> = (0..6).flat_map(|seed| kinds(3, seed)).collect();
        for k in [Kind::Mine, Kind::Lancer, Kind::Jammer, Kind::Carrier] {
            assert!(three.contains(&k), "{k:?} by sector 3");
        }
    }

    #[test]
    fn jammers_and_carriers_come_once_a_sector() {
        for seed in 0..6 {
            let k = kinds(5, seed);
            assert!(k.iter().filter(|k| **k == Kind::Jammer).count() <= 1);
            assert!(k.iter().filter(|k| **k == Kind::Carrier).count() <= 1);
        }
    }
}
