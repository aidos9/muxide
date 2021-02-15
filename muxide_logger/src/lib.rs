mod format;
mod log;
mod logger;
#[macro_use]
mod macros;

pub use format::{Format, FormatItem};
pub use log::{Log, LogLevel};
pub use logger::{FileLogger, StringLogger};

pub mod prelude {
    pub use super::format::{Format, FormatItem};
    pub use super::log::{Log, LogLevel};
    pub use super::logger::{FileLogger, StringLogger};
}
