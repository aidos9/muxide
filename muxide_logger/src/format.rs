use crate::LogLevel;
use chrono::{DateTime, Local};
use std::ops::{Index, IndexMut};

#[derive(Clone, PartialEq, Debug)]
pub enum FormatItem {
    LineNumber,
    ColumnNumber,
    ThreadID,
    ModulePath,
    LogLevel,
    LogString,
    TimeString(String),
    CustomCharacter(char),
    CustomString(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Format {
    items: Vec<FormatItem>,
    thread_id: Option<usize>,
    column: Option<usize>,
    line: Option<usize>,
    file: Option<String>,
    module_path: Option<String>,
    custom_time: Option<DateTime<Local>>,
}

impl Format {
    pub fn new() -> Self {
        return Self {
            items: Vec::new(),
            thread_id: None,
            column: None,
            line: None,
            file: None,
            module_path: None,
            custom_time: None,
        };
    }

    pub fn build_string(self, log_level: LogLevel, log_message: &str) -> String {
        let mut item_strings = Vec::with_capacity(self.items.len());

        for item in self.items {
            let string = match item {
                FormatItem::LineNumber => {
                    if self.line.is_some() {
                        self.line.unwrap().to_string()
                    } else {
                        String::new()
                    }
                }
                FormatItem::ColumnNumber => {
                    if self.column.is_some() {
                        self.column.unwrap().to_string()
                    } else {
                        String::new()
                    }
                }
                FormatItem::ThreadID => {
                    if self.thread_id.is_some() {
                        self.thread_id.unwrap().to_string()
                    } else {
                        String::new()
                    }
                }
                FormatItem::ModulePath => self
                    .module_path
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or(String::new()),
                FormatItem::LogLevel => log_level.to_string(),
                FormatItem::LogString => log_message.to_string(),
                FormatItem::TimeString(fmt_string) => {
                    if self.custom_time.is_some() {
                        self.custom_time.unwrap().format(&fmt_string).to_string()
                    } else {
                        Local::now().format(&fmt_string).to_string()
                    }
                }
                FormatItem::CustomCharacter(ch) => ch.to_string(),
                FormatItem::CustomString(s) => s,
            };

            item_strings.push(string);
        }

        return item_strings.join("");
    }

    pub fn set_thread_id(mut self, id: usize) -> Self {
        self.thread_id = Some(id);

        return self;
    }

    pub fn set_column(mut self, col: usize) -> Self {
        self.column = Some(col);

        return self;
    }

    pub fn set_line(mut self, line: usize) -> Self {
        self.line = Some(line);

        return self;
    }

    pub fn set_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());

        return self;
    }

    pub fn set_module_path(mut self, path: &str) -> Self {
        self.module_path = Some(path.to_string());

        return self;
    }

    pub fn set_constant_time(mut self, time: DateTime<Local>) -> Self {
        self.custom_time = Some(time);

        return self;
    }

    pub fn append(mut self, item: FormatItem) -> Self {
        self.items.push(item);

        return self;
    }

    pub fn remove_last(mut self) -> Self {
        let _ = self.items.pop();

        return self;
    }
}

impl Index<usize> for Format {
    type Output = FormatItem;

    fn index(&self, index: usize) -> &Self::Output {
        return self.items.index(index);
    }
}

impl IndexMut<usize> for Format {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return self.items.index_mut(index);
    }
}

impl Default for Format {
    fn default() -> Self {
        return crate::build_format_from_items!(
            FormatItem::CustomCharacter('['),
            FormatItem::TimeString("%k:%M:%S".to_string()),
            FormatItem::CustomString("] (".to_string()),
            FormatItem::ModulePath,
            FormatItem::CustomCharacter(' '),
            FormatItem::LineNumber,
            FormatItem::CustomCharacter(':'),
            FormatItem::ColumnNumber,
            FormatItem::CustomString(") <Thread: ".to_string()),
            FormatItem::ThreadID,
            FormatItem::CustomString("> ".to_string()),
            FormatItem::LogLevel,
            FormatItem::CustomString(": ".to_string()),
            FormatItem::LogString
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Format, FormatItem};
    use crate::LogLevel;
    use chrono::DateTime;

    #[test]
    fn test_default() {
        assert_eq!(
            Format::default(),
            Format {
                items: vec![
                    FormatItem::CustomCharacter('['),
                    FormatItem::TimeString("%k:%M:%S".to_string()),
                    FormatItem::CustomString("] (".to_string()),
                    FormatItem::ModulePath,
                    FormatItem::CustomCharacter(' '),
                    FormatItem::LineNumber,
                    FormatItem::CustomCharacter(':'),
                    FormatItem::ColumnNumber,
                    FormatItem::CustomString(") <Thread: ".to_string()),
                    FormatItem::ThreadID,
                    FormatItem::CustomString("> ".to_string()),
                    FormatItem::LogLevel,
                    FormatItem::CustomString(": ".to_string()),
                    FormatItem::LogString
                ],
                thread_id: None,
                column: None,
                line: None,
                file: None,
                module_path: None,
                custom_time: None
            }
        )
    }

    #[test]
    fn test_build_default() {
        assert_eq!(
            Format::default()
                .set_column(0)
                .set_line(123)
                .set_module_path("src/log.rs")
                .set_thread_id(0)
                .set_constant_time(DateTime::from(
                    DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap()
                ))
                .build_string(LogLevel::Warning, "Some Warning"),
            "[20:52:37] (src/log.rs 123:0) <Thread: 0> Warning: Some Warning".to_string(),
        )
    }
}
