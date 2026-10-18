//! Tunnel walls: jagged bands along the top and bottom of the field, scrolling
//! with the flight. Each sector has its own style (open space in sector 1, for
//! the beginning, then rock, ice and the wreckage of the war). Heights are a
//! pure function of position, so nothing is stored and a seed replays.

use super::rng::Rng;

/// The deepest a wall reaches into the field, in dots (a cell and a half).
pub const MAX_DEPTH: f32 = 6.0;
/// Scroll speed, in dots per second: the middle star layer's.
const SCROLL: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Open,
    Rock,
    Ice,
    Wreck,
}

#[derive(Clone, Debug)]
pub struct Terrain {
    pub style: Style,
    /// how far the tunnel has scrolled, in dots
    pub scroll: f32,
    phases: [f32; 4],
}

impl Terrain {
    pub fn new(sector: u32, rng: &mut Rng) -> Terrain {
        let style = match sector {
            1 => Style::Open,
            n => [Style::Rock, Style::Ice, Style::Wreck][(n as usize - 2) % 3],
        };
        let mut phase = || rng.range(0.0, std::f32::consts::TAU);
        Terrain { style, scroll: 0.0, phases: [phase(), phase(), phase(), phase()] }
    }

    pub fn update(&mut self, dt: f32) {
        self.scroll += SCROLL * dt;
    }

    /// How far the walls reach in at screen column `x` (dots): (top, bottom).
    pub fn depth(&self, x: f32) -> (f32, f32) {
        if self.style == Style::Open {
            return (0.0, 0.0);
        }
        let u = (x + self.scroll) * 0.045;
        let p = &self.phases;
        let wave = |a: f32, b: f32| {
            let v = (u + a).sin() * 0.6 + (u * 2.3 + b).sin() * 0.4;
            ((v * 0.5 + 0.5) * MAX_DEPTH).clamp(0.0, MAX_DEPTH)
        };
        let (mut top, mut bottom) = (wave(p[0], p[1]), wave(p[2], p[3]));
        // wreckage comes in jagged chunks rather than smooth rock
        if self.style == Style::Wreck {
            let chunk = ((x + self.scroll) / 8.0).floor();
            top = if (chunk as i64).rem_euclid(3) == 0 { top } else { (top - 3.0).max(0.0) };
            bottom = if (chunk as i64).rem_euclid(4) == 1 { bottom } else { (bottom - 3.0).max(0.0) };
        }
        (top, bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_sector_is_open_and_later_walls_stay_shallow() {
        let mut rng = Rng::new(1);
        assert_eq!(Terrain::new(1, &mut rng).depth(40.0), (0.0, 0.0));
        let t = Terrain::new(2, &mut rng);
        for x in 0..400 {
            let (top, bottom) = t.depth(x as f32);
            assert!(top <= MAX_DEPTH && bottom <= MAX_DEPTH);
        }
        assert!((0..400).any(|x| t.depth(x as f32).0 > 2.0), "but they are there");
    }
}
