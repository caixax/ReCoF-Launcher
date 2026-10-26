//! Release side tooling for ReCoF Launcher.
//!
//! Run through the cargo alias defined in `.cargo/config.toml`:
//!
//! ```text
//! cargo xtask check-dashes
//! ```

use std::process::ExitCode;

mod dashes;

const USAGE: &str = "\
ReCoF Launcher release tooling

Usage:
  cargo xtask <command> [options]

Commands:
  check-dashes          Fail if an em dash or an en dash appears in the tree
  help                  Show this message
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");

    let result = match command {
        "check-dashes" => dashes::run(repo_root()),
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
