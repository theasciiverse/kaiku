# Kaiku — CLAUDE.md
**Status:** First draft (v0.1) — captured from initial design conversation, pending review pass.
**Platform:** Asciiverse (Rust / Ratatui / Drawille)
**Genre:** Terminal horde-shooter × typing-under-pressure × incidental language acquisition

---

## 1. Elevator Pitch

Kaiku is a terminal-native horde shooter where enemy hordes are language factions —
German, French, Japanese, and so on — whose communications stream toward you as
falling, color-coded glyph columns, NCMatrix-style. You intercept and type their
words before they land ("RX" — incoming signal). A correctly typed word doesn't
just deal damage — it flips color and re-injects upward into that horde's column
as your own weaponized transmission ("TX" — you've hacked their comms and turned
them against them).

Guns give you a survival floor so typing never gates whether you live. Typing is
where the real damage, targeting, and progression live. Along the way, because
each horde's vocabulary is a small, spaced-repetition pool rather than a random
wordlist, you incidentally pick up real vocabulary in whatever languages you fight.

**Naming note:** "Kaiku" is Finnish for echo. It has no etymological relation to
"haiku" (Japanese) — the near-rhyme is a deliberate, acknowledged coincidence,
used because it reinforces the "short words / compressed moment" design
principle without requiring anyone to get the joke. Its Finnish origin also
gives it a quiet Liero lineage (scrappy, terminal-adjacent, substance-over-marketing
Finnish game heritage) — again as flavor, not as a load-bearing claim. The word
sounds plausibly "foreign" without reading as belonging to any one horde's
nationality, which matters since the game features many.

---

## 2. Prior Art & Licensing

**Inspiration only, zero code lineage:**
- **CMatrix / NCMatrix** (GPL-2.0) — the aesthetic principle borrowed is that
  NCMatrix drives the falling-character display from real network RX/TX packet
  data rather than a fixed animation. Kaiku borrows this *principle*
  (aesthetic driven by real underlying game-state metrics) not any code,
  algorithm implementation, or asset. Clean-room Rust/Ratatui implementation.
- **Typespeed** — general lineage of "type the falling/scrolling word" games.
- **rsync** — conceptual origin of the delta-sync mechanic (see §5.3), included
  here as an optional advanced-difficulty layer, not the core loop it started as.

No GPL code will be read, copied, ported, or referenced during implementation.

---

## 3. Core Loop

1. A wave ("horde") of one or more language factions spawns.
2. Each horde owns a column (or set of columns) with a distinct color.
   Words drawn from that horde's vocabulary pool fall downward (RX) at a
   rate and density tied to that horde's current aggro/threat level —
   a quiet horde barely streams; an aggroed horde floods the column.
3. One word per column is "live" (highlighted/brighter) at a time — the
   next thing you can act on.
4. Player types the live word. Correct completion:
   - Deals damage to the horde/unit associated with that packet.
   - Flips the word's color to the player's signature and re-injects it
     upward into the same column (TX) — a brief visual "shot fired back."
5. Incorrect/abandoned words that hit the bottom of the screen advance the
   horde (damage to player, or horde progress toward the player).
6. Guns fire automatically throughout (see §6) regardless of typing state —
   typing failure is never a hard stop, only a tempo loss.

---

## 4. Horde & Vocabulary System

### 4.1 Horde structure
- Each horde = one language faction, one signature color.
- Horde's word pool = ~15–20 core words, weighted toward high frequency,
  drilled repeatedly across a wave via rotation (spaced repetition, not a
  huge one-shot wordlist). You should see "wa" or "oui" multiple times in
  a single wave — that repetition is the point, not padding.
- Elite/boss units within a horde draw from longer or structurally distinct
  entries (see §4.3).

### 4.2 Incidental learning features
- **Gloss on first kill:** first time a word is cleared in a run, a short
  translation appears briefly near the kill point. No forced pause; ignorable
  if in flow, present if glanced at.
- **Post-run recap:** end-of-run screen lists every word cleared this run
  with translation — a de facto vocab log generated for free from combat stats.
- **Mastery tracking (persistent, cross-run):** a word hit correctly 20+
  times without a miss graduates to "known" — visual marker change in future
  runs (e.g. distinct glyph/color treatment) once known.
- **Design constraint:** no feature may force a pause or interrupt tempo.
  Learning rides alongside the action, never in front of it.

### 4.3 Boss units and grammar
Boss packets require a real grammatical unit, not just a longer word —
e.g. a German compound noun, a French verb conjugated across persons, a
Japanese particle + verb chain. This is the structural-difficulty escalation
within a horde, layered on top of vocabulary difficulty.

