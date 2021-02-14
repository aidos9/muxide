#[macro_export]
macro_rules! log_message {
    ($log_level:expr, $message:expr, $formatter:expr, $logger:expr) => {
        let mut formatter = $formatter;
        formatter.set_module_path(module_path!());
        formatter.set_line(line!() as usize);

        $logger.log_message($log_level, $message, formatter);
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
        );
    };
}
