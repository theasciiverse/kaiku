# Kaiku — CLAUDE.md

**Status:** Fresh start (2026-10-18). This file holds only what has been
decided. Earlier plans (v0.1 vertical rain, v0.2 language-faction shmup) are
**not binding**. They stay in git history (`a3c8492`, `4f1c36c`) as a quarry
of ideas, not a spec. Don't hold the design to them.

## Decided

- **A sideways (horizontal) shoot-'em-up with typing**, Typespeed-style.
  **ZType is the feel to match:** first letter locks a target, typing fires.
- **Fun, challenging and educational.**
- **Terminal-first and all keyboard.** Rust + Ratatui, Asciiverse.
- **Typing is plain a–z.** No accents to type.
- **Movement:** arrow keys and Ctrl+h/j/k/l, on Asciiverse's held-key input
  layer (adapt `entity/src/input.rs`, as Ceres, Brickspire and Marble
  Recovery did). On legacy terminals Ctrl+H arrives as Backspace and Ctrl+J
  as line feed; deal with that when movement is built.
- **Name:** Kaiku (Finnish for "echo").

## Ideas on the table (not decided)

- Translation bonus: type a foreign word, then its English meaning, for a
  power-up.
- Whether foreign languages feature at all.

## References

- `docs/research/2026-10-shmup-review.md`: shmup and typing-game prior art,
  Typespeed's internals, open-source references, licences.
- `../typespeed`: local Typespeed checkout.
- Asciiverse games on GitHub (`theasciiverse/*`): check them before
  designing any terminal infrastructure.
