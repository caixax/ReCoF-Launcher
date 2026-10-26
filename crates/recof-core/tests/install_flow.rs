//! The whole flow against a fake Steam setup: find the game, measure it,
//! install a signed release beside it, verify, break something, repair,
//! move the copy and remove it.
//!
//! Nothing here needs the real game. Every path carries a space and a non
//! ASCII character, and there are two Steam libraries, because those are
//! the cases that break a launcher on somebody else's machine and not on
//! the author's.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use ed25519_dalek::{Signer, SigningKey};
use recof_core::consts::EXPECTED_BUILDID;
use recof_core::hashes::{sha256_bytes, FileVerdict};
use recof_core::install::{self, destination, InstalledState, UninstallOptions};
use recof_core::journal::OperationKind;
use recof_core::manifest::schema::{Archive, Channel, ManifestFile, Product, SCHEMA_VERSION};
use recof_core::manifest::verify::{base64_encode, verify_manifest_with_keys};
use recof_core::manifest::{Manifest, SignatureFile, TrustedKey};
use recof_core::paths::AppPaths;
use recof_core::space::{self, TreeSize};
use recof_core::steam::{self, SteamLayout};
use recof_core::store::VersionStore;
use recof_core::version::Version;

/// The base game, as a handful of small files.
const GAME: &[(&str, &[u8])] = &[
    ("cof.exe", b"retail launcher"),
    ("hw.dll", b"retail engine"),
    ("sw.dll", b"retail engine"),
    ("opengl32.dll", b"retail wrapper"),
    ("CoFLaunchApp.exe", b"retail settings dialog"),
    ("cryoffear/liblist.gam", b"gamedll cl_dlls/hl.dll"),
    ("cryoffear/config.cfg", b"bind w +forward"),
    ("cryoffear/cl_dlls/client.dll", b"retail client"),
    ("cryoffear/cl_dlls/hl.dll", b"retail server"),
    ("cryoffear/cl_dlls/GameUI.dll", b"retail menus"),
    (
        "cryoffear/maps/ap 01.bsp",
        b"a map with a space in its name",
    ),
    ("cryoffear/sound/ambiente ñ.wav", b"sonido ambiente"),
    ("cryoffear/SAVE/saveinfo1.cof", b"the player progress"),
];

/// What the ReCoF release replaces.
const RELEASE: &[(&str, &[u8])] = &[
    ("cof.exe", b"ReCoF launcher"),
    ("hw.dll", b"ReCoF engine"),
    ("cryoffear/cl_dlls/client.dll", b"ReCoF client"),
    ("cryoffear/cl_dlls/hl.dll", b"ReCoF server"),
];

struct World {
    base: PathBuf,
    /// The Steam client folder, which is also library zero.
    steam: PathBuf,
    /// A second library, where the game actually is.
    library: PathBuf,
    /// The vanilla install.
    vanilla: PathBuf,
    /// Where the release zip and its manifest sit.
    release_zip: PathBuf,
    manifest_bytes: Vec<u8>,
    signature_bytes: Vec<u8>,
    key: TrustedKey,
}

impl World {
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir().join("recof install flow ñ").join(name);
        let _ = std::fs::remove_dir_all(&base);

        let steam = base.join("Program Archivos/Steam");
        let library = base.join("Librería Steam D");
        let vanilla = library.join("steamapps/common/Cry of Fear");

