// A console window behind the launcher would look like a crash. Debug
// builds keep it so logs and panics are visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod i18n;
mod screens;
mod theme;
mod worker;

use recof_core::consts;
use recof_core::log_file::FileLogger;
use recof_core::paths::AppPaths;

fn main() -> eframe::Result<()> {
    start_logging();
    log::info!(
        "{} {} starting on {}",
        consts::APP_DISPLAY_NAME,
        consts::LAUNCHER_VERSION,
        recof_platform::PLATFORM_NAME
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 660.0])
            .with_min_inner_size([820.0, 560.0])
            .with_title(consts::APP_DISPLAY_NAME),
        ..Default::default()
    };

    eframe::run_native(
        consts::APP_DISPLAY_NAME,
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

/// Opens the rotating log. A launcher that cannot write a log still runs:
/// losing the log is worse than nothing, but not being able to start is
/// worse than that.
fn start_logging() {
    let Ok(paths) = AppPaths::discover() else {
        return;
    };
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    match FileLogger::new(&paths.logs_dir(), level) {
        Ok(logger) => {
            if let Err(error) = logger.install() {
                eprintln!("the logger could not be installed: {error}");
            }
        }
        Err(error) => eprintln!("the log file could not be opened: {error}"),
    }
}

/// The current time as an RFC 3339 string, in UTC.
///
/// Written out rather than taken from a date crate: the launcher needs one
/// timestamp format in one place, and a dependency for it would be bytes
/// in the executable for nothing.
pub fn now_rfc3339() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    format_rfc3339(seconds)
}

/// Formats a Unix timestamp as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn format_rfc3339(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let rest = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// A timestamp shaped for a file name: `20261026-120000`.
pub fn now_stamp() -> String {
    let text = now_rfc3339();
    text.chars()
        .filter(|character| character.is_ascii_digit() || *character == 'T')
        .collect::<String>()
        .replace('T', "-")
}

/// Howard Hinnant's civil_from_days, the standard way to turn a day count
/// since 1970-01-01 into a date without a calendar library.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_formats_as_the_epoch() {
        assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_timestamps_format_correctly() {
        // Checked against the usual references.
        assert_eq!(format_rfc3339(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(format_rfc3339(1_761_436_800), "2025-10-26T00:00:00Z");
        assert_eq!(format_rfc3339(946_684_800), "2000-01-01T00:00:00Z");
        // A leap day, which is where a hand written calendar goes wrong.
        assert_eq!(format_rfc3339(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(format_rfc3339(1_709_251_199), "2024-02-29T23:59:59Z");
        assert_eq!(format_rfc3339(1_709_251_200), "2024-03-01T00:00:00Z");
        // 1900 was not a leap year and 2000 was, which is the rule a naive
        // implementation gets wrong.
        assert_eq!(format_rfc3339(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn a_time_before_the_epoch_does_not_panic() {
        assert_eq!(format_rfc3339(-1), "1969-12-31T23:59:59Z");
        assert_eq!(format_rfc3339(-86_400), "1969-12-31T00:00:00Z");
    }

    #[test]
    fn now_is_a_plausible_timestamp() {
        let now = now_rfc3339();
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.ends_with('Z'), "{now}");
        // Anything before this project existed means the clock or the
        // arithmetic is wrong.
        assert!(now.as_str() > "2024-01-01T00:00:00Z", "{now}");
    }

    #[test]
    fn a_stamp_is_usable_in_a_file_name() {
        let stamp = now_stamp();
        assert_eq!(stamp.len(), 15, "{stamp}");
        assert!(stamp.chars().all(|c| c.is_ascii_digit() || c == '-'));
    }
}
