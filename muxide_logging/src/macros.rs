use crate::format::Format;
use crate::log::{Log, LogItem, LogLevel};
use crate::DefaultLogger;
use std::ops::DerefMut;

#[macro_export]
/// Logs a message at the error log level.
macro_rules! error {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Error, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Error, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the warning log level.
macro_rules! warning {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Warning, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Warning, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the state change log level.
macro_rules! state_change {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::StateChange, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::StateChange, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the information log level.
macro_rules! info {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Information, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Information, $message, $logger)
    };
}

#[macro_export]
/// Creates the default [Format] with populated line, column and module_path values based on the
/// location where this macro was called.
macro_rules! default_format {
    () => {
        Format::default()
            .set_column(column!() as usize)
            .set_line(line!() as usize)
            .set_module_path(module_path!())
    };
}

#[macro_export]
/// Builds a new [Format] based on a series of [FormatItem](crate::format::FormatItem) objects.
macro_rules! build_format_from_items {
    ($($item:expr),*) => {
        $crate::format::Format::new()$(.append($item))*
    }
}

#[macro_export]
/// Helper macro for logging a message to a logger.
macro_rules! log_message {
    ($log_level:expr, $message:expr, $format:expr, $logger:expr) => {
        $crate::__log_message($log_level, $message, $format, &mut $logger);
    };

    ($log_level:expr, $message:expr, $logger:expr) => {
        $crate::log_message!($log_level, $message, default_format!(), $logger);
    };

    ($log_level:expr, $message:expr) => {
        $crate::__default_log_message($log_level, $message, default_format!());
    };
}

#[doc(hidden)]
/// A wrapper for __log_message that tries to lock the default logger.
pub fn __default_log_message(
    log_level: LogLevel,
    message: &str,
    format: Format,
) -> Option<<DefaultLogger as Log>::ReturnType> {
    if let Ok(mut logger) = crate::DEFAULT_LOGGER.lock() {
        return __log_message(log_level, message, format, logger.deref_mut());
    } else {
        return None;
    }
}

#[doc(hidden)]
/// Internal method used to write the log message to a file. We need a method instead of including
/// the macro because 'let' variables are not supported in some of the contexts we wish to support.
pub fn __log_message<L: Log + Log<ReturnType = T>, T>(
    log_level: LogLevel,
    message: &str,
    format: Format,
    logger: &mut L,
) -> Option<T> {
    let item = LogItem::new(format, log_level, message);

    if logger.can_log_item(&item) {
        return Some(logger.log_item(item));
    } else {
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::format::Format;
    use crate::logger::StringLogger;
    use chrono::DateTime;

    #[test]
    fn test_error_macro() {
        let mut logger = StringLogger::new();
        logger.override_format(default_format!().set_constant_time(DateTime::from(
            DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
        )));
        let content = error!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[20:52:37] ({} {}:32) Error: my message",
                module_path!(),
                line!() - 10
            )
        );
    }

    #[test]
    fn test_warning_macro() {
        let mut logger = StringLogger::new();
        logger.override_format(default_format!().set_constant_time(DateTime::from(
            DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
        )));
        let content = warning!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[20:52:37] ({} {}:32) Warning: my message",
                module_path!(),
                line!() - 10
            )
        );
    }

    #[test]
    fn test_state_change_macro() {
        let mut logger = StringLogger::new();
        logger.override_format(default_format!().set_constant_time(DateTime::from(
            DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
        )));
        let content = state_change!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[20:52:37] ({} {}:32) StateChange: my message",
                module_path!(),
                line!() - 10,
            )
        );
    }

    #[test]
    fn test_info_macro() {
        let mut logger = StringLogger::new();
        logger.override_format(default_format!().set_constant_time(DateTime::from(
            DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
        )));
        let content = info!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[20:52:37] ({} {}:32) Information: my message",
                module_path!(),
                line!() - 10,
            )
        );
    }
}
