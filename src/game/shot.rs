//! Bullets, in dots.

use super::enemy::Side;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Player,
    /// Hurts the player and the other side's ships, never its own.
    Enemy(Side),
}

#[derive(Clone, Debug)]
pub struct Shot {
    pub x: f32,
    pub y: f32,
    /// dots per second
    pub vx: f32,
    pub vy: f32,
    pub owner: Owner,
    /// set when it has hit something
    pub spent: bool,
}

impl Shot {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, owner: Owner) -> Shot {
        Shot { x, y, vx, vy, owner, spent: false }
    }
}

/// Moves shots: the player's at `dt`, enemy fire at `enemy_dt` (slow time).
pub fn update(shots: &mut Vec<Shot>, dt: f32, enemy_dt: f32, w: f32, h: f32) {
    for s in shots.iter_mut() {
        let dt = if s.owner == Owner::Player { dt } else { enemy_dt };
        s.x += s.vx * dt;
        s.y += s.vy * dt;
    }
    shots.retain(|s| !s.spent && s.x >= -4.0 && s.x <= w + 4.0 && s.y >= -4.0 && s.y <= h + 4.0);
}
