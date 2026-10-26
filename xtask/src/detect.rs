//! `cargo xtask detect`: run the detection chain against this machine and
//! print what the launcher would see.
//!
//! Useful while developing, and the same information the bug report
//! collects.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use recof_core::consts;
use recof_core::hashes::{scan_install, FileVerdict};
use recof_core::space::{self, measure_tree};
use recof_core::steam;

pub fn run(measure: bool) -> Result<(), String> {
    println!("ReCoF Launcher detection report");
    println!("expected buildid: {}", consts::EXPECTED_BUILDID);
    println!();

    let steam_path = steam::steam_path().map_err(|error| error.to_string())?;
    match &steam_path {
        Some(path) => println!("Steam:      {}", path.display()),
        None => {
            println!("Steam:      not found in the registry");
            return Ok(());
        }
    }

    let layout = steam::libraries(steam_path.as_ref().unwrap()).map_err(|e| e.to_string())?;
    println!("libraries:  {}", layout.libraries.len());
    for library in &layout.libraries {
        println!("            {}", library.display());
    }
    println!();

    let Some(install) = steam::detect::find_game_in(&layout).map_err(|e| e.to_string())? else {
        println!("Cry of Fear is not installed in any library");
        return Ok(());
    };

    println!("install:    {}", install.root.display());
    println!("installdir: {:?}", install.installdir);
    println!(
        "buildid:    {:?}  {}",
        install.buildid,
        if install.buildid_matches() {
            "as expected"
        } else {
            "NOT the expected build"
        }
    );
    println!(
        "SizeOnDisk: {:?} ({})",
        install.size_on_disk,
        install
            .size_on_disk
            .map(space::format_bytes)
            .unwrap_or_else(|| "unknown".to_owned())
    );
    println!();

    let report = scan_install(&install.root).map_err(|error| error.to_string())?;
    println!("file check:");
    for file in &report.files {
        let verdict = match &file.verdict {
            FileVerdict::Retail => "retail".to_owned(),
            FileVerdict::RetailPatched {
                large_address_aware,
                checksum,
            } => format!("retail, patched (LAA {large_address_aware}, checksum {checksum:#x})"),
            FileVerdict::KnownVariant { message_id } => format!("known variant: {message_id}"),
            FileVerdict::Unknown { sha256, size } => {
                format!("UNKNOWN sha256 {sha256} size {size}")
            }
            FileVerdict::Missing => "MISSING".to_owned(),
        };
        println!("  {:<30} {verdict}", file.path);
    }

    if !report.unexpected_binaries.is_empty() {
        println!();
        println!("unexpected executables:");
        for name in &report.unexpected_binaries {
            println!("  {name}");
        }
    }
    if !report.mod_markers.is_empty() {
        println!();
        println!("known modifications: {}", report.mod_markers.join(", "));
    }
    println!();
    println!("untouched retail: {}", report.is_untouched_retail());

    if measure {
        println!();
        let destination = install
            .library
            .clone()
            .unwrap_or_else(|| install.root.clone());
        let cluster = recof_platform::cluster_size(&destination).map_err(|e| e.to_string())?;
        println!("measuring {} ...", install.root.display());

        let cancel = Arc::new(AtomicBool::new(false));
        let size = measure_tree(&install.root, cluster, &cancel, &mut |partial| {
            print!(
                "\r  {} files, {}",
                partial.files,
                space::format_bytes(partial.bytes)
            );
            use std::io::Write;
            let _ = std::io::stdout().flush();
        })
        .map_err(|error| error.to_string())?
        .map_err(|_| "cancelled".to_owned())?;

        println!();
        println!("  files:        {}", size.files);
        println!("  directories:  {}", size.directories);
        println!("  logical:      {}", space::format_bytes(size.bytes));
        println!(
            "  on disk:      {} (cluster {cluster} B)",
            space::format_bytes(size.bytes_on_disk)
        );

        let app_data = recof_platform::local_app_data()
            .map_err(|e| e.to_string())?
            .join(consts::APP_DIR_NAME);
        let plan = space::SpacePlan::new(size, &destination, &app_data)
            .map_err(|error| error.to_string())?;
        println!();
        println!(
            "  destination {} available {}",
            plan.destination_space.volume,
            space::format_bytes(plan.destination_space.available)
        );
        println!(
            "  app data    {} available {}",
            plan.app_data_space.volume,
            space::format_bytes(plan.app_data_space.available)
        );
        println!(
            "  needed on destination: {}",
            space::format_bytes(plan.destination_needed())
        );
        println!("  fits: {}", plan.fits());
        for shortfall in plan.shortfalls() {
            println!(
                "  SHORT on {}: needs {} more",
                shortfall.volume,
                space::format_bytes(shortfall.missing())
            );
        }
    }

    Ok(())
}
