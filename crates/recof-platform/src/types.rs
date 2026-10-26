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

/// A resolution the display can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
}

impl DisplayMode {
    /// Whether this is wider than four by three, which is what the game's
    /// widescreen flag is for.
    pub fn is_widescreen(&self) -> bool {
        self.width * 3 > self.height * 4
    }
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} x {}", self.width, self.height)
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

    #[test]
    fn widescreen_is_anything_wider_than_four_by_three() {
        assert!(!DisplayMode {
            width: 800,
            height: 600
        }
        .is_widescreen());
        assert!(!DisplayMode {
            width: 1280,
            height: 960
        }
        .is_widescreen());
        assert!(!DisplayMode {
            width: 1280,
            height: 1024
        }
        .is_widescreen());
        assert!(DisplayMode {
            width: 1280,
            height: 720
        }
        .is_widescreen());
        assert!(DisplayMode {
            width: 1920,
            height: 1080
        }
        .is_widescreen());
        assert!(DisplayMode {
            width: 2560,
            height: 1080
        }
        .is_widescreen());
    }

    #[test]
    fn a_mode_reads_as_a_resolution() {
        assert_eq!(
            DisplayMode {
                width: 1920,
                height: 1080
            }
            .to_string(),
            "1920 x 1080"
        );
    }
}
