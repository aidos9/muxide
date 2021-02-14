use crate::Formatter;
use std::io::Write;

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    StateChange,
    Information,
}

pub trait Log {
    type ReturnType;

    fn log_message(
        &self,
        log_level: LogLevel,
        message: String,
        formatter: Formatter,
    ) -> Self::ReturnType;
}
