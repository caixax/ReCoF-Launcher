//! Hashing, and deciding whether an install is the retail one.

pub mod pe;
pub mod scan;
pub mod table;

use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

pub use scan::{scan_install, FileReport, FileVerdict, InstallReport};
pub use table::{known_file, KnownFile, KnownVariant, ModMarker, RETAIL_1_6A};

/// Read this much at a time when hashing a file.
const CHUNK: usize = 1 << 20;

/// SHA-256 of a byte slice, lower case hex.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// SHA-256 of a file, read in chunks so a multi gigabyte file does not have
/// to fit in memory.
pub fn sha256_file(path: &Path) -> crate::Result<String> {
    let mut file = std::fs::File::open(path).map_err(|error| crate::Error::io(path, error))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| crate::Error::io(path, error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// SHA-256 of a file along with its size, which the manifest also records.
pub fn sha256_file_with_size(path: &Path) -> crate::Result<(String, u64)> {
    let size = std::fs::metadata(path)
        .map_err(|error| crate::Error::io(path, error))?
        .len();
    Ok((sha256_file(path)?, size))
}

/// Compares two hex digests without caring about case, and in constant time
/// with respect to where they first differ.
///
/// Hashes are not secrets, so this is about not writing a subtly wrong
/// comparison rather than about timing.
pub fn digests_equal(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.bytes().zip(right.bytes()) {
        difference |= a.to_ascii_lowercase() ^ b.to_ascii_lowercase();
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_the_empty_input() {
        assert_eq!(
            sha256_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hashes_a_known_string() {
        assert_eq!(
            sha256_bytes(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn file_and_byte_hashing_agree() {
        let dir = std::env::temp_dir().join("recof-hash-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample con espacios y ñ.bin");
        // Larger than one chunk, so the loop runs more than once.
        let payload: Vec<u8> = (0..(CHUNK + 1234))
            .map(|index| (index % 251) as u8)
            .collect();
        std::fs::write(&path, &payload).unwrap();

        assert_eq!(sha256_file(&path).unwrap(), sha256_bytes(&payload));
        let (digest, size) = sha256_file_with_size(&path).unwrap();
        assert_eq!(digest, sha256_bytes(&payload));
        assert_eq!(size as usize, payload.len());

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn comparison_ignores_case_but_not_content() {
        let digest = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(digests_equal(digest, &digest.to_uppercase()));
        assert!(!digests_equal(digest, &digest.replace('b', "c")));
        assert!(!digests_equal(digest, ""));
        assert!(!digests_equal(digest, &digest[..63]));
    }
}
