//! One sector: a tunnel between two portals. The war flies through it; partway
//! along, the receiver catches echoes (one true, the others jammed); at the end
//! three gates open, one per echo, and flying into one is the jump.

use super::echo::{self, Echo, EchoState, Transmission};
use super::rng::Rng;
use super::Aabb;

/// Seconds into the sector at which each echo arrives.
const ECHO_TIMES: [f32; 3] = [10.0, 22.0, 34.0];
/// When the portals open (and the war stops sending ships).
const PORTALS_AT: f32 = 58.0;
/// Gates at the end of every sector: one per echo, one of them true.
const GATES: usize = ECHO_TIMES.len();
/// How a blind gate glows: dim, and the same for all of them.
pub const BLIND_GLOW: f32 = 0.15;
/// Portal size in dots: 6×10 half-block pixels.
pub const PORTAL_W: f32 = 12.0;
pub const PORTAL_H: f32 = 20.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Flight,
    Portals,
}

#[derive(Clone, Debug)]
pub struct Portal {
    pub x: f32,
    pub y: f32,
    /// where it stops drifting in
    rest_x: f32,
    /// the echo it answers to; `None` for one that never arrived
    pub echo: Option<u32>,
    pub truth: bool,
}

impl Portal {
    /// Settled in place, and so enterable: a gate still sliding in must not
    /// sweep up a ship that happens to be in its way.
    pub fn open(&self) -> bool {
        (self.x - self.rest_x).abs() < 1.5
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 2.0, self.y + 2.0, self.x + PORTAL_W - 2.0, self.y + PORTAL_H - 2.0)
    }
}

pub struct Sector {
    pub number: u32,
    pub t: f32,
    pub phase: Phase,
    pub echoes: Vec<Echo>,
    pub portals: Vec<Portal>,
    /// which of the arrivals is the true echo
    truth_at: usize,
    arrived: usize,
    /// how near home this sector is, 0..1: true echoes come clearer
    closeness: f32,
    next_id: u32,
    pool: Vec<Transmission>,
}

impl Sector {
    pub fn new(number: u32, closeness: f32, rng: &mut Rng) -> Sector {
        let truth_at = (rng.unit() * ECHO_TIMES.len() as f32) as usize % ECHO_TIMES.len();
        Sector {
            number,
            t: 0.0,
            phase: Phase::Flight,
            echoes: Vec::new(),
            portals: Vec::new(),
            truth_at,
            arrived: 0,
            closeness,
            next_id: number * 100,
            pool: echo::transmissions(),
        }
    }

    /// `others` are the letters other live typing targets (pods) want next: a
    /// new echo starts with none of them either.
    pub fn update(&mut self, dt: f32, w: f32, h: f32, others: &[char], rng: &mut Rng) {
        self.t += dt;
        while self.arrived < ECHO_TIMES.len() && self.t >= ECHO_TIMES[self.arrived] {
            self.arrive(w, h, others, rng);
        }
        for e in &mut self.echoes {
            e.update(dt);
        }
        if self.phase == Phase::Flight && self.t >= PORTALS_AT {
            self.open_portals(w, h, rng);
        }
        for p in &mut self.portals {
            p.x += (p.rest_x - p.x) * (2.0 * dt).min(1.0);
        }
    }

    /// An echo arrives from the right: a real transmission if it is the true
    /// one, a jammed copy otherwise. Its first letter differs from every other
    /// drifting echo's next letter, so one key always names one echo.
    fn arrive(&mut self, w: f32, h: f32, others: &[char], rng: &mut Rng) {
        let truth = self.arrived == self.truth_at;
        self.arrived += 1;
        let mut taken: Vec<char> = self.echoes.iter().filter(|e| e.state == EchoState::Drifting).filter_map(Echo::next_letter).collect();
        taken.extend_from_slice(others);
        for _ in 0..40 {
            let t = &self.pool[(rng.unit() * self.pool.len() as f32) as usize % self.pool.len()];
            let text = if truth { t.text.to_string() } else { echo::jammed(t, &self.pool, rng) };
            let first = text.chars().find(char::is_ascii_alphabetic).map(|c| c.to_ascii_lowercase());
            if first.is_some_and(|c| taken.contains(&c)) {
                continue;
            }
            let clarity = if truth { 0.55 + 0.4 * self.closeness } else { rng.range(0.2, 0.45) };
            // on a whole row, clear of the edges
            let rows = (h / 4.0) as i32;
            let y = (rng.range(1.0, (rows - 2).max(2) as f32).floor()) * 4.0;
            self.next_id += 1;
            log::info!("echo {} truth={truth} clarity={clarity:.2} {text:?}", self.next_id);
            self.echoes.push(Echo::new(self.next_id, text, t.source, truth, clarity, (w + 2.0, y), rng));
            return;
        }
    }

