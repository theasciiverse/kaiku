//! Cues: the moments the game wants heard and felt. The simulation only names
//! them; the main loop turns each into a sound (`audio.rs`) and, for the
//! pilot, a rumble (`pad.rs`). That keeps the game silent and testable.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Cue {
    /// the player's gun
    Shot,
    /// a ship destroyed; `Big` for gunships, lancers, jammers, carriers
    Kill,
    BigKill,
    /// a mine bursting into shrapnel
    Shrapnel,
    ShieldHit,
    LifeLost,
    GameOver,
    /// a right letter typed, a wrong one
    Key,
    Miss,
    /// an echo typed out; a pod's word typed out; a pod shot to pieces
    Decoded,
    Claimed,
    PodBroken,
    Bomb,
    Dash,
    /// a lancer's warning line appears; its beam fires
    Warning,
    Beam,
    /// a jammer starts scrambling the echoes
    Jamming,
    GatesOpen,
    /// the receiver's verdict on a decoded echo
    Jammed,
    Steady,
    /// into a gate: the turn
    Jump,
    ExtraLife,
    /// a shield cell collected: one segment back
    Cell,
    Home,
    /// a letter of the opening crawl, or a repair answered in the cockpit
    Typewriter,
    Repaired,
}
