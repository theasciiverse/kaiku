//! The high score, kept across runs in $XDG_DATA_HOME/kaiku/highscore
//! (or ~/.local/share/kaiku/highscore), as tty-rex keeps its own.

use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("kaiku").join("highscore"))
}

pub fn load() -> u32 {
    path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

pub fn save(score: u32) {
    let Some(p) = path() else { return };
    let result = p.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&p, format!("{score}\n")));
    if let Err(e) = result {
        log::warn!("can't save high score to {}: {e}", p.display());
    }
}
