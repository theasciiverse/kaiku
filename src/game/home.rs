//! Home: Earth rises, Earth's own fighters fly out to meet you and form up
//! around your ship, and the closing crawl (`data/home.txt`) types out a hero's
//! welcome. Then the run's summary, then the credits.

use super::intro;
use super::ship::{SHIP_H, SHIP_W};

/// Seconds into Home when the crawl starts typing: Earth has risen a while.
pub const CRAWL_AT: f32 = 3.0;
/// Seconds the finished crawl stays up before the summary takes its place.
const CRAWL_HOLD: f32 = 3.0;

/// When the first fighter leaves Earth, the gap between them, and how long
/// each takes to reach its place.
const LAUNCH: f32 = 1.0;
const STAGGER: f32 = 0.4;
const FLIGHT: f32 = 4.5;

/// Where the escort forms up, relative to the ship's top-left, in dots: a V
/// opening out behind it, the nearest pair first.
const SLOTS: [(f32, f32); 6] = [(-12.0, -12.0), (-12.0, 12.0), (-24.0, -23.0), (-24.0, 23.0), (-36.0, -34.0), (-36.0, 34.0)];

pub fn text() -> &'static str {
    include_str!("../../data/home.txt").trim_end()
}

/// The crawl revealed `state_t` seconds into Home.
pub fn shown(state_t: f32) -> String {
    intro::shown_of(text(), state_t - CRAWL_AT)
}

/// The crawl is typing.
pub fn crawling(state_t: f32) -> bool {
    (CRAWL_AT..CRAWL_AT + intro::duration_of(text())).contains(&state_t)
}

/// Seconds into Home when the summary replaces the crawl.
pub fn summary_at() -> f32 {
    CRAWL_AT + intro::duration_of(text()) + CRAWL_HOLD
}

/// Where the ship drifts to at home, in dots: left of centre, with room
/// behind it for the escort.
pub fn ship_station(w: f32, h: f32) -> (f32, f32) {
    ((w * 0.16).max(40.0), (h - SHIP_H) / 2.0)
}

/// Earth as it rises, in half-block pixels (one per cell across, two per
/// cell down): centre and radius, growing over the first seconds of Home.
pub fn earth(w: f32, h: f32, state_t: f32) -> (f32, f32, f32) {
    let (pw, ph) = (w / 2.0, h / 2.0);
    let r = ph * (0.2 + 0.16 * ease(state_t / 6.0));
    (pw - r - 4.0, ph / 2.0, r)
}

/// One of Earth's fighters, top-left in dots; `outbound` while it still flies
/// away from Earth (nose to the left), before it swings round into place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escort {
    pub x: f32,
    pub y: f32,
    pub outbound: bool,
}

/// The escort `state_t` seconds into Home: each launches from `earth`, flies
/// out past the ship, loops round and settles into its place beside `ship`
/// (top-left, in dots). Fighters not yet launched aren't there.
pub fn escorts(state_t: f32, ship: (f32, f32), earth: (f32, f32)) -> Vec<Escort> {
    SLOTS
        .iter()
        .enumerate()
        .filter_map(|(i, &(dx, dy))| {
            let t = (state_t - LAUNCH - i as f32 * STAGGER) / FLIGHT;
            if t <= 0.0 {
                return None;
            }
            let t = ease(t);
            let end = (ship.0 + dx, ship.1 + dy);
            // the turn: well behind the place and further out, so the
            // fighter overshoots and comes back round to face Earth
            let side = dy.signum();
            let bend = (end.0 - SHIP_W * 2.5, end.1 + side * SHIP_H * 2.0);
            let start = (earth.0 - SHIP_W / 2.0, earth.1 - SHIP_H / 2.0);
            let u = 1.0 - t;
            let x = u * u * start.0 + 2.0 * u * t * bend.0 + t * t * end.0;
            let y = u * u * start.1 + 2.0 * u * t * bend.1 + t * t * end.1;
            // heading left while x still falls: the curve's x turns round
            // where its derivative crosses zero
            let vx = 2.0 * u * (bend.0 - start.0) + 2.0 * t * (end.0 - bend.0);
            Some(Escort { x, y, outbound: vx < 0.0 })
        })
        .collect()
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_escort_flies_out_and_forms_up() {
        let (ship, earth) = ((40.0, 30.0), (160.0, 35.0));
        assert!(escorts(0.5, ship, earth).is_empty(), "none before launch");
        let early = escorts(LAUNCH + 0.3, ship, earth);
        assert_eq!(early.len(), 1);
        assert!(early[0].outbound && early[0].x > 120.0, "the first one leaves Earth, nose to the left");
        let formed = escorts(summary_at(), ship, earth);
        assert_eq!(formed.len(), SLOTS.len());
        for (e, (dx, dy)) in formed.iter().zip(SLOTS) {
            assert!(!e.outbound, "in place, facing the way the ship faces");
            assert!((e.x - ship.0 - dx).abs() < 0.01 && (e.y - ship.1 - dy).abs() < 0.01);
        }
    }

    #[test]
    fn the_welcome_types_out_then_the_summary() {
        assert_eq!(shown(CRAWL_AT), "");
        assert!(crawling(CRAWL_AT + 1.0));
        assert!(shown(summary_at()).ends_with("WELCOME HOME, HERO."));
        assert!(!crawling(summary_at()));
        assert!(summary_at() > CRAWL_AT + 10.0 && summary_at() < 60.0, "{}", summary_at());
        // the escort is in place before the summary covers it
        assert!(LAUNCH + (SLOTS.len() - 1) as f32 * STAGGER + FLIGHT < summary_at());
    }
}
