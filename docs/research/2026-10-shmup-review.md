# Kaiku: Prior-Art Review (Oct 2026)

Input to the next CLAUDE.md revision. Three sources:

1. A close read of Typespeed 0.6.5 (local checkout, `../typespeed`).
2. A design survey of horizontal shmups and typing hybrids (web research).
3. A survey of open-source shooters, typing games and SRS code. Clones were
   read only; nothing was built or run.

Most shmup "design knowledge" comes from community wikis, forums and reviews,
not formal postmortems. The language-learning research is real but mixed.
Treat everything below as hypotheses to playtest.

---

## 0. The decision this review forces

CLAUDE.md v0.1 describes **vertical** falling columns (NCMatrix rain). The new
direction is a **sideways** shmup. Everything below argues for going
horizontal:

- Terminal cells are about 1:2. Width is the abundant dimension: 80-200+
  columns of approach distance against 24-60 rows.
- Words read left to right, so a horizontal word is a horizontal object.
  Vertical rain has to stack letters or keep words short.
- **Typespeed is already horizontal.** Words scroll left to right across
  22 rows.

Proposed mapping: the ship sits on the left. Rows are **lanes**. RX words fly
in from the right. A captured word flips colour and flies back to the right
as TX. The NCMatrix principle survives unchanged; only the axis rotates. The
rain becomes horizontal "signal traffic" per lane.

---

## 1. Typespeed: what it actually does

Source: `typespeed/src/typespeed.c`, `misc.c`, `rules/*`, `words/*`.

| Aspect | Typespeed's approach | Lesson for Kaiku |
|---|---|---|
| Field | Fixed 80×24. 22 word rows, one input row; widths are hardcoded (`79 - length`, `typespeed.c:603`) | Size to the real terminal. Never hardcode 80. |
| Motion | Each tick, every word moves one cell to the right. `rate = score/step + minspeed` ticks per second, with `step = 175` and `minspeed = 3` (`cwords`, `:268-274`) | Difficulty is a single linear ramp driven by score. Simple, readable, and has no per-word variety. |
| Spawning | A word is added when the total visible characters drop below `score/4 + 1`, or the word count is below `min words` (`:265`). Never a duplicate on screen (`addnewword`, `:220-229`) | Density is tied to the player's own performance. Steal this: it is already "aesthetic driven by game state." |
| Colour | Colour by distance zone: safe (<50), warning (50-65), danger (>65) (`movewords`, `:613-649`) | Colour as proximity is the NCMatrix principle in miniature. |
| Matching | Type the whole word, then press Space or Enter. Matched against **every** word on screen. No prefix lock, no live highlight (`parseinput`, `:706-738`) | Too loose for a shmup: no targeting, no routing. Kaiku needs first-letter lock (§3). |
| Scoring | Score += word length on a hit (`:714`). CPS = (score + words)/time. WPM = CPS×12. Typo ratio drives joke ranks ("Secretary", "TypoKing") | Length-as-payload is the right base. Joke ranks are cheap, beloved flavour. |
| Failure | Game over after 10 words escape. Grace rule: if the last escaping word was being typed correctly, you still get its points (`:891-897`) | The grace rule is a nice touch; keep the idea of forgiving the last moment. |
| Anti-spam | More than 10 wrong Enters in a short span disables highscores (`:873-879`) | Kaiku needs the same guard against blind spamming. |
| Data | Word lists: first line is a description, then one word per line. Rule sets: plain `key = value` files, selectable in a menu | Answers CLAUDE.md §8: hordes as data files, addable without recompiling. |
| Encoding | Word lists are **ISO-8859-1**. Input drops anything above 255 and requires `isprint`. UTF-8 is still on the TODO | Kaiku must be UTF-8/NFC-native from day one, or French and German break. |
| Network | In head-to-head play, a word you clear is sent to the opponent's screen with 50% probability (`:717-725`) | **This is Kaiku's TX mechanic in embryo:** cleared words become ammunition against someone else. |

The whole game loop is about 250 lines. Typespeed proves that the core
"type the moving word" loop is fun with almost no systems. Kaiku's job is to
add stakes (a ship and enemies), targeting and learning without burying that.

---

## 2. Horizontal shmups: what makes them good

