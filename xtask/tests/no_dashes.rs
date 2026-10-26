//! Repository rule enforced as a test, so `cargo test --workspace` catches it
//! before CI does.

#[path = "../src/dashes.rs"]
#[allow(dead_code)]
mod dashes;

#[test]
fn repository_has_no_em_or_en_dashes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf();

    let hits = dashes::scan(&root).expect("the tree can be walked");
    if hits.is_empty() {
        return;
    }

    let mut report = String::new();
    for hit in &hits {
        let shown = hit.path.strip_prefix(&root).unwrap_or(&hit.path);
        report.push_str(&format!(
            "\n  {}:{}:{}  U+{:04X}  {}",
            shown.display(),
            hit.line,
            hit.column,
            hit.dash as u32,
            hit.text.trim()
        ));
    }
    panic!(
        "forbidden dash(es) found:{report}\nuse a comma, a colon, parentheses or a plain hyphen"
    );
}
