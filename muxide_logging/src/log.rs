use crate::format::Format;
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    StateChange,
    Information,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LogItem {
    format: Format,
    message: String,
    level: LogLevel,
}

pub trait Log {
    type ReturnType;

    fn can_log_item(&self, _item: &LogItem) -> bool {
        return true;
    }

    fn log_item(&mut self, item: LogItem) -> Self::ReturnType;
}

impl LogLevel {
    pub const fn as_str(&self) -> &'static str {
        return match self {
            LogLevel::Error => "Error",
            LogLevel::Warning => "Warning",
            LogLevel::StateChange => "StateChange",
            LogLevel::Information => "Information",
        };
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}

impl LogItem {
    pub fn new(format: Format, level: LogLevel, message: &str) -> Self {
        return Self {
            format,
            message: message.to_string(),
            level,
        };
    }

    pub const fn level(&self) -> LogLevel {
        return self.level;
    }

    pub const fn message(&self) -> &String {
        return &self.message;
    }

    pub fn into_message(self) -> String {
        return self.message;
    }

    pub const fn format(&self) -> &Format {
        return &self.format;
    }
}

impl Into<String> for LogItem {
    fn into(self) -> String {
        return self.format.build_string(self.level, &self.message);
    }
}
