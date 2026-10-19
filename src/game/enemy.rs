//! Ships of the two warring sides. They fly in from the right, hunt the player
//! and fight each other. Positions and sizes are in dots.

use super::rng::Rng;
use super::Aabb;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Amber,
    Jade,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Amber => Side::Jade,
            Side::Jade => Side::Amber,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// small, fast, flies through in snaking lines
    Scout,
    /// holds position on the right and fires
    Gunship,
    /// lines up on the player's row, flashes, then dashes straight at it
    Interceptor,
    /// drifts with the tunnel; destroyed, it bursts into shrapnel
    Mine,
    /// holds on the right and fires a telegraphed beam along its row
    Lancer,
    /// scrambles every echo in range while it lives; it never fires
    Jammer,
    /// a capital ship: crosses slowly, fires fans, launches scouts
    Carrier,
}

impl Kind {
    /// Sprite size in dots (half-block pixels are 2×2 dots).
    pub fn size(self) -> (f32, f32) {
        match self {
            Kind::Scout => (10.0, 6.0),
            Kind::Gunship => (14.0, 10.0),
            Kind::Interceptor => (12.0, 6.0),
            Kind::Mine => (6.0, 6.0),
            Kind::Lancer => (14.0, 8.0),
            Kind::Jammer => (12.0, 10.0),
            Kind::Carrier => (32.0, 16.0),
        }
    }

    fn hp(self) -> u8 {
        match self {
            Kind::Scout => 1,
            Kind::Gunship => 4,
            Kind::Interceptor => 2,
            Kind::Mine => 2,
            Kind::Lancer => 4,
            Kind::Jammer => 6,
            Kind::Carrier => 30,
        }
    }

    /// The chance a kill leaves a power-up behind: bigger ships, better odds.
    pub fn drop_chance(self) -> f32 {
        match self {
            Kind::Scout => 0.04,
            Kind::Interceptor => 0.08,
            Kind::Mine => 0.10,
            Kind::Gunship => 0.20,
            Kind::Lancer => 0.25,
            Kind::Jammer => 0.5,
            Kind::Carrier => 1.0,
        }
    }

    pub fn points(self) -> u32 {
        match self {
            Kind::Scout => 100,
            Kind::Gunship => 400,
            Kind::Interceptor => 250,
            Kind::Mine => 150,
            Kind::Lancer => 500,
            Kind::Jammer => 600,
            Kind::Carrier => 3000,
        }
    }
}

/// What a ship does this tick, beyond moving.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    /// one aimed shot
    Fire,
    /// three shots fanned out to the left
    Fan,
    /// launch a scout from the hangar
    Launch,
}

/// A lancer's beam: along its row to the left edge. While `live` is false it is
/// the warning line, and harmless.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Beam {
    pub y: f32,
    pub x_end: f32,
    pub live: bool,
}

const SCOUT_SPEED: f32 = 38.0;
const LEAVE_SPEED: f32 = 60.0;
/// A gunship holds its position this long, then leaves.
const GUNSHIP_STAY: f32 = 9.0;
const INTERCEPT_ENTER: f32 = 70.0;
const INTERCEPT_DASH: f32 = 150.0;
/// The interceptor's tell: it lines up and flashes this long before dashing.
const INTERCEPT_AIM: f32 = 0.7;
/// How fast it slides onto the player's row while aiming, dots per second.
const INTERCEPT_TRACK: f32 = 90.0;
const MINE_DRIFT: f32 = 12.0;
/// The lancer's cycle: rest, warning line, beam.
const LANCE_REST: f32 = 1.6;
const LANCE_WARN: f32 = 1.0;
const LANCE_FIRE: f32 = 0.6;
const LANCER_STAY: f32 = 12.0;
const JAMMER_STAY: f32 = 22.0;
const CARRIER_SPEED: f32 = 7.0;
/// A carrier guarding the gates stays this long before it gives up.
const GUARD_STAY: f32 = 45.0;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub kind: Kind,
    pub side: Side,
    pub x: f32,
    pub y: f32,
    pub hp: u8,
    /// seconds of white hit flash left
    pub flash: f32,
    /// destroyed by the player (gun or bomb), not by the war: such a kill
    /// may leave a power-up behind
    pub by_player: bool,
    base_y: f32,
    phase: f32,
    age: f32,
    hold_x: f32,
    fire_in: f32,
    /// kind-specific: the interceptor's aim/dash, the lancer's cycle
    stage: u8,
    timer: f32,
    launch_in: f32,
}

