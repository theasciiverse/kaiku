//! The typing layer, ZType-style: a letter locks the nearest target (echo or
//! power-up pod) whose next letter it is, and then every key goes to that
//! target until it is done or the lock is dropped (Backspace). Progress stays
//! with the target, so dropping a lock never throws work away.

use super::echo::{Echo, EchoState};
use super::powerup::Pod;

/// Something the player can type out: an echo or a power-up pod.
pub trait Target {
    fn id(&self) -> u32;
    /// still on the field and wanting letters
    fn live(&self) -> bool;
    fn next_letter(&self) -> Option<char>;
    /// how far along the field, for picking the nearest
    fn x(&self) -> f32;
    /// one typed letter; true if it was the right one
    fn feed(&mut self, c: char) -> bool;
    fn finished(&self) -> bool;
    fn set_locked(&mut self, locked: bool);
}

impl Target for Echo {
    fn id(&self) -> u32 {
        self.id
    }
    fn live(&self) -> bool {
        self.state == EchoState::Drifting
    }
    fn next_letter(&self) -> Option<char> {
        Echo::next_letter(self)
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn feed(&mut self, c: char) -> bool {
        self.key(c)
    }
    fn finished(&self) -> bool {
        self.state == EchoState::Decoded
    }
    fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

impl Target for Pod {
    fn id(&self) -> u32 {
        self.id
    }
    fn live(&self) -> bool {
        !self.done && !self.claimed()
    }
    fn next_letter(&self) -> Option<char> {
        Pod::next_letter(self)
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn feed(&mut self, c: char) -> bool {
        self.key(c)
    }
    fn finished(&self) -> bool {
        self.claimed()
    }
    fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    /// right letter, the target still has more
    Hit,
    /// right letter, and that finished target `id`
    Done(u32),
    /// wrong letter for the locked target
    Miss,
    /// no target wanted that letter
    Stray,
}

#[derive(Clone, Debug, Default)]
pub struct Typing {
    pub lock: Option<u32>,
    pub keys: u32,
    pub misses: u32,
}

impl Typing {
    pub fn key(&mut self, c: char, targets: &mut [&mut dyn Target]) -> Key {
        self.keys += 1;
        // a lock on a target that is gone (finished, lost) is no lock
        if self.lock.is_some_and(|id| !targets.iter().any(|t| t.id() == id && t.live())) {
            self.lock = None;
        }
        let target = match self.lock {
            Some(id) => targets.iter_mut().find(|t| t.id() == id),
            None => targets.iter_mut().filter(|t| t.live() && t.next_letter() == Some(c)).min_by(|a, b| a.x().total_cmp(&b.x())),
        };
        let Some(t) = target else {
            self.misses += 1;
            return Key::Stray;
        };
        if !t.feed(c) {
            self.misses += 1;
            return Key::Miss;
        }
        if t.finished() {
            t.set_locked(false);
            self.lock = None;
            return Key::Done(t.id());
        }
        t.set_locked(true);
        self.lock = Some(t.id());
        Key::Hit
    }

    pub fn drop_lock(&mut self, targets: &mut [&mut dyn Target]) {
        for t in targets.iter_mut() {
            t.set_locked(false);
        }
        self.lock = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rng::Rng;

    fn targets(echoes: &mut [Echo]) -> Vec<&mut dyn Target> {
        echoes.iter_mut().map(|e| e as &mut dyn Target).collect()
    }

    fn echo(id: u32, text: &str, x: f32) -> Echo {
        Echo::new(id, text.to_string(), "test", true, 0.5, (x, 20.0), &mut Rng::new(id as u64))
    }

    #[test]
    fn the_first_letter_locks_the_nearest_match() {
        let mut echoes = vec![echo(1, "one small step", 120.0), echo(2, "our moon", 60.0), echo(3, "hello", 10.0)];
        let mut t = Typing::default();
        assert_eq!(t.key('o', &mut targets(&mut echoes)), Key::Hit);
        assert_eq!(t.lock, Some(2), "the nearer of the two o-echoes");
        // locked: an n now is wrong for "our", even though "one" wants n next
        assert_eq!(t.key('n', &mut targets(&mut echoes)), Key::Miss);
        assert_eq!(echoes[0].progress, 0);
    }

    #[test]
    fn decoding_releases_the_lock_and_backspace_keeps_progress() {
        let mut echoes = vec![echo(1, "hi you", 50.0)];
        let mut t = Typing::default();
        t.key('h', &mut targets(&mut echoes));
        t.drop_lock(&mut targets(&mut echoes));
        assert_eq!(t.lock, None);
        // re-lock with the next letter needed
        assert_eq!(t.key('i', &mut targets(&mut echoes)), Key::Hit);
        for c in "yo".chars() {
            t.key(c, &mut targets(&mut echoes));
        }
        assert_eq!(t.key('u', &mut targets(&mut echoes)), Key::Done(1));
        assert_eq!(t.lock, None);
        assert_eq!(t.key('h', &mut targets(&mut echoes)), Key::Stray, "a decoded echo takes no more keys");
    }

    #[test]
    fn pods_and_echoes_share_one_lock() {
        let mut echoes = [echo(1, "hello", 80.0)];
        let mut pods = [Pod::new(7, crate::game::powerup::PowerUp::Rapid, "flask", 40.0, 10.0)];
        let mut t = Typing::default();
        let mut all: Vec<&mut dyn Target> = vec![&mut echoes[0], &mut pods[0]];
        assert_eq!(t.key('f', &mut all), Key::Hit);
        assert_eq!(t.lock, Some(7));
        for c in "las".chars() {
            t.key(c, &mut all);
        }
        assert_eq!(t.key('k', &mut all), Key::Done(7));
    }
}
