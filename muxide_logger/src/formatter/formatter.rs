use crate::LogLevel;

#[derive(Clone, PartialEq, Debug)]
pub struct Formatter {
    items: Vec<FormatItem>,
    separator: char,
    line: Option<usize>,
    module_path: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FormatItem {
    LineNumber,
    ThreadID,
    ModulePath,
    LogLevel,
    LogString,
    TimeString,
    CustomCharacter(char),
    CustomString(String),
}

impl Formatter {
    pub(crate) fn new() -> Self {}
}

impl Default for Formatter {
    fn default() -> Self {
        return Self {
            items: Vec::new(),
            line: None,
            module_path: None,
            separator: ' ',
            log_message: None,
        };
    }
}