impl Enemy {
    pub fn scout(side: Side, x: f32, y: f32, phase: f32, rng: &mut Rng) -> Enemy {
        Enemy::new(Kind::Scout, side, x, y, phase, x, rng)
    }

    pub fn gunship(side: Side, x: f32, y: f32, hold_x: f32, rng: &mut Rng) -> Enemy {
        let phase = rng.range(0.0, std::f32::consts::TAU);
        Enemy::new(Kind::Gunship, side, x, y, phase, hold_x, rng)
    }

    /// Any other kind, holding (if it holds) at `hold_x`. A carrier with a
    /// `hold_x` is a gate guard; with 0 it crosses the tunnel.
    pub fn of(kind: Kind, side: Side, x: f32, y: f32, hold_x: f32, rng: &mut Rng) -> Enemy {
        let phase = rng.range(0.0, std::f32::consts::TAU);
        Enemy::new(kind, side, x, y, phase, hold_x, rng)
    }

    fn new(kind: Kind, side: Side, x: f32, y: f32, phase: f32, hold_x: f32, rng: &mut Rng) -> Enemy {
        Enemy {
            kind,
            side,
            x,
            y,
            hp: kind.hp(),
            by_player: false,
            flash: 0.0,
            base_y: y,
            phase,
            age: 0.0,
            hold_x,
            fire_in: first_shot(kind, rng),
            stage: 0,
            timer: 0.0,
            launch_in: 3.0,
        }
    }

    pub fn aabb(&self) -> Aabb {
        let (w, h) = self.kind.size();
        // a little smaller than the sprite, so grazes don't count
        Aabb::new(self.x + 1.0, self.y + 1.0, self.x + w - 1.0, self.y + h - 1.0)
    }

    /// Where shots leave the ship (its nose, on the left).
    pub fn muzzle(&self) -> (f32, f32) {
        (self.x, self.y + self.kind.size().1 / 2.0)
    }

    pub fn gone(&self) -> bool {
        self.x + self.kind.size().0 < -2.0
    }

    /// The interceptor lining up, about to dash: its tell, for the renderer.
    pub fn aiming(&self) -> bool {
        self.kind == Kind::Interceptor && self.stage == 1
    }

    pub fn beam(&self) -> Option<Beam> {
        if self.kind != Kind::Lancer || !matches!(self.stage, 1 | 2) {
            return None;
        }
        let (x, y) = self.muzzle();
        Some(Beam { y, x_end: x, live: self.stage == 2 })
    }

    /// Moves; returns what the ship does beyond moving. `target_y` is the row
    /// of the player's ship, which interceptors line up on.
    pub fn update(&mut self, dt: f32, w: f32, h: f32, target_y: f32, rng: &mut Rng) -> Action {
        self.age += dt;
        self.flash = (self.flash - dt).max(0.0);
        let (sw, sh) = self.kind.size();
        let mut action = Action::None;
        match self.kind {
            Kind::Scout => {
                self.x -= SCOUT_SPEED * dt;
                self.y = self.base_y + (self.phase + self.age * 2.2).sin() * 7.0;
            }
            Kind::Gunship => self.hold_then_leave(dt, GUNSHIP_STAY, 0.9, 5.0),
            Kind::Interceptor => match self.stage {
                // in fast, to its line
                0 => {
                    self.x -= INTERCEPT_ENTER * dt;
                    if self.x <= self.hold_x {
                        self.stage = 1;
                    }
                }
                // line up on the player's row: the tell
                1 => {
                    let want = target_y - sh / 2.0;
                    self.y += (want - self.y).clamp(-INTERCEPT_TRACK * dt, INTERCEPT_TRACK * dt);
                    self.timer += dt;
                    if self.timer >= INTERCEPT_AIM {
                        self.stage = 2;
                    }
                }
                _ => self.x -= INTERCEPT_DASH * dt,
            },
            Kind::Mine => {
                self.x -= MINE_DRIFT * dt;
                self.y = self.base_y + (self.phase + self.age * 1.3).sin() * 3.0;
            }
            Kind::Lancer => {
                self.hold_then_leave(dt, LANCER_STAY, 0.0, 0.0);
                // only cycles while holding, fully on screen
                if self.age < LANCER_STAY && self.x + sw < w {
                    self.timer += dt;
                    let limit = [LANCE_REST, LANCE_WARN, LANCE_FIRE][self.stage as usize];
                    if self.timer >= limit {
                        self.timer = 0.0;
                        self.stage = (self.stage + 1) % 3;
                    }
                } else {
                    self.stage = 0;
                }
            }
            Kind::Jammer => self.hold_then_leave(dt, JAMMER_STAY, 0.6, 4.0),
            Kind::Carrier => {
                if self.hold_x > 0.0 {
                    // a gate guard: glides in front of the gates and stays
                    self.hold_then_leave(dt, GUARD_STAY, 0.4, 6.0);
                } else {
                    self.x -= CARRIER_SPEED * dt;
                    self.y = self.base_y + (self.phase + self.age * 0.4).sin() * 3.0;
                }
                self.launch_in -= dt;
                if self.launch_in <= 0.0 && self.x + sw < w {
                    self.launch_in = rng.range(3.5, 5.0);
                    action = Action::Launch;
                }
            }
        }
        self.y = self.y.clamp(0.0, (h - sh).max(0.0));
        if action != Action::None {
            return action;
        }
        self.fire_in -= dt;
        // only fire once fully on screen; some kinds never do
        let armed = matches!(self.kind, Kind::Scout | Kind::Gunship | Kind::Carrier);
        if armed && self.fire_in <= 0.0 && self.x + sw < w {
            self.fire_in = match self.kind {
                Kind::Scout => rng.range(2.5, 5.0),
                Kind::Carrier => rng.range(2.2, 3.0),
                _ => rng.range(1.0, 1.8),
            };
            return if self.kind == Kind::Carrier { Action::Fan } else { Action::Fire };
        }
        Action::None
    }

