//! The simulation: no terminal in here, so it runs headless in tests.

pub mod chatter;
pub mod cockpit;
pub mod cue;
pub mod echo;
pub mod enemy;
pub mod fx;
pub mod intro;
pub mod powerup;
pub mod rng;
pub mod route;
pub mod sector;
pub mod ship;
pub mod shot;
pub mod spawn;
pub mod stats;
pub mod stars;
pub mod terrain;
pub mod typing;

use crate::input::Controls;
use chatter::Chatter;
use cockpit::Cockpit;
use cue::Cue;
use enemy::{Action, Enemy, Kind, Side};
use fx::{Particle, Tint};
use powerup::{Drop, Loot, Pod, PodSpawner, PowerUp, Powers};
use rng::Rng;
use route::Route;
use sector::{Phase, Sector};
use ship::{Hit, Ship};
use shot::{Owner, Shot};
use spawn::Spawner;
use stars::Starfield;
use stats::{Chain, Stats};
use terrain::Terrain;
use typing::{Key, Target, Typing};

/// Fixed simulation step: 60 Hz, as in Entity.
pub const DT: f32 = 1.0 / 60.0;
const SHOT_SPEED: f32 = 260.0;
const ENEMY_SHOT_SPEED: f32 = 70.0;
/// Chance that a ship fires at the other side rather than at the player.
const WAR_FIRE: f32 = 0.5;
/// How much a jammer on screen takes off every echo's clarity.
const JAM: f32 = 0.3;
const MINE_SHRAPNEL: usize = 8;
/// Extra lives: at this score, then every so many points more.
const EXTEND_FIRST: u32 = 30_000;
const EXTEND_EVERY: u32 = 60_000;
const MAX_LIVES: u8 = 5;
/// Scraping a tunnel wall: a nag, not a bullet.
const SCRAPE_DAMAGE: f32 = 0.1;
/// Every this many sectors a carrier guards the gates.
const GUARD_EVERY: u32 = 3;
/// A lancer's beam takes this much shield.
const BEAM_DAMAGE: f32 = 0.5;
/// Seconds the field takes to swing round after a portal.
pub const TURN_TIME: f32 = 1.6;
/// How long a receiver note stays up, in seconds.
const NOTE_TIME: f32 = 6.0;

/// An axis-aligned box in dots.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Aabb {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Aabb {
        Aabb { x0, y0, x1, y1 }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }

    pub fn overlaps(&self, o: &Aabb) -> bool {
        self.x0 <= o.x1 && o.x0 <= self.x1 && self.y0 <= o.y1 && o.y0 <= self.y1
    }

    fn center(&self) -> (f32, f32) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// the opening story crawl
    Intro,
    /// the tutorial: repairing the receiver, the first echo
    Cockpit,
    Flight,
    /// through a portal: the field swings round to the new heading
    Turn,
    /// between sectors: the route map, until Enter
    Map,
    /// out of lives: the war goes on without you
    Over,
    /// arrived
    Home,
}

