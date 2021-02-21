use super::layout::Layout;
use super::panel::PanelPtr;
use crate::error::{ErrorType, MuxideError};
use crate::geometry::Size;
use crate::{Color, Config};
use crossterm::style::Color as CrosstermColor;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{stdout, Stdout, Write};

macro_rules! queue_map_err {
    ($($v:expr),*) => {
        queue!($($v),*).map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        });
    };
}

/// Manages the different panels and renders to the terminal the correct output and layout.
pub struct Display {
    config: Config,
    panels: Vec<PanelPtr>,
    selected_panel: Option<PanelPtr>,
    layout: Layout,
    completed_initialization: bool,
    selected_workspace: u8,
    error_message: Option<String>,
}

impl Display {
    /// The text that is displayed when there are no open panels.
    const EMPTY_TEXT: &'static str = "No Panels Open";
    const ERROR_COLOR: Color = Color::new(255, 105, 97);

    /// Create a new "display" instance.
    pub fn new(config: Config) -> Self {
        return Self {
            config,
            layout: Layout::Empty,
            panels: Vec::new(),
            selected_panel: None,
            completed_initialization: false,
            selected_workspace: 0,
            error_message: None,
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
                let size = Self::get_terminal_size()? - Size::new(2, 0);
                let panel = self.init_panel(id, size, (0, 2)); // (col, row)

                self.selected_panel = Some(panel.clone());
                changed.push((id, size));

                Layout::Single { panel }
            }
            Layout::Single { panel } => {
                // -1 cols (center border)
                // -2 rows (workspaces, workspaces bottom border)
                let term_size = Self::get_terminal_size()?;
                let size = term_size - Size::new(2, 1);
                let mut left_size = size;
                left_size.divide_width_by_const(2);
                let right_size = Size::new(size.get_rows(), size.get_cols() - left_size.get_cols());

                let mut left = panel.clone(); // The location stays the same but the size needs to be re-adjusted
                left.set_size(left_size.clone());
                let right = self.init_panel(id, right_size, (2 + left_size.get_cols(), 2)); // + 2 to account for the center border

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
                let size = Self::get_terminal_size()?
                    - (if self.config.get_environment_ref().show_workspaces() {
                        Size::new(2, 0)
                    } else {
                        Size::new(0, 0)
                    });
                let mut panel;

                if left.get_id() == id {
                    panel = right.clone();
                } else if right.get_id() == id {
                    panel = left.clone();
                } else {
                    return Err(ErrorType::NoPanelWithIDError { id }.into_error());
                }

                panel.set_size(size);
                panel.set_location((0, 2)); // (col, row)

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

    /// Switch between the two possible 2-panel layouts.
    pub fn swap_layout(&mut self) -> Result<Vec<(usize, Size)>, MuxideError> {
        if self.layout.is_horizontal_stack() {
            return self.change_layout_vertical();
        } else if self.layout.is_vertical_stack() {
            return self.change_layout_horizontal();
        } else {
            return Ok(Vec::new());
        }
    }

    fn change_layout_vertical(&mut self) -> Result<Vec<(usize, Size)>, MuxideError> {
        let changed;

        let new_layout = match &mut self.layout {
            Layout::HorizontalStack { left, right } => {
                let term_size = Self::get_terminal_size()?;
                let size = term_size - Size::new(3, 0);
                let mut upper_size = size;
                upper_size.divide_height_by_const(2);
                let lower_size =
                    Size::new(size.get_rows() - upper_size.get_rows(), size.get_cols());

                changed = vec![(left.get_id(), upper_size), (right.get_id(), lower_size)];

                left.set_size(upper_size);
                right.set_size(lower_size);
                right.set_location((0, 3 + upper_size.get_rows()));

                Layout::VerticalStack {
                    upper: left.clone(),
                    lower: right.clone(),
                }
            }
            _ => return Ok(Vec::new()),
        };

        self.layout = new_layout;

        return Ok(changed);
    }

    fn change_layout_horizontal(&mut self) -> Result<Vec<(usize, Size)>, MuxideError> {
        todo!();
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

        self.queue_main_borders(
            &mut stdout,
            &size,
            self.layout.requires_vertical_line(),
            self.layout.requires_horizontal_line(),
        )?;

        match &mut self.layout {
            Layout::Empty => {
                let (mut col, mut row) = (size.get_cols(), size.get_rows());

                // Determine the center
                col /= 2;
                // Align the empty text to the center
                col -= Self::EMPTY_TEXT.len() as u16 / 2;

                if self.config.get_environment_ref().show_workspaces() {
                    // -2 for the workspaces bar
                    row -= 2;
                }

                // Determine the center
                row /= 2;
                // Subtract 1 for the height of the text
                row -= 1;

                // Add 1 to offset by the left and top borders. Obviously it is useless having
                // the + and - operations that cancel each other but for clarity's sake they have
                // been used.
                queue!(
                    stdout,
                    cursor::MoveTo(col + 1, row + 1),
                    style::Print(Self::EMPTY_TEXT)
                )
                .map_err(|e| {
                    ErrorType::QueueExecuteError {
                        reason: e.to_string(),
                    }
                    .into_error()
                })?;
            }
            Layout::Single { panel } => {
                let contents = panel.get_content();

                for (r, row) in contents.into_iter().enumerate() {
                    queue_map_err!(stdout, cursor::MoveTo(0, r as u16 + 2))?;

                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;

                    queue_map_err!(stdout, style::ResetColor)?;
                }
            }
            Layout::HorizontalStack { left, right } => {
                let left_contents = left.get_content();
                let right_contents = right.get_content();

                for (r, row) in left_contents.into_iter().enumerate() {
                    queue_map_err!(stdout, cursor::MoveTo(0, r as u16 + 2))?;

                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;

                    queue_map_err!(stdout, style::ResetColor)?;
                }

                queue_map_err!(stdout, style::ResetColor)?;

                for (r, row) in right_contents.into_iter().enumerate() {
                    queue_map_err!(
                        stdout,
                        cursor::MoveTo(left.get_size().get_cols() + 2, r as u16 + 2)
                    )?;

                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;

                    queue_map_err!(stdout, style::ResetColor)?;
                }
            }
            Layout::VerticalStack { lower, upper } => {
                let lower_contents = lower.get_content();
                let upper_contents = upper.get_content();

                for (r, row) in upper_contents.into_iter().enumerate() {
                    queue_map_err!(stdout, cursor::MoveTo(0, r as u16 + 2))?;

                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;

                    queue_map_err!(stdout, style::ResetColor)?;
                }

                queue_map_err!(stdout, style::ResetColor)?;

                for (r, row) in lower_contents.into_iter().enumerate() {
                    queue_map_err!(
                        stdout,
                        cursor::MoveTo(0, upper.get_size().get_rows() + 3 + r as u16)
                    )?;

                    stdout
                        .write(&row)
                        .map_err(|e| ErrorType::new_display_qe_error(e))?;

                    queue_map_err!(stdout, style::ResetColor)?;
                }
            }
            _ => (),
        }

        if self.error_message.is_some() {
            self.queue_error_message(&mut stdout, &size).map_err(|e| {
                ErrorType::QueueExecuteError {
                    reason: e.to_string(),
                }
                .into_error()
            })?;
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
    fn reset_cursor(&self, stdout: &mut Stdout, _terminal_size: &Size) -> Result<(), MuxideError> {
        match &self.selected_panel {
            Some(panel) => {
                let loc = panel.get_cursor_position();

                queue_map_err!(
                    stdout,
                    cursor::MoveTo(loc.column(), loc.row()) // Column, row
                )?;

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
                execute!(stdout, cursor::Hide, cursor::MoveTo(0, 0)).map_err(|e| {
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
        vertical_line: bool,
        horizontal_line: bool,
    ) -> Result<(), MuxideError> {
        let horizontal_character = self.config.get_borders_ref().get_horizontal_char();
        let intersection_character = self.config.get_borders_ref().get_intersection_char();
        let vertical_character = self.config.get_borders_ref().get_vertical_char();

        Self::reset_stdout_style(stdout)?;

        let center_col = terminal_size.get_cols() / 2;
        let center_row = if self.config.get_environment_ref().show_workspaces() {
            (terminal_size.get_rows() - 2) / 2 + 1
        } else {
            terminal_size.get_rows() / 2
        };

        if self.config.get_environment_ref().show_workspaces() {
            // Print the workspaces
            self.queue_workspaces_line(
                stdout,
                (0, 0),
                self.selected_workspace as u16,
                terminal_size.get_cols(),
                vertical_character,
            )
            .map_err(|e| {
                ErrorType::QueueExecuteError {
                    reason: e.to_string(),
                }
                .into_error()
            })?;

            // Print the bottom row

            if vertical_line {
                queue_map_err!(
                    stdout,
                    cursor::MoveTo(0, 1),
                    style::Print(intersection_character),
                    style::Print(
                        horizontal_character
                            .to_string()
                            .repeat(center_col as usize - 1)
                    ),
                    style::Print(intersection_character),
                    style::Print(
                        horizontal_character
                            .to_string()
                            .repeat((terminal_size.get_cols() as usize - 2) - center_col as usize)
                    ),
                    style::Print(intersection_character)
                )?;
            } else {
                queue_map_err!(
                    stdout,
                    cursor::MoveTo(0, 1),
                    style::Print(intersection_character),
                    style::Print(
                        horizontal_character
                            .to_string()
                            .repeat(terminal_size.get_cols() as usize - 2)
                    ),
                    style::Print(intersection_character)
                )?;
            }
        }

        if vertical_line {
            self.queue_vertical_centre_line(
                stdout,
                terminal_size,
                center_col,
                if self.config.get_environment_ref().show_workspaces() {
                    2
                } else {
                    0
                },
            )?;
        }

        if horizontal_line {
            self.queue_horizontal_line(
                stdout,
                terminal_size,
                center_row,
                if vertical_line {
                    Some(center_col)
                } else {
                    None
                },
            )?;
        }

        Self::reset_stdout_style(stdout)?;

        return Ok(());
    }

    fn queue_vertical_centre_line(
        &self,
        stdout: &mut Stdout,
        terminal_size: &Size,
        col: u16,
        starting_row: u16,
    ) -> Result<(), MuxideError> {
        let vertical_character = self.config.get_borders_ref().get_vertical_char();

        Self::reset_stdout_style(stdout)?;

        for r in starting_row..terminal_size.get_rows() {
            queue_map_err!(
                stdout,
                cursor::MoveTo(col, r),
                style::Print(vertical_character)
            )?;
        }

        return Ok(());
    }

    fn queue_horizontal_line(
        &self,
        stdout: &mut Stdout,
        terminal_size: &Size,
        row: u16,
        center_intersection: Option<u16>,
    ) -> Result<(), MuxideError> {
        let horizontal_character = self.config.get_borders_ref().get_horizontal_char();
        let intersection_character = self.config.get_borders_ref().get_intersection_char();

        Self::reset_stdout_style(stdout)?;

        if let Some(col) = center_intersection {
            queue_map_err!(
                stdout,
                cursor::MoveTo(0, row),
                style::Print(horizontal_character.to_string().repeat(col as usize)),
                style::Print(intersection_character),
                style::Print(
                    horizontal_character
                        .to_string()
                        .repeat((terminal_size.get_cols() - col - 1) as usize)
                )
            )?;
        } else {
            queue_map_err!(
                stdout,
                cursor::MoveTo(0, row),
                style::Print(
                    horizontal_character
                        .to_string()
                        .repeat(terminal_size.get_cols() as usize)
                )
            )?;
        }

        return Ok(());
    }

    fn queue_workspaces_line(
        &self,
        stdout: &mut Stdout,
        location: (u16, u16),
        selected_workspace: u16,
        width: u16,
        vertical_character: char,
    ) -> Result<(), crossterm::ErrorKind> {
        // Each workspace cell is 3 character ([1]), plus 1 for spacing, subtract 1 for the last
        // space and add 2 to account for the two border characters.
        // Should look like this:
        // | [1] [2] [3]         |
        // or
        // | [1] [2] [3] [4] ... [10] |
        queue!(stdout, cursor::MoveTo(location.0, location.1))?;
        let selected_color = self
            .config
            .get_environment_ref()
            .selected_workspace_color()
            .crossterm_color(crossterm::style::Color::White);

        if width == 0 {
            queue!(stdout, style::Print(""))?;
        } else if width == 1 {
            queue!(stdout, style::Print(" "))?;
        } else if width < 7 {
            queue!(stdout, style::Print(vertical_character))?;
            queue!(
                stdout,
                style::Print((0..width - 2).map(|_| ' ').collect::<String>())
            )?;
            queue!(stdout, style::Print(vertical_character))?;
        } else if width < 43 {
            queue!(stdout, style::Print(vertical_character))?;
            queue!(
                stdout,
                style::Print(vertical_character),
                style::Print(' '),
                style::SetBackgroundColor(selected_color),
                style::Print(format!("[{}]", selected_workspace)),
                style::ResetColor
            )?;

            if width > 7 {
                queue!(
                    stdout,
                    style::Print((0..(width as usize - 7)).map(|_| ' ').collect::<String>())
                )?;
            }

            queue!(stdout, style::Print(' '))?;
            queue!(stdout, style::Print(vertical_character))?;
        } else {
            queue!(stdout, style::Print(vertical_character))?;

            for i in 0..10 {
                if i == selected_workspace {
                    queue!(
                        stdout,
                        style::Print(' '),
                        style::SetBackgroundColor(selected_color),
                        style::Print(format!("[{}]", selected_workspace)),
                        style::ResetColor
                    )?;
                } else {
                    queue!(stdout, style::Print(format!(" [{}]", i)))?;
                }
            }

            if width > 43 {
                queue!(
                    stdout,
                    style::Print((0..(width as usize - 43)).map(|_| ' ').collect::<String>())
                )?;
            }

            queue!(stdout, style::Print(' '))?;
            queue!(stdout, style::Print(vertical_character))?;
        }

        return Ok(());
    }

    fn queue_error_message(
        &self,
        stdout: &mut Stdout,
        terminal_size: &Size,
    ) -> Result<(), crossterm::ErrorKind> {
        if let Some(text) = self.error_message.as_ref() {
            let error_text;

            if text.len() > terminal_size.get_cols() as usize {
                error_text = format!(
                    "{}...",
                    text.chars().collect::<Vec<char>>()[..terminal_size.get_cols() as usize - 3]
                        .iter()
                        .collect::<String>()
                );
            } else {
                let lhs = (terminal_size.get_cols() as usize - text.len()) / 2;
                error_text = format!(
                    "{}{}{}",
                    (0..lhs).map(|_| ' ').collect::<String>(),
                    text,
                    (0..terminal_size.get_cols() as usize - text.len() - lhs)
                        .map(|_| ' ')
                        .collect::<String>(),
                );
            }

            queue!(
                stdout,
                cursor::MoveTo(0, terminal_size.get_rows()),
                style::SetBackgroundColor(Self::ERROR_COLOR.crossterm_color(CrosstermColor::Red)),
                style::SetForegroundColor(CrosstermColor::White),
                style::Print(error_text),
            )?;
        }

        return Ok(());
    }

    fn reset_stdout_style(stdout: &mut Stdout) -> Result<(), MuxideError> {
        queue_map_err!(stdout, style::ResetColor)?;

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

    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error_message(&mut self) {
        self.error_message = None;
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
