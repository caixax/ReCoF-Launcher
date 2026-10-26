//! Release side tooling for ReCoF Launcher.
//!
//! Run through the cargo alias defined in `.cargo/config.toml`:
//!
//! ```text
//! cargo xtask check-dashes
//! ```

use std::process::ExitCode;

mod dashes;
mod detect;
mod release;
mod smoke;

const USAGE: &str = "\
ReCoF Launcher release tooling

Usage:
  cargo xtask <command> [options]

Commands:
  check-dashes          Fail if an em dash or an en dash appears in the tree
  detect [--measure]    Print what the launcher sees on this machine
  keygen --id <id>      Make a signing key pair in keys/
  pack-recof ...        Pack, sign and check a ReCoF release (docs/releases.md)
  pack-launcher ...     Pack, sign and check a launcher release
  hash-table --from <dir>
                        Hash the in scope files of a real install
  smoke --dest <dir> [--channel beta|stable] [--keep]
                        Install the newest published release into <dir> from
                        the real game, verify, repair and uninstall it, and
                        check the Steam install did not change
  help                  Show this message
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");

    let result = match command {
        "check-dashes" => dashes::run(repo_root()),
        "detect" => detect::run(args.iter().any(|flag| flag == "--measure")),
        "keygen" => release::keygen(&args, &repo_root()),
        "pack-recof" => release::pack_recof(&args, &repo_root()),
        "pack-launcher" => release::pack_launcher(&args, &repo_root()),
        "hash-table" => release::hash_table(&args, &repo_root()),
        "smoke" => smoke::run(&args),
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => {
            eprintln!("xtask: unknown command {other:?}\n");
            print!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root, derived from this crate's manifest directory.
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}
