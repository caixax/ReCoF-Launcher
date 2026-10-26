//! Public keys the launcher trusts.
//!
//! A manifest is accepted only when its detached signature verifies against
//! one of these. The matching private keys live on the release machine and
//! in CI secrets, never in this repository.
//!
//! Rotation works by shipping a launcher that trusts both the old and the
//! new key, cutting releases with the new one, and dropping the old entry a
//! few versions later.
//!
//! Generate a pair with `cargo xtask keygen`, which prints the entry to
//! paste below and writes the private key to `keys/`, which is ignored by
//! git.

/// A key the launcher will accept signatures from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustedKey {
    /// Short name that appears in the signature file, so a launcher can say
    /// which key it was looking for.
    pub id: &'static str,
    /// The 32 byte Ed25519 public key.
    pub bytes: [u8; 32],
    /// Which products this key may sign. A key that only signs the launcher
    /// cannot be used to push game files, and the other way round.
    pub products: &'static [crate::manifest::Product],
}

/// Every key compiled into this build.
///
/// An empty list would mean every signature check fails closed with
/// `err-no-trusted-keys`, which is the right behaviour for a launcher
/// with no key: install nothing rather than everything.
pub const TRUSTED_KEYS: &[TrustedKey] = &[TrustedKey {
    id: "recof-2026a",
    bytes: [
        80, 205, 35, 0, 10, 117, 83, 191, 7, 114, 90, 208, 9, 244, 202, 168, 94, 194, 140, 253,
        240, 51, 157, 80, 226, 6, 154, 181, 252, 193, 210, 57,
    ],
    products: &[
        crate::manifest::Product::Recof,
        crate::manifest::Product::RecofLauncher,
    ],
}];

/// The keys that may sign a given product.
pub fn keys_for(product: crate::manifest::Product) -> impl Iterator<Item = &'static TrustedKey> {
    TRUSTED_KEYS
        .iter()
        .filter(move |key| key.products.contains(&product))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for key in TRUSTED_KEYS {
            assert!(seen.insert(key.id), "duplicate key id {}", key.id);
        }
    }

    #[test]
    fn every_key_names_at_least_one_product() {
        for key in TRUSTED_KEYS {
            assert!(!key.products.is_empty(), "{} signs nothing", key.id);
        }
    }

    #[test]
    fn every_key_is_a_valid_ed25519_point() {
        for key in TRUSTED_KEYS {
            assert!(
                ed25519_dalek::VerifyingKey::from_bytes(&key.bytes).is_ok(),
                "{} is not a usable public key",
                key.id
            );
        }
    }
}
