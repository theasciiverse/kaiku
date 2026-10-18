# Gates of Zendocon: a closer look (Oct 2026)

Research for Kaiku's homage. Gates of Zendocon is a horizontal shmup for the
Atari Lynx (Epyx, published by Atari, 1989).

**About the sources.** The richest source is Robert Krynak's 1992 hints file,
a fan document with a full gate map. The official manual text comes from a
fan transcription. MobyGames, IGN and GameFAQs were unreachable, so quotes
from them come second-hand.

- Wikipedia: https://en.wikipedia.org/wiki/Gates_of_Zendocon
- Manual text: https://mail.retroisle.com/ExtraFiles/gatesofzendocon_manual.txt
- Krynak hints: https://web.archive.org/web/2017/https://www.atariarchives.org/cfn/08/08/0011.php
- Hardcore Gaming 101 (HG101): https://www.hardcoregaming101.net/gates-of-zendocon/

## History

- **Credits:** Peter M. Engelbrite (design, code, art), Matthew Crysdale and
  Arthur Koch (art), Robert Vieira and Alex Rudis (music).
- **Launch:** a North American Lynx launch title in 1989. Japan got it on
  23 Dec 1989, via Mumin.
- **A terminal-lineage fact:** development began before Lynx hardware
  existed. Engelbrite wrote a Lynx emulator on an Apple II, with sprites
  shown as ASCII text.

## Story (manual)

> "The droids of your old nemesis, the evil spider Zendocon, captured you."
> "Zendocon itself sentenced you to wander its web of deadly universes."

The manual's goal is revenge: a duel with the spider. HG101 paraphrases it as
defeating Zendocon "and return home". Kaiku's goal is purely getting home,
which is a nice inversion.

## Structure: gates

- **Size:** "51 interconnected universes." Krynak also counts a hidden
  blank-named universe, ZETA, and the finale.
- **Rules (manual):**
  - Every universe has at least one gate.
  - "You cannot go back through the gate you entered."
  - Some universes have several gates leading to different universes.
  - On the first level, the first gate leads to easy levels and the second
    to harder ones.
- **How you choose:** by where you fly. Gates sit top, bottom, early or late
  in the level; for example, one gate only comes after two waves of attack.
  Most levels have one gate, and a few have three or four.
- **Route shape:** the routes form a graph that branches and rejoins, and
  always moves forward. The shortest route is "fifteen screens, plus
  Zendocon", and there are three equally short paths through one stretch. No
  single run sees every universe.
- **Secret gate:** in one universe a floor pens you into the top half, but
  bombs fall through an invisible hole. Behind it is a gate to a
  blank-named universe, so its code can't be typed in. There you get every
  ally and shoot digitised heads of the developers. It leads back to an
  earlier universe, so you can loop forever.
- **Passwords:** each universe's four-letter name, shown on its base, is its
  password. There is no in-game map; fans drew one.

## Mechanics

- **Controls:** B held fires the laser, B tapped drops bombs (in an arc). A
  held raises a shield that overheats. The thrusters damage enemies behind
  you.
- **Damage in steps:** the first hit takes your shield, the second your
  laser, the third your ship. You start with 5 lives.
- **Docking:** landing on the end-of-level base re-arms you fully. Landing
  while its force field is up kills you.
- **Allies:** rescued aliens follow your ship with their own weapons: one
  fires horizontally from below, one fires straight up, one seeks and rams,
  and one sends a widening wave forward. You can have only one of each, and
  they can be destroyed. All of them explode in the penultimate stage, so you
  face the end alone.
- **Determinism:** enemies appear "in the same places, in the same
  sequence."
- **Scrolling:** always horizontal, with free vertical movement. There are
  no rotation or vertical sections; variety comes from layout. **No source
  describes a special effect when you pass a gate**; the docking sequences
  at the bases were the showpiece. Kaiku's turning playfield is therefore
  our own addition, not a borrowing.
- **Bosses:** a scaling, beating giant heart; dragons you kill with your
  tail jets; an eye you must hit twice. The final spider fight is silent.

## Look, sound, reception

- **Look:** big, detailed sprites with no slowdown, and mostly single-colour
  themed backgrounds (caves, fire, forest, intestines). The parallax reacts
  to height: higher fades the background, lower brings it forward.
- **Sound:** the music was the weak spot (few tracks, repetitive), while the
  effects were praised.
- **At release:** TGM 92, Zero 90, CVG 84. STart called it "limited depth".
  IGN later gave it 7/10.
- **Praised:** replay value from branching, and the passwords.
- **Criticised:** the slow pace, especially in easy mode (you can't speed up
  the scrolling), and the music.

## Rights

Epyx IP passed through Bridgestone to **Epyx Games, LLC** (2024), which
lists Zendocon as licensable. Atari's share of the Lynx versions is unclear.
Treat "Zendocon" and "Epyx" as live trademarks.

## Homage hooks for Kaiku

1. **Four-letter sector names that double as passwords,** shown when you
   dock: our own words, never Zendocon's.
2. **Portal position plus echo is the choice:** top, bottom, early or late
   portals, entered by typing their echo.
3. **The first fork sets the difficulty:** an easy route home or a hard one.
4. **No going back:** one-way tunnels, and a route graph that branches and
   rejoins, with several equally short ways home. The between-stage map
   shows it, which also fixes Zendocon's missing map.
5. **A secret portal whose echo leaks through a wall,** leading to a sector
   with a blank name that can't be typed as a code. It loops back to an
   earlier sector: an echo that returns.
6. **Hidden credits** in that secret sector, as Zendocon did.
7. **Damage that strips abilities in steps:** shield, then weapon, then
   ship. This fits the shield-bar-plus-lives decision.
8. **Docking at the end of a sector re-arms you,** and its timing matters.
9. **Escorts with distinct fire directions,** one of each type. Maybe they
   are lost before the last stretch, so you arrive home alone.
10. **Height-reactive parallax.**
11. **A quiet arrival home,** like Zendocon's silent final fight.
12. **Don't repeat its flaw:** keep the pace up, and let the player push the
    speed.

**Avoid:** the names Zendocon, Gates of Zendocon and Epyx; its universe
codes; its ally and weapon names; a giant-spider final boss; any of its
sprites, music or art. An "inspired by" credit naming it descriptively is
fine.
