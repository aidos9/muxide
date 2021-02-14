use muxide_logger::{log_message, FileLogger, Formatter, Log, LogLevel, PanicLogger};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_macro() {
        let logger = PanicLogger;
        log_message!(LogLevel::Error, "my message".to_string(), logger);
    }
}
