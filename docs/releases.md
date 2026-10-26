# Making a release

Two things get released from two repositories: ReCoF itself, from `caixax/ReCoF`, and this launcher, from `caixax/ReCoF-Launcher`. Both use the same format and the same signing key, and the launcher verifies both the same way.

A release is always three files:

| File | What it is |
|---|---|
| `recof-<version>.zip` | the files to install |
| `recof-manifest.json` | what is in the archive, and what it hashes to |
| `recof-manifest.json.sig` | an Ed25519 signature over those exact bytes |

The launcher's own release uses `recof-launcher-<version>.zip` and `recof-launcher-manifest.json`, and the archive holds one file, the executable. It is a zip rather than a bare executable so that downloading, verifying and extracting is the same code for both.

## The signing key

The launcher trusts a list of public keys compiled into it, in [`crates/recof-core/src/manifest/keys.rs`](../crates/recof-core/src/manifest/keys.rs). Nothing is installed unless the manifest is signed by one of them, so a release signed with any other key is refused, and so is an unsigned one.

To create the key:

```sh
cargo xtask keygen --id recof-2026b
```

That writes `keys/recof-2026b.secret.key` and `keys/recof-2026b.public.key` and prints the entry to paste into `keys.rs`. The `keys/` folder is in `.gitignore` and the private key must never be committed.

Keep a copy of the private key somewhere safe and offline. Losing it does not break anything already installed, but every future release would need a new key, which means shipping a new launcher before anyone can update.

Give CI the same key:

```sh
gh secret set RECOF_LAUNCHER_SIGNING_KEY < keys/recof-2026b.secret.key
```

### Rotating it

Generate the new key, add it to `TRUSTED_KEYS` **alongside** the old one, and ship that launcher. Once enough people are on it, start signing with the new key. Drop the old entry a few versions later. A launcher that only trusts the new key cannot verify anything signed with the old one, so the overlap is what keeps updates working.

## Releasing ReCoF

Build ReCoF and collect what the release installs into one folder, laid out exactly as it goes into the game: `hw.dll` at the top, `cryoffear/cl_dlls/client.dll` under it, and so on.

Then:

```sh
cargo xtask pack-recof \
    --dir  path/to/built/files \
    --version 0.3.0 \
    --channel stable \
    --min-launcher 0.1.0 \
    --buildid 5821301 \
    --notes-url https://github.com/caixax/ReCoF/releases/tag/v0.3.0 \
    --out dist
```

The tool:

- refuses to pack any of Valve's redistributables;
- notices files that are byte identical, records the second one as a copy of the first, and leaves it out of the archive, which is how `sw.dll` rides along with `hw.dll` for free;
- picks up `recof-cvars.json` if the folder has one, so the launcher can build the ReCoF options screen from it;
- checks the manifest it just wrote against the same rules the launcher applies, so a release that would be refused is caught here instead.

Upload all three files from `dist/` to a GitHub release on `caixax/ReCoF`, with the tag `v0.3.0`. Other files may sit next to them, such as a zip for installing by hand: the launcher downloads the archive the manifest names and nothing else.

The release body becomes the notes the launcher shows before a first install and before an update. It is drawn the way GitHub draws it: headings, lists, bold, inline code, links, tables, fenced code and alert blocks such as `> [!WARNING]`. Anything else shows as plain text. Warnings a player needs before installing, such as tools that do not work with this build, belong there.

For a beta, pass `--channel beta` and mark the GitHub release as a prerelease. The tag needs no `-beta` suffix (`v0.4.0-beta.1` and `v0.1.0` both work): the prerelease mark is what keeps a release off the stable channel, which reads `releases/latest`, and GitHub leaves prereleases out of that.

### `--min-launcher`

Set this when a release needs something the current launcher cannot do, for example a new manifest field. A launcher older than this refuses to install it and tells the user to update the launcher first. Leave it at the oldest launcher that works, so updating ReCoF does not force a launcher update for no reason.

### The options file

`recof-cvars.json` describes what the launcher's ReCoF options screen shows. The launcher has no list of its own, so a release that adds an option works with a launcher that predates it.

```json
{
  "schema": 1,
  "cvars": [
    { "name": "rc_rawinput", "type": "bool", "default": "1", "group": "Input",
      "label_en": "Raw mouse input", "desc_en": "Reads the mouse directly." },
    { "name": "rc_prof", "type": "int", "default": "0", "min": 0, "max": 2,
      "group": "Performance", "label_en": "Frame profiler" },
    { "name": "rc_speedometer_pos", "type": "text", "default": ".5 .9",
      "group": "HUD", "label_en": "Speedometer position" },
    { "name": "condebug", "type": "flag", "default": "0", "group": "Testing",
      "label_en": "Console log", "desc_en": "Writes the console to cryoffear/qconsole.log." }
  ]
}
```

