//! Picks up whatever artwork the repository carries.
//!
//! `assets/artwork` is a slot. It ships empty, and the launcher draws its
//! own look when it is empty. Put `hero.png`, `logo.png` or `icon.png` in
//! it and the next build embeds them, with no code change anywhere.
//!
//! Whoever fills it is responsible for having the right to publish what
//! they put there, which is why it is a slot and not a shipped set.

use std::path::Path;

/// The pictures the launcher knows what to do with.
const PIECES: &[(&str, &str)] = &[
    ("hero.png", "bundled_hero"),
    ("logo.png", "bundled_logo"),
    ("icon.png", "bundled_icon"),
];

fn main() {
    let dir = Path::new("assets/artwork");

    // Declared so a build with none of them set does not warn about an
    // unexpected cfg name.
    for (_, flag) in PIECES {
        println!("cargo::rustc-check-cfg=cfg({flag})");
    }

    println!("cargo::rerun-if-changed=assets/artwork");

    for (name, flag) in PIECES {
        let path = dir.join(name);
        println!("cargo::rerun-if-changed={}", path.display());

        if path.is_file() {
            println!("cargo::rustc-cfg={flag}");
            println!("cargo::warning=bundling assets/artwork/{name} into the launcher");
        }
    }
}
