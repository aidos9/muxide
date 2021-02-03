use super::panel::PanelPtr;
use crate::error::{ErrorType, MuxideError};
use crate::geometry::Size;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{stdout, Stdout, Write};

/// Manages the different panels and renders to the terminal the correct output and layout.
pub struct Display {
    panels: Vec<PanelPtr>,
    selected_panel: Option<PanelPtr>,
    layout: Layout,
    prompt_content: String,
    prompt_cursor_offset: u16,
    completed_initialization: bool,
}

/// The different supported layouts of panels
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
    /// The character user for the vertical side borders
    const VERTICAL_BORDER_CHARACTER: char = '|';
    /// The character user for the horizontal side borders
    const HORIZONTAL_BORDER_CHARACTER: char = '-';
    /// The character user for the corner borders
    const CORNER_BORDER_CHARACTER: char = '+';
    /// The text used as a prompt when entering a command
    const PROMPT_STRING: &'static str = "cmd > ";
    /// The text that is displayed when there are no open panels.
    const EMPTY_TEXT: &'static str = "No Panels Open";

    /// Create a new "display" instance.
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

    /// Initializes the terminal for output by taking control of the stdout and clearing the
    /// terminal. This must be run before any other methods are.
    pub fn init(mut self) -> Option<Self> {
        let mut stdout = stdout();
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .ok()?;

        stdout.flush().ok()?;

        self.completed_initialization = true;
        return Some(self);
    }

    /// Set the contents of a panel
    /// Error: If no panel exists with the specified id, or if init has not been run
    pub fn update_panel_content(
        &mut self,
        id: usize,
        content: Vec<Vec<u8>>,
    ) -> Result<(), MuxideError> {
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

    /// Set the text being displayed in the command prompt.
    pub fn set_cmd_content(&mut self, content: String) {
        self.prompt_content = content;
    }

    /// Allows for setting the cursor position in the command prompt
    pub fn set_cmd_offset(&mut self, offset: u16) {
        self.prompt_cursor_offset = offset;
    }

    /// Allows for adding a constant to the cursor position in the command prompt
    pub fn add_cmd_offset(&mut self, offset: u16) {
        self.prompt_cursor_offset += offset;
    }

    /// Allows for subtracting a constant from the cursor position in the command prompt
    pub fn sub_cmd_offset(&mut self, offset: u16) {
        self.prompt_cursor_offset -= offset;
    }

    /// Opens a new panel giving it the specified id. The id should be unique but it is
    /// not enforced by this method. The method will return a vector of all the changed panels
    /// id's and new size.
    pub fn open_new_panel(&mut self, id: usize) -> Result<Vec<(usize, Size)>, MuxideError> {
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
                let right = self.init_panel(id, right_size, (2 + left_size.get_cols(), 1)); // 2 to account for the left and center borders

                changed.push((left.get_id(), left_size));
                changed.push((id, right_size));

                Layout::HorizontalStack { left, right }
            }
            _ => unimplemented!(),
        };

        self.layout = new_layout;

        return Ok(changed);
    }

    pub fn close_panel(&mut self, id: usize) -> Result<Vec<(usize, Size)>, MuxideError> {
        if !self.completed_initialization {
            return Err(ErrorType::DisplayNotRunningError.into_error());
        }

        let mut changed = Vec::new();

        let new_layout = match &self.layout {
            Layout::Empty => {
                return Ok(Vec::new());
            }
            Layout::Single { panel } => {
                if panel.get_id() != id {
                    return Err(ErrorType::NoPanelWithIDError { id }.into_error());
                }

                self.panels.clear();
                self.selected_panel = None;

                Layout::Empty
            }
            Layout::HorizontalStack { left, right } => {
                let size = Self::get_terminal_size()? - Size::new(4, 2);
                let mut panel;
                let other_id;

                if left.get_id() == id {
                    panel = right.clone();
                    other_id = left.get_id();
                } else if right.get_id() == id {
                    panel = left.clone();
                    other_id = right.get_id();
                } else {
                    return Err(ErrorType::NoPanelWithIDError { id }.into_error());
                }

                panel.set_size(size);
                panel.set_location((1, 1));

                for i in 0..self.panels.len() {
                    if self.panels[i].get_id() == id {
                        self.panels.remove(i);
                        break;
                    }
                }

                self.selected_panel = Some(panel.clone());
                changed.push((panel.get_id(), size));

                Layout::Single { panel }
            }
            _ => unimplemented!(),
        };

        self.layout = new_layout;

        return Ok(changed);
    }

    // Initialise a panel by creating a new instance and copying the pointer into the internal tracker.
    fn init_panel(&mut self, id: usize, size: Size, location: (u16, u16)) -> PanelPtr {
        let panel = PanelPtr::new(id, size, location);

        self.panels.push(panel.clone());

        return panel;
    }

    /// Render the contents of the display to stdout.
    pub fn render(&mut self) -> Result<(), MuxideError> {
        if !self.completed_initialization {
            return Ok(());
        }

        let mut stdout = stdout();
        let size = Self::get_terminal_size()?;

        // Clear the terminal
        queue!(stdout, terminal::Clear(ClearType::All)).map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        self.queue_main_borders(&mut stdout, &size)?;

        match &mut self.layout {
            Layout::Empty => {
                let (mut col, mut row) = (size.get_cols(), size.get_rows());

                // -2 For the left and right borders
                col -= 2;
                // Determine the center
                col /= 2;
                // Align the empty text to the center
                col -= Self::EMPTY_TEXT.len() as u16 / 2;

                // -4 for the top, bottom and the command prompt borders
                row -= 4;
                // Determine the center
                row /= 2;
                // Subtract 1 for the height of the text
                row -= 1;

                // Add 1 to offset by the left and top borders. Obviously it is useless having
                // the + and - operations that cancel each other but for clarity's sake they have
                // been used.
                queue!(stdout, cursor::MoveTo(col + 1, row + 1)).map_err(|e| {
                    ErrorType::QueueExecuteError {
                        reason: e.to_string(),
                    }
                    .into_error()
                })?;

                stdout
                    .write(Self::EMPTY_TEXT.as_ref())
                    .map_err(|e| ErrorType::new_display_qe_error(e))?;
            }
            Layout::Single { panel } => {
                let contents = panel.get_content();

                for (r, row) in contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1)).map_err(|e| {
                        ErrorType::QueueExecuteError {
                            reason: e.to_string(),
                        }
                        .into_error()
                    })?;
                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;
                }
            }
            Layout::HorizontalStack { left, right } => {
                let left_contents = left.get_content();
                let right_contents = right.get_content();

                for (r, row) in left_contents.into_iter().enumerate() {
                    queue!(stdout, cursor::MoveTo(1, r as u16 + 1)).map_err(|e| {
                        ErrorType::QueueExecuteError {
                            reason: e.to_string(),
                        }
                        .into_error()
                    })?;
                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;
                }

                queue!(stdout, style::ResetColor).map_err(|e| {
                    ErrorType::QueueExecuteError {
                        reason: e.to_string(),
                    }
                    .into_error()
                })?;

                for (r, row) in right_contents.into_iter().enumerate() {
                    queue!(
                        stdout,
                        cursor::MoveTo(left.get_size().get_cols() + 2, r as u16 + 1)
                    )
                    .map_err(|e| {
                        ErrorType::QueueExecuteError {
                            reason: e.to_string(),
                        }
                        .into_error()
                    })?;
                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;
                }

                Self::queue_vertical_centre_line(
                    &mut stdout,
                    &size,
                    left.get_size().get_cols() + 1,
                )?;
            }
            _ => (),
        }

        self.reset_cursor(&mut stdout, &size).map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        Self::reset_stdout_style(&mut stdout)?;

        return Ok(stdout.flush().map_err(|e| {
            ErrorType::StdoutFlushError {
                reason: format!("{}", e),
            }
            .into_error()
        })?);
    }

    fn get_terminal_size() -> Result<Size, MuxideError> {
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
    fn reset_cursor(&self, stdout: &mut Stdout, terminal_size: &Size) -> Result<(), MuxideError> {
        match &self.selected_panel {
            Some(panel) => {
                let loc = panel.get_cursor_position();

                queue!(
                    stdout,
                    cursor::MoveTo(loc.column(), loc.row()) // Column, row
                )
                .map_err(|e| {
                    ErrorType::QueueExecuteError {
                        reason: e.to_string(),
                    }
                    .into_error()
                })?;

                if panel.get_hide_cursor() {
                    execute!(stdout, cursor::Hide).map_err(|e| {
                        ErrorType::QueueExecuteError {
                            reason: e.to_string(),
                        }
                        .into_error()
                    })?;
                } else {
                    execute!(stdout, cursor::Show).map_err(|e| {
                        ErrorType::QueueExecuteError {
                            reason: e.to_string(),
                        }
                        .into_error()
                    })?;
                }
            }
            None => {
                execute!(
                    stdout,
                    cursor::Show,
                    cursor::MoveTo(
                        Self::PROMPT_STRING.len() as u16 + 1 + self.prompt_cursor_offset,
                        terminal_size.get_rows() - 2
                    ) // Column, row
                )
                .map_err(|e| {
                    ErrorType::QueueExecuteError {
                        reason: e.to_string(),
                    }
                    .into_error()
                })?;
            }
        }

        return Ok(());
    }

    /// Queues the outer border for display in stdout
    fn queue_main_borders(
        &self,
        stdout: &mut Stdout,
        terminal_size: &Size,
    ) -> Result<(), MuxideError> {
        Self::reset_stdout_style(stdout)?;

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
        )
        .map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        // Print the vertical borders
        for i in 1..terminal_size.get_rows() - 3 {
            queue!(
                stdout,
                cursor::MoveTo(0, i),
                style::Print(Self::VERTICAL_BORDER_CHARACTER),
                cursor::MoveTo(terminal_size.get_cols() - 1, i),
                style::Print(Self::VERTICAL_BORDER_CHARACTER),
            )
            .map_err(|e| {
                ErrorType::QueueExecuteError {
                    reason: e.to_string(),
                }
                .into_error()
            })?;
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
        )
        .map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        // Print the prompt and its content
        queue!(
            stdout,
            cursor::MoveTo(0, terminal_size.get_rows() - 2),
            style::Print(Self::VERTICAL_BORDER_CHARACTER),
            style::Print(Self::PROMPT_STRING),
            style::Print(&self.prompt_content),
            cursor::MoveTo(terminal_size.get_cols() - 1, terminal_size.get_rows() - 2),
            style::Print(Self::VERTICAL_BORDER_CHARACTER),
        )
        .map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

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
        )
        .map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        return Ok(());
    }

    fn queue_vertical_centre_line(
        stdout: &mut Stdout,
        terminal_size: &Size,
        col: u16,
    ) -> Result<(), MuxideError> {
        Self::reset_stdout_style(stdout)?;

        for r in 1..terminal_size.get_rows() - 3 {
            queue!(
                stdout,
                cursor::MoveTo(col, r),
                style::Print(Self::VERTICAL_BORDER_CHARACTER)
            )
            .map_err(|e| {
                ErrorType::QueueExecuteError {
                    reason: e.to_string(),
                }
                .into_error()
            })?;
        }

        return Ok(());
    }

    fn reset_stdout_style(stdout: &mut Stdout) -> Result<(), MuxideError> {
        queue!(stdout, style::ResetColor).map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        })?;

        return Ok(());
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
