//! Bullets, in dots.

#[derive(Clone, Debug)]
pub struct Shot {
    pub x: f32,
    pub y: f32,
    /// dots per second (positive: rightwards, the player's)
    pub vx: f32,
}

pub fn update(shots: &mut Vec<Shot>, dt: f32, w: f32) {
    for s in shots.iter_mut() {
        s.x += s.vx * dt;
    }
    shots.retain(|s| s.x >= -4.0 && s.x <= w + 4.0);
}
