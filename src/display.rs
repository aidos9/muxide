use crate::config::Command;
use crate::error::{Error, ErrorType};
use crate::geometry::{Point, Size};
use crate::logic_manager::LogicManager;
use crate::pty::Pty;
use crate::Config;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, style, terminal};
use mio::{Events, Interest, Poll, Token};
use std::cell::RefCell;
use std::io::{stdout, ErrorKind, Read, Stdout, Write};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use vt100::Parser;

macro_rules! wrap_panel_method {
    ($method_name:ident, mut, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        fn $method_name (&mut self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            let mut mut_ref = self.0.borrow_mut();
            return mut_ref.$method_name($($arg_name : $arg_type),*);
        }
    };

        ($method_name:ident, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        fn $method_name (&self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            return self.0.borrow().$method_name($($arg_name : $arg_type),*);
        }
    };

    ($method_name:ident, pub mut, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        pub fn $method_name (&mut self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            let mut mut_ref = self.0.borrow_mut();
            return mut_ref.$method_name($($arg_name),*);
        }
    };

    ($method_name:ident, pub, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        pub fn $method_name (&self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            return self.0.borrow().$method_name($($arg_name : $arg_type),*);
        }
    };
}

#[derive(Clone)]
struct PanelPtr(Rc<RefCell<Panel>>);

pub struct Display {
    panels: Vec<PanelPtr>,
    selected_panel: Option<PanelPtr>,
    layout: Layout,
    prompt_content: String,
    prompt_cursor_offset: u16,
    completed_initialization: bool,
}

struct Panel {
    id: usize,
    size: Size,
    content: Vec<Vec<u8>>,
    hide_cursor: bool,
    cursor_col: u16,
    cursor_row: u16,
    location: (u16, u16), // (col, row). The location in the global space of the top left (the first) cell
}

enum Layout {
    Empty,
    Single {
        panel: PanelPtr,
    },
    VerticalStack {
        lower: PanelPtr,
        upper: PanelPtr,
    },
    HorizontalStack {
        left: PanelPtr,
        right: PanelPtr,
    },
    QuadStack {
        lower_left: PanelPtr,
        lower_right: PanelPtr,
        upper_left: PanelPtr,
        upper_right: PanelPtr,
    },
}

impl Display {
    const VERTICAL_BORDER_CHARACTER: char = '|';
    const HORIZONTAL_BORDER_CHARACTER: char = '-';
    const CORNER_BORDER_CHARACTER: char = '+';
    const PROMPT_STRING: &'static str = "cmd > ";

    pub fn new() -> Self {
        return Self {
            layout: Layout::Empty,
            panels: Vec::new(),
            prompt_content: String::new(),
            prompt_cursor_offset: 0,
            selected_panel: None,
            completed_initialization: false,
        };
    }

    pub fn init(mut self) -> Option<Self> {
        let mut stdout = stdout();
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );

        stdout.flush().ok()?;

