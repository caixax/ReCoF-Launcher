//! Discord Rich Presence, while the game is open.
//!
//! Off unless the user turns it on, and off again the moment the game
//! closes. It shows that somebody is playing ReCoF and which version; it
//! sends no identifier the launcher does not already show on its own
//! screen, and nothing at all when it is disabled.
//!
//! Discord is reached through a local named pipe. There is no network
//! request here, and a Discord that is not running is simply a presence
//! that does not appear.

use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::consts::DISCORD_APP_ID;

/// Why presence is not showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unavailable {
    /// This build has no Discord application id compiled in.
    NotConfigured,
    /// Discord is not running, or refused the connection.
    NotConnected(String),
}

/// The connection, if there is one.
pub struct Presence {
    client: Option<DiscordIpcClient>,
    /// What is on screen now, so the same activity is not sent twice.
    showing: Option<String>,
    /// When the current session started, for the elapsed time Discord
    /// shows.
    since: i64,
}

impl Default for Presence {
    fn default() -> Self {
        Self::new()
    }
}

impl Presence {
    pub fn new() -> Self {
        Presence {
            client: None,
            showing: None,
            since: now_seconds(),
        }
    }

    /// Whether this build can show presence at all.
    pub fn is_configured() -> bool {
        !DISCORD_APP_ID.is_empty()
    }

    /// Connects, if it is not connected already.
    pub fn connect(&mut self) -> Result<(), Unavailable> {
        if !Self::is_configured() {
            return Err(Unavailable::NotConfigured);
        }
        if self.client.is_some() {
            return Ok(());
        }

        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
        client
            .connect()
            .map_err(|error| Unavailable::NotConnected(error.to_string()))?;

        log::info!("connected to Discord");
        self.client = Some(client);
        self.since = now_seconds();
        Ok(())
    }

    /// Shows that ReCoF is running.
    ///
    /// Called repeatedly while the game is open; sending the same thing
    /// again is skipped, so this is cheap to call from a poll.
    pub fn show_playing(&mut self, version: &str) -> Result<(), Unavailable> {
        let state = format!("ReCoF {version}");
        if self.showing.as_deref() == Some(state.as_str()) {
            return Ok(());
        }
        self.connect()?;

        let Some(client) = self.client.as_mut() else {
            return Err(Unavailable::NotConfigured);
        };
        let activity = activity::Activity::new()
            .details("Cry of Fear")
            .state(&state)
            .timestamps(activity::Timestamps::new().start(self.since));

        if let Err(error) = client.set_activity(activity) {
            // A Discord that went away mid session is not an error worth
            // showing: drop the connection and try again next time.
            log::debug!("the Discord activity could not be set: {error}");
            self.client = None;
            self.showing = None;
            return Err(Unavailable::NotConnected(error.to_string()));
        }

        self.showing = Some(state);
        Ok(())
    }

    /// Takes the presence down, which is what happens when the game
    /// closes or the setting is turned off.
    pub fn clear(&mut self) {
        self.showing = None;
        let Some(client) = self.client.as_mut() else {
            return;
        };
        if let Err(error) = client.clear_activity() {
            log::debug!("the Discord activity could not be cleared: {error}");
        }
    }

    /// Disconnects entirely.
    pub fn disconnect(&mut self) {
        self.clear();
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
            log::info!("disconnected from Discord");
        }
    }

    /// Whether something is on screen right now.
    pub fn is_showing(&self) -> bool {
        self.showing.is_some()
    }
}

impl Drop for Presence {
    fn drop(&mut self) {
        // Leaving a stale presence behind after the launcher closes would
        // tell people somebody is playing when they are not.
        self.disconnect();
    }
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_build_with_no_application_id_reports_itself_unconfigured() {
        // This is the state the project ships in until a Discord
        // application exists, and it has to be a clean "not available"
        // rather than a failure.
        if DISCORD_APP_ID.is_empty() {
            assert!(!Presence::is_configured());
            let mut presence = Presence::new();
            assert_eq!(presence.connect(), Err(Unavailable::NotConfigured));
            assert!(!presence.is_showing());
        } else {
            assert!(Presence::is_configured());
        }
    }

    #[test]
    fn showing_and_clearing_never_panic_without_discord() {
        let mut presence = Presence::new();
        // Discord is almost certainly not running under the test runner,
        // and either answer is fine; what matters is that neither call
        // can bring the launcher down.
        let _ = presence.show_playing("0.3.0");
        presence.clear();
        presence.disconnect();
        assert!(!presence.is_showing());
    }

    #[test]
    fn clearing_a_connection_that_was_never_made_is_harmless() {
        let mut presence = Presence::new();
        presence.clear();
        presence.clear();
        assert!(!presence.is_showing());
    }

    #[test]
    fn the_application_id_is_either_empty_or_a_snowflake() {
        // Discord ids are decimal snowflakes. A typo here would mean the
        // presence silently never appears.
        if !DISCORD_APP_ID.is_empty() {
            assert!(
                DISCORD_APP_ID.len() >= 17
                    && DISCORD_APP_ID.bytes().all(|byte| byte.is_ascii_digit()),
                "{DISCORD_APP_ID:?} is not a Discord application id"
            );
        }
    }
}