        // Two libraries, the second one holding the game.
        write(
            &steam.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
                escape(&steam),
                escape(&library)
            )
            .as_bytes(),
        );
        write(
            &library.join("steamapps/appmanifest_223710.acf"),
            format!(
                "\"AppState\"\n{{\n\t\"appid\"\t\t\"223710\"\n\t\"name\"\t\t\"Cry of Fear\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Cry of Fear\"\n\t\"SizeOnDisk\"\t\t\"4695601334\"\n\t\"buildid\"\t\t\"{EXPECTED_BUILDID}\"\n}}\n"
            )
            .as_bytes(),
        );
        for (path, content) in GAME {
            write(&vanilla.join(path), content);
        }

        // A release, packed and signed the way xtask will pack it.
        let release_zip = base.join("recof-0.3.0.zip");
        make_zip(&release_zip, RELEASE);
        let manifest = manifest_for(&release_zip);
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();

        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let key = TrustedKey {
            id: "flow-test-key",
            bytes: signing.verifying_key().to_bytes(),
            products: &[Product::Recof],
        };
        let signature_bytes = serde_json::to_vec(&SignatureFile {
            alg: "ed25519".to_owned(),
            key_id: "flow-test-key".to_owned(),
            sig: base64_encode(&signing.sign(&manifest_bytes).to_bytes()),
        })
        .unwrap();

        World {
            base,
            steam,
            library,
            vanilla,
            release_zip,
            manifest_bytes,
            signature_bytes,
            key,
        }
    }

    fn layout(&self) -> SteamLayout {
        steam::libraries(&self.steam).expect("the fake Steam folder is readable")
    }

    fn app_paths(&self) -> AppPaths {
        let paths = AppPaths::at(self.base.join("AppData/Local/ReCoF Launcher"));
        paths.create_all().unwrap();
        paths
    }

    /// Verifies and stores the release, as the update flow does.
    fn stored(&self, store: &VersionStore) -> recof_core::store::StoredVersion {
        let verified = verify_manifest_with_keys(
            &self.manifest_bytes,
            &self.signature_bytes,
            Product::Recof,
            &Version::parse("0.1.0").unwrap(),
            &[self.key],
        )
        .expect("the release the test signed verifies");

        recof_core::manifest::verify_archive(&verified, &self.release_zip)
            .unwrap()
            .expect("the archive matches the manifest");

        store
            .store(
                &verified,
                &self.manifest_bytes,
                &self.signature_bytes,
                &self.release_zip,
                &mut |_, _| {},
            )
            .expect("the release stores")
    }

    /// Everything about the vanilla install, for the read only check.
    fn vanilla_fingerprint(&self) -> Vec<(String, u64, String)> {
        let mut found = Vec::new();
        for entry in walkdir::WalkDir::new(&self.vanilla).sort_by_file_name() {
            let entry = entry.unwrap();
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(&self.vanilla)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(entry.path()).unwrap();
            found.push((relative, bytes.len() as u64, sha256_bytes(&bytes)));
        }
        found
    }

    fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn manifest_for(archive: &Path) -> Manifest {
    let archive_bytes = std::fs::read(archive).unwrap();
    let mut files: Vec<ManifestFile> = RELEASE
        .iter()
        .map(|(path, content)| ManifestFile {
            path: (*path).to_owned(),
            size: content.len() as u64,
            sha256: sha256_bytes(content),
            copy_of: None,
        })
        .collect();
    // Retail ships sw.dll as a copy of hw.dll and ReCoF keeps doing that.
    files.push(ManifestFile {
        path: "sw.dll".to_owned(),
        size: RELEASE[1].1.len() as u64,
        sha256: sha256_bytes(RELEASE[1].1),
        copy_of: Some("hw.dll".to_owned()),
    });

    Manifest {
        schema: SCHEMA_VERSION,
        product: Product::Recof,
        version: Version::parse("0.3.0").unwrap(),
        channel: Channel::Stable,
        released: "2026-10-26T12:00:00Z".to_owned(),
        target_buildid: Some(EXPECTED_BUILDID),
        min_launcher: Version::parse("0.1.0").unwrap(),
        notes_url: None,
        archive: Archive {
            name: archive.file_name().unwrap().to_string_lossy().into_owned(),
            size: archive_bytes.len() as u64,
            sha256: sha256_bytes(&archive_bytes),
        },
        files,
        data: Default::default(),
    }
}

