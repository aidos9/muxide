use super::subdivision::{SubDivision, SubDivisionSplit};
use super::workspace::Workspace;
use super::{panel::PanelPtr, subdivision::SubdivisionPath};
use crate::geometry::{Point, Size};
use crate::{
    error::{ErrorType, MuxideError},
    geometry::Direction,
};
use crate::{Color, Config};
use crossterm::style::Color as CrosstermColor;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, style, terminal};
use std::{
    collections::HashMap,
    io::{stdout, Stdout, Write},
};

const LOCK_SYMBOL: [&'static str; 13] = [
    "     .--------.",
    "    / .------. \\",
    "   / /        \\ \\",
    "   | |        | |",
    "  _| |________| |_",
    ".' |_|        |_| '.",
    "'._____ ____ _____.'",
    "|     .'____'.     |",
    "'.__.'.'    '.'.__.'",
    "'.__  |      |  __.'",
    "|   '.'.____.'.'   |",
    "'.____'.____.'____.'",
    "'.________________.'",
];

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
    panel_map: HashMap<usize, PanelPtr>, // id, panel
    workspaces: Vec<Workspace>,
    selected_workspace: u8,
    completed_initialization: bool,
    error_message: Option<String>,
    is_locked: bool,
}

impl Display {
    const ERROR_COLOR: Color = Color::new(255, 105, 97);

    /// Create a new "display" instance.
    pub fn new(config: Config) -> Self {
        return Self {
            config,
            panel_map: HashMap::new(),
            workspaces: vec![Workspace::new(); 10],
            completed_initialization: false,
            selected_workspace: 0,
            error_message: None,
            is_locked: false,
        };
    }

