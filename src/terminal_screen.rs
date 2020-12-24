use crate::vte_handler::VTEHandler;
use vte::Params;
use crate::geometry::{Size, Point};

#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug)]
struct Cell {
    content: char,
}

pub struct TerminalScreen {
    main_buffer: Vec<Vec<Cell>>,
    alternate_buffer: Vec<Vec<Cell>>,
    cursor_location: Point<u16>,
    saved_cursor_location: Point<u16>,
    size: Size,
}

impl Cell {
    pub fn new(content: char) -> Self {
        return Self {
            content,
        };
    }

    pub fn blank() -> Self {
        return Self {
            content: ' ',
        };
    }

    pub fn set_content(&mut self, ch: char) {
        self.content = ch;
    }
}

impl TerminalScreen {
    pub fn new(size: Size) -> Self {
        let empty_buffer = vec![vec![Cell::blank(); size.get_cols() as usize]; size.get_rows() as usize];

        return Self {
            main_buffer: empty_buffer.clone(),
            alternate_buffer: empty_buffer,
            cursor_location: Point::new_origin(1, 1, (1, 1)),
            saved_cursor_location: Point::new_origin(1, 1, (1, 1)),
            size,
        };
    }

    pub fn insert_at_cursor(&mut self, ch: char) {
        self.main_buffer[self.cursor_location.row_index() as usize][self.cursor_location.column_index() as usize].set_content(ch);
    }
}

derive_perform!(TerminalScreen);

impl VTEHandler for TerminalScreen {
    fn display_text(&mut self, ch: char) {
        self.insert_at_cursor(ch);
    }

    fn backspace(&mut self) {
        unimplemented!()
    }

    fn horizontal_tab(&mut self) {
        unimplemented!()
    }

    fn line_feed(&mut self) {
        unimplemented!()
    }

    fn carriage_return(&mut self) {
        unimplemented!()
    }

    fn unsupported_execute_byte(&mut self, byte: u8) {
        unimplemented!()
    }

    fn log_hook(&mut self, message: &str) {
        panic!("{}", message.to_string());
    }

    fn log_osc(&mut self, message: &str) {
        panic!("{}", message.to_string());
    }

    fn set_window_title(&mut self, title: &str) {
        unimplemented!()
    }

    fn set_icon_name(&mut self, name: &str) {
        unimplemented!()
    }

    fn insert_character(&mut self, n: u16) {
        unimplemented!()
    }

    fn cursor_up(&mut self, lines: u16) {
        unimplemented!()
    }

    fn cursor_down(&mut self, lines: u16) {
        unimplemented!()
    }

    fn cursor_forward(&mut self, columns: u16) {
        unimplemented!()
    }

    fn cursor_backward(&mut self, columns: u16) {
        unimplemented!()
    }

    fn cursor_next_line(&mut self, lines: u16) {
        unimplemented!()
    }

    fn cursor_previous_line(&mut self, lines: u16) {
        unimplemented!()
    }

    fn cursor_horizontal_absolute(&mut self, column: u16) {
        unimplemented!()
    }

    fn cursor_position(&mut self, line: u16, column: u16) {
        unimplemented!()
    }

    fn set_mode(&mut self, args: &Params) {
        unimplemented!()
    }

    fn reset_mode(&mut self, args: &Params) {
        unimplemented!()
    }

    fn cursor_horizontal_tab(&mut self, n: u16) {
        unimplemented!()
    }

    fn erase_in_display(&mut self, n: u16) {
        unimplemented!()
    }

    fn erase_in_line(&mut self, n: u16) {
        unimplemented!()
    }

    fn insert_line(&mut self, n: u16) {
        unimplemented!()
    }

    fn delete_line(&mut self, n: u16) {
        unimplemented!()
    }

    fn delete_character(&mut self, n: u16) {
        unimplemented!()
    }

    fn scroll_up(&mut self, n: u16) {
        unimplemented!()
    }

    fn scroll_down(&mut self, n: u16) {
        unimplemented!()
    }

    fn erase_character(&mut self, n: u16) {
        unimplemented!()
    }

    fn cursor_backward_tab(&mut self, n: u16) {
        unimplemented!()
    }

    fn repeat_preceding(&mut self, n: u16) {
        unimplemented!()
    }

    fn vertical_position_absolute(&mut self, n: u16) {
        unimplemented!()
    }

    fn vertical_position_relative(&mut self, n: u16) {
        unimplemented!()
    }

    fn selective_erase_in_display(&mut self, n: u16) {
        unimplemented!()
    }

    fn selective_erase_in_line(&mut self, n: u16) {
        unimplemented!()
    }

    fn select_graphic_rendition(&mut self, args: &Params) {
        unimplemented!()
    }

    fn dec_private_set_mode(&mut self, args: &Params) {
        unimplemented!()
    }

    fn dec_private_reset_mode(&mut self, args: &Params) {
        unimplemented!()
    }

    fn log_csi(&mut self, message: &str) {
        panic!("{}", message.to_string());
    }

    fn save_cursor(&mut self) {
        unimplemented!()
    }

    fn restore_cursor(&mut self) {
        unimplemented!()
    }

    fn index(&mut self) {
        unimplemented!()
    }

    fn next_line(&mut self) {
        unimplemented!()
    }

    fn horizontal_tabulation_set(&mut self) {
        unimplemented!()
    }

    fn reverse_index(&mut self) {
        unimplemented!()
    }

    fn reset_to_initial_state(&mut self) {
        unimplemented!()
    }

    fn keypad_application_mode(&mut self) {
        unimplemented!()
    }

    fn keypad_numeric_mode(&mut self) {
        unimplemented!()
    }

    fn log_esc(&mut self, message: &str) {
        panic!("{}", message.to_string());
    }
}