//! Small filesystem helpers the rest of the crate shares.

use std::path::Path;

use crate::{Error, Result};

/// Removes a directory and everything in it, treating "it was not there" as
/// success. Used before writing somewhere that must start empty.
pub fn remove_dir_if_present(path: &Path) -> Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::io(path, error)),
    }
}

/// Removes a file, treating "it was not there" as success.
pub fn remove_file_if_present(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::io(path, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removing_what_is_not_there_is_fine() {
        let missing = std::env::temp_dir().join("recof-fsutil-missing-xyz");
        let _ = std::fs::remove_dir_all(&missing);
        remove_dir_if_present(&missing).unwrap();
        remove_file_if_present(&missing.join("file")).unwrap();
    }

    #[test]
    fn removing_a_populated_directory_works() {
        let root = std::env::temp_dir().join("recof-fsutil-populated");
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        std::fs::write(root.join("a/b/c.txt"), b"x").unwrap();

        remove_dir_if_present(&root).unwrap();
        assert!(!root.exists());
    }
}