        self.completed_initialization = true;
        return Some(self);
    }

    /// Set the contents of a panel
    /// Error: If no panel exists with the specified id, or if init has not been run
    pub fn update_panel_content(&mut self, id: usize, content: Vec<Vec<u8>>) -> Result<(), Error> {
        if !self.completed_initialization {
            return Err(ErrorType::DisplayNotRunningError.into_error());
        }

        for panel in &mut self.panels {
            if panel.get_id() == id {
                panel.set_content(content);
                return Ok(());
            }
        }

        return Err(ErrorType::NoPanelWithIDError { id }.into_error());
    }

    /// Opens a new panel giving it the specified id. The id should be unique but it is
    /// not enforced by this method. The method will return a vector of all the changed panels
    /// id's and new size.
    pub fn open_new_panel(&mut self, id: usize) -> Result<Vec<(usize, Size)>, Error> {
        if !self.completed_initialization {
            return Err(ErrorType::DisplayNotRunningError.into_error());
        }

        let mut changed = Vec::new();

        let new_layout = match &self.layout {
            Layout::Empty => {
                let size = Self::get_terminal_size()?;
                let panel = self.init_panel(id, size - Size::new(4, 2), (1, 1));

                self.selected_panel = Some(panel.clone());
                changed.push((id, size));

                Layout::Single { panel }
            }
            Layout::Single { panel } => {
                // -3 cols (left border, right border, center border)
                // -4 rows (top border, bottom border, cmd input, cmd bottom border)
                let term_size = Self::get_terminal_size()?;
                let size = term_size - Size::new(4, 3);
                let mut left_size = size;
                left_size.divide_width_by_const(2);
                let right_size = Size::new(size.get_rows(), size.get_cols() - left_size.get_cols());

                let mut left = panel.clone(); // The location stays the same but the size needs to be re-adjusted
                left.set_size(left_size.clone());
                let right = self.init_panel(
                    id,
                    right_size,
                    (2 + left_size.get_cols(), 1 + left_size.get_rows()),
                ); // 2 to account for the left and center borders

                changed.push((id, right_size));
                changed.push((left.get_id(), left_size));

                Layout::HorizontalStack { left, right }
            }
            _ => unimplemented!(),
        };

        self.layout = new_layout;

        return Ok(changed);
    }

    fn init_panel(&mut self, id: usize, size: Size, location: (u16, u16)) -> PanelPtr {
        let mut panel = PanelPtr::new(id, size, location);

        self.panels.push(panel.clone());

        return panel;
    }

    pub fn render(&mut self) -> Result<(), Error> {
        if !self.completed_initialization {
            return Ok(());
        }

        let mut stdout = stdout();
        let size = Self::get_terminal_size()?;

        queue!(stdout, terminal::Clear(ClearType::All));

        self.queue_main_borders(&mut stdout, &size);

        match &mut self.layout {
            Layout::Single { panel } => {
                let contents = panel.get_content();

                for (r, row) in contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1));
                    stdout.write(&row);
                }
            }
            Layout::HorizontalStack { left, right } => {
                let left_contents = left.get_content();
                let right_contents = right.get_content();

                for (r, row) in left_contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1));
                    stdout.write(&row);
                }

                queue!(stdout, style::ResetColor);

                for (r, row) in right_contents.into_iter().enumerate() {
                    queue!(
                        stdout,
                        cursor::MoveTo(left.get_size().get_cols() + 2, r as u16 + 1)
                    );
                    stdout.write(&row);
                }

                Self::queue_vertical_centre_line(
                    &mut stdout,
                    &size,
                    left.get_size().get_cols() + 1,
                );
            }
            _ => (),
        }

        self.reset_cursor(&mut stdout, &size);

        queue!(stdout, style::ResetColor);

        return Ok(stdout.flush().map_err(|e| {
            ErrorType::StdoutFlushError {
                reason: format!("{}", e),
            }
            .into_error()
        })?);
    }

    fn get_terminal_size() -> Result<Size, Error> {
        let (cols, rows) = match terminal::size() {
            Ok(t) => t,
            Err(e) => {
                return Err(ErrorType::DetermineTerminalSizeError {
                    reason: e.to_string(),
                }
                .into_error());
            }
        };

        return Ok(Size::new(rows, cols));
    }

    /// Moves the cursor to the correct position and changes it to hidden or visible appropriately
    fn reset_cursor(&self, stdout: &mut Stdout, terminal_size: &Size) {
        match &self.selected_panel {
            Some(panel) => {
                let loc = panel.get_cursor_position();

                queue!(
                    stdout,
                    cursor::MoveTo(loc.column(), loc.row()) // Column, row
                );

                if panel.get_hide_cursor() {
                    execute!(stdout, cursor::Hide);
                } else {
                    execute!(stdout, cursor::Show);
                }
            }
            None => {
                execute!(
                    stdout,
                    cursor::Show,
                    cursor::MoveTo(
                        Self::PROMPT_STRING.len() as u16 + 1,
                        terminal_size.get_rows() - 2
                    ) // Column, row
                );
            }
        }
    }

    /// Queues the outer border for display in stdout
    fn queue_main_borders(&self, stdout: &mut Stdout, terminal_size: &Size) {
        // Print the top row
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            style::Print(Self::CORNER_BORDER_CHARACTER),
            style::Print(
                Self::HORIZONTAL_BORDER_CHARACTER
                    .to_string()
                    .repeat(terminal_size.get_cols() as usize - 2)
            ),
            style::Print(Self::CORNER_BORDER_CHARACTER),
        );

        // Print the vertical borders
        for i in 1..terminal_size.get_rows() - 3 {
            queue!(
                stdout,
                cursor::MoveTo(0, i),
                style::Print(Self::VERTICAL_BORDER_CHARACTER),
                cursor::MoveTo(terminal_size.get_cols() - 1, i),
                style::Print(Self::VERTICAL_BORDER_CHARACTER),
            );
        }

        // Print the horizontal border above the command prompt
        queue!(
            stdout,
            cursor::MoveTo(0, terminal_size.get_rows() - 3),
            style::Print(Self::CORNER_BORDER_CHARACTER),
            style::Print(
                Self::HORIZONTAL_BORDER_CHARACTER
                    .to_string()
                    .repeat(terminal_size.get_cols() as usize - 2)
            ),
            style::Print(Self::CORNER_BORDER_CHARACTER),
        );

        // Print the prompt and its content
        queue!(
            stdout,
            cursor::MoveTo(0, terminal_size.get_rows() - 2),
            style::Print(Self::VERTICAL_BORDER_CHARACTER),
            style::Print(Self::PROMPT_STRING),
            style::Print(&self.prompt_content),
            cursor::MoveTo(terminal_size.get_cols() - 1, terminal_size.get_rows() - 2),
            style::Print(Self::VERTICAL_BORDER_CHARACTER),
        );

        queue!(
            stdout,
            cursor::MoveTo(0, terminal_size.get_rows() - 1),
            style::Print(Self::CORNER_BORDER_CHARACTER),
            style::Print(
                Self::HORIZONTAL_BORDER_CHARACTER
                    .to_string()
                    .repeat(terminal_size.get_cols() as usize - 2)
            ),
            style::Print(Self::CORNER_BORDER_CHARACTER),
        );
    }

    fn queue_vertical_centre_line(stdout: &mut Stdout, terminal_size: &Size, col: u16) {
        for r in 1..terminal_size.get_rows() - 3 {
            queue!(
                stdout,
                cursor::MoveTo(col, r),
                style::Print(Self::VERTICAL_BORDER_CHARACTER)
            );
        }
    }

    fn panel_index_for_id(&self, id: usize) -> Option<usize> {
        for i in 0..self.panels.len() {
            if self.panels[i].get_id() == id {
                return Some(i);
            }
        }

        return None;
    }

    pub fn set_selected_panel(&mut self, id: Option<usize>) {
        if id.is_none() {
            self.selected_panel = None;
            return;
        }

        let id = id.unwrap();

        for panel in &self.panels {
            if panel.get_id() == id {
                self.selected_panel = Some(panel.clone());
                return;
            }
        }

        self.selected_panel = None;
    }

    pub fn update_panel_cursor(&mut self, id: usize, col: u16, row: u16, hide: bool) -> bool {
        let index = match self.panel_index_for_id(id) {
            Some(i) => i,
            None => return false,
        };

        self.panels[index].set_cursor_position(col, row);
        self.panels[index].set_hide_cursor(hide);

        return true;
    }
}

