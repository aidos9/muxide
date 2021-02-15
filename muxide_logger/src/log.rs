use crate::Format;
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    StateChange,
    Information,
}

pub trait Log {
    type ReturnType;

    fn log_message(&self, log_level: LogLevel, message: &str) -> Self::ReturnType;
}

impl LogLevel {
    pub const fn as_str(&self) -> &'static str {
        return match self {
            LogLevel::Error => "Error",
            LogLevel::Warning => "Warning",
            LogLevel::StateChange => "StateChange",
            LogLevel::Information => "Info",
        };
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}
