//! `--demo`: the game plays itself, for an attract screen and the catalog's
//! preview recording. The autopilot plays fair: no invulnerability, and it
//! never peeks at which gate is true. It reads the receiver's verdicts as a
//! player would. It weaves away from fire and beams, keeps off the walls,
//! picks up drops, holds its fire when a pod is in front of it, types echoes
//! and pod words at a human pace, and bombs a crowded screen.

use crate::game::echo::{EchoState, Verdict};
use crate::game::sector::Phase;
use crate::game::shot::Owner;
use crate::game::typing::Target;
use crate::game::{Game, State};
use crate::input::Controls;

/// One letter every this many ticks: about seven a second.
const TYPE_EVERY: u64 = 8;

/// What the autopilot does this tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Moves {
    pub controls: Controls,
    pub letter: Option<char>,
    /// Enter: a bomb in flight, "go on" anywhere else
    pub enter: bool,
    pub dash: bool,
}

#[derive(Default)]
pub struct Autopilot {
    weave: f32,
}

impl Autopilot {
    pub fn moves(&mut self, game: &Game) -> Moves {
        let mut m = Moves::default();
        match game.state {
            State::Intro | State::Cockpit => m.enter = true,
            State::Map => m.enter = game.state_t > 2.5,
            State::Over | State::Home => m.enter = game.state_t > 7.0,
            State::Credits => m.enter = game.state_t > 20.0,
            State::Turn => {}
            State::Flight => self.fly(game, &mut m),
        }
        m
    }

    fn fly(&mut self, game: &Game, m: &mut Moves) {
        self.weave += 0.02;
        let ship = game.ship.aabb();
        let (sx, sy) = game.ship.center();
        // where it would like to be: mid-field, weaving gently, unless there's
        // something to fetch or a gate to fly into
        let mut want = (game.w * 0.18, game.h / 2.0 + self.weave.sin() * game.h * 0.25);
        if let Some(d) = game.drops.iter().filter(|d| d.x > sx - 4.0).min_by(|a, b| a.x.total_cmp(&b.x)) {
            want = (want.0.max(d.x - 30.0), d.y + 3.0);
        }
        if game.sector.phase == Phase::Portals {
            if let Some(p) = self.pick_gate(game) {
                want = (p.x + 6.0, p.y + 10.0);
            }
        }
        let mut dy = (want.1 - sy).clamp(-8.0, 8.0) / 8.0;
        let dx = (want.0 - sx).clamp(-8.0, 8.0) / 8.0;
        // threats: enemy fire ahead on its row, and beams along it
        let mut danger: Option<f32> = None;
        for s in game.shots.iter().filter(|s| s.owner != Owner::Player) {
            let ahead = s.x - ship.x1;
            if (-6.0..60.0).contains(&ahead) && (s.y - sy).abs() < 9.0 {
                danger = Some(danger.map_or(s.y, |d: f32| if (d - sy).abs() < (s.y - sy).abs() { d } else { s.y }));
                if ahead < 14.0 && (s.y - sy).abs() < 5.0 {
                    m.dash = true;
                }
            }
        }
        for b in game.enemies.iter().filter_map(|e| e.beam()) {
            if (b.y - sy).abs() < 9.0 {
                danger = Some(b.y);
            }
        }
        if let Some(y) = danger {
            dy = if y > sy { -1.0 } else { 1.0 };
        }
        // and the walls
        let (top, bottom) = game.terrain.depth(ship.x1);
        if ship.y0 < top + 3.0 {
            dy = dy.max(0.6);
        } else if ship.y1 > game.h - bottom - 3.0 {
            dy = dy.min(-0.6);
        }
        // hold fire when a pod is in the line of fire
        let pod_ahead = game.pods.iter().any(|p| p.x > sx && (p.y + 3.0 - sy).abs() < 7.0);
        m.controls = Controls { dx, dy, fire: !pod_ahead };
        let crowd = game.enemies.iter().filter(|e| e.x < game.w).count();
        m.enter = crowd >= 7 && game.powers.bombs > 0;
        if game.tick.is_multiple_of(TYPE_EVERY) {
            m.letter = self.next_letter(game);
        }
    }

    /// The next letter of what it's typing: the locked target, or else the
    /// nearest one on screen (pods first: power before story).
    fn next_letter(&self, game: &Game) -> Option<char> {
        if let Some(id) = game.typing.lock {
            let echo = game.sector.echoes.iter().find(|e| e.id == id).and_then(Target::next_letter);
            let pod = game.pods.iter().find(|p| p.id == id).and_then(Target::next_letter);
            if let Some(c) = echo.or(pod) {
                return Some(c);
            }
        }
        let edge = game.w - 2.0;
        let pod = game.pods.iter().filter(|p| p.live() && p.x < edge).min_by(|a, b| a.x.total_cmp(&b.x));
        if let Some(p) = pod {
            return Target::next_letter(p);
        }
        game.sector.echoes.iter().filter(|e| e.live() && e.x < edge).min_by(|a, b| a.x.total_cmp(&b.x)).and_then(Target::next_letter)
    }

    /// A gate to fly into, judged only by what the receiver says: a steady
    /// echo's gate, else one not ruled out (named first), once settled.
    fn pick_gate<'a>(&self, game: &'a Game) -> Option<&'a crate::game::sector::Portal> {
        let s = &game.sector;
        let verdict = |p: &crate::game::sector::Portal| s.named(p).map(|e| e.verdict());
        let analysing = s.echoes.iter().any(|e| e.state == EchoState::Decoded && e.verdict() == Verdict::Pending);
        let steady = s.portals.iter().find(|p| verdict(p) == Some(Verdict::Steady));
        // wait for an analysis that's about to finish rather than guess
        if steady.is_none() && analysing {
            return None;
        }
        steady
            .or_else(|| s.portals.iter().find(|p| verdict(p) == Some(Verdict::Pending)))
            .or_else(|| s.portals.iter().find(|p| verdict(p) != Some(Verdict::Jammed)))
            .filter(|p| p.open())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The autopilot, playing fair, gets somewhere: it scores, decodes, and
    /// jumps through gates, over ten minutes of play from sector 1.
    #[test]
    fn the_autopilot_plays_a_real_game() {
        let mut g = Game::new(5);
        g.resize(96, 22);
        g.start_at(1);
        let mut pilot = Autopilot::default();
        let mut jumps = 0;
        for _ in 0..60 * 60 * 10 {
            let m = pilot.moves(&g);
            if let Some(c) = m.letter {
                g.type_letter(c);
            }
            if m.dash {
                g.dash(&m.controls);
            }
            if m.enter {
                match g.state {
                    State::Flight => g.bomb(),
                    State::Map => g.fly_on(),
                    State::Over | State::Home => g.restart(),
                    _ => {}
                }
            }
            let before = g.state;
            g.update(&m.controls);
            if before == State::Flight && g.state == State::Turn {
                jumps += 1;
            }
            g.cues.clear();
        }
        println!("ten minutes: {jumps} jumps, {} true, {} echoes, {} kills, score {}, state {:?}", g.stats.true_jumps, g.stats.echoes, g.stats.kills, g.score, g.state);
        assert!(jumps >= 3, "only {jumps} jumps in ten minutes");
        assert!(g.stats.echoes > 0 || jumps > 3);
    }
}
