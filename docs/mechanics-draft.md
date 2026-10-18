# Kaiku — Mechanics Draft (v1)

First draft of how the game plays. It builds on the decisions in CLAUDE.md;
anything marked *(default)* is a proposal waiting for a playtest. English
only; the game teaches typing.

## The story in one breath

An interplanetary war is raging. A surge throws your ship far off course,
into a pocket of space where Earth's old transmissions bounce around as
garbled echoes. You repair the receiver and start catching them. **The
clearer an echo, the more it points home.** Decode the echoes, take the
right portals, and fly through the war's crossfire until you're home.
(Homage: *Gates of Zendocon*, see `research/2026-10-gates-of-zendocon.md`.)

## Controls

| Keys | Action |
|---|---|
| Arrows, Ctrl+h/j/k/l | Move (held; the input layer is adapted from Entity) |
| Space | Shoot (held = auto-fire) |
| a–z | Typing: lock and decode an echo, or claim a power-up |
| Backspace | Drop the current lock |
| Enter | Smart bomb *(default; a few per sector)* |
| Esc | Pause or quit (asks first, as in Entity) |

Typing and steering never conflict: plain letters are text; Ctrl+letters,
arrows and Space are flight.

## A sector, start to finish

A **sector** is one stage: one tunnel between two portals.

1. **Flight.** The tunnel scrolls right to left. Fleets of the two warring
   sides cross it: some hunt you, some fight each other, and some just pass
   through, firing. You move and shoot. This is the plain shmup, at about
   60–90 seconds a sector *(default)*.
2. **Power-ups.** Now and then a power-up pod drifts in, carrying a short
   word. Type the word to claim it. Shooting a pod destroys it, so keep your
   fire off it.
3. **Echoes.** Partway through, the receiver catches echoes: transmission
   snippets drifting across the field, garbled and flickering (see "Echoes").
   Usually two or three at once *(default)*. **One is true**, from the
   direction of home. The others are false, bounced off the war or faked by
   jammers.
4. **Decoding.** Type an echo's first letter to lock onto it, then keep
   typing. Each correct letter locks in clear. A decoded echo shows its
   **clarity**, which is how strongly it points home.
5. **Portals.** At the end of the sector, one portal appears per decoded
   echo, each tagged with its echo. Fly into one. If you decoded none, a
   single random portal opens. A blind jump is legal, just unlucky.
6. **The turn.** Entering a portal swings the whole field round in a Braille
   rotation animation. Play stays horizontal; the new tunnel's colours,
   walls and heading show where you are going.
7. **The map.** Between sectors, a route map shows where you are, the jumps
   you've taken, and how far home is. **The true portal shortens the way
   home; a false one lengthens it.** It is never game over.

## Echoes

- **Content:** a mashup of Earth's famous public-domain transmissions:
  - Apollo: "The Eagle has landed", "one small step", "Houston, we've had a
    problem", the Apollo 8 Christmas Eve reading.
  - JFK's "We choose to go to the Moon".
  - The Voyager greeting "Hello from the children of planet Earth".
  - The Arecibo message, as a Braille picture echo.

  Snippets can cut between sources mid-line, like a radio jumping stations.
- **Display:** garbled text, drawn as a mix of wrong letters, symbols and
  Braille noise, that flickers into full clear for a moment, still blinking.
  The player reads during the flickers. Typed letters stay clear.
- **Why the flicker matters:** the player can't type what they haven't read,
  so the challenge is catching the clear moments while flying.
- **Clarity:**
  - True echoes flicker clear more often and hold longer, and the garble gets
    thinner the closer you are to home.
  - False echoes flicker less, and some carry a tell: a broken phrase, wrong
    words, mixed-up sources.

  So "the clearer, the more it points home" is both the rule and something
  you can see.
- **Mistakes:** a wrong key flashes the echo and costs a little clarity on
  it, but keeps the lock. Backspace drops the lock.
- **Length:** short early (2–4 words), longer later. The very last echo,
  arriving home, is perfectly clear.

## Typing as a skill curve

Enemy fleets don't carry words; only **power-ups** and **echoes** do.
- **Power-up words follow a typing-tutor curve:**
  - Early sectors use home-row words ("lads", "flask").
  - Then the top row ("quiet", "power").
  - Then the bottom row ("zinc", "beam").
  - Then the whole board.

  The player learns the keyboard without being told.
- **Echoes stay real quotations** and are free text, shorter and simpler
  early on.
- **First-letter rule (ZType):** no two live typing targets on screen share
  a first letter, so one key always names one target.

## Combat

- **Ship:** moves freely in the field, with a small hitbox (visibly smaller
  than the sprite).
- **Fire:** Space fires the main gun forward. Power-ups upgrade it.
- **Getting hit:** a shield bar absorbs hits. When it's empty the next hit
  costs a life. You start with 3 lives *(default)* and respawn in place,
  with brief invulnerability.
- **Enemies:**
  - The two warring sides, each with its own colour and look.
  - Formations fly in, telegraphed.
  - Enemy bullets have one colour and glyph that nothing else uses.
  - Some ships fight each other, so you can sometimes let the war do your
    work.
- **Power-ups** *(defaults)*: spread shot, rapid fire, a shield refill, a
  smart-bomb charge, an escort drone (one of each type, Zendocon-style),
  and slow time.
- **End-of-sector docking** *(later)*: a portal approach that re-arms you if
  you hit it cleanly.

## The route home

- **Shape:** a route graph that branches and rejoins, always moving forward.
  There are several equally short ways home.
- **Length:** the shortest route is about 8 sectors *(default, tune by
  playtest)*. Each false portal adds 1–2 detour sectors.
- **Sector names:** each sector has a four-letter name shown on the map,
  which doubles as a password to resume.
- **The first fork sets the difficulty:** an easier or a harder way home.
- **Secret** *(later)*: a hidden portal whose echo leaks through a tunnel
  wall. It leads to a blank-named sector, holding hidden credits, that loops
  back.

## Intro scene (the tutorial)

1. Dark cockpit, static. The receiver is broken.
2. The player types short repair commands (`reboot`, `tune`, `scan`), which
   is gentle first typing.
3. The first echo arrives, very garbled, then flickers clear: *The Eagle has
   landed.* The player types it, and it locks in clear.
4. A direction lights up, and the first turn swings the field round. The
   game begins.

## Arrival home

The last echo is fully clear and steady: perhaps the Voyager greeting, or a
voice from home. Quiet, no music (a nod to Zendocon's silent finale).
Credits.

## Screen layout and look

- **The play area** sits in the middle. Around it is **Crush-style chrome**:
  - The KAIKU wordmark with a gradient.
  - `╱` hatching fields.
  - Rounded borders.
  - A HUD with shield, lives, bombs and score.
  - **A bottom panel:** the receiver (longer echoes and ship-to-ship
    messages are decoded there) and the power-up illustrations, dim until
    you own them.
  - A small route indicator.
- **ASCII** for text: echoes, power-up words, the HUD.
- **Half-blocks** for ships, enemies and tunnel walls.
- **Braille** for bullets, particles, the starfield and the turn animation.
- **Every visual means something** *(principle carried from earlier
  thinking)*: tunnel colour shows the sector, garble shows clarity, and HUD
  shimmer shows danger.

## Not in v1

Foreign languages, the translation bonus, online scores, sound and music
(maybe later), gamepad, and the secret sector.
