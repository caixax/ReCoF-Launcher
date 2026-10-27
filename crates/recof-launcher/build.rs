//! Picks up whatever artwork the repository carries.
//!
//! `assets/artwork` is a slot. The launcher draws its own look when it is
//! empty, so a clone with nothing in it still builds and still runs. Put
//! a picture in it and the next build embeds it, with no code change
//! anywhere.
//!
//! Whoever fills it is responsible for having the right to publish what
//! they put there, which is why it is a slot and not a shipped set. What
//! this repository carries, and the permission behind it, is written down
//! in CREDITS.md at the root.

use std::path::Path;

/// The pictures the launcher knows what to do with.
const PIECES: &[(&str, &str)] = &[
    ("hero", "bundled_hero"),
    ("logo", "bundled_logo"),
    ("icon", "bundled_icon"),
];

/// Tried in this order. Photographs belong in a JPEG and anything that
/// needs transparency belongs in a PNG, so the slot takes both rather
/// than forcing a picture into the wrong container.
const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];

fn main() {
    let dir = Path::new("assets/artwork");
    let out = std::env::var("OUT_DIR").expect("cargo always sets OUT_DIR");
    let out = Path::new(&out);

    // Declared so a build with none of them set does not warn about an
    // unexpected cfg name.
    for (_, flag) in PIECES {
        println!("cargo::rustc-check-cfg=cfg({flag})");
    }

    println!("cargo::rerun-if-changed=assets/artwork");

    for (name, flag) in PIECES {
        let found = EXTENSIONS
            .iter()
            .map(|extension| dir.join(format!("{name}.{extension}")))
            .inspect(|path| println!("cargo::rerun-if-changed={}", path.display()))
            .find(|path| path.is_file());

        let Some(path) = found else { continue };

        // Copied under one name so the include does not have to know
        // which container the picture arrived in. The decoder sniffs the
        // format from the bytes, so it does not care either.
        let staged = out.join(name);
        std::fs::copy(&path, &staged)
            .unwrap_or_else(|error| panic!("{} could not be staged: {error}", path.display()));

        println!("cargo::rustc-cfg={flag}");
        println!(
            "cargo::warning=bundling {} into the launcher",
            path.display()
        );
    }
}
