//! Making a release: keys, archives, manifests and signatures.
//!
//! The private key never goes in the repository. `keygen` writes it to
//! `keys/`, which `.gitignore` excludes, and prints the public half as
//! the Rust entry to paste into `crates/recof-core/src/manifest/keys.rs`.
//! CI signs with the same key held as a repository secret.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};
use recof_core::hashes::sha256_bytes;
use recof_core::manifest::schema::{
    Archive, Channel, Manifest, ManifestFile, Product, SCHEMA_VERSION,
};
use recof_core::manifest::verify::base64_encode;
use recof_core::manifest::{
    SignatureFile, LAUNCHER_MANIFEST_NAME, MANIFEST_NAME, SIGNATURE_SUFFIX,
};
use recof_core::version::Version;

/// Where a private key is kept when no path is given.
const KEYS_DIR: &str = "keys";

/// The environment variable CI puts the key in.
const KEY_ENV: &str = "RECOF_LAUNCHER_SIGNING_KEY";

/// Name of the launcher executable inside its release archive.
const LAUNCHER_EXE: &str = "ReCoF-Launcher.exe";

/// Files a release must never contain, whatever is in the folder: Valve's
/// redistributables, which are not ours to ship.
///
/// The rebuilt `CoFLaunchApp.exe` is allowed. It carries the images and
/// sounds of the retail settings dialog, and shipping it is the project
/// owner's decision.
const NEVER_SHIP: &[&str] = &[
    "steam_api.dll",
    "steam_api_c.dll",
    "steamclient.dll",
    "tier0.dll",
    "tier0_s.dll",
    "vstdlib.dll",
    "vstdlib_s.dll",
    "mss32.dll",
    "vgui.dll",
    "vgui2.dll",
];

/// Generates a signing key pair.
pub fn keygen(args: &[String], root: &Path) -> Result<(), String> {
    let id = value_of(args, "--id").unwrap_or_else(|| "recof-2026b".to_owned());
    let out = root.join(value_of(args, "--out").unwrap_or_else(|| KEYS_DIR.to_owned()));

    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|error| format!("no randomness available: {error}"))?;
    let signing = SigningKey::from_bytes(&seed);
    let verifying = signing.verifying_key();

    std::fs::create_dir_all(&out).map_err(|error| format!("{}: {error}", out.display()))?;
    let private_path = out.join(format!("{id}.secret.key"));
    if private_path.exists() {
        return Err(format!(
            "{} already exists; delete it first if you really mean to replace the key",
            private_path.display()
        ));
    }

    // Base64 of the 32 byte seed, which is what the environment variable
    // and the CI secret hold too.
    std::fs::write(&private_path, base64_encode(&seed))
        .map_err(|error| format!("{}: {error}", private_path.display()))?;

    let public_path = out.join(format!("{id}.public.key"));
    std::fs::write(&public_path, base64_encode(&verifying.to_bytes()))
        .map_err(|error| format!("{}: {error}", public_path.display()))?;

    println!("private key: {}", private_path.display());
    println!("public key:  {}", public_path.display());
    println!();
    println!("The private key is ignored by git. Keep a copy somewhere safe:");
    println!("losing it means every future release needs a new launcher.");
    println!();
    println!("Paste this into crates/recof-core/src/manifest/keys.rs:");
    println!();
    println!("pub const TRUSTED_KEYS: &[TrustedKey] = &[TrustedKey {{");
    println!("    id: {id:?},");
    println!("    bytes: {:?},", verifying.to_bytes());
    println!("    products: &[Product::Recof, Product::RecofLauncher],");
    println!("}}];");
    println!();
    println!("And put the private key in the {KEY_ENV} repository secret:");
    println!();
    println!("  gh secret set {KEY_ENV} < {}", private_path.display());

    Ok(())
}

