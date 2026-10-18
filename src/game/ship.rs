//! The player's ship. Positions are in Braille dots (2 per cell across, 4 down);
//! the sprite is half-block pixels of 2×2 dots.

use super::Aabb;
use crate::input::Controls;

/// Sprite size in dots (9×5 half-block pixels).
pub const SHIP_W: f32 = 18.0;
pub const SHIP_H: f32 = 10.0;
/// Dots per second. A dot is about square on a 1:2 cell, so one speed serves both axes.
const SPEED: f32 = 80.0;
/// Shield lost per hit: a bullet takes a quarter, a ram half.
pub const BULLET_DAMAGE: f32 = 0.25;
pub const RAM_DAMAGE: f32 = 0.5;
/// Grace after a shield hit, so one burst isn't four hits at once.
const HIT_GRACE: f32 = 0.4;
/// Invulnerable after losing a life (respawn in place).
const RESPAWN_GRACE: f32 = 2.0;

/// What a hit cost.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hit {
    Shield,
    Life,
    Dead,
}

#[derive(Clone, Debug)]
pub struct Ship {
    /// top-left corner, in dots
    pub x: f32,
    pub y: f32,
    pub shield: f32,
    pub lives: u8,
    /// seconds of invulnerability left
    pub grace: f32,
    cooldown: f32,
}

impl Ship {
    pub fn new(h: f32) -> Ship {
        Ship { x: 8.0, y: (h - SHIP_H) / 2.0, shield: 1.0, lives: 3, grace: 0.0, cooldown: 0.0 }
    }

    /// Moves within the field; returns true when the gun fires this tick, at
    /// most once every `interval` seconds.
    pub fn update(&mut self, c: &Controls, dt: f32, w: f32, h: f32, interval: f32) -> bool {
        let (mut dx, mut dy) = (c.dx as f32, c.dy as f32);
        if dx != 0.0 && dy != 0.0 {
            dx *= std::f32::consts::FRAC_1_SQRT_2;
            dy *= std::f32::consts::FRAC_1_SQRT_2;
        }
        self.x += dx * SPEED * dt;
        self.y += dy * SPEED * dt;
        self.clamp(w, h);
        self.grace = (self.grace - dt).max(0.0);
        self.cooldown -= dt;
        if c.fire && self.cooldown <= 0.0 {
            self.cooldown = interval;
            return true;
        }
        false
    }

    pub fn clamp(&mut self, w: f32, h: f32) {
        self.x = self.x.clamp(0.0, (w - SHIP_W).max(0.0));
        self.y = self.y.clamp(0.0, (h - SHIP_H).max(0.0));
    }

    /// The hitbox: the cockpit and body, much smaller than the sprite.
    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 6.0, self.y + 3.0, self.x + 14.0, self.y + 7.0)
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + SHIP_W / 2.0, self.y + SHIP_H / 2.0)
    }

    /// Takes a hit unless in grace. The shield absorbs hits; once it is empty the
    /// next hit costs a life, and the ship respawns in place with a full shield.
    pub fn hit(&mut self, damage: f32) -> Option<Hit> {
        if self.grace > 0.0 {
            return None;
        }
        if self.shield > 0.0 {
            self.shield = (self.shield - damage).max(0.0);
            self.grace = HIT_GRACE;
            return Some(Hit::Shield);
        }
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            return Some(Hit::Dead);
        }
        self.shield = 1.0;
        self.grace = RESPAWN_GRACE;
        Some(Hit::Life)
    }

    /// Where shots leave the ship.
    pub fn nose(&self) -> (f32, f32) {
        (self.x + SHIP_W, self.y + SHIP_H / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shield_takes_hits_then_lives_go() {
        let mut s = Ship::new(40.0);
        for _ in 0..4 {
            assert_eq!(s.hit(BULLET_DAMAGE), Some(Hit::Shield));
            s.grace = 0.0;
        }
        assert_eq!(s.shield, 0.0);
        assert_eq!(s.hit(BULLET_DAMAGE), Some(Hit::Life), "an empty shield: the next hit costs a life");
        assert_eq!((s.lives, s.shield), (2, 1.0), "respawn in place, shield full");
        assert_eq!(s.hit(BULLET_DAMAGE), None, "invulnerable after respawning");
    }

    #[test]
    fn the_last_life_ends_the_run() {
        let mut s = Ship::new(40.0);
        s.lives = 1;
        s.shield = 0.0;
        assert_eq!(s.hit(RAM_DAMAGE), Some(Hit::Dead));
    }
}
