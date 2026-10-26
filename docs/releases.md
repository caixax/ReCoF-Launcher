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

- refuses to pack `CoFLaunchApp.exe`, which embeds art and sound taken out of the retail executable, or any of Valve's redistributables;
- notices files that are byte identical, records the second one as a copy of the first, and leaves it out of the archive, which is how `sw.dll` rides along with `hw.dll` for free;
- picks up `recof-cvars.json` if the folder has one, so the launcher can build the ReCoF options screen from it;
- checks the manifest it just wrote against the same rules the launcher applies, so a release that would be refused is caught here instead.

Upload all three files from `dist/` to a GitHub release on `caixax/ReCoF`, with the tag `v0.3.0`. The release body becomes the changelog the launcher shows before updating.

For a beta, pass `--channel beta`, tag it `v0.4.0-beta.1`, and mark the GitHub release as a prerelease. The stable channel reads `releases/latest`, which GitHub already filters to skip prereleases, so a beta is invisible to it.

### `--min-launcher`

Set this when a release needs something the current launcher cannot do, for example a new manifest field. A launcher older than this refuses to install it and tells the user to update the launcher first. Leave it at the oldest launcher that works, so updating ReCoF does not force a launcher update for no reason.

## Releasing the launcher

Pushing a tag does it:

```sh
git tag v0.2.0
git push origin v0.2.0
```

`.github/workflows/release.yml` builds, tests, packs, signs with the repository secret and publishes. To do it by hand:

```sh
cargo build --release --target x86_64-pc-windows-msvc
cargo xtask pack-launcher \
    --exe target/x86_64-pc-windows-msvc/release/ReCoF-Launcher.exe \
    --version 0.2.0 \
    --out dist
```

## Testing a release before publishing it

The launcher installs from a local file, through exactly the same verification: Maintenance, then "Install from a file". Put the archive, `recof-manifest.json` and `recof-manifest.json.sig` in one folder and pick the archive. A local file is not trusted more than a downloaded one, so this tests the real path.

This is also how testers work while the repository is private: a private repository's releases are only visible to people with read access, so the launcher cannot fetch them.

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
- [ ] All three files are attached to the GitHub release.
- [ ] The launcher on a clean machine finds the update, shows the changelog, and installs it.
