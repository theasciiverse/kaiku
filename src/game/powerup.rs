//! Power-up pods: capsules drifting through the tunnel, each carrying a word.
//! Type the word to claim the power-up; shoot the pod and it's gone. The words
//! follow a typing tutor's curve: home row first, then the top row, the bottom
//! row, then long words across the whole board.

use super::rng::Rng;
use super::Aabb;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// In the order the panel shows them.
pub enum PowerUp {
    Spread,
    Rapid,
    Shield,
    Bomb,
    Drone,
    Slow,
}

impl PowerUp {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        ["spread shot", "rapid fire", "shield", "smart bomb", "drone", "slow time"][self.index()]
    }
}

/// Words for a sector: its keyboard level from `data/words.txt`.
pub fn words(sector: u32) -> Vec<&'static str> {
    let level = match sector {
        0 | 1 => "home",
        2 => "top",
        3 => "bottom",
        _ => "all",
    };
    let mut current = "";
    let mut out = Vec::new();
    for line in include_str!("../../data/words.txt").lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = name;
        } else if current == level {
            out.push(line);
        }
    }
    out
}

/// Pod sprite size in dots (4×3 half-block pixels).
pub const POD_W: f32 = 8.0;
pub const POD_H: f32 = 6.0;
const DRIFT: f32 = 16.0;
/// Shots a pod survives: enough to notice and ease off the trigger.
pub const POD_HP: u8 = 3;
const DRIFT_LOCKED: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct Pod {
    pub id: u32,
    pub kind: PowerUp,
    pub word: &'static str,
    pub progress: usize,
    pub locked: bool,
    pub flash: f32,
    pub x: f32,
    pub y: f32,
    base_y: f32,
    age: f32,
    /// hits it can still take before it's lost
    pub hp: u8,
    /// claimed, shot or drifted off
    pub done: bool,
}

impl Pod {
    pub fn new(id: u32, kind: PowerUp, word: &'static str, x: f32, y: f32) -> Pod {
        Pod { id, kind, word, progress: 0, locked: false, flash: 0.0, x, y, base_y: y, age: 0.0, hp: POD_HP, done: false }
    }

    pub fn next_letter(&self) -> Option<char> {
        self.word.chars().nth(self.progress)
    }

    /// Feeds one typed letter; returns true if it was the right one.
    pub fn key(&mut self, c: char) -> bool {
        if self.next_letter() == Some(c) {
            self.progress += 1;
            true
        } else {
            self.flash = 0.15;
            false
        }
    }

    /// A shot hits it; returns true when that one breaks it.
    pub fn shot(&mut self) -> bool {
        self.hp = self.hp.saturating_sub(1);
        self.flash = 0.1;
        if self.hp == 0 {
            self.done = true;
        }
        self.done
    }

    pub fn claimed(&self) -> bool {
        self.progress == self.word.len()
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x, self.y, self.x + POD_W, self.y + POD_H)
    }

    pub fn update(&mut self, dt: f32, h: f32) {
        self.age += dt;
        self.flash = (self.flash - dt).max(0.0);
        self.x -= if self.locked { DRIFT_LOCKED } else { DRIFT } * dt;
        self.y = (self.base_y + (self.age * 1.6).sin() * 4.0).clamp(0.0, h - POD_H);
        if self.x + POD_W < 0.0 {
            self.done = true;
        }
    }
}

/// Seconds between pods, and which power-ups come how often.
const EVERY: (f32, f32) = (11.0, 17.0);
const WEIGHTS: [(PowerUp, f32); 6] = [
    (PowerUp::Spread, 1.0),
    (PowerUp::Rapid, 1.0),
    (PowerUp::Shield, 1.4),
    (PowerUp::Bomb, 0.9),
    (PowerUp::Drone, 0.7),
    (PowerUp::Slow, 0.6),
];

#[derive(Clone, Debug)]
pub struct PodSpawner {
    next: f32,
    next_id: u32,
}

impl Default for PodSpawner {
    fn default() -> PodSpawner {
        PodSpawner { next: 6.0, next_id: 1 }
    }
}

