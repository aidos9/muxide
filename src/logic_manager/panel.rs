use vt100::Parser;

/// Represents a panel, i.e. the output for a process. It tracks the contents being
/// displayed and assigns an id.
pub struct Panel {
    parser: Parser,
    id: usize,
    current_scrollback: usize,
}

impl Panel {
    pub fn new(id: usize, parser: Parser) -> Self {
        return Self {
            parser,
            id,
            current_scrollback: 0,
        };
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.current_scrollback += lines;
        let previous = self.parser.screen().scrollback();
        self.parser.set_scrollback(self.current_scrollback);

        if self.parser.screen().scrollback() == previous {
            self.current_scrollback -= lines;
        }
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.current_scrollback = self.current_scrollback.checked_sub(lines).unwrap_or(0);
        self.parser.set_scrollback(self.current_scrollback);
    }

    pub fn clear_scrollback(&mut self) {
        self.current_scrollback = 0;
        self.parser.set_scrollback(self.current_scrollback);
    }

    pub fn id(&self) -> usize {
        return self.id;
    }

    pub fn parser_ref(&self) -> &Parser {
        return &self.parser;
    }

    pub fn parser_mut(&mut self) -> &mut Parser {
        return &mut self.parser;
    }

    pub fn current_scrollback(&self) -> usize {
        return self.current_scrollback;
    }
}
