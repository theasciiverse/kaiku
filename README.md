# Kaiku: Echoes Home

A sideways shoot-'em-up for the terminal, with typing. An Asciiverse game.

An interplanetary war is raging. A surge has flung your ship far off course,
into a pocket of space where Earth's old transmissions bounce around as
garbled echoes. Fly and fight through the crossfire, decode the echoes by
typing them, and take the right gate each time: the clearer the echo, the
nearer home.

## Play

```
cargo run --release
```

| Keys | |
|---|---|
| Arrows, or Ctrl+h/j/k/l | fly |
| Space | fire (hold) |
| a-z | type: lock onto an echo or a power-up's word, and decode it |
| Backspace | drop the lock (progress is kept) |
| Enter | smart bomb; on the map, fly on |
| Tab | dash: a burst of speed you can't be hit in |
| Esc | pause (y quits) |
| F1 | the controls card (the game waits) |
| F8 | sound and music on/off (`--mute` starts silent, `--no-music` keeps only the effects) |

**Two players:** plug in a gamepad and one flies while the other types.

| Gamepad (the pilot) | |
|---|---|
| Left stick or d-pad | fly (the stick in proportion) |
| Bottom face button (Xbox A, Switch B) | fire (hold); confirm on the map and the end screens |
| Right face button or left bumper | smart bomb |
| Left face button or right bumper | dash |
| Start / + | pause |
| Select / − | the controls card |

The pad rumbles: hard when you're hit, big for bombs and jumps, and a small
pulse when comms decodes an echo or claims a power-up.

The keyboard (comms) types the echoes and power-up words; it can still fly
too, so one player can do it all. Only SDL-mapped pads are used;
`KAIKU_ANY_PAD=1` accepts any.

- **Echoes** drift in garbled and flicker into clear. Type the first letter
  to lock on, then the rest. Spaces fill themselves in. A decoded echo is
  analysed; a jammed one rules out its gate.
- **Gates:** three at the end of every sector, and only one leads closer to
  home. A wrong gate is a detour, never the end.
- **Power-ups** fly in carrying a word: type it to claim the power-up, and
  don't shoot it. The words walk the keyboard row by row as you go. Ships you
  shoot down sometimes leave one behind too (a small glowing diamond, no
  word): fly into it. Bigger ships drop more often; carriers always do.
- **The war:** two sides fighting each other as much as you. Watch for the
  interceptor's flash, the lancer's warning line, and the jammer that
  scrambles your echoes.

Terminals with the kitty keyboard protocol (kitty, foot, WezTerm, Ghostty
and others) give exact key presses and releases. Elsewhere Kaiku reads key
repeat timing; if held keys feel wrong, try `KAIKU_INPUT=latch`.

Flags: `--skip-intro`, `--sector N` (start in a later sector), `--demo` (the
game plays itself; any key quits), `--credits`, `--mute`, `--no-music`.

## Credits and sources

- **Echoes:** public-domain transmissions from NASA missions (Mercury,
  Gemini, Apollo) and John F. Kennedy's speeches, plus the Voyager Golden
  Record greeting (its rights still to be checked). See `data/echoes.txt`.
- **Inspired by:** ZType, Typespeed, and *Gates of Zendocon* (Epyx, 1989),
  whose worlds are linked by gates. Homage only: no names, code or art taken.
- **Input:** the held-key layer is adapted from Asciiverse's Entity.

Design notes are in `CLAUDE.md` and `docs/`.
