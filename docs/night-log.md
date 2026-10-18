# Night log — 2026-10-18

You asked for work on other enemies, gameplay and other things overnight.
This is what got done, what I decided on your behalf, and what to look at
first. Fourteen commits, from `7e37f86` to the end of the night; each can be
reviewed or reverted on its own. Tests went from 51 to 75, all passing.
Clippy is clean.

I could not play it. Everything was checked with tests, headless rendered
frames, a bot that flies the whole game, and two short runs of the real
binary in a pseudo-terminal (no panics). The feel still needs your hands.

## Try first

```
cargo run --release -- --sector 3     # straight into the fuller war
cargo run --release                   # the whole thing from the crawl
```

## What's new

**Five new enemies** (`7e37f86`), each with one clear job and a readable
tell:

| Ship | From sector | Tell / what it does |
|---|---|---|
| Interceptor | 1 | stops, slides onto your row **flashing**, then dashes |
| Mine | 2 | drifts; destroyed by *anyone*, it bursts into 8 shards |
| Lancer | 2 | a dim **dotted line** along its row, then a beam (half a shield) |
| Jammer | 2 | never fires; while on screen every echo garbles more and the receiver says *⚠ jamming*. Shoot it down to clear the signal |
| Carrier | 3 | capital ship, 30 hits: crosses slowly, fires 3-shot fans, launches scouts; too big to ram |

**Waves grow by sector.** Sector 1 keeps to scouts, gunships, skirmishes
and interceptors. From sector 2 come mines, lancers and jammers; from 3,
carriers. Jammers and carriers come at most once per sector, and each
sector runs 6% faster than the last, down to 70% of the starting gap.

**Gate guards** (`a4275de`): every third sector a carrier glides in to hold
in front of the gates as they open.

**Tunnel walls** (`9f1a3be`): from sector 2, jagged bands of rock, ice or
the war's wreckage along the top and bottom, at most a cell and a half
deep. A scrape costs a little shield and pushes you off. Sector 1 stays open
space.

**Scoring and the run:**
- **Kill chains** (`a375130`): kills within 2.5 s of each other chain; every
  5 links add ×1, up to ×8. The multiplier shows next to the score, and any
  hit breaks the chain.
- **Run summary** at SIGNAL LOST or HOME: score and best, sectors (true /
  detours), kills and best chain, echoes and pods decoded, typing accuracy,
  time.
- **High score** saved in `~/.local/share/kaiku/highscore`, as tty-rex does.
- **Extra lives** at 30,000, then every 60,000, up to 5 (`5e3a6f8`).
- **The shield recharges at every jump**, a nod to Zendocon's docking
  re-arm.

**Fairness and comfort:**
- **Enemy fire is drawn over everything**, echo text and words included
  (`be4edca`). Before this, a word could hide a bullet.
- **Pods take 3 hits**, cracking visibly, before they're lost (`8c89dac`).
  Held auto-fire used to kill them before you could type a letter.
- **Calmer echo blink**, about 3 Hz instead of 6.
- **Esc now reads as a pause** ("y: quit · any other key: fly on").

**Atmosphere:** ship-to-ship chatter (`48ee0f1`). With no echoes about, the
receiver picks up the fleets talking to each other, e.g. *JADE-5: that
civilian is still out there, flying home*. The lines are in
`data/chatter.txt`. Nine more public-domain transmissions joined
`data/echoes.txt` too.

**Tools:**
- `--sector N` and `--skip-intro`.
- A bot test that flies the crawl, cockpit, every sector, map and turn
  through to home.
- Two probes, run with `--ignored`:

  | Probe | Result |
  |---|---|
  | `probe` (naive pilot: weaves, fires, avoids walls) | ~99 s in sector 1, ~64 s in 2, ~59 s by 5 |
  | `frame_cost` (busy frame at 200×60) | ~0.3 ms |

**A bug fixed** (`9b0cef0`): far out on detours, the *true* echo could read
fainter than the fakes, because closeness went negative. It's now clamped.

**Found by a later bug-hunt pass** (`77c4868`): shrinking the terminal to
a sliver mid-game crashed (a pod's clamp). It's fixed, and a test now draws
every state at sizes from 10×4 to 240×70. Also (`376dad5`): a letter could
lock onto an echo or pod still entering off the right edge, unseen. Only
on-screen targets can be locked now. And (`36bcaeb`): a new echo could
share its first letter with a live pod, so one key could mean either; echoes
now avoid pods' letters too.

## Decisions I made for you

All of these are easy to change; most are a single constant.

| Decision | Where |
|---|---|
| Fleet names Amber and Jade stay working names; chatter callsigns use them | `data/chatter.txt`, `enemy.rs` |
| Which sector each enemy arrives in; a guard every 3rd sector | `spawn.rs` `Wave::weight`, `GUARD_EVERY` |
| Chain window 2.5 s, ×1 per 5 kills, cap ×8 | `stats.rs` |
| Extra lives at 30k / every 60k, max 5 | `EXTEND_*`, `MAX_LIVES` in `game/mod.rs` |
| Shield recharges at every jump | `fly_on` |
| Scrape 0.1, beam 0.5 of the shield | `SCRAPE_DAMAGE`, `BEAM_DAMAGE` |
| Jammer takes 0.3 clarity off every echo | `JAM` |
| Pods survive 3 hits | `POD_HP` in `powerup.rs` |
| Chatter lines and wording | `data/chatter.txt` |

## Not done, and why

- **Sound.** It needs the ALSA headers to build, and installing them needs
  sudo: `sudo apt install pkg-config libasound2-dev`. Then rodio, as tty-rex
  uses, for laser blips, explosions, a decode chime and the crawl's
  typewriter tick.
- **Design calls that are yours:**
  - sector names as passwords;
  - docking at the gates;
  - the secret sector;
  - the first fork setting the difficulty (Zendocon's rule).

  The hooks are in `docs/mechanics-draft.md`.
- **The cockpit scene polish** you flagged. I left it alone.

## Worth a look

- **Walls and echoes share the top and bottom rows.** Text is drawn over
  wall pixels, so it stays readable, but check it on your terminal.
- **The gates phase has no time limit.** Enemies stop coming (except a
  guard), so you can take your time choosing. That's deliberate, but say if
  you want pressure there.
- **Legacy terminals without the kitty protocol** repeat only the newest
  key held. Holding Space and steering at once relies on the 0.45 s fire
  linger, so it's worth trying in GNOME Terminal.
