//! The route home: sectors laid out by how many jumps they are from home.
//! Layer 0 is home itself; the surge leaves you in layer `START_DISTANCE`.
//! A true portal moves you one layer nearer, a jammed one one layer further
//! out (a detour). Every sector has a four-letter name (a Gates of Zendocon
//! nod), shown on the map; later they double as passwords.

use super::rng::Rng;

/// Jumps between the surge and home along the shortest route.
pub const START_DISTANCE: usize = 6;
/// The furthest out a detour can take you.
pub const MAX_DISTANCE: usize = 10;

/// Zendocon's own universe codes, never to be generated (homage, not copy).
const AVOID: [&str; 12] = ["BASE", "RAZE", "TRYX", "BREX", "ZYBX", "NERB", "BROT", "ZETA", "BARE", "SSSS", "NEAR", "HOME"];

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Route {
    /// layers[d] are the sectors d jumps from home; layers[0] is home
    pub layers: Vec<Vec<Node>>,
    /// (layer, index) of where you are
    pub at: (usize, usize),
    /// every sector visited, in order
    pub visited: Vec<(usize, usize)>,
}

impl Route {
    pub fn new(rng: &mut Rng) -> Route {
        let mut names = Vec::new();
        let layers: Vec<Vec<Node>> = (0..=MAX_DISTANCE)
            .map(|d| {
                let n = if d == 0 { 1 } else { 2 + (rng.unit() * 2.0) as usize };
                (0..n).map(|_| Node { name: if d == 0 { "HOME".to_string() } else { name(rng, &mut names) } }).collect()
            })
            .collect();
        let at = (START_DISTANCE, (rng.unit() * layers[START_DISTANCE].len() as f32) as usize);
        Route { layers, at, visited: vec![at] }
    }

    pub fn distance(&self) -> usize {
        self.at.0
    }

    pub fn here(&self) -> &Node {
        &self.layers[self.at.0][self.at.1]
    }

    /// Jumps: one layer nearer home for a true portal, one further out for a
    /// false one, to some sector in that layer.
    pub fn jump(&mut self, truth: bool, rng: &mut Rng) {
        let d = self.at.0;
        let next = if truth { d.saturating_sub(1) } else { (d + 1).min(MAX_DISTANCE) };
        let n = self.layers[next].len();
        let mut i = (rng.unit() * n as f32) as usize % n;
        // pinned at the edge: at least move to another sector there
        if next == d && n > 1 && i == self.at.1 {
            i = (i + 1) % n;
        }
        self.at = (next, i);
        self.visited.push(self.at);
    }
}

/// A pronounceable four-letter sector name, consonant-vowel-consonant-vowel or
/// with a consonant ending, unique and never one of Zendocon's.
fn name(rng: &mut Rng, taken: &mut Vec<String>) -> String {
    const C: &[u8] = b"BDFGKLMNPRSTVZ";
    const V: &[u8] = b"AEIOU";
    let pick = |rng: &mut Rng, set: &[u8]| set[(rng.unit() * set.len() as f32) as usize % set.len()] as char;
    loop {
        let s: String = if rng.unit() < 0.6 {
            [pick(rng, C), pick(rng, V), pick(rng, C), pick(rng, V)].iter().collect()
        } else {
            [pick(rng, C), pick(rng, V), pick(rng, V), pick(rng, C)].iter().collect()
        };
        if !AVOID.contains(&s.as_str()) && !taken.contains(&s) {
            taken.push(s.clone());
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_jumps_lead_home_and_false_ones_detour() {
        let mut rng = Rng::new(3);
        let mut r = Route::new(&mut rng);
        assert_eq!(r.distance(), START_DISTANCE);
        r.jump(false, &mut rng);
        assert_eq!(r.distance(), START_DISTANCE + 1);
        for _ in 0..START_DISTANCE + 1 {
            r.jump(true, &mut rng);
        }
        assert_eq!(r.distance(), 0);
        assert_eq!(r.here().name, "HOME");
        assert_eq!(r.visited.len(), START_DISTANCE + 3);
    }

    #[test]
    fn names_are_four_letters_unique_and_not_zendocons() {
        let mut rng = Rng::new(8);
        let r = Route::new(&mut rng);
        let names: Vec<&str> = r.layers[1..].iter().flatten().map(|n| n.name.as_str()).collect();
        assert!(names.iter().all(|n| n.len() == 4 && !AVOID.contains(n)));
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }
}