### 4.4 Multi-horde waves
Later waves place two (or more) hordes' columns on screen simultaneously,
forcing rapid language/script code-switching mid-combo. This is the game's
hard-mode identity — reflex speed matters less than which-language-am-I-in
recognition speed.

---

## 5. Combat Layer

### 5.1 Guns (the floor)
- Always-on or simple-input (e.g. spacebar / auto-fire-nearest).
- Low damage, untargeted or auto-target-nearest.
- Purpose: guarantee survivability during a bad typing beat. Typing input
  is never gated behind staying alive — the reverse should never be true either.

### 5.2 Typed words (the real layer)
- Damage is gated by horde/color signature match — you must be "in" the
  right column/target to route damage there.
- Short words = fast target routing, low payload, low exposure time.
- Long/rare words = larger payload, longer exposure (you're vulnerable
  mid-type).

### 5.3 Delta-sync (optional advanced mechanic, rsync-derived)
For elite/high-value packets only (not the base loop): a validated word
can be "re-synced" with a single-character mutation after a delay. Rather
than full retype, a checksum-style indicator flags the mismatched span and
the player patches only that segment. Full retype is reserved for rare
"full sync" packets or corruption events (packet flagged bad after being
cleared — punishes complacency). This layer is a stretch goal for later
phases, not part of the MVP loop — flag for revisit once the core RX/TX
loop is proven fun on its own.

### 5.4 Upgrades vs. super weapons
- **Upgrades (consistency reward):** N correct-signature words in a row
  without a miss → permanent-for-the-run buffs (reload speed, auto-target
  radius, extra lives). This is literally your combo acting as a "speedup
  ratio" multiplier, echoing rsync's real reported stat.
- **Super weapons (completion reward):** clearing a full signature set
  (all active hordes' live words typed correctly in sequence, "full sync"
  vs. incremental) triggers a rare, telegraphed screen-clear or major AOE.

---

## 6. Aesthetic Architecture (NCMatrix principle)

The rain **is** the game state, not a backdrop:

- **RX columns** — one per active horde, color-coded, density/fall-speed
  scaled to that horde's live aggro/threat level.
- **TX columns** — same physical column, reversed direction, palette-swapped
  to the player's signature, populated by successfully captured words.
- **Global tint/brightness** — driven by the player's live combo/speedup
  ratio, so the whole screen's temperature shifts with performance, not
  just a side-meter.
- No purely decorative animation layer — every visual variable should map
  to a real tracked game-state number, per the NCMatrix design principle
  this game is named in the spirit of.

---

## 7. Script Tiering & Rollout Plan

**Rendering risk is real and should gate scope, not block the whole project.**

- **Phase 1 — MVP:** single horde, Latin script only (pick one language,
  e.g. German or French). Prove the RX/TX capture-reinject loop, gun floor,
  and combo/upgrade layer in isolation. No multi-horde, no boss grammar yet.
- **Phase 2 — Multi-horde:** add 2–3 more Latin-script hordes (Spanish,
  Italian, Scandinavian). Introduce simultaneous multi-column waves and
  boss/grammar units.
- **Phase 3 — Script-tiered rollout:** Japanese horde as flagship "wow"
  content. Contingent on a `drawprobe` glyph-fidelity pass for kana
  rendering in the Braille/box-drawing terminal context. Ship a
  **romaji-mode fallback** (wa, ka, desu in Latin transliteration) first
  if kana isn't ready — still thematically Japanese, doesn't block release.
- **Phase 4 — Persistence & polish:** cross-run mastery tracking, recap
  screen refinement, delta-sync stretch mechanic (§5.3), additional hordes.

**Explicit sequencing note:** given Asciiverse's ship-first doctrine, do not
attempt Phases 2–4 concurrently with Phase 1. The MVP must be fun as a
single-horde loop before anything else is added.

---

## 8. Open Questions for Next Review Pass

- Exact word-pool sourcing/authoring workflow — hand-authored given
  fluency in Italian/Spanish/French/some Japanese/Scandinavian/German,
  but need a lightweight format (data file, not hardcoded) so hordes are
  addable without recompiling.
- Terminal rendering approach for column-based rain — confirm against
  Ratatui widget capabilities and `drawprobe`'s glyph-fidelity signal, the
  pattern used elsewhere in Asciiverse.
- Exact combo/speedup math and threshold tuning — needs playtesting, not
  spec-time guessing.
- Whether delta-sync (§5.3) survives as a real feature or gets cut entirely
  after Phase 1 playtesting — don't over-invest in it before the core loop
  is validated.
- Difficulty floor for total typing novices — should the very first minutes
  of Phase 1 be "just type the word," ramping into signature-matching, so
  new players don't bounce immediately.
