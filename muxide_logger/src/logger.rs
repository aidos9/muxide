use crate::{Formatter, Log, LogLevel};
use std::fs::File;
use std::sync::{Arc, Mutex};

pub struct Builder {}

pub struct FileLogger {
    file: Arc<Mutex<File>>,
}

pub struct StringLogger;

impl Log for StringLogger {
    type ReturnType = String;

    fn log_message(
        &self,
        log_level: LogLevel,
        message: String,
        mut formatter: Formatter,
    ) -> Self::ReturnType {
        panic!("{:?}, {:?}", formatter, message);
    }
}
