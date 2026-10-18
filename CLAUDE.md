# Kaiku — CLAUDE.md
**Status:** v0.2 (2026-10-18). Rotates the v0.1 vertical-rain design into a
sideways shmup built on ZType's model. Reasoning and sources are in
`docs/research/2026-10-shmup-review.md`. Pending a playtest of the MVP.
**Platform:** Asciiverse (Rust / Ratatui / Braille)
**Genre:** Sideways typing shoot-'em-up × incidental language acquisition

---

## 1. Elevator Pitch

Kaiku is ZType turned sideways, with shmup structure on top. It is
terminal-first and played entirely on the keyboard.

Your ship sits at the left of the screen. Language factions (German,
French, Japanese and so on) fly in from the right in waves, each enemy
carrying a word. Type a word's first letter to lock on, then finish the
word; every correct key fires a shot along it. Enemies fire back with
missiles that carry single letters or short words, and you type those to
shoot them down. Arrow keys or Ctrl+h/j/k/l move the ship. Bosses, branching
zones, combo upgrades and specials provide the shmup structure.

The goal is that the game is fun, challenging and educational, and that it
uses the full width and breadth of the keyboard: every finger, the number
row, punctuation. Each faction's vocabulary is a small,
spaced-repetition pool of real words, not a random word list, so you pick up
vocabulary in the languages you fight.

**Naming note:** "Kaiku" is Finnish for echo. It has no etymological
relation to "haiku" (Japanese). The near-rhyme is a deliberate, acknowledged
coincidence, kept because it reinforces the "short words / compressed
moment" design principle without anyone needing to get the joke. The
Finnish origin also gives it a quiet Liero lineage (scrappy,
terminal-adjacent, substance-over-marketing Finnish game heritage), as
flavour, not a load-bearing claim. Typespeed is Finnish too. The word sounds
plausibly "foreign" without reading as belonging to any one horde's
nationality, which matters since the game features many.

---

## 2. Prior Art

Full review: `docs/research/2026-10-shmup-review.md`.

- **ZType:** the feel to match. First-letter lock, shots along the word,
  typed enemy missiles, and an emergency EMP on Enter.
- **Typespeed:** the ancestor. It is already horizontal: words cross 22 rows
  of an 80×24 screen, speed rises linearly with score, and colour shows
  distance. In head-to-head play, a cleared word is sent to your opponent's
  screen, which is the RX/TX idea in embryo. A local checkout is at
  `../typespeed`.
- **Horizontal shmups:** Gradius (upgrade choice; avoid its death spiral),
  R-Type (one versatile tool), Darius (branching zones), Thunder Force
  (losing only the equipped weapon on death), Sine Mora (time as health),
  Ikaruga (polarity chains), DoDonPachi (a visible chain gauge).
- **NCMatrix:** the aesthetic principle that every visual is driven by
  real state (§6).
- **rsync:** the origin of the delta-sync stretch mechanic (§5.4).
- **Asciiverse:** `entity/src/input.rs` (held-key input, §7) and the Braille
  games' rendering.

Licensing: permissive code (MIT/Apache/BSD) may be reused with attribution.
GPL code is for reading only, unless Kaiku itself is licensed GPL; Kaiku's
licence is still open (§10). Word lists are written by us, not taken from
other games.

---

## 3. Core Loop

1. A wave from one faction (later two or more) enters from the right in
   formations. Each enemy carries a word from that faction's pool.
2. **Lock:** the first letter you type locks the nearest or soonest-arriving
   enemy whose word starts with it. **Two live words on screen never share a
   first letter**; spawning re-rolls to avoid it, so one keystroke always
   names one target.
3. **Fire (TX):** each correct key fires a shot at the locked enemy.
   Finishing the word destroys it, and the word flips to the player's colour
   and flies back out to the right, as your transmission.
   - A wrong key breaks the streak but keeps the lock.
   - Backspace or Esc drops the lock, at a combo cost.
4. **Enemy fire (RX):** enemies launch missiles that carry a single letter,
   and later a short high-frequency word ("ja", "oui", "wa"). You type a
   missile to shoot it down, or move out of its path.
5. **Move:** arrows or Ctrl+h/j/k/l steer the ship (§7). Movement is for
   dodging and positioning. It is never required in order to aim, because
   shots home on the locked target.
6. **Hits** drain the ship's **Signal** (§5.1). The wave ends with a short
   accuracy and words summary, then the next wave or a zone branch (§4.5).

Typing failure is a tempo loss, never a hard stop. No mechanic forces a mode
switch: typing and steering happen on separate keys, so you can do both at
any time.

---

## 4. Horde & Vocabulary System

### 4.1 Horde structure
- Each horde is one language faction with one signature colour.
- A horde's word pool is about 15–20 core words, weighted towards high
  frequency and drilled repeatedly across a wave. Seeing "wa" or "oui"
  several times in one wave is the point, not padding.
- Enemy size follows word length. Short words are fast scouts and missiles;
  long words are heavies, which are slower (Tux Typing slows long words by
  `0.92^(len-1)`), carry a bigger payload and take longer to kill.
