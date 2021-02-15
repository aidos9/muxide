#[macro_export]
macro_rules! error {
    ($message:expr) => {
        $crate::log_message!($crate::LogLevel::Error, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::LogLevel::Error, $message, $logger)
    };
}

#[macro_export]
macro_rules! warning {
    ($message:expr) => {
        $crate::log_message!($crate::LogLevel::Warning, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::LogLevel::Warning, $message, $logger)
    };
}

#[macro_export]
macro_rules! state_change {
    ($message:expr) => {
        $crate::log_message!($crate::LogLevel::StateChange, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::LogLevel::StateChange, $message, $logger)
    };
}

#[macro_export]
macro_rules! info {
    ($message:expr) => {
        $crate::log_message!($crate::LogLevel::Information, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::LogLevel::Information, $message, $logger)
    };
}

#[macro_export]
macro_rules! log_message {
    ($log_level:expr, $message:expr, $formatter:expr, $logger:expr) => {
        let mut formatter = $formatter;
        formatter.set_module_path(module_path!());
        formatter.set_line(line!() as usize);

        $logger.log_message($log_level, $message, formatter)
    };

    ($log_level:expr, $message:expr, $logger:expr) => {
        log_message!($log_level, $message, $crate::Formatter::default(), $logger)
    };

    ($log_level:expr, $message:expr) => {
        log_message!(
            $log_level,
            $message,
            $crate::Formatter::default(),
            $crate::DEFAULT_LOGGER
        )
    };
}

#[macro_export]
macro_rules! build_format_from_items {
    ($($item:expr),*) => {
        $crate::Format::new()$(.append($item))*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;
    use crate::StringLogger;

    #[test]
    fn test_log_macro() {
        let logger = StringLogger::new(Format::default());
        assert_eq!(
            log_message!(LogLevel::Error, "my message".to_string(), logger),
            ""
        );
    }
}
