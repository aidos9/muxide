use crate::format::Format;
use crate::log::{Log, LogItem};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
/// The default logger, writes any new logs to a file by appending.
pub struct FileLogger {
    /// The file to write to. We have an optional value so that the user can open a file on demand.
    file: Option<File>,
    /// Whether we should panic on IO errors or ignore them.
    panic_on_fail: bool,
    /// A custom Format to use as an override.
    override_format: Option<Format>,
}

#[derive(Clone, Debug, PartialEq)]
/// An alternative logger, primarily used for testing purposes. However
pub struct StringLogger {
    override_format: Option<Format>,
}

impl FileLogger {
    pub fn new() -> Self {
        return Self {
            file: None,
            panic_on_fail: false,
            override_format: None,
        };
    }

    pub fn set_panic_on_fail(&mut self, b: bool) {
        self.panic_on_fail = b;
    }

    pub fn set_override(&mut self, override_format: Format) {
        self.override_format = Some(override_format);
    }

    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        self.file = Some(OpenOptions::new().append(true).create(true).open(path)?);

        return Ok(());
    }

    pub fn close_file(&mut self) {
        self.file = None;
    }
}

impl Log for FileLogger {
    type ReturnType = ();

    fn log_item(&mut self, item: LogItem) -> Self::ReturnType {
        if let Some(file) = &mut self.file {
            let text = match self.override_format.as_ref() {
                Some(format) => {
                    let new_format = Format::merged(format, item.format());

                    new_format.build_string(item.level(), &item.into_message())
                }
                None => item.into(),
            };

            let res = writeln!(file, "{}", text);

            if self.panic_on_fail {
                res.unwrap()
            }

            let res = file.flush();

            if self.panic_on_fail {
                res.unwrap();
            }
        }
    }
}

impl StringLogger {
    pub const fn new() -> Self {
        return Self {
            override_format: None,
        };
    }

    /// Override format will only merge the value prioritising having a value over not having one.
    pub fn override_format(&mut self, format: Format) {
        self.override_format = Some(format);
    }
}

impl Log for StringLogger {
    type ReturnType = String;

    fn log_item(&mut self, item: LogItem) -> Self::ReturnType {
        return match self.override_format.as_ref() {
            Some(format) => {
                let new_format = Format::merged(format, item.format());

                new_format.build_string(item.level(), &item.into_message())
            }
            None => item.into(),
        };
    }
}
