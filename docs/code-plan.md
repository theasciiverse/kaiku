# Kaiku — Code Plan (v1)

Rust + Ratatui, following Asciiverse conventions as Entity and tty-rex do:
- `ratatui = "0.30"`, using its re-exported `ratatui::crossterm`.
- `log` + `env_logger`.
- A fixed 60 Hz tick.
- Optional kitty keyboard flags.

The simulation, input and rendering are separate layers. The simulation
never touches the terminal, so it is testable headless.

## Layout

```
Cargo.toml
src/
  main.rs        terminal setup, kitty probe, 60 Hz loop (Entity's pattern)
  input.rs       held keys (adapted from entity/src/input.rs) + typed letters
  typing.rs      first-letter lock, progress, mistakes
  echo.rs        echo text, garble, flicker, clarity, true/false
  game/
    mod.rs       Game state machine: Intro, Flight, Turn, Map, Home, Over
    ship.rs      movement, gun, shield, lives
    enemy.rs     the two warring sides, formations, enemy fire
    shot.rs      bullets (player and enemy)
    powerup.rs   word-carrying pods
    sector.rs    one tunnel: spawn script, echoes, portals
    route.rs     route graph, distance home, sector names
  render/
    mod.rs       compose layers into the ratatui buffer
    canvas.rs    one Braille + half-block pixel canvas over the play area
    chrome.rs    Crush-style frame: wordmark, gradients, ╱ hatching, HUD
    turn.rs      the rotation animation
    map.rs       the between-sector route map
data/
  echoes.toml    transmission snippets (public domain), with sources
  words.toml     power-up words, grouped by keyboard row
docs/            design and research
```

## Key decisions

- **World units are Braille dots:** 2 per cell across and 4 down. Half-block
  sprites are drawn in 2×2-dot pixels, so the two styles share one
  coordinate space. Text (echoes, words) is anchored to whole cells.
- **Input:**
  - **Movement** (arrows, Ctrl+h/j/k/l) and **fire** (Space) go through
    Entity's three tiers (exact, timing, latched). Ctrl+H and Ctrl+J: in raw
    mode crossterm should decode `0x08` and `0x0A` as Ctrl+letter, as long
    as Backspace sends `0x7F` (most terminals). Verify this against the
    crossterm source once the crate is fetched.
  - **Typed letters:** plain a–z with no Ctrl, `Press` only, queued with a
    timestamp for the typing layer. Entity's kitty flags (1|2|8) are fine
    here, because typing is a–z only.
- **Determinism:** each sector is seeded, so tests can replay a sector.
- **Data files** are embedded with `include_str!` for v1, and parsed with
  `toml` + `serde`.
- **No `drawprobe` in v1.** It isn't available on this machine. Add it
  later if Braille fidelity checks are wanted, as tty-rex does.

## Milestones

Each milestone ends playable and committed.

1. **M0 — Skeleton:** the loop, input, and a ship that moves and shoots in an
   empty starfield (Braille). Clean quit, panic hook, logging.
2. **M1 — Shmup:** enemy formations from both sides, enemy fire, collisions,
   the shield bar and lives, game over. A first pass at the chrome and HUD.
3. **M2 — Echoes:** garbled echoes, flicker, typing lock and decode,
   clarity, end-of-sector portals.
4. **M3 — Journey:** the turn animation, the route graph, the
   between-sector map, true and false portals changing the distance home,
   arrival home.
5. **M4 — Power-ups:** word pods following the keyboard-row curve, and the
   power-up effects.
6. **M5 — Intro scene:** the receiver repair tutorial.
7. **Then:** playtest. The open question is whether the mix of flying,
   shooting and typing is fun.

## Tests

- **input:** Entity's timing tests, carried over, plus a–z routing (Ctrl+h
  moves, h types).
- **typing:** lock rules, unique first letters, mistakes.
- **echo:** garble is deterministic from a seed; typed letters stay clear.
- **game:** a seeded sector runs headless to completion.
