# ReCoF Launcher

Installer, updater and launcher for [ReCoF](https://github.com/caixax/ReCoF), the improved recompilation of Cry of Fear.

It finds your Steam copy of Cry of Fear, copies it to a folder you choose, applies ReCoF on top of the copy, keeps it updated from GitHub Releases with signature and hash checks, backs up your saves, and starts either build.

**Your Steam installation is never modified.** The launcher reads and hashes it, and nothing more. ReCoF always lives in a separate copy.

This is a standalone MIT licensed project. It is not a replacement for `CoFLaunchApp.exe` and has no relationship with it.

## What it does

- Finds Cry of Fear through the Steam registry key, `libraryfolders.vdf` and `appmanifest_223710.acf`, or through a folder you pick by hand.
- Checks the install against the known SHA-256 hashes of retail 1.6a and reports anything unexpected, without blocking you.
- Works out the real size of the copy and checks it against the free space on both the destination volume and the volume holding `%LOCALAPPDATA%`.
- Copies the game with a cancelable progress bar, then applies ReCoF through a recoverable journal, so an interrupted install is never left half done.
- Updates ReCoF and itself from GitHub Releases, verifying an Ed25519 signature over the manifest, the SHA-256 of the archive against both the manifest and the GitHub asset digest, and every extracted file.
- Keeps the last few versions so you can roll back.
- Backs up your saves on demand and automatically before every update.
- Imports your progress from the vanilla install, one way only.
- Verifies and repairs, moves the installation, and uninstalls cleanly.
- Starts ReCoF directly, or vanilla Cry of Fear through Steam.

## Rules it follows

1. The Steam installation is read only. The launcher never writes, deletes or renames anything inside it.
2. The launcher writes in exactly two places: the ReCoF copy folder you chose, and `%LOCALAPPDATA%\ReCoF Launcher`. The one documented exception is `shortcuts.vdf`, for adding ReCoF to Steam as a non Steam game, which is opt in, requires Steam to be closed, and backs the file up first.
3. No retail game file is ever distributed or downloaded. The base game always comes from your own copy, and a release carries only ReCoF's rebuilt binaries (the rebuilt `CoFLaunchApp.exe` includes the images and sounds of the retail settings dialog).
4. Every download is verified by signature and hash before it is used. If any check fails, nothing is installed.
5. No operation may leave a half finished installation.
6. ReCoF is never installed inside the Steam folder or in `Program Files`.

## Requirements

- Windows 10 or 11, 64 bit.
- Cry of Fear on Steam, build 5821301 (retail 1.6a).
- Free disk space for a full copy of the game, about 4.4 GiB, plus room for the version store and backups. The launcher measures this and tells you before it starts.

No Visual C++ redistributable and no .NET runtime are needed: the launcher is a single executable of about 7 MB, built against the static CRT. See [docs/binary-size.md](docs/binary-size.md) for what that is made of and what was measured to get there.

## Installing

Every [release](https://github.com/caixax/ReCoF-Launcher/releases) has the same launcher in two forms:

- **`ReCoF-Launcher-<version>-setup.exe`** installs it for your Windows user in `%LOCALAPPDATA%\Programs\ReCoF Launcher`, with a Start menu entry, an optional desktop shortcut, and an entry in the list of installed apps. It needs no administrator rights, which is also what lets the launcher update itself in that folder.
- **`ReCoF-Launcher.exe`** is the portable launcher: one file, nothing to install. Put it in any folder you can write to.

Both keep their settings, save backups and downloaded releases in `%LOCALAPPDATA%\ReCoF Launcher`, and both update themselves from GitHub Releases. Uninstalling the launcher leaves a ReCoF copy where it is, so remove that first from Maintenance, Uninstall if you want it gone too. The uninstaller removes the launcher's settings and downloads only when asked, and never the save backups.

## Building

```sh
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

The executable lands in `target/x86_64-pc-windows-msvc/release/ReCoF-Launcher.exe`.

Useful commands:

```sh
cargo test --workspace          # unit and integration tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo xtask check-dashes        # repository rule, see below
cargo xtask detect --measure    # what the launcher sees on this machine
cargo xtask help                # every command
```

`cargo xtask detect` is the first thing to run when something is not found: it prints the Steam path, every library, the app manifest, the hash of each in scope file and the space check, which is also what the bug report collects.

## Working on this repository

After cloning, enable the versioned hooks:

```sh
git config core.hooksPath .githooks
```

The `commit-msg` hook strips attribution trailers and rejects commit messages containing an em dash or an en dash.

**No em dashes (U+2014) and no en dashes (U+2013) anywhere in this repository**, including code, comments, documentation, translation files and UI strings. Use a comma, a colon, parentheses or a plain hyphen. This is enforced by `cargo xtask check-dashes`, by the `no_dashes` test, by the `commit-msg` hook, and by CI.

## Layout

| Path | Contents |
|---|---|
| `crates/recof-core` | Install, verification, journal, update and backup logic. No UI. |
| `crates/recof-platform` | The only crate allowed to touch platform APIs. Windows backend plus Linux stubs. |
| `crates/recof-launcher` | The egui application and the embedded translations. |
| `xtask` | Release tooling: key generation, packing, signing, repository checks. |
| `docs/` | How releases are made, what the binary size is made of, and ideas not implemented. |

## Translating

Translations live in `crates/recof-launcher/i18n/<locale>.ftl`, one [Fluent](https://projectfluent.org/) file per language, embedded in the binary. English is the source language. Languages other than English and Spanish are machine assisted first drafts and are marked in the file as pending review by a native speaker. Corrections are very welcome.

## Making a release

Documented in [docs/releases.md](docs/releases.md). In short: generate the signing key once with `cargo xtask keygen`, keep the private half out of the repository, and let `cargo xtask pack-recof` or `pack-launcher` build the archive, the manifest and the signature. Pushing a `v*` tag does the launcher's own release through CI.

## Out of scope for now

- LiveSplit, autosplitters and speedrun tooling.
- Native Linux and Steam Deck. The `recof-platform` crate keeps the door open.
- Saving space with hardlinks or shared assets. See [docs/ideas.md](docs/ideas.md).
- Installing ReCoF inside the Steam folder.

## Credits

The launcher draws its own look, so it runs with no artwork at all. The background it ships with is by Greez (https://www.steamgriddb.com/hero/2549), used with permission. See [CREDITS.md](CREDITS.md) for that permission, for the font, and for what to do if you add artwork of your own.

## License

MIT, see [LICENSE](LICENSE). ReCoF itself is a separate project under its own terms; this launcher downloads nothing except what that project publishes.
