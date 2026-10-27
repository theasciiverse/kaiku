//! The opening story crawl, after the NES Metroid intro: the text types itself
//! out letter by letter, a blank line is a breath. The text is `data/intro.txt`;
//! the closing crawl at home (`super::home`) types out the same way.

/// Letters per second.
const CPS: f32 = 26.0;
/// A line break costs this many letters of time; a blank line twice that.
const LINE_PAUSE: f32 = 6.0;

pub fn text() -> &'static str {
    include_str!("../../data/intro.txt").trim_end()
}

/// The time each character of `text` appears at, in seconds, in text order.
fn times(text: &str) -> impl Iterator<Item = (char, f32)> + '_ {
    let mut t = 0.0;
    let mut prev = ' ';
    text.chars().map(move |c| {
        t += match c {
            '\n' if prev == '\n' => 2.0 * LINE_PAUSE,
            '\n' => LINE_PAUSE,
            _ => 1.0,
        } / CPS;
        prev = c;
        (c, t)
    })
}

/// Seconds until the whole text is out.
pub fn duration() -> f32 {
    duration_of(text())
}

/// The text revealed after `secs`.
pub fn shown(secs: f32) -> String {
    shown_of(text(), secs)
}

/// Seconds until all of `text` is out.
pub fn duration_of(text: &str) -> f32 {
    times(text).last().map_or(0.0, |(_, t)| t)
}

/// As much of `text` as is revealed after `secs`.
pub fn shown_of(text: &str, secs: f32) -> String {
    times(text).take_while(|&(_, t)| t <= secs).map(|(c, _)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crawl_types_out_and_ends() {
        assert_eq!(shown(0.0), "");
        assert!(shown(1.0).len() > 10);
        assert!(shown(1.0).len() < shown(3.0).len());
        assert_eq!(shown(duration()), text());
        assert!(duration() > 10.0 && duration() < 60.0, "{}", duration());
    }
}