| Game | What it's praised for | Lesson for Kaiku |
|---|---|---|
| **Gradius** | A power-up bar: the player chooses when to spend capsules | Upgrade *choice*, not just upgrade accrual. Warning: "Gradius Syndrome." Dying strips every upgrade, so players restart instead of continuing. |
| **Gradius V** | Respawn in place; your lost Options float on screen so you can grab them back | The fix for the death spiral. |
| **R-Type** | The Force pod: one flexible tool that is both shield and weapon. Levels play as puzzles | One readable tool gives more depth than a weapon zoo. |
| **Darius** | Branching zones: about 26 zones, 7 per run, with route-dependent endings | **The branch can be a language choice:** "next: French zone or German zone." Cheap replay value. |
| **Thunder Force III/IV** | Switch weapons any time, ship speed control. Dying costs only the equipped weapon | A gentle penalty instead of a full wipe. |
| **Einhänder** | Weapons taken from downed enemies; complaints about accidental pickups | Pickups in a cramped grid need deliberate input. |
| **Ikaruga** (vertical, but design canon) | Polarity: absorb your own colour; chains of 3 same-colour kills | **Language = polarity.** A chain on one faction builds the multiplier; switching faction mid-chain is the risk. |
| **DoDonPachi** | Chaining, with a visible decaying chain gauge | A visible meter beats a hidden timer. That suits the "global tint = combo" idea. |
| **Battle Garegga** | Hidden rank that rises as you play well; experts die on purpose to lower it | Anti-pattern: if difficulty scales, show it, and never reward dying. |
| **Psyvariar** | Grazing bullets fills XP | Proximity as reward, but players weren't taught it. |
| **Sine Mora** | **Time is health:** hits cost seconds, kills add seconds | Directly fits "typing failure = tempo loss, not a hard stop." |
| **Steredenn** | Roguelite: procedural waves, drafted weapons, heal at bosses | The structural template for a game whose content is generated from data files. |

Principles that apply across the genre:

- **Hitboxes:** the player's hitbox is much smaller than the sprite. Enemy
  bullets are the clearest thing on screen, in a colour and glyph nothing
  else uses.
- **Telegraph every threat.** Hand-authored waves reward memorising; procedural
  waves reward reflexes. Kaiku's content is data-driven, so lean towards
  reflexes.
- **Respawn in place.** Checkpoints are for memorisation games; Kaiku is
  about flow.
- **Risk/reward through proximity:** grazing, point-blank kills, chains.
  Kaiku's equivalent is **lateness**: a word finished close to the ship pays
  more and risks more.
- **Bosses:** phases, readable weak points, learnable patterns. Typing of the
  Dead's bosses already include "punishes typos" and multiple-choice question
  bosses. Grammar bosses (§4.3) fit straight in.
- **Short stages, deep scoring.** Replay comes from score systems and routes,
  not length.

---

## 3. Typing hybrids: what works, what fails

| Game | Takeaway |
|---|---|
| **The Typing of the Dead** | On rails; typing is the only agency, and that works because the camera does the navigating. Its drills made it a real typing tutor. |
| **ZType** | **The first keystroke locks the nearest word starting with that letter.** Never spawn two on-screen words with the same first letter. Start letters are spread across the keyboard. Streak multiplier +1 at 25/50/100, reset on miss. |
| **k-type** (ZType fork for Japanese) | Entries are `[display, typed]` pairs: show あ, type `a`. This is exactly the romaji fallback, and it generalises to "show the gloss, type the word." |
| **Epistory / Nanotale** | Typed spell modifiers are lovely. But wave fights pin you in place with no defence except typing faster, which is the frustration Kaiku's gun floor exists to prevent. |
| **The Textorcist** | The closest existing game: bullet-hell plus typing prayers. Reviewers agree that typing and dodging at once is the hard part, and for some it never clicked. **The biggest warning for Kaiku.** |
| **Tux Typing: Fish Cascade** | Suffix match on a typed buffer: no lock, no backspace, wrong keys are just garbage. Extremely forgiving; a ready-made novice mode. Fall speed `0.92^(len-1)`, so long words move slower. |
| **Tux Typing: Comet Zap** | Score is weighted by distance: early kills pay more. A miss costs points. |
| **Voidrunner, Type'EmUp** (itch.io) | Jam-scale horizontal typing shooters. Typed movement commands are too slow. No established hit in this niche: an opportunity, and also a sign the problem is hard. |

---

## 4. The hands problem (the core design risk)

Both hands are on the letters, so the ship has to move some other way.

| Option | Verdict |
|---|---|
| Typing *is* the steering: locking a word moves the ship to that word's lane | Zero extra input. Loses dodging unless combined with something else. |
| Non-letter keys (arrows, Tab/Shift-Tab, Space) for a lane dodge | Costs a fraction of a second away from home row. Works as an *optional* skill layer. |
| Modal input (fly mode / type mode) | Breaks tempo and violates §4.2. **Rejected.** |
| Auto-shield or a tempo buffer as the floor | Absorbs what the player couldn't dodge (Sine Mora model). |
| Typed movement commands | Too slow. **Rejected.** |