/// Packs the ReCoF game files.
pub fn pack_recof(args: &[String], root: &Path) -> Result<(), String> {
    let dir = root.join(require(args, "--dir")?);
    let version = parse_version(&require(args, "--version")?)?;
    let channel = parse_channel(args)?;
    let out = root.join(value_of(args, "--out").unwrap_or_else(|| "dist".to_owned()));
    let min_launcher =
        parse_version(&value_of(args, "--min-launcher").unwrap_or_else(|| "0.1.0".to_owned()))?;
    let buildid = match value_of(args, "--buildid") {
        Some(text) => text
            .parse::<u64>()
            .map_err(|_| format!("{text:?} is not a buildid"))?,
        None => recof_core::consts::EXPECTED_BUILDID,
    };

    let mut files = collect(&dir)?;
    if files.is_empty() {
        return Err(format!("{} holds nothing to pack", dir.display()));
    }
    refuse_what_must_not_ship(&files)?;
    let copies = fold_copies(&mut files);

    let archive_name = format!("recof-{version}.zip");
    let data = match value_of(args, "--cvars") {
        Some(name) => BTreeMap::from([("cvars".to_owned(), name)]),
        None => cvars_if_present(&files),
    };

    pack(
        &dir,
        &out,
        &archive_name,
        Product::Recof,
        version,
        channel,
        min_launcher,
        Some(buildid),
        files,
        copies,
        data,
        args,
        root,
    )
}

/// Packs the launcher itself.
///
/// The launcher ships as a zip holding one executable rather than a bare
/// exe, so downloading, verifying and extracting is the same code as for
/// a game release.
pub fn pack_launcher(args: &[String], root: &Path) -> Result<(), String> {
    let exe = root.join(require(args, "--exe")?);
    if !exe.is_file() {
        return Err(format!("{} is not a file", exe.display()));
    }
    let version = parse_version(&require(args, "--version")?)?;
    let channel = parse_channel(args)?;
    let out = root.join(value_of(args, "--out").unwrap_or_else(|| "dist".to_owned()));

    // A staging folder holding the one file, so the packer below is the
    // same for both products.
    let staging = out.join("launcher-staging");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|error| format!("{}: {error}", staging.display()))?;
    std::fs::copy(&exe, staging.join(LAUNCHER_EXE))
        .map_err(|error| format!("{}: {error}", exe.display()))?;

    let files = collect(&staging)?;
    let archive_name = format!("recof-launcher-{version}.zip");

    let outcome = pack(
        &staging,
        &out,
        &archive_name,
        Product::RecofLauncher,
        version,
        channel,
        // A launcher release never demands a newer launcher than itself.
        Version::new(0, 1, 0),
        None,
        files,
        Vec::new(),
        BTreeMap::new(),
        args,
        root,
    );

    let _ = std::fs::remove_dir_all(&staging);
    outcome
}

