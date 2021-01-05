use crate::config::Command;
use crate::error::{Error, ErrorType};
use crate::geometry::{Point, Size};
use crate::logic_manager::LogicManager;
use crate::pty::PTY;
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
    init_command: String,
    layout: Layout,
    initialized_output: bool,
    panels: Vec<PanelPtr>,
    prompt_content: String,
    selected_panel: Option<PanelPtr>,
    changed_state: bool,
    processor: LogicManager,
    continue_execution: bool,
    cursor_position_prompt: u16,
    config: Config,
}

struct Panel {
    pty: PTY,
    parser: Parser,
    size: Size,
    poll: Poll,
    events: Events,
    location: (u16, u16), // The top left first cell
    id: usize,
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

    pub fn new(init_command: &str, config: Config) -> Self {
        return Self {
            init_command: init_command.to_string(),
            layout: Layout::Empty,
            initialized_output: false,
            panels: Vec::new(),
            prompt_content: String::new(),
            selected_panel: None,
            changed_state: false,
            processor: LogicManager::new(),
            continue_execution: true,
            cursor_position_prompt: 0,
            config,
        };
    }

    pub fn quit(&self) -> bool {
        return !self.continue_execution;
    }

    pub fn init(mut self) -> Option<Self> {
        let mut stdout = stdout();
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );

        stdout.flush().ok()?;

        self.initialized_output = true;
        return Some(self);
    }

    pub fn receive_input(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        if !self.initialized_output | !self.continue_execution {
            return Err(ErrorType::DisplayNotRunning.into_error());
        }

        match self.processor.process_bytes(bytes.clone()) {
            Some(Command::QuitCommand) => {
                self.continue_execution = false;
                return Ok(());
            }
            Some(Command::EnterInputCommand) => {
                self.set_selected_panel(None);
                self.changed_state = true;
                return Ok(());
            }
            Some(Command::StopInputCommand) => {
                if self.panels.len() > 0 {
                    self.set_selected_panel(Some(self.panels[0].clone()));
                    self.changed_state = true;
                    return Ok(());
                }
            }
            Some(Command::OpenPanelCommand) => {
                return self.open_new_panel();
            }
            Some(_) => unimplemented!("Handling commands"), // A command was received in the future this will be handled
            None => (),
        }

        self.prompt_content = self.processor.get_cmd_buffer_string();
        self.changed_state = self.processor.redraw_required();

        match &mut self.selected_panel {
            Some(panel) => {
                panel.receive_input(bytes).unwrap();
            }
            None => (),
        }

        return Ok(());
    }

    pub fn open_new_panel(&mut self) -> Result<(), Error> {
        if !self.initialized_output | !self.continue_execution {
            return Ok(());
        }

        let new_layout = match &self.layout {
            Layout::Empty => {
                let size = Self::get_terminal_size()?;
                let panel = self.init_panel(size - Size::new(4, 2), (1, 1))?;

                self.set_selected_panel(Some(panel.clone()));

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
                left.resize(left_size.clone());
                let right = self.init_panel(
                    right_size,
                    (2 + left_size.get_cols(), 1 + left_size.get_rows()),
                )?; // 2 to account for the left and center borders

                Layout::HorizontalStack { left, right }
            }
            _ => unimplemented!(),
        };

        self.layout = new_layout;

        return Ok(());
    }

    fn init_panel(&mut self, size: Size, location: (u16, u16)) -> Result<PanelPtr, Error> {
        let mut panel = PanelPtr::new(
            &self.init_command,
            size,
            self.panels.len(),
            location,
            self.config.get_thread_time(),
        )?;
        panel.register();

        self.panels.push(panel.clone());

        return Ok(panel);
    }

    /// Returns true if a render is required
    pub fn pre_render(&mut self) -> Result<bool, Error> {
        if !self.initialized_output | !self.continue_execution {
            return Ok(false);
        }

        let mut changed = self.changed_state;
        self.changed_state = false;

        for panel in &mut self.panels {
            changed = changed || panel.read_pty_content()?;
        }

        return Ok(changed);
    }

    pub fn render(&mut self) -> Result<(), Error> {
        if !self.initialized_output | !self.continue_execution {
            return Ok(());
        }

        let mut stdout = stdout();
        let size = Self::get_terminal_size()?;

        queue!(stdout, terminal::Clear(ClearType::All));

        self.queue_main_borders(&mut stdout, &size);

        match &mut self.layout {
            Layout::Single { panel } => {
                let contents = panel.get_display_contents();

                for (r, row) in contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1));
                    stdout.write(&row);
                }
            }
            Layout::HorizontalStack { left, right } => {
                let left_contents = left.get_display_contents();
                let right_contents = right.get_display_contents();

                for (r, row) in left_contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1));
                    stdout.write(&row);
                }

                queue!(stdout, style::ResetColor);

                for (r, row) in right_contents.into_iter().enumerate() {
                    queue!(
                        stdout,
                        cursor::MoveTo(left.size().get_cols() + 2, r as u16 + 1)
                    );
                    stdout.write(&row);
                }

                Self::queue_vertical_centre_line(&mut stdout, &size, left.size().get_cols() + 1);
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
                let loc = panel.cursor_position();

                queue!(
                    stdout,
                    cursor::MoveTo(loc.column(), loc.row()) // Column, row
                );

                if panel.hide_cursor() {
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

    fn set_selected_panel(&mut self, panel_ptr: Option<PanelPtr>) {
        self.processor.set_command_mode(panel_ptr.is_none());
        self.selected_panel = panel_ptr;
    }
}

