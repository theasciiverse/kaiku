//! Explosion and hit particles. They carry a tint, not a colour: what a tint
//! looks like is the renderer's business.

use super::enemy::Side;
use super::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tint {
    Side(Side),
    Player,
    Spark,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    vx: f32,
    vy: f32,
    /// seconds left, and the total, so the renderer can fade it
    pub life: f32,
    pub max: f32,
    pub tint: Tint,
}

pub fn burst(out: &mut Vec<Particle>, x: f32, y: f32, n: usize, speed: f32, tint: Tint, rng: &mut Rng) {
    for _ in 0..n {
        let a = rng.range(0.0, std::f32::consts::TAU);
        let v = rng.range(0.25, 1.0) * speed;
        let life = rng.range(0.25, 0.7);
        out.push(Particle { x, y, vx: a.cos() * v, vy: a.sin() * v, life, max: life, tint });
    }
}

pub fn update(particles: &mut Vec<Particle>, dt: f32) {
    for p in particles.iter_mut() {
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.vx *= 1.0 - 2.5 * dt;
        p.vy *= 1.0 - 2.5 * dt;
        p.life -= dt;
    }
    particles.retain(|p| p.life > 0.0);
}