    /// Initializes the terminal for output by taking control of the stdout and clearing the
    /// terminal. This must be run before any other methods are.
    pub fn init(mut self) -> Option<Self> {
        let origin = if self.config.get_environment_ref().show_workspaces() {
            Point::new(0, 2)
        } else {
            Point::new(0, 0)
        };

        let dimensions = if self.config.get_environment_ref().show_workspaces() {
            Self::get_terminal_size().ok()? - Size::new(2, 0)
        } else {
            Self::get_terminal_size().ok()?
        };

        for workspace in &mut self.workspaces {
            workspace.root_subdivision = SubDivision::new(origin, dimensions);
        }

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

    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    pub fn unlock(&mut self) {
        self.is_locked = false;
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

        if let Some(panel) = self.panel_map.get_mut(&id) {
            panel.set_content(content);
            return Ok(());
        } else {
            return Err(ErrorType::NoPanelWithIDError { id }.into_error());
        }
    }

    pub fn next_panel_details(&self) -> Result<(SubdivisionPath, Size, Point<u16>), MuxideError> {
        return self
            .root_subdivision()
            .next_panel_details()
            .ok_or(ErrorType::NoAvailableSubdivision.into_error());
    }

    /// Opens a new panel giving it the specified id. The id should be unique but it is
    /// not enforced by this method. The method will return a vector of all the changed panels
    /// id's and new size.
    pub fn open_new_panel(
        &mut self,
        id: usize,
        panel_path: SubdivisionPath,
        size: Size,
        origin: Point<u16>,
    ) -> Result<Vec<(usize, Size)>, MuxideError> {
        if !self.completed_initialization {
            return Err(ErrorType::DisplayNotRunningError.into_error());
        }

        let panel = self.init_panel(id, (origin.column(), origin.row()));

        self.root_subdivision_mut()
            .open_panel_at_path(panel, panel_path)?;

        return Ok(vec![(id, size)]);
    }

    pub fn close_panel(&mut self, id: usize) -> Result<(), MuxideError> {
        if !self.completed_initialization {
            return Err(ErrorType::DisplayNotRunningError.into_error());
        }

        if !self.root_subdivision_mut().close_panel_with_id(id) {
            panic!("No panel with an id: {}", id);
        } else {
            if let Some(panel) = self.selected_panel() {
                if panel.get_id() == id {
                    self.selected_workspace_mut().selected_panel =
                        self.selected_workspace().panels.first().map(|p| p.clone());
                }
            }

            self.panel_map.remove(&id);

            return Ok(());
        }
    }

    /// Subdivide the currently selected panel into two panels split with a vertical line down the middle
    pub fn subdivide_selected_panel_vertical(&mut self) -> Result<Vec<(usize, Size)>, MuxideError> {
        return self.subdivide_selected_panel(SubDivisionSplit::Vertical);
    }

    /// Subdivide the currently selected panel into two panels split with a horizontal line down the middle
    pub fn subdivide_selected_panel_horizontal(
        &mut self,
    ) -> Result<Vec<(usize, Size)>, MuxideError> {
        return self.subdivide_selected_panel(SubDivisionSplit::Horizontal);
    }

    pub fn focus_direction(&mut self, direction: Direction) -> Option<usize> {
        let id = self.selected_panel().map(|p| p.get_id())?;
        return self.root_subdivision_mut().focus_next_id(id, direction);
    }

    /// Returns the index of the newly selected panel.
    pub fn switch_to_workspace(&mut self, workspace: u8) -> Result<Option<usize>, MuxideError> {
        if workspace >= 10 {
            return Err(ErrorType::NoWorkspaceWithID(workspace as usize).into_error());
        }

        self.selected_workspace = workspace;
        return Ok(self.selected_panel().map(|p| p.get_id()));
    }

    /// Subdivide the currently selected panel into two panels split with the specified line down the middle
    fn subdivide_selected_panel(
        &mut self,
        direction: SubDivisionSplit,
    ) -> Result<Vec<(usize, Size)>, MuxideError> {
        let id = self.selected_panel().map(|p| p.get_id());
        let (sz, success) = self.root_subdivision_mut().split_panel(id, direction);

        if !success {
            return Err(ErrorType::FailedSubdivision.into_error());
        }

        return Ok(if let Some(sz) = sz {
            vec![(self.selected_panel().unwrap().get_id(), sz)]
        } else {
            Vec::new()
        });
    }

    // Initialise a panel by creating a new instance and copying the pointer into the internal tracker. Location: (col, row).
    fn init_panel(&mut self, id: usize, location: (u16, u16)) -> PanelPtr {
        let panel = PanelPtr::new(id, location);

        self.panel_map.insert(id, panel.clone());

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

        if self.is_locked {
            Self::render_locked(&mut stdout, &size)?;
        } else {
            self.queue_main_borders(&mut stdout, &size)?;

            self.root_subdivision().render(&mut stdout, &self.config)?;
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

    fn render_locked(stdout: &mut Stdout, size: &Size) -> Result<(), MuxideError> {
        let starting_row = (size.get_rows() - LOCK_SYMBOL.len() as u16) / 2;
        let starting_col = (size.get_cols() - LOCK_SYMBOL[LOCK_SYMBOL.len() - 1].len() as u16) / 2;

        queue_map_err!(stdout, style::ResetColor)?;

        for i in 0..LOCK_SYMBOL.len() as u16 {
            queue_map_err!(
                stdout,
                cursor::MoveTo(starting_col, starting_row + i),
                style::Print(LOCK_SYMBOL[i as usize])
            )?;
        }

        return Ok(());
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
        if self.is_locked {
            execute!(stdout, cursor::Hide, cursor::MoveTo(0, 0)).map_err(|e| {
                ErrorType::QueueExecuteError {
                    reason: e.to_string(),
                }
                .into_error()
            })?;

            return Ok(());
        }

        match self.selected_panel() {
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
    ) -> Result<(), MuxideError> {
        let horizontal_character = self.config.get_borders_ref().get_horizontal_char();
        let intersection_character = self.config.get_borders_ref().get_intersection_char();
        let vertical_character = self.config.get_borders_ref().get_vertical_char();

        Self::reset_stdout_style(stdout)?;

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

        Self::reset_stdout_style(stdout)?;

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

    fn selected_workspace(&self) -> &Workspace {
        return self
            .workspaces
            .get(self.selected_workspace as usize)
            .unwrap();
    }

    fn selected_workspace_mut(&mut self) -> &mut Workspace {
        return self
            .workspaces
            .get_mut(self.selected_workspace as usize)
            .unwrap();
    }

    fn selected_panel(&self) -> Option<&PanelPtr> {
        return self.selected_workspace().selected_panel.as_ref();
    }

    fn root_subdivision(&self) -> &SubDivision {
        return &self.selected_workspace().root_subdivision;
    }

    fn root_subdivision_mut(&mut self) -> &mut SubDivision {
        return &mut self.selected_workspace_mut().root_subdivision;
    }

    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error_message(&mut self) {
        self.error_message = None;
    }

    pub fn set_selected_panel(&mut self, id: Option<usize>) {
        if id.is_none() {
            self.selected_workspace_mut().selected_panel = None;
            return;
        }

        let id = id.unwrap();

        self.selected_workspace_mut().selected_panel = self.panel_map.get(&id).map(|p| p.clone());
    }

    pub fn update_panel_cursor(&mut self, id: usize, col: u16, row: u16, hide: bool) -> bool {
        if let Some(panel) = self.panel_map.get_mut(&id) {
            panel.set_cursor_position(col, row);
            panel.set_hide_cursor(hide);
            return true;
        } else {
            return false;
        }
    }
}