- Elites and bosses use longer or structurally distinct entries (§4.3).

### 4.2 Data format
Hordes are data files, addable without recompiling: one TOML file per horde.
Each entry keeps the **display** separate from the **answer**, as k-type
does. That one schema covers romaji mode, glosses and retrieval
drills:

```toml
[[word]]
display = "Grüße"           # what flies at you, spelled correctly
gloss   = "greetings"
freq    = 2140              # frequency rank, drives pool weighting
tags    = ["noun"]
# answer = "..."            # optional; defaults to the display folded to ASCII
```

**You only ever type plain a–z.** Nobody has time for accents in a shmup.
The display keeps the real spelling, so you still see it; the answer is the
display folded to lowercase ASCII: diacritics stripped, `ß`→`ss`, `æ`→`ae`,
`ø`→`o`. So `Grüße` is typed `grusse` and `été` is typed `ete`. Set `answer`
only when the fold is wrong or for romaji (`display = "わ"`, `answer = "wa"`).

### 4.3 Bosses and grammar
Boss phases require a real grammatical unit, not just a longer word: a
German compound noun, a French verb conjugated across persons, a Japanese
particle plus verb chain. A boss can also run a typo-punishing phase, or a
choice phase where you type the right one of two forms (Typing of the Dead
does both).

### 4.4 Learning features
- **Gloss on first kill:** the first time a word is cleared in a run, a
  short translation appears briefly near the kill. No pause.
- **Post-run recap:** every word cleared this run, with translation.
- **Retrieval for known words:** once a word is "known", it sometimes flies
  in showing only its gloss ("butterfly"), and you type the word. Copying a
  visible word teaches spelling; recalling it from the gloss teaches meaning.
- **Spaced repetition:**
  - *In-wave:* a simple rotation, where a recently missed word comes back
    sooner.
  - *Across runs:* FSRS (`rs-fsrs` crate, MIT) picks each horde's pool and
    decides "known". Map each result to an FSRS rating: fast and clean =
    Easy, clean = Good, slow or corrected = Hard, missed = Again.
- **Constraint:** no learning feature may pause or interrupt tempo.
- **Caveat:** research says incidental learning drops under high load. The
  gloss, recap and retrieval drills carry the teaching, not raw exposure.
  Measure it with a before-and-after quiz in playtests; don't assume it.

### 4.5 Multi-horde waves and zone branches
- Later waves field two or more factions at once, forcing language and
  script code-switching mid-combo. This is the hard-mode identity:
  recognising which language you are in matters more than raw speed.
- **Language as polarity (Ikaruga):** consecutive kills on the same faction
  build the chain multiplier. Switching faction is the risk you choose.
- **Branches (Darius):** after a boss, choose the next zone, which is the
  next language ("Französisch or Italiano?").

---

## 5. Combat Layer

