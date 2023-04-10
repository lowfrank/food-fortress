//! Little logging library to log program failure. We are allowed to panic if we can't
//! log, because we have nowhere else to log to.

use std::fmt;
use std::fs;
use std::io::Write;

use super::backend::today;

/// Log datetime format
const DATETIME_LOG_FORMAT: &str = "%Y-%m-%d %H:%M:%S:%3f";

/// Path to the log file
pub const LOG: &str = "log\\log.log";

/// Log level
#[derive(Debug)]
enum Level {
    Warning,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn log(msg: impl fmt::Display, level: Level) {
    let msg = format!(
        "{} {}: {}\n",
        today().format(DATETIME_LOG_FORMAT),
        level,
        msg
    );
    fs::OpenOptions::new()
        .append(true)
        .open(LOG)
        .unwrap() // If there is an error, there is nowhere else we can log it
        .write_all(msg.as_bytes())
        .unwrap(); // If there is an error, there is nowhere else we can log it
}

/// Helper for [`log`] with error level. This function panics!
/// Call this when the situation is unrecoverable
pub fn error(err: impl std::error::Error) -> ! {
    log(err, Level::Error);
    panic!();
}

/// Helper for [`log`] with warning level
pub fn warning(msg: impl fmt::Display) {
    log(msg, Level::Warning);
}
