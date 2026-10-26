//! Repository rule: no em dash (U+2014) and no en dash (U+2013) anywhere.
//!
//! Checked in code, documentation, translation files and anything else that
//! is text. Binary files and generated directories are skipped.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Directory names that are never scanned, at any depth.
const SKIPPED_DIRS: &[&str] = &[".git", "target", "dist", "node_modules"];

/// File extensions that hold bytes, not text.
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "ico", "icns", "bmp", "webp", "zip", "gz", "exe", "dll", "pdb",
    "bin", "sig", "ttf", "otf", "ttc", "woff", "woff2", "wav", "ogg", "mp3", "vdf-bin",
];

/// One occurrence of a forbidden dash.
#[derive(Debug)]
pub struct Hit {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub dash: char,
    pub text: String,
}

/// Scans `root` and returns an error listing every occurrence found.
pub fn run(root: PathBuf) -> Result<(), String> {
    let hits = scan(&root)?;
    if hits.is_empty() {
        println!("check-dashes: clean");
        return Ok(());
    }

    let mut report = format!("{} forbidden dash(es) found:\n", hits.len());
    for hit in &hits {
        let shown = hit.path.strip_prefix(&root).unwrap_or(&hit.path);
        report.push_str(&format!(
            "  {}:{}:{}  U+{:04X}  {}\n",
            shown.display(),
            hit.line,
            hit.column,
            hit.dash as u32,
            hit.text.trim()
        ));
    }
    report.push_str("use a comma, a colon, parentheses or a plain hyphen instead");
    Err(report)
}

/// Walks the tree and collects every em dash and en dash in a text file.
pub fn scan(root: &Path) -> Result<Vec<Hit>, String> {
    let mut hits = Vec::new();

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            // The root itself has to pass the filter.
            entry.depth() == 0 || !SKIPPED_DIRS.contains(&name.as_ref())
        });

    for entry in walker {
        let entry = entry.map_err(|error| format!("walking {}: {error}", root.display()))?;
        if !entry.file_type().is_file() || is_binary(entry.path()) {
            continue;
        }

        let bytes = match std::fs::read(entry.path()) {
            Ok(bytes) => bytes,
            Err(error) => return Err(format!("reading {}: {error}", entry.path().display())),
        };
        // A file that is not valid UTF-8 is not a translation or a source
        // file, so it cannot hold a dash we care about.
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };

        collect(entry.path(), &text, &mut hits);
    }

    hits.sort_by(|a, b| (&a.path, a.line, a.column).cmp(&(&b.path, b.line, b.column)));
    Ok(hits)
}

fn collect(path: &Path, text: &str, hits: &mut Vec<Hit>) {
    for (line_index, line) in text.lines().enumerate() {
        for (column_index, character) in line.chars().enumerate() {
            if character == '\u{2014}' || character == '\u{2013}' {
                hits.push(Hit {
                    path: path.to_path_buf(),
                    line: line_index + 1,
                    column: column_index + 1,
                    dash: character,
                    text: line.to_string(),
                });
            }
        }
    }
}

fn is_binary(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let lowered = extension.to_ascii_lowercase();
            BINARY_EXTENSIONS.contains(&lowered.as_str())
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_both_dash_kinds_with_positions() {
        let mut hits = Vec::new();
        collect(
            Path::new("x.rs"),
            "fine line\nbad \u{2014} here\nbad \u{2013}\n",
            &mut hits,
        );

        assert_eq!(hits.len(), 2);
        assert_eq!(
            (hits[0].line, hits[0].column, hits[0].dash),
            (2, 5, '\u{2014}')
        );
        assert_eq!(
            (hits[1].line, hits[1].column, hits[1].dash),
            (3, 5, '\u{2013}')
        );
    }

    #[test]
    fn plain_hyphen_and_minus_are_allowed() {
        let mut hits = Vec::new();
        collect(Path::new("x.rs"), "a-b, 3 - 4, --flag\n", &mut hits);
        assert!(hits.is_empty());
    }

    #[test]
    fn column_counts_characters_not_bytes() {
        let mut hits = Vec::new();
        collect(
            Path::new("x.ftl"),
            "instalación \u{2014} lista\n",
            &mut hits,
        );
        assert_eq!(hits[0].column, "instalación ".chars().count() + 1);
    }

    #[test]
    fn binary_extensions_are_skipped() {
        assert!(is_binary(Path::new("a/b/icon.PNG")));
        assert!(is_binary(Path::new("release.zip")));
        assert!(!is_binary(Path::new("main.rs")));
        assert!(!is_binary(Path::new("README.md")));
        assert!(!is_binary(Path::new("Makefile")));
    }
}
