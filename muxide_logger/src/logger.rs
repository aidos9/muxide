use crate::{Format, Log, LogLevel};
use std::fs::File;
use std::sync::{Arc, Mutex};

pub struct Builder {}

pub struct FileLogger {
    file: Arc<Mutex<File>>,
}

pub struct StringLogger {
    format: Format,
}

impl StringLogger {
    pub fn new(format: Format) -> Self {
        return Self { format };
    }
}

impl Log for StringLogger {
    type ReturnType = String;

    fn log_message(&self, log_level: LogLevel, message: &str) -> Self::ReturnType {
        panic!("{:?}", message);
    }
}
