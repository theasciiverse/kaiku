//! The run's record, for the summary at the end, and the kill chain that
//! multiplies the score.

/// Seconds a chain survives without a kill.
pub const CHAIN_WINDOW: f32 = 2.5;
/// Kills per step of the multiplier, and its cap.
const CHAIN_STEP: u32 = 5;
const MAX_MULTIPLIER: u32 = 8;

/// Kills in quick succession build a chain; every five links add one to the
/// score multiplier, up to ×8. Taking a hit breaks it.
#[derive(Clone, Debug, Default)]
pub struct Chain {
    pub count: u32,
    /// seconds left before it breaks
    pub timer: f32,
    pub best: u32,
}

impl Chain {
    pub fn multiplier(&self) -> u32 {
        (1 + self.count / CHAIN_STEP).min(MAX_MULTIPLIER)
    }

    /// A kill: the chain grows; returns the multiplier it scores at.
    pub fn link(&mut self) -> u32 {
        self.count += 1;
        self.timer = CHAIN_WINDOW;
        self.best = self.best.max(self.count);
        self.multiplier()
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
        if self.timer == 0.0 {
            self.count = 0;
        }
    }

    pub fn snap(&mut self) {
        self.count = 0;
        self.timer = 0.0;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub kills: u32,
    pub sectors: u32,
    pub true_jumps: u32,
    pub false_jumps: u32,
    pub echoes: u32,
    pub pods: u32,
    pub keys: u32,
    pub misses: u32,
    pub seconds: f32,
}

impl Stats {
    /// Typing accuracy, 0..100.
    pub fn accuracy(&self) -> u32 {
        if self.keys == 0 {
            return 100;
        }
        ((self.keys - self.misses.min(self.keys)) * 100 + self.keys / 2) / self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chain_multiplies_and_breaks() {
        let mut c = Chain::default();
        for _ in 0..4 {
            assert_eq!(c.link(), 1);
        }
        assert_eq!(c.link(), 2, "five kills: ×2");
        c.tick(CHAIN_WINDOW);
        assert_eq!((c.count, c.multiplier(), c.best), (0, 1, 5), "too slow: it breaks, the best stays");
        for _ in 0..100 {
            c.link();
        }
        assert_eq!(c.multiplier(), MAX_MULTIPLIER);
    }

    #[test]
    fn accuracy_rounds_and_never_divides_by_zero() {
        let s = Stats { keys: 3, misses: 1, ..Stats::default() };
        assert_eq!(s.accuracy(), 67);
        assert_eq!(Stats::default().accuracy(), 100);
    }
}
