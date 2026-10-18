//! Ship-to-ship chatter: the war talking to itself on the receiver, now and
//! then, while nothing else is coming in. The lines are `data/chatter.txt`.

use super::enemy::Side;
use super::rng::Rng;

#[derive(Clone, Debug)]
pub struct Line {
    pub side: Side,
    pub callsign: &'static str,
    pub words: &'static str,
}

pub fn lines() -> Vec<Line> {
    include_str!("../../data/chatter.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut parts = l.splitn(3, '|').map(str::trim);
            let side = match parts.next()? {
                "amber" => Side::Amber,
                "jade" => Side::Jade,
                _ => return None,
            };
            Some(Line { side, callsign: parts.next()?, words: parts.next()? })
        })
        .collect()
}

/// Seconds between lines, and how long one stays on the receiver.
const EVERY: (f32, f32) = (12.0, 20.0);
const SHOWN: f32 = 5.0;

pub struct Chatter {
    pool: Vec<Line>,
    next: f32,
    /// the line on the receiver, and seconds it has left
    pub current: Option<(usize, f32)>,
}

impl Chatter {
    pub fn new() -> Chatter {
        Chatter { pool: lines(), next: 8.0, current: None }
    }

    pub fn line(&self) -> Option<&Line> {
        self.current.map(|(i, _)| &self.pool[i])
    }

    pub fn update(&mut self, dt: f32, rng: &mut Rng) {
        if let Some((_, t)) = &mut self.current {
            *t -= dt;
            if *t <= 0.0 {
                self.current = None;
            }
        }
        self.next -= dt;
        if self.next <= 0.0 && !self.pool.is_empty() {
            self.next = rng.range(EVERY.0, EVERY.1);
            self.current = Some(((rng.unit() * self.pool.len() as f32) as usize % self.pool.len(), SHOWN));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chatter_file_parses_and_lines_come_and_go() {
        assert!(lines().len() >= 10);
        assert!(lines().iter().any(|l| l.side == Side::Jade) && lines().iter().any(|l| l.side == Side::Amber));
        let mut c = Chatter::new();
        let mut rng = Rng::new(1);
        let mut seen = false;
        for _ in 0..60 * 30 {
            c.update(1.0 / 60.0, &mut rng);
            seen |= c.line().is_some();
        }
        assert!(seen);
    }
}
