use chrono::{DateTime, Utc};
use std::ops::{Index, IndexMut};

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

#[derive(Clone, PartialEq, Debug)]
pub struct Format {
    items: Vec<FormatItem>,
    separator: Option<char>,
    thread_id: Option<usize>,
    column: Option<usize>,
    line: Option<usize>,
    file: Option<String>,
    module_path: Option<String>,
    custom_time: Option<DateTime<Utc>>,
}

impl Format {
    pub fn new() -> Self {
        return Self {
            items: Vec::new(),
            separator: None,
            thread_id: None,
            column: None,
            line: None,
            file: None,
            module_path: None,
            custom_time: None,
        };
    }

    pub fn build_string(self, log_message: &str) {

    }

    pub fn set_separator(mut self, separator: char) -> Self {
        self.separator = Some(separator);

        return self;
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

    pub fn set_

    pub fn set_constant_time(mut self, time: DateTime<Utc>) -> Self {
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
