//! Just enough PE parsing to recognise a patched binary as the retail one.
//!
//! The community 4 GB patch flips `IMAGE_FILE_LARGE_ADDRESS_AWARE` in the
//! COFF characteristics and fills in the optional header checksum, which
//! changes the file hash without changing a single instruction. A launcher
//! that only compared exact hashes would call a perfectly normal install
//! unknown.
//!
//! Normalising means clearing that one flag and zeroing the checksum before
//! hashing. Every retail 1.6a binary ships with a zero checksum and without
//! the flag, so a normalised retail file hashes to the value already
//! recorded in `docs/versions.csv`, and a patched file hashes to the same
//! value. Verified against the real Steam install: all seven in scope
//! binaries have `CheckSum` zero.

/// `IMAGE_FILE_LARGE_ADDRESS_AWARE`.
const LARGE_ADDRESS_AWARE: u16 = 0x0020;

/// `IMAGE_DOS_SIGNATURE`, "MZ".
const DOS_MAGIC: &[u8; 2] = b"MZ";

/// `IMAGE_NT_SIGNATURE`, "PE\0\0".
const NT_MAGIC: &[u8; 4] = b"PE\0\0";

/// Offset of `e_lfanew` in the DOS header.
const E_LFANEW: usize = 0x3c;

/// Offset of `Characteristics` inside the COFF file header, which itself
/// starts right after the four byte NT signature.
const CHARACTERISTICS_IN_COFF: usize = 18;

/// Size of the COFF file header.
const COFF_HEADER_SIZE: usize = 20;

/// Offset of `CheckSum` inside the optional header. Same in PE32 and PE32+.
const CHECKSUM_IN_OPTIONAL: usize = 64;

/// What a normalisation attempt produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Normalized<'a> {
    /// The file is not a PE, so there is nothing to normalise.
    NotPe,
    /// The file is a PE and these are its bytes with the large address flag
    /// cleared and the checksum zeroed.
    Pe {
        bytes: std::borrow::Cow<'a, [u8]>,
        /// Whether the original had the large address flag set.
        large_address_aware: bool,
        /// The checksum the original carried.
        checksum: u32,
    },
}

/// Clears the large address flag and the checksum, so a 4 GB patched binary
/// hashes the same as the retail file it was made from.
///
/// Returns the input untouched when it is not a PE, when it is truncated, or
/// when the headers do not make sense: a file the launcher cannot parse is
/// reported as an unknown hash, which is the honest answer.
pub fn normalize(bytes: &[u8]) -> Normalized<'_> {
    let Some(layout) = Layout::read(bytes) else {
        return Normalized::NotPe;
    };

    let characteristics = read_u16(bytes, layout.characteristics_offset);
    let checksum = read_u32(bytes, layout.checksum_offset);
    let large_address_aware = characteristics & LARGE_ADDRESS_AWARE != 0;

    if !large_address_aware && checksum == 0 {
        // Already in normal form, so no copy is needed.
        return Normalized::Pe {
            bytes: std::borrow::Cow::Borrowed(bytes),
            large_address_aware,
            checksum,
        };
    }

    let mut copy = bytes.to_vec();
    let cleared = characteristics & !LARGE_ADDRESS_AWARE;
    copy[layout.characteristics_offset..layout.characteristics_offset + 2]
        .copy_from_slice(&cleared.to_le_bytes());
    copy[layout.checksum_offset..layout.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    Normalized::Pe {
        bytes: std::borrow::Cow::Owned(copy),
        large_address_aware,
        checksum,
    }
}

/// Reads the headers the launcher cares about, or nothing if the file is not
/// a PE it can make sense of.
struct Layout {
    characteristics_offset: usize,
    checksum_offset: usize,
}

