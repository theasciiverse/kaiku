# Kaiku

**Status:** Fresh start (2026-10-18). This file holds only what has been
decided. Earlier plans (v0.1 vertical rain, v0.2 language-faction shmup) are
**not binding**. They stay in git history (`a3c8492`, `7579e6b`) as a quarry
of ideas, not a spec. Don't hold the design to them.

## Decided

- **A sideways (horizontal) shoot-'em-up with typing**, Typespeed-style.
  **ZType is the feel to match:** first letter locks a target, typing fires.
- **A fair mix of typing and playing:** moving and shooting matter as much
  as typing. Space shoots; arrows and Ctrl+h/j/k/l move; Ctrl and other
  non-letter keys are free for actions.
- **Full title: *Kaiku: Echoes Home*.** The story (David, 2026-10-27): you
  are a fighter in an intergalactic war. A savage blast hurls your ship far
  beyond the front line, deep into enemy space where no human has ever been;
  both fleets are hostile, and to get home you must find the right portals by
  their echo. At home Earth's fleet turns out to escort you in, and you are
  welcomed as a hero (`src/game/home.rs`). A little homage to *Gates of Zendocon* (Epyx,
  Atari Lynx, 1989), the horizontal shmup whose worlds are linked by gates.
- **What the echoes are:** a mashup of Earth's famous transmissions (Apollo
  ("The Eagle has landed"), JFK's "We choose to go to the Moon", the
  Arecibo message), bouncing around the region of space the pilot is stranded in.
  **The clearer an echo, the more it points home.** Use public-domain
  sources only (NASA, US government, Arecibo); no music or commercial
  broadcasts.
- **Intro scene:** the pilot manages to repair the ship's receiver, which
  starts catching snippets of these transmissions, all garbled.
- **Echoes:** during play an echo shows as garbled text, flickering now
  and then into full clear while still blinking and garbled. Typing it out
  makes it clear, letter by letter. **The echoes are what guide you home:**
  each one you type points the way to the next portal or tunnel.
- **Three gates at the end of every sector; only one gets you closer to
  home.** Wrong gate = a longer path home, never game over. (Proposed: the
  war's jamming sends false echoes alongside the true one.)
- **A stage map between stages** shows your progress home: where you are,
  the jumps taken, how far is left.
- **Getting hit:** a shield bar plus a few lives.
- **Typing is for echoes and power-ups.** Shooting is the combat; typing
  decodes echoes and wins power-ups (for example by typing the word a
  power-up carries).
- **Turns and tunnels:** when an echo points the way, the whole play area
  turns, so you feel you are heading in a new direction. There are
  different tunnels to take on your way home.
  To try first: the turn is a full-field rotation animation (Braille), and
  play stays horizontal afterwards; the new tunnel's look and the map show
  the new heading.
- **Fun, challenging and educational.**
- **Terminal-first and all keyboard.** Rust + Ratatui, Asciiverse.
- **Bottom panel:** the lower part of the screen is reserved for a message
  row (the **receiver**, for decoding longer echoes and ship-to-ship
  communication) and **power-up illustrations**.
- **Look: a lovely mix of ASCII, Braille and half-blocks.** A natural split:
  ASCII for text (words, echoes, HUD); half-blocks (`▀▄`, two colours a
  cell) for chunky sprites, tunnel walls and terrain; Braille (2×4 dots) for
  fine detail such as bullets, particles, starfield and the turn animation.
  Asciiverse precedents: Ceres, Panzer and Calvary (half-blocks); Entity and
  Prince of Braille (block glyphs plus Braille).
- **Chrome around the play area is visually stunning too,** inspired by
  Charm's Crush TUI: a custom wordmark built from letterforms (one letter
  stretched), horizontal colour gradients, `╱` diagonal hatch fields and
  gradient `╱╱╱` after titles, rounded borders, shimmering indicators.
  Crush is FSL-1.1-MIT (source-available, MIT after two years): read it
  for ideas (`internal/ui/logo`, `internal/ui/styles`), don't copy code.
- **Typing is plain a-z.** No accents to type.
- **Movement:** arrow keys and Ctrl+h/j/k/l, on Asciiverse's held-key input
  layer (adapt `entity/src/input.rs`, as Ceres, Brickspire and Marble
  Recovery did). On legacy terminals Ctrl+H arrives as Backspace and Ctrl+J
  as line feed; deal with that when movement is built.
- **First version: English only, and it teaches typing.** Foreign languages
  are for later, if at all.
- **Two players:** a gamepad flies (the pilot) while the keyboard types (comms).
  Plug in a pad and it's co-op; either alone can still do both.
- **The fleets are Amber and Jade.** The opening crawl, the closing crawl, the
  fleet chatter and the credits (`data/intro.txt`, `data/home.txt`,
  `data/chatter.txt`, `data/credits.txt`) are the approved texts (intro, home
  crawl and two chatter lines rewritten for the new story 2026-10-27). Never
  "an Asciiverse game" anywhere: Asciiverse is a place of equals. The last echo, at home, is "we leave as we came" (Apollo 17).
  The Voyager greeting is out (2026-10-19): its rights were never checked.
- **Name:** Kaiku (Finnish for "echo").

## Ideas on the table (not decided)

- Later, if foreign languages come in: a translation bonus (type a foreign
  word, then its English meaning, for a power-up).

## References

- `docs/research/2026-10-shmup-review.md`: shmup and typing-game prior art,
  Typespeed's internals, open-source references, licences.
- `../typespeed`: local Typespeed checkout.
- Asciiverse games on GitHub (`theasciiverse/*`): check them before
  designing any terminal infrastructure.

## Build and run

Gamepad support (on by default, via gilrs) needs `pkg-config` and
`libudev-dev`; `cargo build --no-default-features` builds without it.

```
cargo run --release                     # play
cargo run --release -- --sector 3       # straight into sector 3 (carriers, lancers, jammers)
cargo run --release -- --skip-intro     # straight into sector 1
cargo run --release -- --demo           # the game plays itself (attract mode, preview recordings)
KAIKU_INPUT=latch cargo run --release   # latched keys, for terminals where timing feels wrong
RUST_LOG=debug cargo run --release 2>kaiku.log
cargo test
KAIKU_PRINT=1 cargo test a_frame -- --nocapture   # print a headless frame
cargo test --release probe -- --ignored --nocapture   # difficulty probe: naive-pilot survival per sector
```

Plan and milestones: `docs/code-plan.md`. Mechanics: `docs/mechanics-draft.md`.