/// Builds the NSIS installer for the launcher.
///
/// The script is `installer/recof-launcher.nsi`; this finds `makensis` and
/// hands it the version and the paths. The installer is not what the
/// launcher updates itself from, which is the signed zip, so it carries no
/// manifest: its SHA-256 is printed for the release notes instead.
pub fn pack_installer(args: &[String], root: &Path) -> Result<(), String> {
    let exe = root.join(require(args, "--exe")?);
    if !exe.is_file() {
        return Err(format!("{} is not a file", exe.display()));
    }
    let version = parse_version(&require(args, "--version")?)?;
    let out = root.join(value_of(args, "--out").unwrap_or_else(|| "dist".to_owned()));
    std::fs::create_dir_all(&out).map_err(|error| format!("{}: {error}", out.display()))?;
    let makensis = find_makensis(value_of(args, "--makensis"))?;

    let installer = out.join(installer_name(&version));
    let script = root.join("installer").join("recof-launcher.nsi");
    let absolute = |path: &Path| {
        std::path::absolute(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    };

    let output = std::process::Command::new(&makensis)
        .arg("/V2")
        .arg("/INPUTCHARSET")
        .arg("UTF8")
        .arg(format!("/DVERSION={version}"))
        .arg(format!("/DVERSION_NUMERIC={}", numeric_version(&version)))
        .arg(format!("/DEXE={}", absolute(&exe)))
        .arg(format!("/DLICENSE={}", absolute(&root.join("LICENSE"))))
        .arg(format!("/DOUTFILE={}", absolute(&installer)))
        .arg(&script)
        .output()
        .map_err(|error| format!("{}: {error}", makensis.display()))?;
    if !output.status.success() {
        return Err(format!(
            "makensis failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let bytes =
        std::fs::read(&installer).map_err(|error| format!("{}: {error}", installer.display()))?;
    println!("{}", installer.display());
    println!();
    println!("version:  {version}");
    println!("size:     {} bytes", bytes.len());
    println!("sha256:   {}", sha256_bytes(&bytes));
    Ok(())
}

/// What the installer of a version is called.
pub fn installer_name(version: &Version) -> String {
    format!("ReCoF-Launcher-{version}-setup.exe")
}

/// Windows version resources take four numbers and nothing else, so a
/// prerelease such as `1.1.0-beta.1` is stamped `1.1.0.0`.
fn numeric_version(version: &Version) -> String {
    format!("{}.{}.{}.0", version.major, version.minor, version.patch)
}

/// `makensis`: the one asked for, then the one on the path, then where
/// the NSIS installer puts it.
fn find_makensis(asked: Option<String>) -> Result<std::path::PathBuf, String> {
    if let Some(path) = asked.or_else(|| std::env::var("MAKENSIS").ok()) {
        return Ok(std::path::PathBuf::from(path));
    }
    let on_path = std::process::Command::new("makensis")
        .arg("/VERSION")
        .output()
        .is_ok_and(|output| output.status.success());
    if on_path {
        return Ok(std::path::PathBuf::from("makensis"));
    }
    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(base) = std::env::var_os(variable) {
            let candidate = Path::new(&base).join("NSIS").join("makensis.exe");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(
        "makensis was not found. Install NSIS (winget install NSIS.NSIS), or pass \
         --makensis or set MAKENSIS to its path."
            .to_owned(),
    )
}

/// A file about to be packed.
struct Entry {
    relative: String,
    bytes: Vec<u8>,
}

/// Everything under a directory, as relative paths with forward slashes.
fn collect(dir: &Path) -> Result<Vec<Entry>, String> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(dir)
            .map_err(|error| format!("{}: {error}", entry.path().display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(entry.path())
            .map_err(|error| format!("{}: {error}", entry.path().display()))?;
        found.push(Entry { relative, bytes });
    }
    Ok(found)
}

/// Stops a release that carries something it has no right to.
fn refuse_what_must_not_ship(files: &[Entry]) -> Result<(), String> {
    for entry in files {
        let name = entry
            .relative
            .rsplit('/')
            .next()
            .unwrap_or(&entry.relative)
            .to_lowercase();
        if NEVER_SHIP.contains(&name.as_str()) {
            return Err(format!(
                "{} must never be in a release: it belongs to Valve. Remove it \
                 from the folder and pack again.",
                entry.relative
            ));
        }
    }
    Ok(())
}

/// Finds files that are byte identical to another one and records them as
/// copies, so the archive carries the content once.
///
/// Retail ships `sw.dll` as a copy of `hw.dll` and ReCoF keeps doing it,
/// which saves two megabytes in every download.
fn fold_copies(files: &mut Vec<Entry>) -> Vec<(String, String, u64, String)> {
    let mut copies = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();

    files.retain(|entry| {
        let digest = sha256_bytes(&entry.bytes);
        match seen.get(&digest) {
            Some(original) => {
                copies.push((
                    entry.relative.clone(),
                    original.clone(),
                    entry.bytes.len() as u64,
                    digest,
                ));
                false
            }
            None => {
                seen.insert(digest, entry.relative.clone());
                true
            }
        }
    });

    copies
}

/// Notes a cvar data file if the folder has one under its usual name.
fn cvars_if_present(files: &[Entry]) -> BTreeMap<String, String> {
    const NAME: &str = "recof-cvars.json";
    if files.iter().any(|entry| entry.relative == NAME) {
        BTreeMap::from([("cvars".to_owned(), NAME.to_owned())])
    } else {
        BTreeMap::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn pack(
    source: &Path,
    out: &Path,
    archive_name: &str,
    product: Product,
    version: Version,
    channel: Channel,
    min_launcher: Version,
    buildid: Option<u64>,
    files: Vec<Entry>,
    copies: Vec<(String, String, u64, String)>,
    data: BTreeMap<String, String>,
    args: &[String],
    root: &Path,
) -> Result<(), String> {
    let _ = source;
    std::fs::create_dir_all(out).map_err(|error| format!("{}: {error}", out.display()))?;

    // The archive.
    let archive_path = out.join(archive_name);
    {
        let file = std::fs::File::create(&archive_path)
            .map_err(|error| format!("{}: {error}", archive_path.display()))?;
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for entry in &files {
            writer
                .start_file(&entry.relative, options)
                .map_err(|error| format!("{}: {error}", entry.relative))?;
            writer
                .write_all(&entry.bytes)
                .map_err(|error| format!("{}: {error}", entry.relative))?;
        }
        writer
            .finish()
            .map_err(|error| format!("{}: {error}", archive_path.display()))?;
    }

    let archive_bytes = std::fs::read(&archive_path)
        .map_err(|error| format!("{}: {error}", archive_path.display()))?;

    // The manifest.
    let mut manifest_files: Vec<ManifestFile> = files
        .iter()
        .map(|entry| ManifestFile {
            path: entry.relative.clone(),
            size: entry.bytes.len() as u64,
            sha256: sha256_bytes(&entry.bytes),
            copy_of: None,
        })
        .collect();
    for (path, original, size, digest) in copies {
        manifest_files.push(ManifestFile {
            path,
            size,
            sha256: digest,
            copy_of: Some(original),
        });
    }

    let manifest = Manifest {
        schema: SCHEMA_VERSION,
        product,
        version: version.clone(),
        channel,
        released: value_of(args, "--released").unwrap_or_else(now_rfc3339),
        target_buildid: buildid,
        min_launcher,
        notes_url: value_of(args, "--notes-url"),
        archive: Archive {
            name: archive_name.to_owned(),
            size: archive_bytes.len() as u64,
            sha256: sha256_bytes(&archive_bytes),
        },
        files: manifest_files,
        data,
    };

    // Checked here rather than discovered by a user: a manifest this tool
    // writes must pass the same rules the launcher applies.
    recof_core::manifest::check_manifest(&manifest, product, &Version::new(9999, 0, 0))
        .map_err(|error| format!("the manifest this would publish is not valid: {error}"))?;

    let manifest_name = match product {
        Product::Recof => MANIFEST_NAME,
        Product::RecofLauncher => LAUNCHER_MANIFEST_NAME,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("the manifest could not be written: {error}"))?;
    let manifest_path = out.join(manifest_name);
    std::fs::write(&manifest_path, &manifest_bytes)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;

    // The signature, over exactly those bytes.
    let (key_id, signing) = load_key(args, root)?;
    let signature = SignatureFile {
        alg: "ed25519".to_owned(),
        key_id: key_id.clone(),
        sig: base64_encode(&signing.sign(&manifest_bytes).to_bytes()),
    };
    let signature_path = out.join(format!("{manifest_name}{SIGNATURE_SUFFIX}"));
    std::fs::write(
        &signature_path,
        serde_json::to_vec_pretty(&signature)
            .map_err(|error| format!("the signature could not be written: {error}"))?,
    )
    .map_err(|error| format!("{}: {error}", signature_path.display()))?;

    println!("{}", archive_path.display());
    println!("{}", manifest_path.display());
    println!("{}", signature_path.display());
    println!();
    println!("product:  {}", product.as_str());
    println!("version:  {version}");
    println!("channel:  {}", channel.as_str());
    println!("files:    {}", manifest.files.len());
    println!("archive:  {} bytes", manifest.archive.size);
    println!("sha256:   {}", manifest.archive.sha256);
    println!("signed:   {key_id}");

    Ok(())
}

/// Loads the signing key, from a path, the environment or `keys/`.
fn load_key(args: &[String], root: &Path) -> Result<(String, SigningKey), String> {
    let id = value_of(args, "--key-id").unwrap_or_else(|| "recof-2026b".to_owned());

    let encoded = if let Some(path) = value_of(args, "--key") {
        std::fs::read_to_string(root.join(&path)).map_err(|error| format!("{path}: {error}"))?
    } else if let Ok(from_env) = std::env::var(KEY_ENV) {
        from_env
    } else {
        let path = root.join(KEYS_DIR).join(format!("{id}.secret.key"));
        std::fs::read_to_string(&path).map_err(|_| {
            format!(
                "no signing key. Pass --key, set {KEY_ENV}, or run `cargo xtask keygen` \
                 to make one at {}",
                path.display()
            )
        })?
    };

    let seed =
        base64_decode(encoded.trim()).ok_or_else(|| "the signing key is not base64".to_owned())?;
    let seed: [u8; 32] = seed.try_into().map_err(|bytes: Vec<u8>| {
        format!("the signing key is {} bytes, expected 32", bytes.len())
    })?;

    Ok((id, SigningKey::from_bytes(&seed)))
}

/// Prints the retail hash table from a real install, for regenerating
/// `crates/recof-core/src/hashes/table.rs` after a game update.
pub fn hash_table(args: &[String], root: &Path) -> Result<(), String> {
    let from = root.join(require(args, "--from")?);
    if !from.is_dir() {
        return Err(format!("{} is not a folder", from.display()));
    }

    println!(
        "// Generated by `cargo xtask hash-table --from {}`",
        from.display()
    );
    for entry in recof_core::hashes::RETAIL_1_6A {
        let path = from.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        match std::fs::read(&path) {
            Ok(bytes) => {
                let digest = sha256_bytes(&bytes);
                let matches = if digest == entry.sha256 {
                    "same"
                } else {
                    "DIFFERENT"
                };
                println!(
                    "{:<32} size: {:<9} sha256: {}  ({matches})",
                    entry.path,
                    bytes.len(),
                    digest
                );
            }
            Err(error) => println!("{:<32} {error}", entry.path),
        }
    }
    Ok(())
}

fn parse_version(text: &str) -> Result<Version, String> {
    Version::parse(text).map_err(|error| error.to_string())
}

fn parse_channel(args: &[String]) -> Result<Channel, String> {
    match value_of(args, "--channel").as_deref() {
        None | Some("stable") => Ok(Channel::Stable),
        Some("beta") => Ok(Channel::Beta),
        Some(other) => Err(format!("{other:?} is not a channel; use stable or beta")),
    }
}

/// The value after a flag, for example `--version 1.2.3`.
fn value_of(args: &[String], flag: &str) -> Option<String> {
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1).cloned()
}

fn require(args: &[String], flag: &str) -> Result<String, String> {
    value_of(args, flag).ok_or_else(|| format!("{flag} is required"))
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.decode(text).ok()
}

/// A timestamp for the manifest, in UTC.
pub(crate) fn now_rfc3339() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);

    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    let year = if month <= 2 { year + 1 } else { year };

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join("recof-release-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn entry(relative: &str, bytes: &[u8]) -> Entry {
        Entry {
            relative: relative.to_owned(),
            bytes: bytes.to_vec(),
        }
    }

    #[test]
    fn flags_are_read_from_the_argument_list() {
        let args: Vec<String> = ["pack-recof", "--version", "1.2.3", "--channel", "beta"]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        assert_eq!(value_of(&args, "--version"), Some("1.2.3".to_owned()));
        assert_eq!(value_of(&args, "--missing"), None);
        assert_eq!(parse_channel(&args).unwrap(), Channel::Beta);
        assert!(require(&args, "--dir").is_err());
    }

    #[test]
    fn an_unknown_channel_is_refused() {
        let args = vec!["--channel".to_owned(), "nightly".to_owned()];
        assert!(parse_channel(&args).is_err());
    }

    #[test]
    fn the_installer_is_named_and_stamped_from_the_version() {
        let release = Version::parse("1.0.0").unwrap();
        assert_eq!(installer_name(&release), "ReCoF-Launcher-1.0.0-setup.exe");
        assert_eq!(numeric_version(&release), "1.0.0.0");

        let beta = Version::parse("1.1.0-beta.1").unwrap();
        assert_eq!(
            installer_name(&beta),
            "ReCoF-Launcher-1.1.0-beta.1-setup.exe"
        );
        assert_eq!(numeric_version(&beta), "1.1.0.0");
    }

    #[test]
    fn identical_files_are_folded_into_a_copy() {
        let mut files = vec![
            entry("hw.dll", b"the engine"),
            entry("sw.dll", b"the engine"),
            entry("cof.exe", b"the launcher"),
        ];
        let copies = fold_copies(&mut files);

        assert_eq!(files.len(), 2, "the archive carries the content once");
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].0, "sw.dll");
        assert_eq!(copies[0].1, "hw.dll");
        assert_eq!(copies[0].2, "the engine".len() as u64);
    }

    #[test]
    fn the_rebuilt_settings_dialog_may_ship() {
        let files = vec![
            entry("hw.dll", b"engine"),
            entry("CoFLaunchApp.exe", b"rebuilt settings dialog"),
        ];
        assert!(refuse_what_must_not_ship(&files).is_ok());
    }

    #[test]
    fn a_release_carrying_valves_redistributables_is_refused() {
        for name in ["steam_api.dll", "tier0.dll", "cryoffear/vgui2.dll"] {
            let files = vec![entry(name, b"not ours")];
            assert!(
                refuse_what_must_not_ship(&files).is_err(),
                "{name} should be refused"
            );
        }
    }

    #[test]
    fn an_ordinary_release_passes_the_check() {
        let files = vec![
            entry("hw.dll", b"engine"),
            entry("cryoffear/cl_dlls/client.dll", b"client"),
            entry("recof-cvars.json", b"{}"),
        ];
        assert!(refuse_what_must_not_ship(&files).is_ok());
        assert_eq!(
            cvars_if_present(&files).get("cvars").map(String::as_str),
            Some("recof-cvars.json")
        );
    }

    #[test]
    fn a_folder_with_no_cvar_file_notes_none() {
        let files = vec![entry("hw.dll", b"engine")];
        assert!(cvars_if_present(&files).is_empty());
    }

    #[test]
    fn collecting_walks_a_tree_into_relative_paths() {
        let base = scratch("collect");
        std::fs::create_dir_all(base.join("cryoffear/cl_dlls")).unwrap();
        std::fs::write(base.join("hw.dll"), b"engine").unwrap();
        std::fs::write(base.join("cryoffear/cl_dlls/hl.dll"), b"server").unwrap();

        let found = collect(&base).unwrap();
        let names: Vec<&str> = found.iter().map(|entry| entry.relative.as_str()).collect();
        assert_eq!(names, ["cryoffear/cl_dlls/hl.dll", "hw.dll"]);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_timestamp_looks_like_rfc_3339() {
        let now = now_rfc3339();
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.ends_with('Z'));
        assert!(now.as_str() > "2024-01-01T00:00:00Z", "{now}");
    }

    /// The whole thing: generate a key, pack a release, and check that the
    /// launcher's own verification accepts it.
    #[test]
    fn a_packed_release_verifies_with_the_key_that_signed_it() {
        let base = scratch("round-trip");
        let files = base.join("files");
        std::fs::create_dir_all(files.join("cryoffear/cl_dlls")).unwrap();
        std::fs::write(files.join("hw.dll"), b"the rebuilt engine").unwrap();
        std::fs::write(files.join("sw.dll"), b"the rebuilt engine").unwrap();
        std::fs::write(
            files.join("cryoffear/cl_dlls/hl.dll"),
            b"the rebuilt server",
        )
        .unwrap();

        let keygen_args: Vec<String> = ["--id", "test-key", "--out", "keys"]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        keygen(&keygen_args, &base).expect("a key is generated");

        let pack_args: Vec<String> = [
            "--dir",
            "files",
            "--version",
            "0.3.0",
            "--out",
            "dist",
            "--key-id",
            "test-key",
            "--key",
            "keys/test-key.secret.key",
        ]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();
        pack_recof(&pack_args, &base).expect("the release packs");

        let dist = base.join("dist");
        let manifest_bytes = std::fs::read(dist.join(MANIFEST_NAME)).unwrap();
        let signature_bytes =
            std::fs::read(dist.join(format!("{MANIFEST_NAME}{SIGNATURE_SUFFIX}"))).unwrap();

        // The public half, as keygen printed it for keys.rs.
        let public = std::fs::read_to_string(base.join("keys/test-key.public.key")).unwrap();
        let public: [u8; 32] = base64_decode(public.trim()).unwrap().try_into().unwrap();
        let trusted = recof_core::manifest::TrustedKey {
            id: "test-key",
            bytes: public,
            products: &[Product::Recof, Product::RecofLauncher],
        };

        let verified = recof_core::manifest::verify::verify_manifest_with_keys(
            &manifest_bytes,
            &signature_bytes,
            Product::Recof,
            &Version::new(9999, 0, 0),
            &[trusted],
        )
        .expect("the launcher accepts what the tool signed");

        assert_eq!(verified.version, Version::parse("0.3.0").unwrap());
        assert_eq!(verified.files.len(), 3);
        // sw.dll is a copy, so the archive holds hw.dll once.
        let sw = verified.file("sw.dll").unwrap();
        assert_eq!(sw.copy_of.as_deref(), Some("hw.dll"));

        // And the archive hash the manifest claims is the real one.
        let archive = dist.join("recof-0.3.0.zip");
        recof_core::manifest::verify_archive(&verified, &archive)
            .unwrap()
            .expect("the archive matches its manifest");

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn keygen_refuses_to_overwrite_an_existing_key() {
        let base = scratch("no-overwrite");
        let args: Vec<String> = ["--id", "test-key"]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        keygen(&args, &base).unwrap();

        let second = keygen(&args, &base);
        assert!(second.is_err(), "a key must not be silently replaced");
        assert!(second.unwrap_err().contains("already exists"));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn packing_without_a_key_says_how_to_make_one() {
        let base = scratch("no-key");
        std::fs::create_dir_all(base.join("files")).unwrap();
        std::fs::write(base.join("files/hw.dll"), b"engine").unwrap();

        let args: Vec<String> = ["--dir", "files", "--version", "0.3.0", "--out", "dist"]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();

        // The environment variable would make this pass on a machine that
        // happens to have it set, so it is cleared for the test.
        let saved = std::env::var(KEY_ENV).ok();
        unsafe { std::env::remove_var(KEY_ENV) };

        let outcome = pack_recof(&args, &base);
        if let Some(value) = saved {
            unsafe { std::env::set_var(KEY_ENV, value) };
        }

        let message = outcome.unwrap_err();
        assert!(message.contains("keygen"), "{message}");

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn packing_an_empty_folder_is_refused() {
        let base = scratch("empty");
        std::fs::create_dir_all(base.join("files")).unwrap();
        let args: Vec<String> = ["--dir", "files", "--version", "0.3.0"]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        assert!(pack_recof(&args, &base).is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }
}