pub struct Game {
    /// field size in dots
    pub w: f32,
    pub h: f32,
    pub state: State,
    pub ship: Ship,
    pub shots: Vec<Shot>,
    pub enemies: Vec<Enemy>,
    pub particles: Vec<Particle>,
    pub stars: Starfield,
    pub score: u32,
    /// the best score ever, as loaded at start; `record` once this run beats it
    pub best: u32,
    pub record: bool,
    pub chain: Chain,
    pub stats: Stats,
    /// the score at which the next extra life comes
    next_extend: u32,
    pub tick: u64,
    pub sector: Sector,
    pub terrain: Terrain,
    pub chatter: Chatter,
    pub typing: Typing,
    pub pods: Vec<Pod>,
    /// power-ups destroyed ships left behind, for the pilot to fly into
    pub drops: Vec<Drop>,
    pub powers: Powers,
    /// seconds since the last smart bomb, for its flash
    pub bomb_t: f32,
    /// the power-up just claimed or lost, and seconds left to show it
    pub power_note: Option<(String, f32)>,
    pods_spawner: PodSpawner,
    pub route: Route,
    /// seconds spent in the current state (Turn, Map, Home)
    pub state_t: f32,
    /// which way the field swings at a turn: -1 or 1
    pub turn_dir: f32,
    /// the tutorial scene, while it runs
    pub cockpit: Option<Cockpit>,
    /// after the crawl: the controls card, before the cockpit
    pub intro_card: bool,
    /// the turn under way is the one out of the cockpit
    first_turn: bool,
    /// moments to be heard and felt, drained by the main loop each frame
    pub cues: Vec<Cue>,
    /// a line for the receiver (jump results), and seconds it has left
    pub note: Option<(String, f32)>,
    spawner: Spawner,
    rng: Rng,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut rng = Rng::new(seed);
        let sector = Sector::new(1, 0.0, &mut rng);
        let route = Route::new(&mut rng);
        let terrain = Terrain::new(1, &mut rng);
        Game {
            w: 0.0,
            h: 0.0,
            state: State::Flight,
            ship: Ship::new(0.0),
            shots: Vec::new(),
            enemies: Vec::new(),
            particles: Vec::new(),
            stars: Starfield::default(),
            score: 0,
            best: 0,
            record: false,
            chain: Chain::default(),
            stats: Stats::default(),
            next_extend: EXTEND_FIRST,
            tick: 0,
            sector,
            terrain,
            chatter: Chatter::new(),
            typing: Typing::default(),
            pods: Vec::new(),
            drops: Vec::new(),
            powers: Powers::default(),
            bomb_t: 10.0,
            power_note: None,
            pods_spawner: PodSpawner::default(),
            route,
            state_t: 0.0,
            turn_dir: 1.0,
            cockpit: None,
            intro_card: false,
            first_turn: false,
            cues: Vec::new(),
            note: None,
            spawner: Spawner::default(),
            rng,
        }
    }

    /// A fresh run in the same field.
    pub fn restart(&mut self) {
        let mut g = Game::new(self.rng.next_u64());
        g.best = self.best.max(self.score);
        g.resize_dots(self.w, self.h);
        *self = g;
        log::info!("restart");
    }

    /// The run is over, one way or the other.
    pub fn finished(&self) -> bool {
        matches!(self.state, State::Over | State::Home)
    }

    /// Starts straight in the tunnel at sector `n`, skipping the opening.
    pub fn start_at(&mut self, n: u32) {
        let closeness = self.closeness();
        self.sector = Sector::new(n.max(1), closeness, &mut self.rng);
        self.terrain = Terrain::new(self.sector.number, &mut self.rng);
        self.set_state(State::Flight);
    }

    /// Begins with the opening crawl instead of straight in the tunnel.
    pub fn start_intro(&mut self) {
        self.set_state(State::Intro);
    }

    /// A key during the crawl: the first finishes the text, the next flies.
    pub fn intro_key(&mut self) {
        if self.state != State::Intro {
            return;
        }
        if self.state_t < intro::duration() {
            self.state_t = intro::duration();
        } else if !self.intro_card {
            // the story told: now how to fly
            self.intro_card = true;
        } else {
            self.intro_card = false;
            self.cockpit = Some(Cockpit::new(&mut self.rng));
            self.set_state(State::Cockpit);
        }
    }

    /// Enter in the cockpit: skip the tutorial, straight into the first turn.
    pub fn skip_cockpit(&mut self) {
        if self.state == State::Cockpit {
            self.leave_cockpit();
        }
    }

    fn leave_cockpit(&mut self) {
        self.cockpit = None;
        self.turn_dir = 1.0;
        self.first_turn = true;
        self.set_state(State::Turn);
    }

    /// Fits the field to the play area, in cells. The first fit places the ship.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.resize_dots(cols as f32 * 2.0, rows as f32 * 4.0);
    }

    fn resize_dots(&mut self, w: f32, h: f32) {
        if (w, h) == (self.w, self.h) {
            return;
        }
        let first = self.w == 0.0;
        self.w = w;
        self.h = h;
        self.stars = Starfield::new(w, h, &mut self.rng);
        if first {
            self.ship = Ship::new(h);
        }
        self.ship.clamp(w, h);
        log::info!("field {w}x{h} dots");
    }

    pub fn update(&mut self, c: &Controls) {
        self.tick += 1;
        let crawl = (self.state == State::Intro).then(|| intro::shown(self.state_t).trim_end().len());
        self.state_t += DT;
        if crawl.is_some_and(|n| intro::shown(self.state_t).trim_end().len() > n) {
            self.cues.push(Cue::Typewriter);
        }
        // through a portal the stars streak past
        // the stars stream past in flight, streak through a turn, and hold
        // still while the ship drifts dead in the cockpit scene
        let warp = match self.state {
            State::Turn => 6.0,
            State::Cockpit => 0.0,
            _ => 1.0,
        };
        self.stars.update(DT, warp, self.w, self.h, &mut self.rng);
        if self.state == State::Turn && self.state_t >= TURN_TIME {
            // the first turn, out of the cockpit, goes straight into sector 1
            let next = match () {
                _ if std::mem::take(&mut self.first_turn) => State::Flight,
                _ if self.route.distance() == 0 => State::Home,
                _ => State::Map,
            };
            self.set_state(next);
        }
        if let Some(c) = &mut self.cockpit {
            if c.update(DT) {
                self.leave_cockpit();
            }
        }
        self.bomb_t += DT;
        self.powers.tick(DT);
        self.chain.tick(DT);
        if self.state == State::Flight {
            self.stats.seconds += DT;
            self.terrain.update(DT);
            self.scrape_walls();
            self.chatter.update(DT, &mut self.rng);
        }
        for note in [&mut self.note, &mut self.power_note] {
            if let Some((_, t)) = note {
                *t -= DT;
                if *t <= 0.0 {
                    *note = None;
                }
            }
        }
        // a dash leaves a trail of afterimages
        if self.ship.dash > 0.0 && self.tick.is_multiple_of(2) {
            let (cx, cy) = self.ship.center();
            fx::burst(&mut self.particles, cx - 4.0, cy, 2, 6.0, Tint::Player, &mut self.rng);
        }
        let interval = if self.powers.rapid { 0.06 } else { 0.11 };
        if self.state == State::Flight && self.ship.update(c, DT, self.w, self.h, interval) {
            let (x, y) = self.ship.nose();
            self.shots.push(Shot::new(x, y, SHOT_SPEED, 0.0, Owner::Player));
            self.cues.push(Cue::Shot);
            if self.powers.spread {
                for vy in [-45.0, 45.0] {
                    self.shots.push(Shot::new(x - 2.0, y, SHOT_SPEED * 0.95, vy, Owner::Player));
                }
            }
        }
        if self.state == State::Flight {
            let station = (self.ship.x, self.ship.y);
            if let Some(d) = &mut self.powers.drone {
                if d.update(DT, station) {
                    self.shots.push(Shot::new(d.x + 6.0, d.y + 2.0, SHOT_SPEED, 0.0, Owner::Player));
                }
            }
        }
        // slow time: the war, its fire, echoes and pods all run slow; you don't
        let war_dt = if self.powers.slow > 0.0 { DT * powerup::SLOW_FACTOR } else { DT };
        if self.state == State::Flight && self.sector.phase == Phase::Flight {
            self.spawner.update(DT, self.w, self.h, self.sector.number, &mut self.rng, &mut self.enemies);
        }
        if self.state == State::Flight {
            let was = self.sector.phase;
            let pod_letters: Vec<char> = self.pods.iter().filter(|p| p.live()).filter_map(Target::next_letter).collect();
            let verdicts: Vec<_> = self.sector.echoes.iter().map(|e| e.verdict()).collect();
            self.sector.update(war_dt, self.w, self.h, &pod_letters, &mut self.rng);
            for (e, before) in self.sector.echoes.iter().zip(verdicts) {
                match (before, e.verdict()) {
                    (echo::Verdict::Pending, echo::Verdict::Jammed) => self.cues.push(Cue::Jammed),
                    (echo::Verdict::Pending, echo::Verdict::Steady) => self.cues.push(Cue::Steady),
                    _ => {}
                }
            }
            if was == Phase::Flight && self.sector.phase == Phase::Portals {
                self.cues.push(Cue::GatesOpen);
            }
            if was == Phase::Flight && self.sector.phase == Phase::Portals && self.sector.number.is_multiple_of(GUARD_EVERY) {
                self.send_guard();
            }
            self.enter_portal();
        }
        let target_y = self.ship.center().1;
        for i in 0..self.enemies.len() {
            let beam = self.enemies[i].beam().map(|b| b.live);
            let action = self.enemies[i].update(war_dt, self.w, self.h, target_y, &mut self.rng);
            match (beam, self.enemies[i].beam().map(|b| b.live)) {
                (None, Some(false)) => self.cues.push(Cue::Warning),
                (Some(false), Some(true)) => self.cues.push(Cue::Beam),
                _ => {}
            }
            match action {
                Action::None => {}
                Action::Fire => self.enemy_fires(i),
                Action::Fan => self.enemy_fan(i),
                Action::Launch => {
                    let e = &self.enemies[i];
                    let (x, y) = (e.x, e.y + e.kind.size().1 / 2.0 - 3.0);
                    let side = e.side;
                    self.enemies.push(Enemy::scout(side, x, y, 0.0, &mut self.rng));
                }
            }
        }
        // a jammer on screen scrambles every echo in range
        let jam = if self.enemies.iter().any(|e| e.kind == Kind::Jammer && e.hp > 0 && e.x < self.w) { JAM } else { 0.0 };
        if jam > 0.0 && !self.sector.echoes.is_empty() && self.sector.echoes.iter().all(|e| e.jam == 0.0) {
            self.cues.push(Cue::Jamming);
        }
        for e in &mut self.sector.echoes {
            e.jam = jam;
        }
        if self.state == State::Flight && self.sector.phase == Phase::Flight {
            let taken = self.taken_letters();
            if let Some(p) = self.pods_spawner.update(DT, self.w, self.h, self.sector.number, &taken, &mut self.rng) {
                self.pods.push(p);
            }
        }
        for p in &mut self.pods {
            p.update(war_dt, self.h);
        }
        self.pods.retain(|p| !p.done);
        shot::update(&mut self.shots, DT, war_dt, self.w, self.h);
        self.collide();
        // a mine destroyed, by anyone, bursts into shrapnel
        let bursts: Vec<((f32, f32), Side)> = self.enemies.iter().filter(|e| e.kind == Kind::Mine && e.hp == 0).map(|e| (e.aabb().center(), e.side)).collect();
        for ((x, y), side) in bursts {
            for k in 0..MINE_SHRAPNEL {
                let a = k as f32 / MINE_SHRAPNEL as f32 * std::f32::consts::TAU;
                self.shots.push(Shot::new(x, y, a.cos() * ENEMY_SHOT_SPEED, a.sin() * ENEMY_SHOT_SPEED, Owner::Enemy(side)));
            }
        }
        for e in self.enemies.iter().filter(|e| e.hp == 0 && e.by_player) {
            let (cx, cy) = e.aabb().center();
            // a power-up, or failing that perhaps a shield cell, a little apart
            let drop = Drop::roll(e.kind.drop_chance(), cx, cy, &mut self.rng).or_else(|| Drop::roll_cell(e.kind.cell_chance(), cx, cy, &mut self.rng));
            if let Some(d) = drop {
                log::debug!("drop {:?} from {:?}", d.kind, e.kind);
                self.drops.push(d);
            }
            if e.kind == Kind::Carrier {
                // a carrier leaves both
                self.drops.extend(Drop::roll_cell(1.0, cx + 10.0, cy, &mut self.rng));
            }
        }
        for d in &mut self.drops {
            d.update(war_dt, self.h);
        }
        if self.state == State::Flight {
            let ship = self.ship.aabb();
            let grabbed: Vec<Loot> = self.drops.iter_mut().filter(|d| !d.done && d.aabb().overlaps(&ship)).map(|d| {
                d.done = true;
                d.kind
            }).collect();
            for kind in grabbed {
                self.collect(kind);
            }
        }
        self.drops.retain(|d| !d.done);
        for e in self.enemies.iter().filter(|e| e.hp == 0) {
            self.cues.push(match e.kind {
                Kind::Scout | Kind::Interceptor => Cue::Kill,
                Kind::Mine => Cue::Shrapnel,
                _ => Cue::BigKill,
            });
        }
        self.enemies.retain(|e| e.hp > 0 && !e.gone());
        fx::update(&mut self.particles, DT);
        self.extends();
        if self.tick.is_multiple_of(30) {
            log::trace!("tick {} enemies {} shots {} shield {} lives {}", self.tick, self.enemies.len(), self.shots.len(), self.ship.shield, self.ship.lives);
        }
    }

    /// A typed letter, for the typing layer.
    pub fn type_letter(&mut self, c: char) {
        if let (State::Cockpit, Some(cockpit)) = (self.state, &mut self.cockpit) {
            let log = cockpit.log.len();
            let missed = cockpit.flash;
            cockpit.key(c);
            self.cues.push(match () {
                _ if cockpit.log.len() > log => Cue::Repaired,
                // a miss sets the flash; a hit leaves it as it was
                _ if cockpit.flash > missed => Cue::Miss,
                _ => Cue::Key,
            });
            return;
        }
        if self.state != State::Flight {
            return;
        }
        // only what you can see: a target still entering off the right edge
        // can't be locked by a letter you had no way of reading
        let edge = self.w - 2.0;
        let mut targets: Vec<&mut dyn Target> = self
            .sector
            .echoes
            .iter_mut()
            .map(|e| e as &mut dyn Target)
            .chain(self.pods.iter_mut().map(|p| p as &mut dyn Target))
            .filter(|t| t.x() < edge)
            .collect();
        let key = self.typing.key(c, &mut targets);
        self.stats.keys += 1;
        if matches!(key, Key::Miss | Key::Stray) {
            self.stats.misses += 1;
        }
        self.cues.push(if matches!(key, Key::Miss | Key::Stray) { Cue::Miss } else { Cue::Key });
        let Key::Done(id) = key else { return };
        if let Some(e) = self.sector.echo(id) {
            log::info!("decoded echo {id} truth={} {:?}", e.truth, e.text);
            self.score += 50 * e.text.len() as u32;
            self.stats.echoes += 1;
            self.cues.push(Cue::Decoded);
        } else if let Some(i) = self.pods.iter().position(|p| p.id == id) {
            self.stats.pods += 1;
            self.cues.push(Cue::Claimed);
            let pod = self.pods.remove(i);
            self.claim(pod);
        }
    }

    /// The pilot flies into a drop: a power-up, or a cell of shield back, no
    /// typing needed.
    fn collect(&mut self, loot: Loot) {
        log::info!("collected a {loot:?} drop");
        let kind = match loot {
            Loot::Power(kind) => kind,
            Loot::Cell => {
                self.ship.shield = (self.ship.shield + powerup::CELL).min(1.0);
                self.score += 25;
                self.cues.push(Cue::Cell);
                return;
            }
        };
        self.score += 100;
        self.stats.pods += 1;
        self.powers.take(kind, (self.ship.x, self.ship.y));
        if kind == PowerUp::Shield {
            self.ship.shield = 1.0;
        }
        self.cues.push(Cue::Claimed);
        self.power_note = Some((format!("+ {}", kind.name()), 2.5));
    }

    fn claim(&mut self, pod: Pod) {
        log::info!("claimed {:?} with {:?}", pod.kind, pod.word);
        self.score += 25 * pod.word.len() as u32;
        self.powers.take(pod.kind, (self.ship.x, self.ship.y));
        if pod.kind == PowerUp::Shield {
            self.ship.shield = 1.0;
        }
        let (cx, cy) = (pod.x + powerup::POD_W / 2.0, pod.y + powerup::POD_H / 2.0);
        fx::burst(&mut self.particles, cx, cy, 14, 40.0, Tint::Player, &mut self.rng);
        self.power_note = Some((format!("+ {}", pod.kind.name()), 2.5));
    }

    /// Letters the live typing targets want next: a new target starts with
    /// none of them, so one key always names one target.
    fn taken_letters(&self) -> Vec<char> {
        let echoes = self.sector.echoes.iter().filter(|e| e.live()).filter_map(Target::next_letter);
        let pods = self.pods.iter().filter(|p| p.live()).filter_map(Target::next_letter);
        echoes.chain(pods).collect()
    }

    pub fn drop_lock(&mut self) {
        let mut targets: Vec<&mut dyn Target> =
            self.sector.echoes.iter_mut().map(|e| e as &mut dyn Target).chain(self.pods.iter_mut().map(|p| p as &mut dyn Target)).collect();
        self.typing.drop_lock(&mut targets);
    }

    /// Tab (or the pad's X / right bumper): a dash, if one is ready, with a
    /// trail of afterimages.
    pub fn dash(&mut self, c: &Controls) {
        if self.state == State::Flight && self.ship.start_dash(c) {
            self.cues.push(Cue::Dash);
            let (cx, cy) = self.ship.center();
            fx::burst(&mut self.particles, cx, cy, 10, 25.0, Tint::Player, &mut self.rng);
            log::debug!("dash");
        }
    }

    /// Enter: a smart bomb, if one is carried. Every ship on screen goes, and
    /// all enemy fire.
    pub fn bomb(&mut self) {
        if self.state != State::Flight || self.powers.bombs == 0 {
            return;
        }
        self.powers.bombs -= 1;
        self.bomb_t = 0.0;
        self.cues.push(Cue::Bomb);
        for e in self.enemies.iter_mut().filter(|e| e.x < self.w && e.hp > 0) {
            e.hp = 0;
            e.by_player = true;
            let m = self.chain.link();
            self.score += e.kind.points() * m;
            self.stats.kills += 1;
            let (cx, cy) = e.aabb().center();
            fx::burst(&mut self.particles, cx, cy, 18, 55.0, Tint::Side(e.side), &mut self.rng);
        }
        self.shots.retain(|s| s.owner == Owner::Player);
        log::info!("smart bomb, {} left", self.powers.bombs);
    }

    /// How near home, 0 (where the surge left you) to 1.
    pub fn closeness(&self) -> f32 {
        // detours take you further out than the surge did: still 0, never less,
        // or the true echo would read fainter than the fakes
        (1.0 - self.route.distance() as f32 / route::START_DISTANCE as f32).clamp(0.0, 1.0)
    }

    fn set_state(&mut self, state: State) {
        log::info!("state {:?} -> {state:?}", self.state);
        match state {
            State::Turn => self.cues.push(Cue::Jump),
            State::Home => self.cues.push(Cue::Home),
            State::Over => self.cues.push(Cue::GameOver),
            _ => {}
        }
        if matches!(state, State::Over | State::Home) && self.score > self.best {
            self.best = self.score;
            self.record = true;
        }
        self.state = state;
        self.state_t = 0.0;
    }

    /// The jump: a true portal brings home one step closer, a false one adds a
    /// detour. The field swings round, then the map shows where you are.
    fn enter_portal(&mut self) {
        let ship = self.ship.aabb();
        let Some(p) = self.sector.portals.iter().find(|p| p.open() && p.aabb().overlaps(&ship)) else { return };
        let (truth, blind) = (p.truth, self.sector.named(p).is_none());
        let from = self.route.at.1;
        self.route.jump(truth, &mut self.rng);
        self.stats.sectors += 1;
        if truth {
            self.stats.true_jumps += 1;
        } else {
            self.stats.false_jumps += 1;
        }
        let d = self.route.distance();
        let line = match (truth, blind) {
            (true, _) if d == 0 => "the echoes were true · home".to_string(),
            (true, false) => format!("true echo · closer to home · {d} jumps left"),
            (true, true) => format!("a lucky blind gate · {d} jumps left"),
            (false, false) => format!("the echo was jammed · a detour · {d} jumps left"),
            (false, true) => format!("a blind gate, off course · {d} jumps left"),
        };
        log::info!("jump from sector {} truth={truth} to {} distance={d}", self.sector.number, self.route.here().name);
        self.note = Some((line, NOTE_TIME));
        // the field swings the way the route bends on the map
        self.turn_dir = if self.route.at.1 >= from { 1.0 } else { -1.0 };
        self.enemies.clear();
        self.shots.clear();
        self.pods.clear();
        self.drops.clear();
        self.typing = Typing::default();
        self.set_state(State::Turn);
    }

    /// Leaves the map: the next sector begins.
    pub fn fly_on(&mut self) {
        if self.state != State::Map {
            return;
        }
        let closeness = self.closeness();
        self.sector = Sector::new(self.sector.number + 1, closeness, &mut self.rng);
        self.terrain = Terrain::new(self.sector.number, &mut self.rng);
        self.spawner = Spawner::default();
        self.pods_spawner = PodSpawner::default();
        self.ship.x = 8.0;
        // through the gate the shield recharges, as docking re-arms in Zendocon
        self.ship.shield = 1.0;
        self.note = None;
        self.set_state(State::Flight);
    }

    /// Extra lives at score milestones: the first at `EXTEND_FIRST`, then every
    /// `EXTEND_EVERY`, up to `MAX_LIVES`.
    fn extends(&mut self) {
        while self.score >= self.next_extend {
            self.next_extend += EXTEND_EVERY;
            if self.ship.lives < MAX_LIVES {
                self.ship.lives += 1;
                self.cues.push(Cue::ExtraLife);
                self.power_note = Some(("+ extra life".to_string(), 2.5));
                log::info!("extra life at {}", self.score);
            }
        }
    }

    /// Ship `i` fires: at the nearest ship of the other side half the time, if
    /// there is one, otherwise at the player.
    fn enemy_fires(&mut self, i: usize) {
        let e = &self.enemies[i];
        let (mx, my) = e.muzzle();
        let rival = self
            .enemies
            .iter()
            .filter(|o| o.side == e.side.other() && o.x < self.w)
            .map(|o| o.aabb().center())
            .min_by(|a, b| dist2(*a, (mx, my)).total_cmp(&dist2(*b, (mx, my))));
        let target = match rival {
            Some(r) if self.rng.unit() < WAR_FIRE => r,
            _ if self.state == State::Flight => self.ship.center(),
            // no player: fire straight ahead
            _ => (mx - 1.0, my),
        };
        let (dx, dy) = (target.0 - mx, target.1 - my);
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let side = e.side;
        self.shots.push(Shot::new(mx, my, dx / len * ENEMY_SHOT_SPEED, dy / len * ENEMY_SHOT_SPEED, Owner::Enemy(side)));
    }

    /// The ship against the tunnel walls: a scrape costs shield, and the wall
    /// pushes the ship back into the open.
    fn scrape_walls(&mut self) {
        let b = self.ship.aabb();
        let (mut top, mut bottom) = (0.0f32, 0.0f32);
        for x in [b.x0, (b.x0 + b.x1) / 2.0, b.x1] {
            let (t, d) = self.terrain.depth(x);
            top = top.max(t);
            bottom = bottom.max(d);
        }
        let floor = self.h - bottom;
        if b.y0 < top {
            self.ship.y += top - b.y0;
        } else if b.y1 > floor {
            self.ship.y -= b.y1 - floor;
        } else {
            return;
        }
        self.ship_hit(SCRAPE_DAMAGE);
    }

    /// A carrier comes to hold in front of the gates as they open.
    fn send_guard(&mut self) {
        let side = if self.rng.unit() < 0.5 { Side::Amber } else { Side::Jade };
        let y = self.h / 2.0 - 8.0;
        let hold = self.w * 0.55;
        self.enemies.push(Enemy::of(Kind::Carrier, side, self.w + 2.0, y, hold, &mut self.rng));
        self.note = Some(("⚠ a carrier guards the gates".to_string(), NOTE_TIME));
        log::info!("gate guard in sector {}", self.sector.number);
    }

    /// A carrier's salvo: three shots fanned out ahead of it.
    fn enemy_fan(&mut self, i: usize) {
        let e = &self.enemies[i];
        let (mx, my) = e.muzzle();
        let side = e.side;
        for a in [-0.35f32, 0.0, 0.35] {
            let (sin, cos) = a.sin_cos();
            self.shots.push(Shot::new(mx, my, -cos * ENEMY_SHOT_SPEED, sin * ENEMY_SHOT_SPEED, Owner::Enemy(side)));
        }
    }

    fn collide(&mut self) {
        let flying = self.state == State::Flight;
        let ship_box = self.ship.aabb();
        let mut hits: Vec<f32> = Vec::new();
        // lancer beams: a band two dots either side of the row, to the left edge
        if flying {
            for b in self.enemies.iter().filter(|e| e.hp > 0).filter_map(Enemy::beam).filter(|b| b.live) {
                if Aabb::new(0.0, b.y - 2.0, b.x_end, b.y + 2.0).overlaps(&ship_box) {
                    hits.push(BEAM_DAMAGE);
                }
            }
        }
        for s in self.shots.iter_mut().filter(|s| !s.spent) {
            let owner = s.owner;
            // keep your fire off the pods: a shot pod is lost
            if owner == Owner::Player {
                if let Some(p) = self.pods.iter_mut().find(|p| !p.done && p.aabb().contains(s.x, s.y)) {
                    s.spent = true;
                    let broken = p.shot();
                    if broken {
                        self.cues.push(Cue::PodBroken);
                    }
                    let n = if broken { 10 } else { 3 };
                    fx::burst(&mut self.particles, s.x, s.y, n, 30.0, Tint::Spark, &mut self.rng);
                    continue;
                }
            }
            let hurts = |e: &Enemy| match owner {
                Owner::Player => true,
                Owner::Enemy(side) => e.side != side,
            };
            if let Some(e) = self.enemies.iter_mut().find(|e| e.hp > 0 && hurts(e) && e.aabb().contains(s.x, s.y)) {
                s.spent = true;
                e.hp -= 1;
                e.flash = 0.08;
                if e.hp == 0 {
                    let (cx, cy) = e.aabb().center();
                    fx::burst(&mut self.particles, cx, cy, 18, 50.0, Tint::Side(e.side), &mut self.rng);
                    if owner == Owner::Player {
                        e.by_player = true;
                        let m = self.chain.link();
                        self.score += e.kind.points() * m;
                        self.stats.kills += 1;
                    }
                } else {
                    fx::burst(&mut self.particles, s.x, s.y, 3, 30.0, Tint::Spark, &mut self.rng);
                }
                continue;
            }
            if flying && owner != Owner::Player && ship_box.contains(s.x, s.y) && self.ship.grace <= 0.0 {
                s.spent = true;
                hits.push(ship::BULLET_DAMAGE);
            }
        }
        if flying {
            // carriers are too big to ram: they just hurt
            for e in self.enemies.iter_mut().filter(|e| e.hp > 0 && e.aabb().overlaps(&ship_box)) {
                if e.kind == Kind::Carrier {
                    hits.push(ship::RAM_DAMAGE);
                    continue;
                }
                // a ram destroys the rammer either way
                e.hp = 0;
                let (cx, cy) = e.aabb().center();
                fx::burst(&mut self.particles, cx, cy, 18, 50.0, Tint::Side(e.side), &mut self.rng);
                hits.push(ship::RAM_DAMAGE);
            }
        }
        for damage in hits {
            self.ship_hit(damage);
        }
    }

    fn ship_hit(&mut self, damage: f32) {
        let (cx, cy) = self.ship.center();
        let hit = self.ship.hit(damage);
        if hit.is_some() {
            self.chain.snap();
        }
        match hit {
            None => {}
            Some(Hit::Shield) => {
                self.cues.push(Cue::ShieldHit);
                fx::burst(&mut self.particles, cx, cy, 6, 35.0, Tint::Spark, &mut self.rng);
            }
            Some(Hit::Life) => {
                self.cues.push(Cue::LifeLost);
                fx::burst(&mut self.particles, cx, cy, 30, 60.0, Tint::Player, &mut self.rng);
                if let Some(lost) = self.powers.lose_one() {
                    self.power_note = Some((format!("- {}", lost.name()), 2.5));
                }
                log::info!("life lost, {} left", self.ship.lives);
            }
            Some(Hit::Dead) => {
                fx::burst(&mut self.particles, cx, cy, 50, 70.0, Tint::Player, &mut self.rng);
                self.set_state(State::Over);
                log::info!("game over, score {}", self.score);
            }
        }
    }
}

