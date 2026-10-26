//! Types both backends return, so callers never see a platform specific one.

/// What a volume has room for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSpace {
    /// The mount point the figures belong to, for example `D:\`.
    pub volume: String,
    /// Bytes the current user may still write, quota included.
    pub available: u64,
    /// Size of the volume.
    pub total: u64,
}

impl DiskSpace {
    /// Bytes already in use.
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// Whether `needed` bytes fit, leaving `headroom` bytes untouched.
    pub fn fits(&self, needed: u64, headroom: u64) -> bool {
        self.available >= needed.saturating_add(headroom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(available: u64) -> DiskSpace {
        DiskSpace {
            volume: "T:\\".to_owned(),
            available,
            total: 1000,
        }
    }

    #[test]
    fn fits_accounts_for_headroom() {
        assert!(space(500).fits(400, 100));
        assert!(!space(500).fits(401, 100));
        assert!(space(500).fits(500, 0));
    }

    #[test]
    fn fits_does_not_overflow() {
        assert!(!space(u64::MAX - 1).fits(u64::MAX, u64::MAX));
    }
}
