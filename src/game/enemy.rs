//! Ships of the two warring sides. They fly in from the right, hunt the player
//! and fight each other. Positions and sizes are in dots.

use super::rng::Rng;
use super::Aabb;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Amber,
    Jade,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Amber => Side::Jade,
            Side::Jade => Side::Amber,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// small, fast, flies through in snaking lines
    Scout,
    /// holds position on the right and fires
    Gunship,
}

impl Kind {
    /// Sprite size in dots (half-block pixels are 2×2 dots).
    pub fn size(self) -> (f32, f32) {
        match self {
            Kind::Scout => (10.0, 6.0),
            Kind::Gunship => (14.0, 10.0),
        }
    }

    fn hp(self) -> u8 {
        match self {
            Kind::Scout => 1,
            Kind::Gunship => 4,
        }
    }

    pub fn points(self) -> u32 {
        match self {
            Kind::Scout => 100,
            Kind::Gunship => 400,
        }
    }
}

const SCOUT_SPEED: f32 = 38.0;
const LEAVE_SPEED: f32 = 60.0;
/// A gunship holds its position this long, then leaves.
const GUNSHIP_STAY: f32 = 9.0;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub kind: Kind,
    pub side: Side,
    pub x: f32,
    pub y: f32,
    pub hp: u8,
    /// seconds of white hit flash left
    pub flash: f32,
    base_y: f32,
    phase: f32,
    age: f32,
    hold_x: f32,
    fire_in: f32,
}

impl Enemy {
    pub fn scout(side: Side, x: f32, y: f32, phase: f32, rng: &mut Rng) -> Enemy {
        Enemy::new(Kind::Scout, side, x, y, phase, x, rng)
    }

    pub fn gunship(side: Side, x: f32, y: f32, hold_x: f32, rng: &mut Rng) -> Enemy {
        let phase = rng.range(0.0, std::f32::consts::TAU);
        Enemy::new(Kind::Gunship, side, x, y, phase, hold_x, rng)
    }

    fn new(kind: Kind, side: Side, x: f32, y: f32, phase: f32, hold_x: f32, rng: &mut Rng) -> Enemy {
        Enemy { kind, side, x, y, hp: kind.hp(), flash: 0.0, base_y: y, phase, age: 0.0, hold_x, fire_in: first_shot(kind, rng) }
    }

    pub fn aabb(&self) -> Aabb {
        let (w, h) = self.kind.size();
        // a little smaller than the sprite, so grazes don't count
        Aabb::new(self.x + 1.0, self.y + 1.0, self.x + w - 1.0, self.y + h - 1.0)
    }

    /// Where shots leave the ship (its nose, on the left).
    pub fn muzzle(&self) -> (f32, f32) {
        (self.x, self.y + self.kind.size().1 / 2.0)
    }

    pub fn gone(&self) -> bool {
        self.x + self.kind.size().0 < -2.0
    }

    /// Moves; returns true when the ship fires this tick.
    pub fn update(&mut self, dt: f32, w: f32, h: f32, rng: &mut Rng) -> bool {
        self.age += dt;
        self.flash = (self.flash - dt).max(0.0);
        let (_, sh) = self.kind.size();
        match self.kind {
            Kind::Scout => {
                self.x -= SCOUT_SPEED * dt;
                self.y = self.base_y + (self.phase + self.age * 2.2).sin() * 7.0;
            }
            Kind::Gunship => {
                if self.age < GUNSHIP_STAY {
                    self.x += (self.hold_x - self.x) * (1.5 * dt).min(1.0);
                } else {
                    self.x -= LEAVE_SPEED * dt;
                }
                self.y = self.base_y + (self.phase + self.age * 0.9).sin() * 5.0;
            }
        }
        self.y = self.y.clamp(0.0, (h - sh).max(0.0));
        self.fire_in -= dt;
        // only fire once fully on screen
        if self.fire_in <= 0.0 && self.x + self.kind.size().0 < w {
            self.fire_in = match self.kind {
                Kind::Scout => rng.range(2.5, 5.0),
                Kind::Gunship => rng.range(1.0, 1.8),
            };
            return true;
        }
        false
    }
}

fn first_shot(kind: Kind, rng: &mut Rng) -> f32 {
    match kind {
        Kind::Scout => rng.range(1.5, 4.0),
        Kind::Gunship => rng.range(1.2, 2.0),
    }
}