impl PanelPtr {
    pub fn new(
        command: &str,
        size: Size,
        id: usize,
        location: (u16, u16),
        thread_sleep_time: Duration,
    ) -> Result<Self, Error> {
        return Ok(Self(Rc::new(RefCell::new(Panel::new(
            command,
            size,
            id,
            location,
            thread_sleep_time,
        )?))));
    }

    wrap_panel_method!(read_pty_content, pub mut, => Result<bool, Error>);
    wrap_panel_method!(register, pub mut,);
    wrap_panel_method!(get_display_contents, pub, => Vec<Vec<u8>>);
    wrap_panel_method!(receive_input, pub mut, bytes: Vec<u8> => Result<(), Error>);
    wrap_panel_method!(cursor_position, pub, => Point<u16>);
    wrap_panel_method!(hide_cursor, pub, => bool);
    wrap_panel_method!(resize, pub mut, size: Size => Result<(), Error>);
    wrap_panel_method!(size, pub, => Size);
}

impl Panel {
    const BUFFER_SIZE: usize = 4096;
    const SCROLLBACK_LEN: usize = 120;

    pub fn new(
        command: &str,
        size: Size,
        id: usize,
        location: (u16, u16),
        thread_sleep_time: Duration,
    ) -> Result<Self, Error> {
        let mut pty = PTY::new(command, &size, thread_sleep_time)?;
        let parser = Parser::new(size.get_rows(), size.get_cols(), Self::SCROLLBACK_LEN);

        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                return Err(ErrorType::PollCreationError {
                    reason: format!("{}", e),
                }
                .into_error());
            }
        };

        return Ok(Self {
            pty,
            parser,
            size,
            poll,
            events: Events::with_capacity(128),
            id,
            location,
        });
    }

    pub fn register(&mut self) {
        self.poll
            .registry()
            .register(&mut self.pty, Token(0), Interest::READABLE);
    }

    pub fn read_pty_content(&mut self) -> Result<bool, Error> {
        match self
            .poll
            .poll(&mut self.events, Some(Duration::from_millis(100)))
        {
            Ok(_) => (),
            Err(e) => {
                match e.kind() {
                    // We will treat these two errors as non-terminal
                    ErrorKind::TimedOut | ErrorKind::Interrupted => {
                        return Ok(false);
                    }
                    _ => {
                        return Err(ErrorType::PollingError {
                            reason: format!("{}", e),
                        }
                        .into_error())
                    }
                }
            }
        }

        let mut changed = false;

        for event in self.events.iter() {
            match event.token() {
                Token(0) => {
                    let mut buffer = [0; Self::BUFFER_SIZE];
                    let count = match self.pty.read(&mut buffer) {
                        Ok(c) => c,
                        Err(e) => {
                            return Err(ErrorType::IOError {
                                read: true,
                                target: "PTY".to_string(),
                                reason: format!("{}", e),
                            }
                            .into_error())
                        }
                    };

                    self.parser.process(&buffer[..count]);
                    changed = true;
                }
                _ => (),
            }
        }

        return Ok(changed);
    }

    pub fn get_display_contents(&self) -> Vec<Vec<u8>> {
        return self
            .parser
            .screen()
            .rows_formatted(0, self.parser.screen().size().1)
            .collect();
    }

    pub fn resize(&mut self, size: Size) -> Result<(), Error> {
        self.size = size;
        self.parser
            .set_size(self.size.get_rows(), self.size.get_cols());

        return self.pty.resize(&self.size);
    }

    /// Writes bytes to the PTY
    pub fn receive_input(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        return self.pty.write_all(&bytes).map_err(|e| {
            ErrorType::IOError {
                read: false,
                target: "PTY".to_string(),
                reason: format!("{}", e),
            }
            .into_error()
        });
    }

    /// Returns the cursor position in the global space.
    pub fn cursor_position(&self) -> Point<u16> {
        let (row, col) = self.parser.screen().cursor_position();
        return Point::new_origin(col, row, self.location);
    }

    /// Returns if the current panel wants the cursor hidden
    pub fn hide_cursor(&self) -> bool {
        return self.parser.screen().hide_cursor();
    }

    /// Returns the size of the current panel.
    pub fn size(&self) -> Size {
        return self.size;
    }
}