impl PanelPtr {
    pub fn new(id: usize, size: Size, location: (u16, u16)) -> Self {
        return Self(Rc::new(RefCell::new(Panel::new(id, size, location))));
    }

    wrap_panel_method!(get_cursor_position, pub, => Point<u16>);
    wrap_panel_method!(set_cursor_position, pub mut, col: u16, row: u16);
    wrap_panel_method!(set_content, pub mut, content: Vec<Vec<u8>>);
    wrap_panel_method!(get_content, pub, => Vec<Vec<u8>>);
    wrap_panel_method!(get_id, pub, => usize);
    wrap_panel_method!(set_size, pub mut, size: Size);
    wrap_panel_method!(get_size, pub, => Size);
    wrap_panel_method!(get_hide_cursor, pub, => bool);
    wrap_panel_method!(set_hide_cursor, pub mut, hide: bool);
}

impl Panel {
    pub fn new(id: usize, size: Size, location: (u16, u16)) -> Self {
        return Self {
            content: Vec::new(),
            size,
            id,
            location,
            hide_cursor: false,
            cursor_col: 0,
            cursor_row: 0,
        };
    }

    /// Returns the cursor position in the global space.
    pub fn get_cursor_position(&self) -> Point<u16> {
        return Point::new_origin(self.cursor_col, self.cursor_row, self.location);
    }

    pub fn set_cursor_position(&mut self, col: u16, row: u16) {
        self.cursor_col = col;
        self.cursor_row = row;
    }

    /// Set the content of this panel
    pub fn set_content(&mut self, content: Vec<Vec<u8>>) {
        self.content = content;
    }

    /// Returns an immutable reference to the content of this panel
    pub fn get_content(&self) -> Vec<Vec<u8>> {
        return self.content.clone();
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn get_size(&self) -> Size {
        return self.size;
    }

    pub fn get_hide_cursor(&self) -> bool {
        return self.hide_cursor;
    }

    pub fn set_hide_cursor(&mut self, hide: bool) {
        self.hide_cursor = hide;
    }
}