fn dist2(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

#[cfg(test)]
mod tests {
    use super::enemy::Side;
    use super::*;

    fn game() -> Game {
        let mut g = Game::new(7);
        g.resize(80, 20);
        g
    }

    #[test]
    fn the_ship_stays_in_the_field_and_fires() {
        let mut g = game();
        let c = Controls { dx: 1.0, dy: -1.0, fire: true };
        for _ in 0..30 {
            g.update(&c);
        }
        assert!(g.shots.iter().any(|s| s.owner == Owner::Player), "held fire shoots");
        for _ in 0..600 {
            g.update(&c);
        }
        assert!(g.ship.x + ship::SHIP_W <= g.w && g.ship.y >= 0.0);
    }

    #[test]
    fn player_fire_kills_and_scores() {
        let mut g = game();
        g.enemies.push(Enemy::scout(Side::Amber, 60.0, 30.0, 0.0, &mut Rng::new(1)));
        let e = g.enemies[0].aabb().center();
        g.shots.push(Shot::new(e.0, e.1, 0.0, 0.0, Owner::Player));
        g.collide();
        assert_eq!(g.enemies[0].hp, 0);
        assert_eq!(g.score, 100);
    }

    #[test]
    fn a_side_never_hurts_its_own_ships_but_hurts_the_other() {
        let mut g = game();
        g.enemies.push(Enemy::scout(Side::Amber, 60.0, 30.0, 0.0, &mut Rng::new(1)));
        g.enemies.push(Enemy::scout(Side::Jade, 100.0, 30.0, 0.0, &mut Rng::new(1)));
        let (a, j) = (g.enemies[0].aabb().center(), g.enemies[1].aabb().center());
        g.shots.push(Shot::new(a.0, a.1, 0.0, 0.0, Owner::Enemy(Side::Amber)));
        g.shots.push(Shot::new(j.0, j.1, 0.0, 0.0, Owner::Enemy(Side::Amber)));
        g.collide();
        assert_eq!(g.enemies[0].hp, 1, "Amber fire passes through Amber");
        assert_eq!(g.enemies[1].hp, 0, "and kills Jade");
        assert_eq!(g.score, 0, "war kills don't score");
    }

    #[test]
    fn enemy_fire_drains_the_shield_and_then_lives() {
        let mut g = game();
        let hit = |g: &mut Game| {
            let (x, y) = g.ship.center();
            g.ship.grace = 0.0;
            g.shots.push(Shot::new(x, y, 0.0, 0.0, Owner::Enemy(Side::Jade)));
            g.collide();
        };
        for _ in 0..4 {
            hit(&mut g);
        }
        assert_eq!(g.ship.shield, 0.0);
        hit(&mut g);
        assert_eq!(g.ship.lives, 2);
        for _ in 0..10 {
            hit(&mut g);
        }
        assert_eq!(g.state, State::Over);
    }

    #[test]
    fn a_long_seeded_run_stays_bounded() {
        let mut g = game();
        // three minutes of flying in circles and firing
        for t in 0..60 * 180 {
            let dy = if (t / 90) % 2 == 0 { 1.0 } else { -1.0 };
            g.update(&Controls { dx: 0.0, dy, fire: true });
        }
        assert!(g.enemies.len() <= 20 && g.shots.len() < 400 && g.particles.len() < 2000);
        assert!(g.score > 0, "something got shot down");
    }

    #[test]
    fn a_portal_turns_the_field_then_shows_the_map_then_flies_on() {
        let mut g = game();
        for _ in 0..60 * 62 {
            g.update(&Controls::default());
            g.ship.grace = 1.0;
        }
        assert_eq!(g.sector.phase, Phase::Portals);
        let p = g.sector.portals[0].clone();
        g.ship.x = p.x;
        g.ship.y = p.y + 4.0;
        g.update(&Controls::default());
        assert_eq!(g.state, State::Turn);
        assert!(g.enemies.is_empty(), "the war stays behind");
        for _ in 0..(TURN_TIME * 60.0) as usize + 2 {
            g.update(&Controls::default());
        }
        assert_eq!(g.state, State::Map);
        g.fly_on();
        assert_eq!((g.state, g.sector.number), (State::Flight, 2));
    }

    #[test]
    fn the_last_true_jump_arrives_home() {
        let mut g = game();
        g.route.at = (1, 0);
        for _ in 0..60 * 62 {
            g.update(&Controls::default());
            g.ship.grace = 1.0;
        }
        g.sector.portals[0].truth = true;
        let p = g.sector.portals[0].clone();
        g.ship.x = p.x;
        g.ship.y = p.y + 4.0;
        for _ in 0..(TURN_TIME * 60.0) as usize + 4 {
            g.update(&Controls::default());
        }
        assert_eq!(g.state, State::Home);
    }

    #[test]
    fn the_cockpit_leads_through_the_first_turn_into_sector_one() {
        let mut g = game();
        g.start_intro();
        g.intro_key();
        g.intro_key();
        assert!(g.intro_card, "the controls card after the story");
        g.intro_key();
        assert_eq!(g.state, State::Cockpit);
        for c in "reboottunescantheeaglehaslanded".chars() {
            g.type_letter(c);
        }
        for _ in 0..60 * 2 {
            g.update(&Controls::default());
        }
        assert_eq!(g.state, State::Turn);
        for _ in 0..(TURN_TIME * 60.0) as usize + 2 {
            g.update(&Controls::default());
        }
        assert_eq!((g.state, g.sector.number), (State::Flight, 1), "no map before the first sector");
    }

    #[test]
    fn typing_a_pod_word_claims_its_power_and_shooting_it_loses_it() {
        let mut g = game();
        g.pods.push(Pod::new(10_001, PowerUp::Rapid, "flask", 100.0, 20.0));
        for c in "flask".chars() {
            g.type_letter(c);
        }
        assert!(g.powers.rapid && g.pods.is_empty());
        g.pods.push(Pod::new(10_002, PowerUp::Bomb, "glad", 100.0, 20.0));
        for k in 0..powerup::POD_HP {
            assert!(!g.pods[0].done, "it survives {k} hits");
            g.shots.push(Shot::new(102.0, 22.0, 0.0, 0.0, Owner::Player));
            g.collide();
        }
        assert!(g.pods[0].done, "a shot pod is lost in the end");
    }

    #[test]
    fn a_smart_bomb_clears_the_screen() {
        let mut g = game();
        g.powers.bombs = 1;
        g.enemies.push(Enemy::gunship(Side::Jade, 100.0, 20.0, 100.0, &mut Rng::new(1)));
        g.shots.push(Shot::new(50.0, 20.0, -70.0, 0.0, Owner::Enemy(Side::Jade)));
        g.bomb();
        assert!(g.enemies.iter().all(|e| e.hp == 0) && g.shots.is_empty());
        assert_eq!((g.powers.bombs, g.score), (0, 400));
        g.bomb();
        assert_eq!(g.score, 400, "no bombs, no bomb");
    }

    #[test]
    fn a_destroyed_mine_bursts_into_shrapnel() {
        let mut g = game();
        g.enemies.push(Enemy::of(Kind::Mine, Side::Amber, 100.0, 30.0, 0.0, &mut Rng::new(1)));
        g.enemies[0].hp = 1;
        // one tick so it has settled into its bob, then a shot at its heart
        g.enemies[0].update(DT, g.w, g.h, 0.0, &mut Rng::new(1));
        let c = g.enemies[0].aabb().center();
        g.shots.push(Shot::new(c.0, c.1, 0.0, 0.0, Owner::Player));
        g.update(&Controls::default());
        assert_eq!(g.shots.iter().filter(|s| s.owner == Owner::Enemy(Side::Amber)).count(), MINE_SHRAPNEL);
    }

    #[test]
    fn a_live_beam_hurts_and_a_warning_does_not() {
        let mut g = game();
        // a lancer on the ship's row
        let row = g.ship.center().1 - 4.0;
        g.enemies.push(Enemy::of(Kind::Lancer, Side::Jade, 120.0, row, 120.0, &mut Rng::new(1)));
        let mut warned = false;
        for _ in 0..60 * 3 {
            g.update(&Controls::default());
            if g.enemies[0].beam().is_some_and(|b| !b.live) {
                warned = true;
                assert_eq!(g.ship.shield, 1.0, "the warning line is harmless");
            }
        }
        assert!(warned && g.ship.shield < 1.0, "then the beam hits");
    }

    #[test]
    fn a_jammer_scrambles_echoes_until_it_is_shot_down() {
        let mut g = game();
        for _ in 0..60 * 11 {
            g.update(&Controls::default());
            g.ship.grace = 1.0;
        }
        g.enemies.push(Enemy::of(Kind::Jammer, Side::Amber, 120.0, 2.0, 120.0, &mut Rng::new(1)));
        g.update(&Controls::default());
        assert!(!g.sector.echoes.is_empty() && g.sector.echoes.iter().all(|e| e.jam > 0.0));
        let jammer = g.enemies.iter().position(|e| e.kind == Kind::Jammer).unwrap();
        g.enemies[jammer].hp = 0;
        g.update(&Controls::default());
        g.update(&Controls::default());
        assert!(g.sector.echoes.iter().all(|e| e.jam == 0.0));
    }

    #[test]
    fn kills_chain_into_a_multiplier_and_a_hit_breaks_it() {
        let mut g = game();
        for i in 0..6 {
            g.enemies.push(Enemy::scout(Side::Amber, 60.0 + i as f32 * 12.0, 30.0, 0.0, &mut Rng::new(1)));
        }
        let centres: Vec<_> = g.enemies.iter().map(|e| e.aabb().center()).collect();
        for (x, y) in centres {
            g.shots.push(Shot::new(x, y, 0.0, 0.0, Owner::Player));
        }
        g.collide();
        assert_eq!(g.score, 4 * 100 + 2 * 200, "from the fifth kill on, double");
        assert_eq!(g.stats.kills, 6);
        g.ship.grace = 0.0;
        g.ship_hit(ship::BULLET_DAMAGE);
        assert_eq!(g.chain.count, 0);
        assert_eq!(g.chain.best, 6);
    }

    #[test]
    fn a_new_best_is_recorded_at_the_end() {
        let mut g = game();
        g.best = 100;
        g.score = 250;
        g.set_state(State::Over);
        assert!(g.record && g.best == 250 && g.finished());
    }

    /// A bot that knows the answers: it types every echo it can, steers into
    /// the true gate, flies on from each map, and is never hit. The whole
    /// journey must run, crawl to home, without a hitch.
    #[test]
    fn a_bot_can_fly_the_whole_way_home() {
        let mut g = game();
        g.start_intro();
        g.intro_key();
        g.intro_key();
        g.intro_key();
        g.skip_cockpit();
        for _ in 0..60 * 60 * 20 {
            g.ship.grace = 1.0;
            match g.state {
                State::Map => g.fly_on(),
                State::Home => break,
                State::Flight => {
                    // type the next letter of the nearest drifting echo that's on screen
                    let next = g.sector.echoes.iter().filter(|e| e.live() && e.x < g.w - 10.0).min_by(|a, b| a.x.total_cmp(&b.x)).and_then(Target::next_letter);
                    if let Some(c) = next {
                        g.type_letter(c);
                    }
                    if let Some(p) = g.sector.portals.iter().find(|p| p.truth) {
                        g.ship.x = p.x;
                        g.ship.y = p.y + 4.0;
                    }
                }
                _ => {}
            }
            g.update(&Controls { dx: 0.0, dy: 0.0, fire: true });
        }
        assert_eq!(g.state, State::Home, "stuck in {:?} at sector {}", g.state, g.sector.number);
        assert_eq!(g.stats.true_jumps as usize, route::START_DISTANCE);
        assert!(g.stats.echoes > 0 && g.stats.kills > 0 && g.score > 0);
    }

    /// Not a check: a difficulty probe. A naive pilot weaves and fires and
    /// never dodges; how long does it last per sector? Run with
    /// `cargo test probe -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn probe_survival_per_sector() {
        const SEEDS: u64 = 24;
        for level in [1, 2, 3, 5] {
            let mut total = 0.0;
            for seed in 0..SEEDS {
                let mut g = Game::new(seed);
                g.resize(96, 22);
                g.sector = Sector::new(level, 0.0, &mut Rng::new(seed));
                g.terrain = Terrain::new(level, &mut Rng::new(seed));
                let mut t = 0;
                let mut dy = 1.0;
                while g.state == State::Flight && t < 60 * 60 * 5 {
                    // weave, turning back before the walls, as a player would
                    let b = g.ship.aabb();
                    let (top, bottom) = g.terrain.depth(b.x1);
                    if (t % 70 == 0) || b.y0 < top + 3.0 || b.y1 > g.h - bottom - 3.0 {
                        dy = if b.y0 < top + 3.0 { 1.0 } else if b.y1 > g.h - bottom - 3.0 { -1.0 } else { -dy };
                    }
                    g.update(&Controls { dx: 0.0, dy, fire: true });
                    // the sector never ends: keep the war coming
                    if g.sector.phase == Phase::Portals {
                        g.sector = Sector::new(level, 0.0, &mut Rng::new(t as u64));
                    }
                    t += 1;
                }
                total += t as f32 / 60.0;
            }
            println!("sector {level}: a naive pilot lasts {:.0} s on average", total / SEEDS as f32);
        }
    }

    #[test]
    fn every_third_sector_a_carrier_guards_the_gates() {
        for (number, guarded) in [(2, false), (3, true)] {
            let mut g = game();
            g.sector = Sector::new(number, 0.0, &mut Rng::new(2));
            for _ in 0..60 * 59 {
                g.update(&Controls::default());
                g.ship.grace = 1.0;
            }
            let guard = g.enemies.iter().any(|e| e.kind == Kind::Carrier && e.x > g.w * 0.4);
            assert_eq!(guard, guarded, "sector {number}");
        }
    }

    #[test]
    fn extra_lives_come_at_milestones_up_to_five() {
        let mut g = game();
        g.score = EXTEND_FIRST;
        g.update(&Controls::default());
        assert_eq!(g.ship.lives, 4);
        g.score = EXTEND_FIRST + 3 * EXTEND_EVERY;
        g.update(&Controls::default());
        assert_eq!(g.ship.lives, MAX_LIVES);
        assert_eq!(g.next_extend, EXTEND_FIRST + 4 * EXTEND_EVERY, "milestones pass even at the cap");
    }

    #[test]
    fn the_shield_recharges_through_the_gate() {
        let mut g = game();
        g.ship.shield = 0.25;
        g.state = State::Map;
        g.fly_on();
        assert_eq!(g.ship.shield, 1.0);
    }

    #[test]
    fn scraping_a_wall_hurts_and_pushes_the_ship_back() {
        let mut g = game();
        g.terrain = Terrain::new(2, &mut Rng::new(3));
        // find a column where the ceiling reaches down
        g.terrain.scroll = (0..2000).map(|s| s as f32).find(|&s| {
            g.terrain.scroll = s;
            g.terrain.depth(g.ship.aabb().x0).0 > 4.0
        }).unwrap();
        g.ship.y = 0.0;
        g.scrape_walls();
        assert!(g.ship.shield < 1.0);
        assert!(g.ship.aabb().y0 >= g.terrain.depth(g.ship.aabb().x0).0 - 0.01);
    }

    #[test]
    fn far_out_on_a_detour_the_true_echo_is_still_the_clearest() {
        let mut g = game();
        g.route.at = (route::MAX_DISTANCE, 0);
        assert_eq!(g.closeness(), 0.0);
        let mut rng = Rng::new(5);
        let mut s = Sector::new(4, g.closeness(), &mut rng);
        for _ in 0..60 * 40 {
            s.update(DT, 160.0, 72.0, &[], &mut rng);
        }
        let truth = s.echoes.iter().find(|e| e.truth).unwrap().clarity;
        assert!(s.echoes.iter().filter(|e| !e.truth).all(|e| e.clarity < truth));
    }

    #[test]
    fn a_target_still_off_screen_cannot_be_locked() {
        let mut g = game();
        g.pods.push(Pod::new(10_001, PowerUp::Rapid, "flask", g.w + 2.0, 20.0));
        g.type_letter('f');
        assert_eq!(g.typing.lock, None);
        g.pods[0].x = g.w - 20.0;
        g.type_letter('f');
        assert_eq!(g.typing.lock, Some(10_001));
    }

    #[test]
    fn a_new_echo_never_shares_a_first_letter_with_a_live_pod() {
        for seed in 0..20 {
            let mut g = Game::new(seed);
            g.resize(80, 20);
            // pods wanting every letter but a handful
            for (i, c) in "abcdefghijklmnopqrstuvw".chars().enumerate() {
                let word: &'static str = Box::leak(c.to_string().into_boxed_str());
                g.pods.push(Pod::new(20_000 + i as u32, PowerUp::Shield, word, 300.0, 10.0));
            }
            for _ in 0..60 * 11 {
                g.update(&Controls::default());
                g.ship.grace = 1.0;
            }
            for e in g.sector.echoes.iter() {
                let first = e.letters().next().unwrap();
                assert!(!"abcdefghijklmnopqrstuvw".contains(first), "seed {seed}: {:?}", e.text);
            }
        }
    }

    #[test]
    fn a_gate_still_sliding_in_cannot_sweep_up_the_ship() {
        let mut g = game();
        for _ in 0..60 * 58 + 10 {
            g.update(&Controls::default());
            g.ship.grace = 1.0;
        }
        assert_eq!(g.sector.phase, Phase::Portals);
        // park the ship where the first gate is passing right now
        let p = g.sector.portals[0].clone();
        assert!(!p.open());
        g.ship.x = (p.x - 4.0).min(g.w - ship::SHIP_W);
        g.ship.y = p.y + 4.0;
        g.update(&Controls::default());
        assert_eq!(g.state, State::Flight, "no jump while it slides in");
    }

    #[test]
    fn moments_become_cues() {
        let mut g = game();
        g.enemies.push(Enemy::scout(Side::Amber, 60.0, 30.0, 0.0, &mut Rng::new(1)));
        let c = g.enemies[0].aabb().center();
        g.shots.push(Shot::new(c.0, c.1, 0.0, 0.0, Owner::Player));
        g.update(&Controls { dx: 0.0, dy: 0.0, fire: true });
        assert!(g.cues.contains(&Cue::Kill) && g.cues.contains(&Cue::Shot));
        g.cues.clear();
        g.ship.grace = 0.0;
        g.ship_hit(ship::BULLET_DAMAGE);
        g.dash(&Controls::default());
        assert_eq!(g.cues, vec![Cue::ShieldHit, Cue::Dash]);
    }

    #[test]
    fn the_stars_hold_still_in_the_cockpit() {
        let mut g = game();
        g.start_intro();
        g.intro_key();
        g.intro_key();
        g.intro_key();
        let before: Vec<f32> = g.stars.stars.iter().map(|s| s.x).collect();
        for _ in 0..120 {
            g.update(&Controls::default());
        }
        let after: Vec<f32> = g.stars.stars.iter().map(|s| s.x).collect();
        assert_eq!(before, after, "the ship is adrift, not flying");
    }

    #[test]
    fn a_carrier_shot_down_leaves_a_drop_the_pilot_can_fly_into() {
        let mut g = game();
        g.enemies.push(Enemy::of(Kind::Carrier, Side::Jade, 100.0, 30.0, 0.0, &mut Rng::new(1)));
        g.enemies[0].hp = 1;
        let c = g.enemies[0].aabb().center();
        g.shots.push(Shot::new(c.0, c.1, 0.0, 0.0, Owner::Player));
        g.update(&Controls::default());
        assert_eq!(g.drops.len(), 2, "a carrier always drops, and a cell besides");
        g.drops.truncate(1);
        let d = g.drops[0].clone();
        g.ship.x = d.x - 6.0;
        g.ship.y = d.y - 3.0;
        let pods = g.stats.pods;
        g.update(&Controls::default());
        assert!(g.drops.is_empty() && g.stats.pods == pods + 1, "flown into, collected");
    }

    #[test]
    fn the_wars_own_kills_leave_nothing() {
        let mut g = game();
        g.enemies.push(Enemy::of(Kind::Carrier, Side::Jade, 100.0, 30.0, 0.0, &mut Rng::new(1)));
        g.enemies[0].hp = 1;
        let c = g.enemies[0].aabb().center();
        g.shots.push(Shot::new(c.0, c.1, 0.0, 0.0, Owner::Enemy(Side::Amber)));
        g.update(&Controls::default());
        assert!(g.drops.is_empty());
    }

    #[test]
    fn a_shield_cell_restores_one_segment() {
        let mut g = game();
        g.ship.shield = 0.5;
        let (x, y) = g.ship.center();
        g.drops.extend(Drop::roll_cell(1.0, x, y, &mut Rng::new(1)));
        g.update(&Controls::default());
        assert!((g.ship.shield - 0.6).abs() < 1e-5, "{}", g.ship.shield);
        assert!(g.cues.contains(&Cue::Cell));
    }
}