impl Layout {
    fn read(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < E_LFANEW + 4 || &bytes[0..2] != DOS_MAGIC {
            return None;
        }
        let nt_offset = read_u32(bytes, E_LFANEW) as usize;

        // A silly e_lfanew must not become a huge allocation or a panic.
        let nt_end = nt_offset.checked_add(4 + COFF_HEADER_SIZE)?;
        if nt_offset < E_LFANEW + 4 || nt_end > bytes.len() {
            return None;
        }
        if &bytes[nt_offset..nt_offset + 4] != NT_MAGIC {
            return None;
        }

        let coff_offset = nt_offset + 4;
        let optional_offset = coff_offset + COFF_HEADER_SIZE;
        let checksum_offset = optional_offset.checked_add(CHECKSUM_IN_OPTIONAL)?;
        if checksum_offset.checked_add(4)? > bytes.len() {
            return None;
        }

        // The optional header magic is 0x10b for PE32 and 0x20b for PE32+.
        // Anything else is not a Windows image this launcher should touch.
        let magic = read_u16(bytes, optional_offset);
        if magic != 0x010b && magic != 0x020b {
            return None;
        }

        Some(Layout {
            characteristics_offset: coff_offset + CHARACTERISTICS_IN_COFF,
            checksum_offset,
        })
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal but structurally valid PE32 image.
    fn fake_pe(characteristics: u16, checksum: u32) -> Vec<u8> {
        let nt_offset = 0x80usize;
        let mut bytes = vec![0u8; nt_offset + 4 + COFF_HEADER_SIZE + 224];
        bytes[0..2].copy_from_slice(DOS_MAGIC);
        bytes[E_LFANEW..E_LFANEW + 4].copy_from_slice(&(nt_offset as u32).to_le_bytes());
        bytes[nt_offset..nt_offset + 4].copy_from_slice(NT_MAGIC);

        let coff = nt_offset + 4;
        // Machine i386, as every Cry of Fear binary is.
        bytes[coff..coff + 2].copy_from_slice(&0x014cu16.to_le_bytes());
        bytes[coff + CHARACTERISTICS_IN_COFF..coff + CHARACTERISTICS_IN_COFF + 2]
            .copy_from_slice(&characteristics.to_le_bytes());

        let optional = coff + COFF_HEADER_SIZE;
        bytes[optional..optional + 2].copy_from_slice(&0x010bu16.to_le_bytes());
        bytes[optional + CHECKSUM_IN_OPTIONAL..optional + CHECKSUM_IN_OPTIONAL + 4]
            .copy_from_slice(&checksum.to_le_bytes());

        // Some payload after the headers so the files are not all identical.
        for (index, byte) in bytes.iter_mut().enumerate().skip(optional + 224) {
            *byte = (index % 251) as u8;
        }
        bytes
    }

    #[test]
    fn a_retail_shaped_binary_is_already_normal() {
        let retail = fake_pe(0x010f, 0);
        match normalize(&retail) {
            Normalized::Pe {
                bytes,
                large_address_aware,
                checksum,
            } => {
                assert!(!large_address_aware);
                assert_eq!(checksum, 0);
                assert_eq!(bytes.as_ref(), retail.as_slice());
                assert!(matches!(bytes, std::borrow::Cow::Borrowed(_)));
            }
            other => panic!("expected a PE, got {other:?}"),
        }
    }

    #[test]
    fn the_large_address_patch_normalizes_back_to_retail() {
        let retail = fake_pe(0x010f, 0);
        // What the 4 GB patch produces: the flag set and a real checksum.
        let patched = fake_pe(0x010f | LARGE_ADDRESS_AWARE, 0x0002_1a7f);
        assert_ne!(retail, patched);

        let Normalized::Pe {
            bytes,
            large_address_aware,
            checksum,
        } = normalize(&patched)
        else {
            panic!("the patched file is still a PE");
        };
        assert!(large_address_aware);
        assert_eq!(checksum, 0x0002_1a7f);
        assert_eq!(bytes.as_ref(), retail.as_slice());
    }

    #[test]
    fn a_checksum_alone_is_also_normalized() {
        let retail = fake_pe(0x210e, 0);
        let signed = fake_pe(0x210e, 0xdead_beef);
        let Normalized::Pe { bytes, .. } = normalize(&signed) else {
            panic!("still a PE");
        };
        assert_eq!(bytes.as_ref(), retail.as_slice());
    }

    #[test]
    fn a_real_difference_survives_normalization() {
        let mut other = fake_pe(0x010f, 0);
        let last = other.len() - 1;
        other[last] ^= 0xff;
        let Normalized::Pe { bytes, .. } = normalize(&other) else {
            panic!("still a PE");
        };
        assert_ne!(bytes.as_ref(), fake_pe(0x010f, 0).as_slice());
    }

    #[test]
    fn a_non_pe_file_is_left_alone() {
        assert_eq!(normalize(b"not an executable at all"), Normalized::NotPe);
        assert_eq!(normalize(b""), Normalized::NotPe);
        assert_eq!(normalize(b"MZ"), Normalized::NotPe);
    }

    #[test]
    fn a_truncated_pe_is_not_parsed() {
        let mut bytes = fake_pe(0x010f, 0);
        bytes.truncate(0x90);
        assert_eq!(normalize(&bytes), Normalized::NotPe);
    }

    #[test]
    fn an_absurd_header_offset_does_not_panic() {
        let mut bytes = fake_pe(0x010f, 0);
        bytes[E_LFANEW..E_LFANEW + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(normalize(&bytes), Normalized::NotPe);

        let mut bytes = fake_pe(0x010f, 0);
        bytes[E_LFANEW..E_LFANEW + 4].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(normalize(&bytes), Normalized::NotPe);
    }

    #[test]
    fn a_wrong_optional_header_magic_is_refused() {
        let mut bytes = fake_pe(0x010f, 0);
        let optional = 0x80 + 4 + COFF_HEADER_SIZE;
        bytes[optional..optional + 2].copy_from_slice(&0x0107u16.to_le_bytes());
        assert_eq!(normalize(&bytes), Normalized::NotPe);
    }
}
