//! How long the game has been played, and when it was last played.
//!
//! The launcher already asks the system every few seconds whether the
//! game is running, for the Discord presence. This counts the time
//! between those answers, so it needs nothing the launcher was not
//! already doing.
//!
//! Counted from the poll rather than from a session that is opened and
//! closed, because a launcher that is killed, or a machine that loses
//! power, would never reach the closing half. The most that can be lost
//! is whatever has not been written yet, which is a minute at the
//! outside.
//!
//! This is the launcher's own count and nothing else. Steam's hours for
//! retail Cry of Fear belong to Steam, and nothing here reads or touches
//! them.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::IoResultExt;
use crate::Result;

/// A tick longer than this is not play, it is a launcher that was asleep.
///
/// Windows stops handing out frames to a minimised window and suspends
/// the machine on a lid close, so one poll can be followed by another an
/// hour later with no game time in between. Anything beyond a generous
/// multiple of the poll interval is treated as a gap, not as play.
const LONGEST_CREDIBLE_TICK: Duration = Duration::from_secs(60);

/// What the launcher knows about time spent in the game.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Playtime {
    /// Total seconds the game has been seen running.
    pub seconds: u64,
    /// When it was last seen running, in seconds since the epoch.
    pub last_played: Option<u64>,
    /// How many times it has been started.
    pub sessions: u32,
}

impl Playtime {
    /// Reads the count, treating anything unreadable as nothing counted.
    ///
    /// A missing file is the normal state before the first launch, and a
    /// corrupt one is not worth refusing to start over.
    pub fn load(dir: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(file(dir)) else {
            return Playtime::default();
        };
        serde_json::from_str(&text).unwrap_or_else(|error| {
            log::warn!("the playtime file could not be read, starting from zero: {error}");
            Playtime::default()
        })
    }

    /// Writes the count through a temporary file, the way the settings
    /// are written: a launcher killed mid write must not leave a file
    /// that reads as zero hours.
    pub fn save(&self, dir: &Path) -> Result<()> {
        use std::io::Write;

        std::fs::create_dir_all(dir).at(dir)?;
        let text =
            serde_json::to_string_pretty(self).expect("a struct of numbers always serialises");

        let path = file(dir);
        let temporary = path.with_extension("json.tmp");
        {
            let mut handle = std::fs::File::create(&temporary).at(&temporary)?;
            handle.write_all(text.as_bytes()).at(&temporary)?;
            handle.sync_all().at(&temporary)?;
        }
        std::fs::rename(&temporary, &path).at(&path)?;
        Ok(())
    }

    /// Credits one poll in which the game was seen running.
    ///
    /// `since_last` is the time between this answer and the one before
    /// it. Returns whether anything was counted, so the caller can decide
    /// whether the file is worth rewriting.
    pub fn credit(&mut self, since_last: Duration) -> bool {
        self.last_played = Some(now_seconds());
        if since_last.is_zero() || since_last > LONGEST_CREDIBLE_TICK {
            // The gap is not time at the keyboard. The stamp above still
            // moves, because the game is running right now whatever the
            // launcher was doing before.
            return false;
        }
        self.seconds += since_last.as_secs();
        true
    }

    /// Records that a session has begun, which is the moment the game
    /// goes from not running to running.
    pub fn begin_session(&mut self) {
        self.sessions = self.sessions.saturating_add(1);
        self.last_played = Some(now_seconds());
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.seconds)
    }

    /// Whole hours and leftover minutes, which is how it reads on screen.
    pub fn hours_and_minutes(&self) -> (u64, u64) {
        (self.seconds / 3600, (self.seconds % 3600) / 60)
    }

    /// Whether anything has been played at all.
    pub fn is_empty(&self) -> bool {
        self.seconds == 0 && self.last_played.is_none()
    }

    /// Whole days since it was last played, where zero means today.
    ///
    /// `None` when it has never been played, or when the stamp is in the
    /// future, which means the clock moved and any answer would be a lie.
    pub fn days_since_last(&self) -> Option<u64> {
        let last = self.last_played?;
        let now = now_seconds();
        if last > now {
            return None;
        }
        Some((now - last) / 86_400)
    }
}

fn file(dir: &Path) -> PathBuf {
    dir.join("playtime.json")
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-playtime-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn nothing_played_yet_reads_as_nothing() {
        let played = Playtime::default();
        assert!(played.is_empty());
        assert_eq!(played.hours_and_minutes(), (0, 0));
        assert_eq!(played.days_since_last(), None);
    }

    #[test]
    fn an_ordinary_poll_is_counted() {
        let mut played = Playtime::default();
        assert!(played.credit(Duration::from_secs(3)));
        assert!(played.credit(Duration::from_secs(3)));
        assert_eq!(played.seconds, 6);
        assert!(!played.is_empty());
    }

    #[test]
    fn a_suspended_machine_does_not_earn_hours() {
        // The lid was closed for two hours with the game open. Counting
        // the gap would hand out time nobody played.
        let mut played = Playtime::default();
        assert!(!played.credit(Duration::from_secs(7200)));
        assert_eq!(played.seconds, 0);
        // The game is running now, so the stamp still moved.
        assert!(played.last_played.is_some());
    }

    #[test]
    fn hours_and_minutes_split_the_way_they_are_read() {
        let played = Playtime {
            seconds: 2 * 3600 + 53 * 60 + 40,
            ..Default::default()
        };
        assert_eq!(played.hours_and_minutes(), (2, 53));
    }

    #[test]
    fn a_session_count_cannot_wrap() {
        let mut played = Playtime {
            sessions: u32::MAX,
            ..Default::default()
        };
        played.begin_session();
        assert_eq!(played.sessions, u32::MAX);
    }

    #[test]
    fn a_clock_that_went_backwards_says_nothing_rather_than_lying() {
        let played = Playtime {
            last_played: Some(now_seconds() + 10_000),
            ..Default::default()
        };
        assert_eq!(played.days_since_last(), None);
    }

    #[test]
    fn played_today_is_zero_days_ago() {
        let played = Playtime {
            last_played: Some(now_seconds() - 60),
            ..Default::default()
        };
        assert_eq!(played.days_since_last(), Some(0));
    }

    #[test]
    fn it_survives_a_round_trip_through_the_file() {
        let dir = scratch("round-trip");
        let played = Playtime {
            seconds: 9_999,
            last_played: Some(1_700_000_000),
            sessions: 12,
        };
        played.save(&dir).unwrap();
        assert_eq!(Playtime::load(&dir), played);
    }

    #[test]
    fn a_corrupt_file_starts_from_zero_instead_of_refusing_to_start() {
        let dir = scratch("corrupt");
        std::fs::write(file(&dir), b"{ this is not json").unwrap();
        assert_eq!(Playtime::load(&dir), Playtime::default());
    }

    #[test]
    fn a_file_from_a_later_version_keeps_what_it_understands() {
        // A field this build does not know about must not throw away the
        // hours, which is what serde(default) on a plain struct buys.
        let dir = scratch("from-the-future");
        std::fs::write(file(&dir), br#"{"seconds": 120, "something_new": true}"#).unwrap();
        assert_eq!(Playtime::load(&dir).seconds, 120);
    }
}