    /// Glides to `hold_x`, bobbing, for `stay` seconds, then leaves left.
    fn hold_then_leave(&mut self, dt: f32, stay: f32, bob_speed: f32, bob: f32) {
        if self.age < stay {
            self.x += (self.hold_x - self.x) * (1.5 * dt).min(1.0);
        } else {
            self.x -= LEAVE_SPEED * dt;
        }
        self.y = self.base_y + (self.phase + self.age * bob_speed).sin() * bob;
    }
}

fn first_shot(kind: Kind, rng: &mut Rng) -> f32 {
    match kind {
        Kind::Scout => rng.range(1.5, 4.0),
        Kind::Carrier => rng.range(2.0, 3.0),
        _ => rng.range(1.2, 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(e: &mut Enemy, secs: f32, target_y: f32) -> Vec<Action> {
        let mut rng = Rng::new(4);
        (0..(secs * 60.0) as usize).map(|_| e.update(1.0 / 60.0, 200.0, 80.0, target_y, &mut rng)).filter(|a| *a != Action::None).collect()
    }

    #[test]
    fn an_interceptor_lines_up_on_the_player_then_dashes() {
        let mut e = Enemy::of(Kind::Interceptor, Side::Amber, 200.0, 10.0, 150.0, &mut Rng::new(1));
        run(&mut e, 1.0, 60.0);
        assert!(e.aiming(), "the tell");
        run(&mut e, 0.35, 60.0);
        assert!(e.aiming() && (e.y + 3.0 - 60.0).abs() < 2.0, "on the player's row: {}", e.y);
        let x = e.x;
        run(&mut e, 0.5, 60.0);
        assert!(x - e.x > 60.0, "and dashing");
    }

    #[test]
    fn a_lancer_warns_before_its_beam_and_the_warning_is_harmless() {
        let mut e = Enemy::of(Kind::Lancer, Side::Jade, 150.0, 20.0, 150.0, &mut Rng::new(1));
        run(&mut e, 1.7, 0.0);
        assert_eq!(e.beam().map(|b| b.live), Some(false), "warning first");
        run(&mut e, 1.0, 0.0);
        assert_eq!(e.beam().map(|b| b.live), Some(true), "then the beam");
        assert!(run(&mut e, 5.0, 0.0).is_empty(), "lancers never fire bullets");
    }

    #[test]
    fn a_carrier_launches_and_fires_fans() {
        let mut e = Enemy::of(Kind::Carrier, Side::Amber, 150.0, 20.0, 0.0, &mut Rng::new(1));
        let actions = run(&mut e, 12.0, 0.0);
        assert!(actions.contains(&Action::Launch) && actions.contains(&Action::Fan));
    }

    #[test]
    fn jammers_and_mines_never_fire() {
        for kind in [Kind::Jammer, Kind::Mine] {
            let mut e = Enemy::of(kind, Side::Amber, 150.0, 20.0, 150.0, &mut Rng::new(1));
            assert!(run(&mut e, 10.0, 0.0).is_empty(), "{kind:?}");
        }
    }
}