    /// Three gates, always: one per echo, and only the true echo's gate leads
    /// nearer home. A gate whose echo was decoded shows its text and glows
    /// with its clarity; the others are blind (`???`), all alike.
    fn open_portals(&mut self, w: f32, h: f32, rng: &mut Rng) {
        self.phase = Phase::Portals;
        let mut gates: Vec<(Option<u32>, bool)> = self.echoes.iter().map(|e| (Some(e.id), e.truth)).collect();
        // an echo that never arrived still leaves a gate, a false one
        while gates.len() < GATES {
            gates.push((None, false));
        }
        let n = gates.len();
        let gap = (h - PORTAL_H * n as f32) / (n as f32 + 1.0);
        // shuffle, so a gate's place says nothing about its echo
        for i in (1..n).rev() {
            gates.swap(i, (rng.unit() * (i + 1) as f32) as usize % (i + 1));
        }
        for (i, (echo, truth)) in gates.into_iter().enumerate() {
            let y = gap + i as f32 * (PORTAL_H + gap);
            self.portals.push(Portal { x: w + 4.0, y, rest_x: w - PORTAL_W - 16.0, echo, truth });
        }
        log::info!("gates open: {n}");
    }

    pub fn echo(&self, id: u32) -> Option<&Echo> {
        self.echoes.iter().find(|e| e.id == id)
    }

    /// A gate's echo if it has been decoded (decoding late still counts); a
    /// gate without one is blind.
    pub fn named(&self, p: &Portal) -> Option<&Echo> {
        p.echo.and_then(|id| self.echo(id)).filter(|e| e.state == EchoState::Decoded)
    }

    /// How brightly a gate glows: its echo's clarity once decoded, else dim
    /// and the same as every other blind gate.
    pub fn glow(&self, p: &Portal) -> f32 {
        self.named(p).map_or(BLIND_GLOW, |e| e.clarity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &mut Sector, secs: f32, rng: &mut Rng) {
        for _ in 0..(secs * 60.0) as usize {
            s.update(1.0 / 60.0, 160.0, 72.0, &[], rng);
        }
    }

    #[test]
    fn three_echoes_arrive_and_exactly_one_is_true() {
        let mut rng = Rng::new(11);
        let mut s = Sector::new(1, 0.0, &mut rng);
        run(&mut s, 40.0, &mut rng);
        assert_eq!(s.echoes.len(), 3);
        assert_eq!(s.echoes.iter().filter(|e| e.truth).count(), 1);
        let truth = s.echoes.iter().find(|e| e.truth).unwrap();
        assert!(s.echoes.iter().filter(|e| !e.truth).all(|e| e.clarity < truth.clarity), "the true echo is the clearest");
    }

    #[test]
    fn three_gates_always_one_true_and_decoded_ones_are_named() {
        let mut rng = Rng::new(12);
        let mut s = Sector::new(1, 0.0, &mut rng);
        run(&mut s, 12.0, &mut rng);
        let text: String = s.echoes[0].letters().collect();
        for c in text.chars() {
            s.echoes[0].key(c);
        }
        run(&mut s, 50.0, &mut rng);
        assert_eq!(s.phase, Phase::Portals);
        assert_eq!(s.portals.len(), 3);
        assert_eq!(s.portals.iter().filter(|p| p.truth).count(), 1, "only one leads home");
        let named: Vec<u32> = s.portals.iter().filter_map(|p| s.named(p)).map(|e| e.id).collect();
        assert_eq!(named, vec![s.echoes[0].id], "only the decoded echo names its gate");

        let mut blind = Sector::new(2, 0.0, &mut rng);
        run(&mut blind, 60.0, &mut rng);
        assert_eq!(blind.portals.len(), 3);
        assert!(blind.portals.iter().all(|p| blind.named(p).is_none() && blind.glow(p) == BLIND_GLOW), "undecoded gates all look alike");
    }
}
