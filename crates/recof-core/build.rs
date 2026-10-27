//! Watches the one build time variable the crate reads.
//!
//! Without this, changing `RECOF_DISCORD_APP_ID` would not rebuild the
//! crate that bakes it in, and the presence would quietly keep the old
//! value, which is a long afternoon for whoever is looking for it.

fn main() {
    println!("cargo::rerun-if-env-changed=RECOF_DISCORD_APP_ID");
}