**Movement keys: arrows *and* Ctrl+h/j/k/l** (vim directions). Ctrl+hjkl
keeps both hands on the home row: the left pinky holds Ctrl, the right hand
steers with the vim keys, and letters typed without Ctrl stay text. Two
legacy-terminal collisions to deal with:

- **Ctrl+H is Backspace (0x08)** in many terminals.
- **Ctrl+J is line feed (0x0A)**, which may decode as Enter.

The kitty `DISAMBIGUATE_ESCAPE_CODES` flag separates both. Without kitty,
Ctrl+H/J need a check (what crossterm decodes in raw mode), and possibly a
fallback such as Alt+hjkl or remapping Backspace to Ctrl+W/Ctrl+U-style
erase. Ctrl+W and Ctrl+U are the conventional erase keys in typing games;
Typespeed uses ^U. Held Ctrl+hjkl goes through the same Entity input tiers
as the arrows.

**Recommendation:** lane-snap on lock, plus an optional dodge on non-letter
keys, plus hits draining a tempo/"signal" buffer rather than killing.

"The full width and breadth of the keyboard" can be honoured *without*
movement keys:

- Spread target start letters across the whole layout (ZType's trick) so
  every finger works.
- Put **weapons and specials on the number row and punctuation**, which
  typists rarely touch, so a full-board typist is rewarded.
- Treat accents, ß and capitals as *bonus payload* rather than obstacles.

---

## 5. Terminal technical lessons

**Held keys are already solved in Asciiverse. Kaiku reuses that work and
doesn't redesign it.** `entity/src/input.rs` is the reference, and Marble
Recovery, Brickspire and Ceres all adapted it. It has three tiers:

- **Exact:** kitty protocol, or Windows key-up. Real presses and releases.
- **Timing:** everywhere else. A key counts as held while its OS repeats
  keep arriving. The repeat delay and interval are learned from the events
  themselves; the pre-repeat window is capped; the most recent direction
  wins.
- **Latched:** opt-in via an environment variable. Direction stays on until
  the opposite key.

Ceres adds two things: it latches a sub-frame tap until the next tick reads
it, and when exact input is unavailable it says so at start-up. Logging
follows the Asciiverse convention: raw input events, state changes, and a
snapshot about every 500 ms. So arrow-key flight works on every terminal,
exactly where kitty is available and well enough elsewhere.

**What's new for Kaiku is text, not held keys.** No other Asciiverse game
mixes held movement keys with typed accented text:

1. **Check the enhancement flags against typed text.** Entity pushes
   `DISAMBIGUATE | REPORT_EVENT_TYPES | REPORT_ALL_KEYS_AS_ESCAPE_CODES`.
   Per the kitty spec, flag 8 (all keys as escape codes) stops the terminal
   sending composed text: dead-key é, AltGr characters, IME. crossterm also
   doesn't support flag 16 (associated text), which would bring it back.
   Kaiku may need flags 1|2 only, accepting that letters then give no
   releases (it doesn't need them) while arrows still do. **Test this in
   kitty, foot, WezTerm and ghostty with a German and a French layout
   before committing.**
2. **Accept `KeyEventKind::Press` only** for typing. Windows sends releases
   natively, so without this filter every letter registers twice.
3. **Unicode:** normalise word data and input to NFC, compare by `char`, and
   case-fold with `to_lowercase`. Known bugs to avoid: ttyper mixes byte
   length and char index (`test/mod.rs:84`); Tux Typing upper-cases with
   arithmetic.
4. **Lenient accents mode by default:** accept `e`→é and `ss`→ß, with
   optional `ae/oe/ue`, at a payload penalty. Strict mode pays full. Many
   players type French and German on US layouts.
5. **Rendering:** ratatui's buffer diff, no full clears (they flicker on
   Windows Terminal). Words stay on whole cells. Bullets and effects use
   Braille (2×4) sub-cells, as in the Asciiverse Braille games
   (prince-of-braille, tty-rex, entity).
6. **Input thread plus timestamps** (tetro-tui does this): keypress
   timestamps give exact per-word latency for scoring and SRS ratings. Check
   whether the Asciiverse loop already timestamps events before adding it.

---

## 6. Learning: the research says be careful

- **Meta-analyses are positive but depend on design.** Kao 2014: d≈0.7
  across 25 studies. Tsai & Tsai 2018 (26 studies): there is no effect when
  only a non-game variable differs.
- **Incidental learning fails under high cognitive load.** Studies found that
  salience and conscious attention predicted acquisition, and exposure
  frequency didn't. A frantic shooter is the worst case. In Kaiku, the
  **gloss and post-run recap carry the learning**, more than the combat
  itself.
- **Copying a word you can see is transcription, not retrieval.** It teaches
  spelling and recognition, not meaning. Once a word is "known", sometimes
  show **only the gloss** (type `Schmetterling` when shown "butterfly"). The
  k-type `[display, answer]` schema makes this free.
- **Spaced repetition in two layers:**
  - *In-wave:* simple rotation; a recently missed word comes back sooner.
  - *Across runs:* FSRS via `rs-fsrs` (MIT, depends only on chrono) picks
    each horde's 15-20-word pool and decides "known". Map results to FSRS
    ratings: fast and clean = Easy, clean = Good, slow or corrected = Hard,
    missed = Again.

Proposed word-entry schema (per horde, TOML):

```toml
[[word]]
display = "Schmetterling"   # what flies at you
answer  = "schmetterling"   # what you type (NFC); romaji goes here for kana
alt     = ["schmetterling"] # lenient-mode alternatives
gloss   = "butterfly"
freq    = 2140              # frequency rank, drives pool weighting
tags    = ["noun", "animal"]
```

---

## 7. Open-source references

| Reference | Licence | Why it matters |
|---|---|---|
| **Asciiverse `entity/src/input.rs`** (and Ceres, Brickspire, Marble Recovery) | ours | **The** input layer: three tiers (exact, timing, latched) |
| tetro-tui (Rust, crossterm) | MIT | Timestamped input thread |
| crossterm / ratatui | MIT | The stack |
| rusty-rain | Apache-2.0 | Matrix-rain rendering in Rust |
| CleanCut/invaders | MIT/Apache | Minimal Rust terminal shooter skeleton; also a counterexample (no fixed timestep, movement driven by key repeat) |
| ttyper, toipe, thokr | MIT | Rust typing input; ttyper has frequency-ordered word lists (check where they come from before bundling) |
| rs-fsrs | MIT | SRS scheduler, small enough to embed |
| Tux Typing | GPL-2.0 | Fish Cascade and Comet Zap design |
| Project Starfighter, Tumiki Fighters | GPL-3.0, BSD-2 | Genuine open-source horizontal shmups |
| ninvaders, moon-buggy | GPL | Classic terminal action games (C/ncurses) |
| k-type | ZType base is proprietary | Target-lock and display/answer design only |
| JMdict (EDRDG) | CC BY-SA | The source for Japanese dictionary data if needed (attribution and share-alike apply) |

**Licensing:** permissive code (MIT/Apache/BSD) can be reused with
attribution. GPL code is for reading only, unless Kaiku itself goes GPL.
That includes Tux Typing's word lists, which are GPL data. Author Kaiku's
own lists.

---

## 8. Design lessons, condensed

1. Go horizontal: ship on the left, lanes as rows, RX from the right, TX
   back out.
2. First-letter lock on the soonest-arriving word. Never two live words with
   the same first letter.
3. Locking snaps the ship to the word's lane. An optional dodge on
   non-letter keys. No modes, ever.
4. Tempo is health (Sine Mora): hits drain a signal buffer, kills refill it.
5. The guns plus a post-hit shield always cover the player mid-word.
6. Language = polarity: same-faction chains build the multiplier.
7. A visible, decaying combo meter drives the global tint.
8. Visible difficulty scaling (aggro per horde). Never reward dying.
9. Partial upgrade decay on a hit, never a full wipe. Respawn in place.
10. One reserved colour and glyph for enemy bullets.
11. Payload = length × lateness, plus an accent bonus in strict mode.
12. Retrieval drills for known words: show the gloss, type the word.
13. FSRS across runs, simple rotation within a wave.
14. Darius-style branches as a language choice between stages.
15. Tux Fish Cascade suffix-matching as the novice on-ramp (§8 open
    question).
16. Everything is data: hordes, rules and waves in text files, Typespeed-style.

## 9. Open risks

- **Divided attention (Textorcist):** typing plus dodging may be tiring
  rather than fun. The MVP must test this first.
- **Learning under load:** measure it with a before-and-after word quiz;
  don't assume it.
- **Small pools against the first-letter rule:** 15-20 words with unique
  live first letters limits how many words can be live at once. Check this
  against real word lists.
- **Terminal variance:** cell aspect, Braille rendering, kana width.
- **Dead keys versus kitty flag 8:** the one input question Asciiverse hasn't
  answered yet (§5). Only about 8 lanes fit at 24 rows.
- **Dead keys and IME** for kana make Phase 3 hard, which supports shipping
  the romaji fallback first.
