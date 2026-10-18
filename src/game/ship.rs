//! The player's ship. Positions are in Braille dots (2 per cell across, 4 down);
//! the sprite is half-block pixels of 2×2 dots.

use crate::input::Controls;

/// Sprite size in dots (9×5 half-block pixels).
pub const SHIP_W: f32 = 18.0;
pub const SHIP_H: f32 = 10.0;
/// Dots per second. A dot is about square on a 1:2 cell, so one speed serves both axes.
const SPEED: f32 = 80.0;
const FIRE_INTERVAL: f32 = 0.11;

#[derive(Clone, Debug)]
pub struct Ship {
    /// top-left corner, in dots
    pub x: f32,
    pub y: f32,
    pub shield: f32,
    pub lives: u8,
    cooldown: f32,
}

impl Ship {
    pub fn new(h: f32) -> Ship {
        Ship { x: 8.0, y: (h - SHIP_H) / 2.0, shield: 1.0, lives: 3, cooldown: 0.0 }
    }

    /// Moves within the field; returns true when the gun fires this tick.
    pub fn update(&mut self, c: &Controls, dt: f32, w: f32, h: f32) -> bool {
        let (mut dx, mut dy) = (c.dx as f32, c.dy as f32);
        if dx != 0.0 && dy != 0.0 {
            dx *= std::f32::consts::FRAC_1_SQRT_2;
            dy *= std::f32::consts::FRAC_1_SQRT_2;
        }
        self.x += dx * SPEED * dt;
        self.y += dy * SPEED * dt;
        self.clamp(w, h);
        self.cooldown -= dt;
        if c.fire && self.cooldown <= 0.0 {
            self.cooldown = FIRE_INTERVAL;
            return true;
        }
        false
    }

    pub fn clamp(&mut self, w: f32, h: f32) {
        self.x = self.x.clamp(0.0, (w - SHIP_W).max(0.0));
        self.y = self.y.clamp(0.0, (h - SHIP_H).max(0.0));
    }

    /// Where shots leave the ship.
    pub fn nose(&self) -> (f32, f32) {
        (self.x + SHIP_W, self.y + SHIP_H / 2.0)
    }
}