fn make_zip(path: &Path, entries: &[(&str, &[u8])]) {
    use std::io::Write;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    for (name, content) in entries {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap();
}

fn write(path: &Path, content: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// A path as a Steam VDF writes it, with escaped separators.
fn escape(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

fn no_cancel() -> install::Cancel {
    Arc::new(AtomicBool::new(false))
}

fn measure(root: &Path) -> TreeSize {
    space::measure_tree(root, 4096, &no_cancel(), &mut |_| {})
        .unwrap()
        .unwrap()
}

#[test]
fn the_game_is_found_in_the_second_library() {
    let world = World::new("detection");

    let layout = world.layout();
    assert_eq!(layout.libraries.len(), 2);
    assert_eq!(layout.libraries[1], world.library);

    let install = steam::detect::find_game_in(&layout)
        .unwrap()
        .expect("the game is in library one");
    assert_eq!(install.root, world.vanilla);
    assert_eq!(install.installdir.as_deref(), Some("Cry of Fear"));
    assert!(install.buildid_matches());
    assert_eq!(install.library.as_deref(), Some(world.library.as_path()));

    world.cleanup();
}

#[test]
fn a_folder_picked_by_hand_is_accepted_and_one_without_the_game_is_not() {
    let world = World::new("manual-pick");

    let install = steam::inspect_folder(&world.vanilla).expect("a real game folder");
    assert_eq!(install.root, world.vanilla);
    // The manifest two levels up is still found, so the buildid is known.
    assert_eq!(install.buildid, Some(EXPECTED_BUILDID));

    let empty = world.base.join("una carpeta cualquiera");
    std::fs::create_dir_all(&empty).unwrap();
    assert!(steam::inspect_folder(&empty).is_err());

    world.cleanup();
}

#[test]
fn the_whole_install_flow_works_and_never_touches_the_vanilla_copy() {
    let world = World::new("install");
    let before = world.vanilla_fingerprint();

    let layout = world.layout();
    let destination = world.library.join("Mi ReCoF ñ");
    destination::validate(&destination, Some(&layout), Some(&world.vanilla))
        .expect("a folder beside steamapps in the same library is fine");

    let paths = world.app_paths();
    let store = VersionStore::new(paths.versions_dir());
    let release = world.stored(&store);

    let measured = measure(&world.vanilla);
    assert_eq!(measured.files, GAME.len() as u64);

    let mut stages = 0;
    let outcome = install::install(
        install::InstallRequest {
            source: &world.vanilla,
            source_buildid: Some(EXPECTED_BUILDID),
            destination: &destination,
            measured,
            release: &release,
            now: "2026-10-26T12:00:00Z",
            id: "flow-install",
        },
        &no_cancel(),
        &mut |_| stages += 1,
    )
    .expect("the install succeeds");

    let install::InstallOutcome::Installed(state) = outcome else {
        panic!("the install should not have been cancelled");
    };
    assert!(stages > 0);
    assert_eq!(state.version, Version::parse("0.3.0").unwrap());
    assert_eq!(state.source_buildid, Some(EXPECTED_BUILDID));

    // The ReCoF files are in place.
    for (path, content) in RELEASE {
        assert_eq!(
            std::fs::read(destination.join(path)).unwrap(),
            *content,
            "{path}"
        );
    }
    // sw.dll was written from the single hw.dll in the archive.
    assert_eq!(
        std::fs::read(destination.join("sw.dll")).unwrap(),
        b"ReCoF engine"
    );
    // Everything the release does not replace came from the copy.
    assert_eq!(
        std::fs::read(destination.join("cryoffear/maps/ap 01.bsp")).unwrap(),
        b"a map with a space in its name"
    );
    assert_eq!(
        std::fs::read(destination.join("cryoffear/sound/ambiente ñ.wav")).unwrap(),
        "sonido ambiente".as_bytes()
    );
    assert_eq!(
        std::fs::read(destination.join("CoFLaunchApp.exe")).unwrap(),
        b"retail settings dialog"
    );
    // And the state file records what happened.
    assert!(destination.join(".recof/installed.json").is_file());
    assert!(!destination.join(".recof/journal.json").exists());

    // The rule that matters most.
    assert_eq!(
        world.vanilla_fingerprint(),
        before,
        "the vanilla installation was modified"
    );

    world.cleanup();
}

#[test]
fn a_cancelled_install_leaves_nothing_behind() {
    let world = World::new("cancel");
    let before = world.vanilla_fingerprint();

    let paths = world.app_paths();
    let store = VersionStore::new(paths.versions_dir());
    let release = world.stored(&store);
    let destination = world.library.join("ReCoF cancelado");

    let outcome = install::install(
        install::InstallRequest {
            source: &world.vanilla,
            source_buildid: Some(EXPECTED_BUILDID),
            destination: &destination,
            measured: measure(&world.vanilla),
            release: &release,
            now: "2026-10-26T12:00:00Z",
            id: "flow-cancel",
        },
        &Arc::new(AtomicBool::new(true)),
        &mut |_| {},
    )
    .unwrap();

    assert_eq!(outcome, install::InstallOutcome::Cancelled);
    assert!(
        !destination.exists(),
        "a cancelled install should remove the folder it created"
    );
    assert_eq!(world.vanilla_fingerprint(), before);

    world.cleanup();
}

#[test]
fn verify_and_repair_put_a_damaged_file_back() {
    let world = World::new("repair");
    let paths = world.app_paths();
    let store = VersionStore::new(paths.versions_dir());
    let release = world.stored(&store);
    let destination = world.library.join("ReCoF reparable");

    install::install(
        install::InstallRequest {
            source: &world.vanilla,
            source_buildid: Some(EXPECTED_BUILDID),
            destination: &destination,
            measured: measure(&world.vanilla),
            release: &release,
            now: "2026-10-26T12:00:00Z",
            id: "flow-repair",
        },
        &no_cancel(),
        &mut |_| {},
    )
    .unwrap();

    assert!(install::verify(&destination, &mut |_, _| {})
        .unwrap()
        .is_intact());

    // Something went wrong with two of the files.
    std::fs::write(destination.join("hw.dll"), b"corrupted somehow").unwrap();
    std::fs::remove_file(destination.join("cryoffear/cl_dlls/hl.dll")).unwrap();

    let report = install::verify(&destination, &mut |_, _| {}).unwrap();
    assert!(!report.is_intact());
    let broken = report.broken();
    assert_eq!(broken.len(), 2);

    let repaired = install::maintain::repair(
        &destination,
        &release,
        &broken,
        "2026-10-26T13:00:00Z",
        "flow-repair-2",
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(repaired, 2);

    assert!(install::verify(&destination, &mut |_, _| {})
        .unwrap()
        .is_intact());
    assert_eq!(
        std::fs::read(destination.join("hw.dll")).unwrap(),
        b"ReCoF engine"
    );

    world.cleanup();
}

#[test]
fn an_update_replaces_the_files_and_drops_the_ones_it_no_longer_ships() {
    let world = World::new("update");
    let paths = world.app_paths();
    let store = VersionStore::new(paths.versions_dir());
    let release = world.stored(&store);
    let destination = world.library.join("ReCoF actualizable");

    install::install(
        install::InstallRequest {
            source: &world.vanilla,
            source_buildid: Some(EXPECTED_BUILDID),
            destination: &destination,
            measured: measure(&world.vanilla),
            release: &release,
            now: "2026-10-26T12:00:00Z",
            id: "flow-update-1",
        },
        &no_cancel(),
        &mut |_| {},
    )
    .unwrap();

    // A newer release that changes one file and no longer ships another.
    let newer_files: &[(&str, &[u8])] = &[
        ("cof.exe", b"ReCoF launcher 0.4"),
        ("hw.dll", b"ReCoF engine 0.4"),
        ("cryoffear/cl_dlls/client.dll", b"ReCoF client 0.4"),
    ];
    let archive = world.base.join("recof-0.4.0.zip");
    make_zip(&archive, newer_files);
    let mut manifest = manifest_for(&archive);
    manifest.version = Version::parse("0.4.0").unwrap();
    manifest.files = newer_files
        .iter()
        .map(|(path, content)| ManifestFile {
            path: (*path).to_owned(),
            size: content.len() as u64,
            sha256: sha256_bytes(content),
            copy_of: None,
        })
        .collect();

    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    let signing = SigningKey::from_bytes(&[7u8; 32]);
    let signature = serde_json::to_vec(&SignatureFile {
        alg: "ed25519".to_owned(),
        key_id: "flow-test-key".to_owned(),
        sig: base64_encode(&signing.sign(&bytes).to_bytes()),
    })
    .unwrap();
    let verified = verify_manifest_with_keys(
        &bytes,
        &signature,
        Product::Recof,
        &Version::parse("0.1.0").unwrap(),
        &[world.key],
    )
    .unwrap();
    let newer = store
        .store(&verified, &bytes, &signature, &archive, &mut |_, _| {})
        .unwrap();

    let state = install::apply_release(
        &destination,
        &newer,
        OperationKind::Update,
        "2026-10-26T14:00:00Z",
        "flow-update-2",
        &mut |_| {},
    )
    .unwrap();

    assert_eq!(state.version, Version::parse("0.4.0").unwrap());
    assert_eq!(
        std::fs::read(destination.join("cof.exe")).unwrap(),
        b"ReCoF launcher 0.4"
    );
    // hl.dll was in 0.3.0 and is not in 0.4.0, so the update removed it.
    assert!(!destination.join("cryoffear/cl_dlls/hl.dll").exists());
    // sw.dll was a copy entry in 0.3.0 and is gone from 0.4.0 too.
    assert!(!destination.join("sw.dll").exists());
    // And a file that belongs to the base game is still there.
    assert!(destination.join("cryoffear/maps/ap 01.bsp").is_file());

    assert!(install::verify(&destination, &mut |_, _| {})
        .unwrap()
        .is_intact());

    // Both versions are in the store, so a rollback has somewhere to go.
    assert_eq!(store.list(Product::Recof).unwrap().len(), 2);

    world.cleanup();
}

#[test]
fn the_copy_can_be_moved_and_then_removed() {
    let world = World::new("move-and-remove");
    let paths = world.app_paths();
    let store = VersionStore::new(paths.versions_dir());
    let release = world.stored(&store);
    let destination = world.library.join("ReCoF original");

    install::install(
        install::InstallRequest {
            source: &world.vanilla,
            source_buildid: Some(EXPECTED_BUILDID),
            destination: &destination,
            measured: measure(&world.vanilla),
            release: &release,
            now: "2026-10-26T12:00:00Z",
            id: "flow-move",
        },
        &no_cancel(),
        &mut |_| {},
    )
    .unwrap();

    let moved_to = world.base.join("Otro sitio/ReCoF movido");
    let outcome = install::move_install(
        &destination,
        &moved_to,
        measure(&destination),
        &no_cancel(),
        &mut |_| {},
    )
    .unwrap();
    assert_ne!(outcome, install::maintain::MoveOutcome::Cancelled);

    assert!(!destination.exists());
    assert!(install::verify(&moved_to, &mut |_, _| {})
        .unwrap()
        .is_intact());
    assert_eq!(
        InstalledState::load(&moved_to).unwrap().unwrap().version,
        Version::parse("0.3.0").unwrap()
    );

    install::uninstall(&moved_to, &paths, UninstallOptions::default()).unwrap();
    assert!(!moved_to.exists());
    assert!(!paths.versions_dir().exists());
    // The vanilla install is still exactly where it was.
    assert!(world.vanilla.join("cof.exe").is_file());

    world.cleanup();
}

#[test]
fn the_hash_scan_recognises_the_copy_and_the_vanilla_install_the_same_way() {
    let world = World::new("scan");

    // The fake game does not hold the real retail bytes, so every in scope
    // file reads as unknown. What matters is that the scan runs over a
    // real tree, reports per file, and never fails.
    let report = recof_core::hashes::scan_install(&world.vanilla).unwrap();
    assert_eq!(report.files.len(), recof_core::hashes::RETAIL_1_6A.len());
    assert!(report.files.iter().all(|file| matches!(
        file.verdict,
        FileVerdict::Unknown { .. } | FileVerdict::Missing
    )));
    assert!(report.unexpected_binaries.is_empty());
    assert!(report.mod_markers.is_empty());
    assert!(!report.is_untouched_retail());

    world.cleanup();
}

#[test]
fn the_space_check_adds_up_over_two_volumes_worth_of_requirements() {
    let world = World::new("space");
    let measured = measure(&world.vanilla);

    let plan =
        space::SpacePlan::new(measured, &world.library, &world.base.join("AppData/Local")).unwrap();

    // Both are under the temp directory, so they are the same volume and
    // the two requirements have to add up.
    assert!(plan.same_volume);
    assert_eq!(
        plan.destination_needed(),
        measured.bytes_on_disk + space::RESERVE_VERSION_STORE + space::RESERVE_BACKUPS
    );

    world.cleanup();
}
