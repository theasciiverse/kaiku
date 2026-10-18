# Kaiku — CLAUDE.md

**Status:** Fresh start (2026-10-18). This file holds only what has been
decided. Earlier plans (v0.1 vertical rain, v0.2 language-faction shmup) are
**not binding**. They stay in git history (`a3c8492`, `4f1c36c`) as a quarry
of ideas, not a spec. Don't hold the design to them.

## Decided

- **A sideways (horizontal) shoot-'em-up with typing**, Typespeed-style.
  **ZType is the feel to match:** first letter locks a target, typing fires.
- **A fair mix of typing and playing:** moving and shooting matter as much
  as typing. Space shoots; arrows and Ctrl+h/j/k/l move; Ctrl and other
  non-letter keys are free for actions.
- **Story: Echo Home.** An interplanetary war is raging. A surge has
  thrown your ship far off course, and to get home you must find the right
  portals by their echo. A little homage to *Gates of Zendocon* (Epyx,
  Atari Lynx, 1989), the horizontal shmup whose worlds are linked by gates.
- **Echoes:** during play an echo shows as garbled text, flickering now
  and then into full clear while still blinking and garbled. Typing it out
  makes it clear, letter by letter. Once it is typed, it guides you in the
  right direction.
- **Turns and tunnels:** when an echo points the way, the whole play area
  turns, so you feel you are heading in a new direction. There are
  different tunnels to take on your way home.
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
