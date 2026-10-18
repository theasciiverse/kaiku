//! The typing layer, ZType-style: a letter locks the nearest echo whose next
//! letter it is, and then every key goes to that echo until it is decoded or
//! the lock is dropped (Backspace). Progress stays with the echo, so dropping
//! a lock never throws work away.

use super::echo::{Echo, EchoState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    /// right letter, the echo still has more
    Hit,
    /// right letter, and that finished the echo
    Decoded(u32),
    /// wrong letter for the locked echo
    Miss,
    /// no echo wanted that letter
    Stray,
}

#[derive(Clone, Debug, Default)]
pub struct Typing {
    pub lock: Option<u32>,
    pub keys: u32,
    pub misses: u32,
}

impl Typing {
    pub fn key(&mut self, c: char, echoes: &mut [Echo]) -> Key {
        self.keys += 1;
        // a lock on an echo that is gone (decoded, lost) is no lock
        if self.lock.is_some_and(|id| !echoes.iter().any(|e| e.id == id && e.state == EchoState::Drifting)) {
            self.lock = None;
        }
        let target = match self.lock {
            Some(id) => echoes.iter_mut().find(|e| e.id == id),
            None => echoes
                .iter_mut()
                .filter(|e| e.state == EchoState::Drifting && e.next_letter() == Some(c))
                .min_by(|a, b| a.x.total_cmp(&b.x)),
        };
        let Some(e) = target else {
            self.misses += 1;
            return Key::Stray;
        };
        if !e.key(c) {
            self.misses += 1;
            return Key::Miss;
        }
        if e.state == EchoState::Decoded {
            self.lock = None;
            return Key::Decoded(e.id);
        }
        e.locked = true;
        self.lock = Some(e.id);
        Key::Hit
    }

    pub fn drop_lock(&mut self, echoes: &mut [Echo]) {
        for e in echoes.iter_mut() {
            e.locked = false;
        }
        self.lock = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rng::Rng;

    fn echo(id: u32, text: &str, x: f32) -> Echo {
        Echo::new(id, text.to_string(), "test", true, 0.5, (x, 20.0), &mut Rng::new(id as u64))
    }

    #[test]
    fn the_first_letter_locks_the_nearest_match() {
        let mut echoes = vec![echo(1, "one small step", 120.0), echo(2, "our moon", 60.0), echo(3, "hello", 10.0)];
        let mut t = Typing::default();
        assert_eq!(t.key('o', &mut echoes), Key::Hit);
        assert_eq!(t.lock, Some(2), "the nearer of the two o-echoes");
        // locked: an n now is wrong for "our", even though "one" wants n next
        assert_eq!(t.key('n', &mut echoes), Key::Miss);
        assert_eq!(echoes[0].progress, 0);
    }

    #[test]
    fn decoding_releases_the_lock_and_backspace_keeps_progress() {
        let mut echoes = vec![echo(1, "hi you", 50.0)];
        let mut t = Typing::default();
        t.key('h', &mut echoes);
        t.drop_lock(&mut echoes);
        assert_eq!(t.lock, None);
        // re-lock with the next letter needed
        assert_eq!(t.key('i', &mut echoes), Key::Hit);
        for c in "yo".chars() {
            t.key(c, &mut echoes);
        }
        assert_eq!(t.key('u', &mut echoes), Key::Decoded(1));
        assert_eq!(t.lock, None);
        assert_eq!(t.key('h', &mut echoes), Key::Stray, "a decoded echo takes no more keys");
    }
}
