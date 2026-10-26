//! Rotating log files.
//!
//! The log is where the technical detail of an error goes; the user sees a
//! translated sentence instead. It is also most of what the bug report
//! collects, so it has to be readable by a person and bounded in size.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::IoResultExt;
use crate::Result;

/// Rotate once the current file passes this.
pub const MAX_BYTES: u64 = 1024 * 1024;

/// How many rotated files are kept, newest first.
pub const KEEP_FILES: usize = 5;

const CURRENT: &str = "recof-launcher.log";

/// A `log` implementation writing to a rotating file.
pub struct FileLogger {
    inner: Mutex<Inner>,
    level: log::LevelFilter,
}

struct Inner {
    directory: PathBuf,
    file: Option<std::fs::File>,
    written: u64,
}

impl FileLogger {
    /// Opens, or creates, the log in `directory`.
    pub fn new(directory: &Path, level: log::LevelFilter) -> Result<Self> {
        std::fs::create_dir_all(directory).at(directory)?;
        let path = directory.join(CURRENT);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .at(&path)?;
        let written = file.metadata().at(&path)?.len();

        Ok(FileLogger {
            inner: Mutex::new(Inner {
                directory: directory.to_path_buf(),
                file: Some(file),
                written,
            }),
            level,
        })
    }

    /// Installs this as the global logger. Only the first call takes.
    pub fn install(self) -> std::result::Result<(), log::SetLoggerError> {
        let level = self.level;
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(level);
        Ok(())
    }

    /// The current log file, for the bug report and the open folder button.
    pub fn current_path(directory: &Path) -> PathBuf {
        directory.join(CURRENT)
    }

    /// The last `lines` lines of the current log, for the bug report.
    pub fn tail(directory: &Path, lines: usize) -> String {
        let path = Self::current_path(directory);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return String::new();
        };
        let all: Vec<&str> = text.lines().collect();
        let start = all.len().saturating_sub(lines);
        all[start..].join("\n")
    }
}

impl Inner {
    fn rotate(&mut self) {
        self.file = None;

        // recof-launcher.log.4 is dropped, 3 becomes 4, and so on.
        let oldest = self.directory.join(format!("{CURRENT}.{KEEP_FILES}"));
        let _ = std::fs::remove_file(oldest);
        for index in (1..KEEP_FILES).rev() {
            let from = self.directory.join(format!("{CURRENT}.{index}"));
            let to = self.directory.join(format!("{CURRENT}.{}", index + 1));
            let _ = std::fs::rename(from, to);
        }
        let _ = std::fs::rename(
            self.directory.join(CURRENT),
            self.directory.join(format!("{CURRENT}.1")),
        );

        self.file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.directory.join(CURRENT))
            .ok();
        self.written = 0;
    }

    fn write_line(&mut self, line: &str) {
        if self.written >= MAX_BYTES {
            self.rotate();
        }
        if let Some(file) = self.file.as_mut() {
            if writeln!(file, "{line}").is_ok() {
                self.written += line.len() as u64 + 1;
            }
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{:<5} {} {}",
            record.level(),
            record.target(),
            record.args()
        );

        // A poisoned lock means another thread panicked while logging.
        // Carrying on without the log is better than panicking again.
        if let Ok(mut inner) = self.inner.lock() {
            inner.write_line(&line);
        }
    }

    fn flush(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(file) = inner.file.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Log;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-log-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn record(logger: &FileLogger, message: &str) {
        logger.log(
            &log::Record::builder()
                .level(log::Level::Info)
                .target("test")
                .args(format_args!("{message}"))
                .build(),
        );
    }

    #[test]
    fn writes_lines_to_the_current_file() {
        let base = scratch("writes");
        let logger = FileLogger::new(&base, log::LevelFilter::Info).unwrap();
        record(&logger, "hola");
        record(&logger, "adios");
        logger.flush();

        let text = std::fs::read_to_string(FileLogger::current_path(&base)).unwrap();
        assert!(text.contains("hola"));
        assert!(text.contains("adios"));
        assert!(text.contains("INFO"));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_level_below_the_filter_is_dropped() {
        let base = scratch("level");
        let logger = FileLogger::new(&base, log::LevelFilter::Warn).unwrap();
        record(&logger, "this is only info");
        logger.flush();

        let text = std::fs::read_to_string(FileLogger::current_path(&base)).unwrap();
        assert!(text.is_empty());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn rotation_keeps_a_bounded_number_of_files() {
        let base = scratch("rotate");
        let logger = FileLogger::new(&base, log::LevelFilter::Info).unwrap();

        // Force several rotations with a line long enough to pass the cap
        // quickly.
        let long = "x".repeat(200_000);
        for round in 0..(KEEP_FILES + 3) {
            for _ in 0..6 {
                record(&logger, &long);
            }
            record(&logger, &format!("round {round}"));
        }
        logger.flush();

        let files: Vec<String> = std::fs::read_dir(&base)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            files.len() <= KEEP_FILES + 1,
            "the log grew to {} files: {files:?}",
            files.len()
        );
        assert!(files.contains(&CURRENT.to_owned()));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_tail_returns_the_last_lines() {
        let base = scratch("tail");
        let logger = FileLogger::new(&base, log::LevelFilter::Info).unwrap();
        for index in 0..50 {
            record(&logger, &format!("line {index}"));
        }
        logger.flush();

        let tail = FileLogger::tail(&base, 5);
        assert_eq!(tail.lines().count(), 5);
        assert!(tail.contains("line 49"));
        assert!(!tail.contains("line 44"));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_tail_of_a_missing_log_is_empty_rather_than_an_error() {
        let base = scratch("no-log");
        assert!(FileLogger::tail(&base, 10).is_empty());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn opening_an_existing_log_appends_rather_than_truncating() {
        let base = scratch("append");
        std::fs::write(FileLogger::current_path(&base), b"from a previous run\n").unwrap();

        let logger = FileLogger::new(&base, log::LevelFilter::Info).unwrap();
        record(&logger, "from this run");
        logger.flush();

        let text = std::fs::read_to_string(FileLogger::current_path(&base)).unwrap();
        assert!(text.contains("from a previous run"));
        assert!(text.contains("from this run"));

        std::fs::remove_dir_all(&base).unwrap();
    }
}