impl PodSpawner {
    /// Spawns a pod when it's time. `taken` are the letters live typing targets
    /// want next: the pod's word starts with none of them.
    pub fn update(&mut self, dt: f32, w: f32, h: f32, sector: u32, taken: &[char], rng: &mut Rng) -> Option<Pod> {
        self.next -= dt;
        if self.next > 0.0 {
            return None;
        }
        self.next = rng.range(EVERY.0, EVERY.1);
        let pool: Vec<&str> = words(sector).into_iter().filter(|w| !w.chars().next().is_some_and(|c| taken.contains(&c))).collect();
        if pool.is_empty() {
            return None;
        }
        let word = pool[(rng.unit() * pool.len() as f32) as usize % pool.len()];
        let total: f32 = WEIGHTS.iter().map(|w| w.1).sum();
        let mut roll = rng.unit() * total;
        let kind = WEIGHTS.iter().find(|(_, wt)| {
            roll -= wt;
            roll <= 0.0
        });
        let kind = kind.map_or(PowerUp::Shield, |k| k.0);
        let y = rng.range(4.0, (h - POD_H - 8.0).max(5.0));
        self.next_id += 1;
        log::info!("pod {kind:?} {word:?}");
        Some(Pod::new(10_000 + self.next_id, kind, word, w + 2.0, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "asdfghjkl";
    const TOP: &str = "qwertyuiop";

    #[test]
    fn words_follow_the_keyboard_rows() {
        assert!(words(1).iter().all(|w| w.chars().all(|c| HOME.contains(c))), "sector 1: home row only");
        assert!(words(2).iter().all(|w| w.chars().all(|c| HOME.contains(c) || TOP.contains(c))), "sector 2: plus the top row");
        assert!(words(3).iter().any(|w| w.chars().any(|c| "zxcvbnm".contains(c))));
        assert!(words(9).iter().all(|w| w.len() >= 9), "then long words");
    }

    #[test]
    fn a_pod_never_shares_a_first_letter_with_a_live_target() {
        let mut s = PodSpawner::default();
        let mut rng = Rng::new(3);
        for _ in 0..50 {
            s.next = 0.0;
            let taken = ['s', 'd', 'f', 'g'];
            if let Some(p) = s.update(0.1, 160.0, 72.0, 1, &taken, &mut rng) {
                assert!(!taken.contains(&p.word.chars().next().unwrap()), "{}", p.word);
            }
        }
    }

    #[test]
    fn typing_the_word_claims_the_pod() {
        let mut p = Pod::new(1, PowerUp::Rapid, "flask", 50.0, 10.0);
        for c in "flasx".chars() {
            p.key(c);
        }
        assert!(!p.claimed());
        p.key('k');
        assert!(p.claimed());
    }
}

/// Seconds slow time lasts, and how slow the war runs meanwhile.
pub const SLOW_TIME: f32 = 8.0;
pub const SLOW_FACTOR: f32 = 0.45;
pub const MAX_BOMBS: u8 = 3;

/// An escort drone: hovers above the ship and fires forward.
#[derive(Clone, Debug)]
pub struct Drone {
    pub x: f32,
    pub y: f32,
    cooldown: f32,
}

impl Drone {
    pub fn new(x: f32, y: f32) -> Drone {
        Drone { x, y, cooldown: 0.0 }
    }

    /// Follows its station above the ship; returns true when it fires.
    pub fn update(&mut self, dt: f32, ship: (f32, f32)) -> bool {
        let (tx, ty) = (ship.0 + 4.0, (ship.1 - 10.0).max(0.0));
        self.x += (tx - self.x) * (6.0 * dt).min(1.0);
        self.y += (ty - self.y) * (6.0 * dt).min(1.0);
        self.cooldown -= dt;
        if self.cooldown <= 0.0 {
            self.cooldown = 0.32;
            return true;
        }
        false
    }
}

/// What the ship has. Weapon power-ups (spread, rapid, drone) stack; losing a
/// life costs the newest one, never all of them.
#[derive(Clone, Debug, Default)]
pub struct Powers {
    pub spread: bool,
    pub rapid: bool,
    pub drone: Option<Drone>,
    pub bombs: u8,
    /// seconds of slow time left
    pub slow: f32,
    /// seconds the shield icon stays lit after a refill
    pub shield_glow: f32,
    stack: Vec<PowerUp>,
}

impl Powers {
    /// Lit icons in the panel, in `PowerUp` order.
    pub fn owned(&self) -> [bool; 6] {
        [self.spread, self.rapid, self.shield_glow > 0.0, self.bombs > 0, self.drone.is_some(), self.slow > 0.0]
    }

    /// Takes a claimed power-up. Shield refills are the caller's (the ship's).
    pub fn take(&mut self, kind: PowerUp, ship: (f32, f32)) {
        match kind {
            PowerUp::Spread => self.spread = true,
            PowerUp::Rapid => self.rapid = true,
            PowerUp::Drone => self.drone = self.drone.take().or(Some(Drone::new(ship.0, ship.1))),
            PowerUp::Bomb => self.bombs = (self.bombs + 1).min(MAX_BOMBS),
            PowerUp::Slow => self.slow = SLOW_TIME,
            PowerUp::Shield => self.shield_glow = 1.5,
        }
        if matches!(kind, PowerUp::Spread | PowerUp::Rapid | PowerUp::Drone) {
            self.stack.retain(|&k| k != kind);
            self.stack.push(kind);
        }
    }

    /// A life lost: the newest weapon power-up goes.
    pub fn lose_one(&mut self) -> Option<PowerUp> {
        let kind = self.stack.pop()?;
        match kind {
            PowerUp::Spread => self.spread = false,
            PowerUp::Rapid => self.rapid = false,
            PowerUp::Drone => self.drone = None,
            _ => {}
        }
        Some(kind)
    }

    pub fn tick(&mut self, dt: f32) {
        self.slow = (self.slow - dt).max(0.0);
        self.shield_glow = (self.shield_glow - dt).max(0.0);
    }
}

#[cfg(test)]
mod power_tests {
    use super::*;

    #[test]
    fn a_lost_life_costs_only_the_newest_weapon() {
        let mut p = Powers::default();
        p.take(PowerUp::Spread, (0.0, 0.0));
        p.take(PowerUp::Drone, (0.0, 0.0));
        p.take(PowerUp::Bomb, (0.0, 0.0));
        assert_eq!(p.lose_one(), Some(PowerUp::Drone));
        assert!(p.spread && p.drone.is_none() && p.bombs == 1, "bombs and older weapons stay");
        assert_eq!(p.owned(), [true, false, false, true, false, false]);
    }
}