### 5.1 Signal (health)
- The ship has a **Signal** meter. Hits from missiles or collisions drain
  it; kills refill a little of it (Sine Mora's "time is health").
- A short invulnerability shield follows every hit, so a bad typing beat
  never cascades into a wipe.
- **Respawn in place**, no checkpoints.

### 5.2 Payload
- Short words route targets fast, carry a low payload and expose you
  briefly.
- Long or rare words carry a large payload and expose you for longer.
- **Lateness:** a word finished closer to the ship pays more (risk/reward,
  like a shmup's point-blank kill).

### 5.3 Combo, upgrades, specials
- **Streak multiplier** (ZType): +1 at streak tiers (for example 25, 50,
  100 correct keys). A miss lowers it **one tier**, never to zero.
- **Upgrades** (consistency reward): sustained streaks unlock run-long
  buffs, such as faster shots, splash damage on long words, or more Signal.
  Taking a hit costs one upgrade tier, never all of them, to avoid
  Gradius's death spiral.
- **Translation bonus (power-up):** a marked bonus enemy carries a foreign
  word. Type it, and it flips to ask for the English meaning; type that too
  ("schmetterling", then "butterfly") for a power-up or a large payload.
  English glosses for bonus words must be short and typable in a–z, and
  `alt` lists other accepted answers ("hello" or "hi").
- **Specials on non-letter keys:**
  - **Enter = EMP**, which clears nearby enemies and missiles. A few charges
    per wave.
  - **The number row** holds the specials and weapons that upgrades earn.
    Typists rarely use those keys, so a full-keyboard typist is rewarded.
- **Full sync:** clearing every live word of all active hordes in sequence,
  without a miss, triggers a rare, telegraphed screen-clear.

### 5.4 Delta-sync (stretch, rsync-derived)
For elite packets only: a cleared word comes back after a delay with one
character mutated, and a checksum-style marker flags the changed span, so
you patch only that span. A "corruption event" re-flags a cleared word as
bad. **Not in the MVP.** Revisit only after the core loop is proven fun.

---

## 6. Aesthetic Architecture (NCMatrix principle)

Every visual variable maps to a tracked game-state number. There is no
purely decorative layer.

- **RX traffic:** incoming enemies, words and missiles in the faction's
  colour. Density and speed show that horde's live aggro.
- **TX traffic:** your shots and the cleared words flying back out,
  palette-swapped to the player's signature colour.
- **Proximity colour:** a word warms as it nears the ship, as Typespeed
  does with its distance zones.
- **Global tint and brightness** follow the live combo multiplier.
- **Readability rules:**
  - Words always sit on whole cells.
  - Bullets and effects may use Braille (2×4) sub-cells.
  - Enemy missiles get a colour and glyph that nothing else uses.
  - The locked word is highlighted, with the typed prefix marked.

---

## 7. Input

**Reuse Asciiverse's held-key layer:** adapt `entity/src/input.rs`, as
Marble Recovery, Brickspire and Ceres did. It has three tiers:
- **Exact:** kitty keyboard protocol, or Windows key-up.
- **Timing:** held while OS key repeats keep arriving, with the repeat
  timing learned from the events.
- **Latched:** opt-in via an environment variable.

Follow Ceres too: latch a sub-frame tap until the next tick, and say at
start-up when exact input isn't available.

| Keys | Action |
|---|---|
| Letters a–z | Typing: lock and fire |
| Arrows, Ctrl+h/j/k/l | Move |
| Backspace / Esc | Drop lock |
| Enter | EMP |
| Number row, punctuation | Specials |
| Ctrl+W / Ctrl+U | Clear the typed prefix |
| Pause key | Pause (to be decided) |

What is new for Kaiku, because no other Asciiverse game mixes held movement
with typed text:
- **Typing accepts `KeyEventKind::Press` only.** Windows reports releases
  natively, so without this filter every letter would register twice.
- Typing is plain a–z (§4.2), so Entity's kitty flags work unchanged.
- **Ctrl+H is Backspace and Ctrl+J is line feed** in legacy terminals.
  Kitty's disambiguate flag separates them. Without kitty, check what
  crossterm decodes in raw mode, and fall back (Alt+hjkl, or erase via
  Ctrl+W/U) if they collide.
- Timestamp input events, so per-word latency feeds scoring and FSRS
  ratings.

Logging follows the Asciiverse convention: `log` + `env_logger`, every raw
input event, state changes, and a state snapshot about every 500 ms.

---

## 8. Layout

- The playfield sizes to the real terminal; nothing is hardcoded to 80×24.
- Width is the approach distance; rows are the vertical space the ship and
  enemies move in.
- About 8 usable rows of traffic fit at 24 rows; more on larger terminals.
- Kana width (East Asian Width) and Braille fidelity follow `drawprobe`'s
  signal, as elsewhere in Asciiverse.

---

## 9. Rollout Plan

Rendering and input risk gate the scope; they don't block the project.

- **Phase 1 (MVP):** one Latin-script horde (French or German). Includes:
  - First-letter lock and homing shots
  - Typed missiles
  - Arrow and Ctrl+hjkl movement
  - Signal and shield
  - The EMP
  - The streak multiplier
  - Gloss on first kill and the post-run recap
  - Hordes as data files

  The first minutes are "just type the word". Missiles and movement ramp in
  after that, so total novices don't bounce. A forgiving novice mode à la
  Tux Typing's Fish Cascade (no lock, no backspace, suffix match) is an
  option if playtests need it.

  **The MVP's question:** is typing plus occasional steering fun, or
  tiring? The Textorcist is the warning here.
- **Phase 2 (structure):**
  - 2–3 more Latin-script hordes (Spanish, Italian, Scandinavian)
  - Multi-horde waves and polarity chains
  - Bosses with grammar phases
  - Zone branches
  - Upgrades and number-row specials
- **Phase 3 (scripts):** a Japanese horde as the flagship. Romaji mode
  ships first (`display = "わ"`, `answer = "wa"`, or plain romaji display),
  with kana after a `drawprobe` fidelity pass and IME testing.
- **Phase 4 (persistence and polish):**
  - FSRS across runs, "known" status and retrieval drills
  - Recap refinement
  - Delta-sync (if it survives)
  - More hordes

**Sequencing:** following Asciiverse's ship-first doctrine, Phases 2–4 do
not start until Phase 1 is fun as a single-horde loop.

---

## 10. Open Questions

- **Foreign languages at all? Not decided yet.** The game may be English at
  its core, with other languages coming in as bonus content such as the
  translation power-up (§5.3) rather than as whole factions. This decides
  how much of §4 (hordes as languages, polarity, zone branches, Japanese)
  survives.

- **Signal vs. lives:** is a drain-and-refill meter clearer than discrete
  lives in a terminal HUD?
- **Movement's role:** pure dodging, or should position also matter, for
  example lateness payload or lane-based enemy formations? Is lane-snap on
  lock worth keeping as an option?
- **Pool size vs. the unique-first-letter rule:** with 15–20 words, how many
  can be live at once? Check this against real word lists.
- **Ctrl+H/J on legacy terminals:** §7.
- **Combo tiers, Signal drain and refill, and spawn rates:** tune by
  playtest, not at spec time.
- **Kaiku's own licence:** this decides whether GPL references can ever be
  more than reading material.
- **Delta-sync:** keep or cut after Phase 1.