- `type` is `bool` (passed as `+name 1` or `+name 0`), `int` or `float` (with optional `min` and `max`), `text`, or `flag`. A flag is a launch option rather than a console variable: it is passed as `-name` when on and left out when off, so its `name` has no dash.
- A value may hold ASCII letters, digits, spaces, dots and underscores, and nothing else. The engine splits its command line at every `+` and `-`, drops bytes outside printable ASCII, and its console parser ends a token at other punctuation, so a value with any of those would not arrive whole. That rules out negative numbers. Spaces are fine: the value is passed quoted, and the engine keeps the quotes for its console.
- A value the user chose in the launcher is always passed, even at its default: the game saves many options in `config.cfg`, and leaving one out would keep whatever the console set last. An option the user never touched is not passed, and the launcher shows the value from the copy's `config.cfg` for it when there is one. A flag is passed whenever it is on, including when it is on by default.
- The engine runs the command line (`stuffcmds` at the end of `valve.rc`) before it runs `config.cfg`, so a saved option would undo the value it was started with. Before starting the game the launcher therefore also writes each chosen value into the line of `cryoffear/config.cfg` that already names it, leaving every other byte as it was. An option the file does not name yet only needs the command line. Nothing in the options file says which options are saved: whatever `config.cfg` holds is what gets written.
- `key` is optional: a Fluent message id the launcher uses for the label, and its `desc` attribute for the description, when it has that translation. Otherwise `label_en` and `desc_en` are shown. Commands meant for binds, such as `+rc_ducktap`, do not belong in the file.
- The whole command line, the path of `cof.exe` included, has to stay within 1000 bytes. The engine copies it into a 1024 byte buffer without checking the length, so the launcher refuses to start the game past that and says why.

## Releasing the launcher

Pushing a tag does it:

```sh
git tag v0.2.0
git push origin v0.2.0
```

`.github/workflows/release.yml` builds, tests, packs, signs with the repository secret, builds the installer and publishes five files:

| File | For |
|---|---|
| `ReCoF-Launcher-<version>-setup.exe` | Players who want it installed: per user, no administrator rights, Start menu entry, listed in installed apps |
| `ReCoF-Launcher.exe` | Players who want it portable |
| `recof-launcher-<version>.zip`, `recof-launcher-manifest.json`, `.sig` | The launcher itself, when it updates: the signed zip it downloads and checks |

The installer goes to `%LOCALAPPDATA%\Programs` precisely so that the self update, which swaps the running executable for the verified one from the zip, works there without elevation. Its uninstall entry has no version on purpose: after the first self update it would be wrong.

The version in `Cargo.toml` has to match the tag. A tag with a prerelease part, such as `v1.1.0-beta.1`, is signed for the beta channel and published as a prerelease; any other tag is stable.

To do it by hand (the installer needs NSIS: `winget install NSIS.NSIS`):

```sh
cargo build --release --target x86_64-pc-windows-msvc
cargo xtask pack-launcher \
    --exe target/x86_64-pc-windows-msvc/release/ReCoF-Launcher.exe \
    --version 1.0.0 \
    --out dist
cargo xtask pack-installer \
    --exe target/x86_64-pc-windows-msvc/release/ReCoF-Launcher.exe \
    --version 1.0.0 \
    --out dist
```

The installer script is [`installer/recof-launcher.nsi`](../installer/recof-launcher.nsi). It speaks the launcher's eleven languages; its own few sentences are first drafts outside English and Spanish, like the launcher's.

## Testing a release before publishing it

The launcher installs from a local file, through exactly the same verification: "Install from a file" on the install screen for a first install, or Maintenance, then "Install from a file" for a copy that exists. Put the archive, `recof-manifest.json` and `recof-manifest.json.sig` in one folder and pick the archive. A local file is not trusted more than a downloaded one, so this tests the real path.

This is also how testers work while the repository is private: a private repository's releases are only visible to people with read access, so the launcher cannot fetch them.

Once a release is published, run the whole path against it with the real game on the machine:

```sh
cargo xtask smoke --dest "D:/recof-smoke/Cry of Fear ReCoF"
```

It finds the game, asks GitHub for the newest release on the beta channel (`--channel stable` for the other one), reads its notes, fetches and verifies the launcher's own newest release down to the executable a self update would swap in, downloads and verifies the ReCoF release, copies the game and applies the release, verifies the copy, damages two files and repairs them, builds the command line with every option set, and uninstalls. It lists and hashes the Steam install before and after, and fails if anything in it changed. `--keep` leaves the copy in place to play. It is the launcher's own code without the window, so a failure here is a failure a player would hit.

## Checking a real install

To see the hashes of a retail installation, for example after a Steam update changes the build:

```sh
cargo xtask hash-table --from "C:/Program Files (x86)/Steam/steamapps/common/Cry of Fear"
```

It prints each in scope file with its size and SHA-256 and says whether it still matches the table in [`crates/recof-core/src/hashes/table.rs`](../crates/recof-core/src/hashes/table.rs).

`cargo xtask detect` runs the whole detection chain and prints what the launcher would see, which is the first thing to ask for in a bug report.

## Checklist

- [ ] The version in `Cargo.toml` matches the tag.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo xtask check-dashes` is clean.
- [ ] The release notes say what changed, in plain language.
- [ ] `--min-launcher` is right.
- [ ] All three files are attached to the GitHub release (five for a launcher release).
- [ ] For a launcher release, the installer installs and uninstalls cleanly: `ReCoF-Launcher-<version>-setup.exe /S`, then `Uninstall.exe /S`.
- [ ] `cargo xtask smoke --dest <folder>` passes against the published release.
- [ ] The launcher on a clean machine finds the update, shows the changelog, and installs it.
