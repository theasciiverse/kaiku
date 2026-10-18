//! The simulation: no terminal in here, so it runs headless in tests.

pub mod echo;
pub mod enemy;
pub mod fx;
pub mod rng;
pub mod sector;
pub mod ship;
pub mod shot;
pub mod spawn;
pub mod stars;
pub mod typing;

use crate::input::Controls;
use enemy::Enemy;
use fx::{Particle, Tint};
use rng::Rng;
use sector::{Phase, Sector};
use ship::{Hit, Ship};
use shot::{Owner, Shot};
use spawn::Spawner;
use stars::Starfield;
use typing::{Key, Typing};

/// Fixed simulation step: 60 Hz, as in Entity.
pub const DT: f32 = 1.0 / 60.0;
const SHOT_SPEED: f32 = 260.0;
const ENEMY_SHOT_SPEED: f32 = 70.0;
/// Chance that a ship fires at the other side rather than at the player.
const WAR_FIRE: f32 = 0.5;
/// Jumps between the surge and home along the shortest route.
pub const START_DISTANCE: u32 = 6;
/// How long a receiver note stays up, in seconds.
const NOTE_TIME: f32 = 6.0;

/// An axis-aligned box in dots.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
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
    Flight,
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
    pub tick: u64,
    pub sector: Sector,
    pub typing: Typing,
    /// jumps left to home
    pub distance: u32,
    /// a line for the receiver (jump results), and seconds it has left
    pub note: Option<(String, f32)>,
    spawner: Spawner,
    rng: Rng,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut rng = Rng::new(seed);
        let sector = Sector::new(1, 0.0, &mut rng);
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
            tick: 0,
            sector,
            typing: Typing::default(),
            distance: START_DISTANCE,
            note: None,
            spawner: Spawner::default(),
            rng,
        }
    }

    /// A fresh run in the same field.
    pub fn restart(&mut self) {
        let mut g = Game::new(self.rng.next_u64());
        g.resize_dots(self.w, self.h);
        *self = g;
        log::info!("restart");
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
        self.stars.update(DT, 1.0, self.w, self.h, &mut self.rng);
        if self.state == State::Flight && self.ship.update(c, DT, self.w, self.h) {
            let (x, y) = self.ship.nose();
            self.shots.push(Shot::new(x, y, SHOT_SPEED, 0.0, Owner::Player));
        }
        if self.sector.phase == Phase::Flight {
            self.spawner.update(DT, self.w, self.h, &mut self.rng, &mut self.enemies);
        }
        if self.state == State::Flight {
            self.sector.update(DT, self.w, self.h, &mut self.rng);
            self.enter_portal();
        }
        if let Some((_, t)) = &mut self.note {
            *t -= DT;
            if *t <= 0.0 {
                self.note = None;
            }
        }
        for i in 0..self.enemies.len() {
            if self.enemies[i].update(DT, self.w, self.h, &mut self.rng) {
                self.enemy_fires(i);
            }
        }
        shot::update(&mut self.shots, DT, self.w, self.h);
        self.collide();
        self.enemies.retain(|e| e.hp > 0 && !e.gone());
        fx::update(&mut self.particles, DT);
        if self.tick.is_multiple_of(30) {
            log::trace!("tick {} enemies {} shots {} shield {} lives {}", self.tick, self.enemies.len(), self.shots.len(), self.ship.shield, self.ship.lives);
        }
    }

    /// A typed letter, for the typing layer.
    pub fn type_letter(&mut self, c: char) {
        if self.state != State::Flight {
            return;
        }
        if let Key::Decoded(id) = self.typing.key(c, &mut self.sector.echoes) {
            if let Some(e) = self.sector.echo(id) {
                log::info!("decoded echo {id} truth={} {:?}", e.truth, e.text);
                self.score += 50 * e.text.len() as u32;
            }
        }
    }

    pub fn drop_lock(&mut self) {
        self.typing.drop_lock(&mut self.sector.echoes);
    }

    /// How near home, 0 (where the surge left you) to 1.
    pub fn closeness(&self) -> f32 {
        1.0 - self.distance as f32 / START_DISTANCE as f32
    }

    /// The jump: a true portal brings home one step closer, a false one adds a
    /// detour. Then the next sector begins.
    fn enter_portal(&mut self) {
        let ship = self.ship.aabb();
        let Some(p) = self.sector.portals.iter().find(|p| p.aabb().overlaps(&ship)) else { return };
        let truth = p.truth;
        let blind = p.echo.is_none();
        self.distance = if truth { self.distance.saturating_sub(1) } else { self.distance + 1 };
        let line = match (truth, blind) {
            (true, _) if self.distance == 0 => "the echoes were true: home".to_string(),
            (true, false) => format!("true echo · closer to home · {} jumps left", self.distance),
            (true, true) => format!("a lucky blind jump · {} jumps left", self.distance),
            (false, false) => format!("the echo was jammed · a detour · {} jumps left", self.distance),
            (false, true) => format!("a blind jump, off course · {} jumps left", self.distance),
        };
        log::info!("jump from sector {} truth={truth} distance={}", self.sector.number, self.distance);
        self.note = Some((line, NOTE_TIME));
        if self.distance == 0 {
            self.state = State::Home;
            return;
        }
        let closeness = self.closeness();
        self.sector = Sector::new(self.sector.number + 1, closeness, &mut self.rng);
        self.typing = Typing::default();
        self.enemies.clear();
        self.shots.clear();
        self.spawner = Spawner::default();
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

    fn collide(&mut self) {
        let flying = self.state == State::Flight;
        let ship_box = self.ship.aabb();
        let mut hits: Vec<f32> = Vec::new();
        for s in self.shots.iter_mut() {
            let owner = s.owner;
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
                        self.score += e.kind.points();
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
            for e in self.enemies.iter_mut().filter(|e| e.hp > 0 && e.aabb().overlaps(&ship_box)) {
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
        match self.ship.hit(damage) {
            None => {}
            Some(Hit::Shield) => fx::burst(&mut self.particles, cx, cy, 6, 35.0, Tint::Spark, &mut self.rng),
            Some(Hit::Life) => {
                fx::burst(&mut self.particles, cx, cy, 30, 60.0, Tint::Player, &mut self.rng);
                log::info!("life lost, {} left", self.ship.lives);
            }
            Some(Hit::Dead) => {
                fx::burst(&mut self.particles, cx, cy, 50, 70.0, Tint::Player, &mut self.rng);
                self.state = State::Over;
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
        let c = Controls { dx: 1, dy: -1, fire: true };
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
            let dy = if (t / 90) % 2 == 0 { 1 } else { -1 };
            g.update(&Controls { dx: 0, dy, fire: true });
        }
        assert!(g.enemies.len() <= 20 && g.shots.len() < 400 && g.particles.len() < 2000);
        assert!(g.score > 0, "something got shot down");
    }
}
