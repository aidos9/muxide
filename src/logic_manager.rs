use crate::channel_controller::{ChannelController, ChannelID, PtyMessage, ServerMessage};
use crate::command::Command;
use crate::config::Config;
use crate::display::Display;
use crate::error::{ErrorType, MuxideError};
use crate::geometry::{Direction, Size};
use crate::hasher;
use crate::input_manager::InputManager;
use crate::pty::Pty;
use binary_set::BinaryTreeSet;
use muxide_logging::error;
use nix::poll;
use rand::Rng;
use std::os::unix::io::AsRawFd;
use termion::event::{self, Event};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use vt100::Parser;

/// The timeout used when we poll the PTY for if it is available.
const POLL_TIMEOUT_MS: i32 = 100;
/// THe timeout used when reporting an error.
const ERROR_TIMEOUT_MS: u64 = 100;
/// THe timeout used when writing to a file.
const FILE_TIMEOUT_MS: u64 = 750;

/// This method runs a pty, handling shutdown messages, stdin and stdout.
/// It should be spawned in a thread.
async fn pty_manager(mut p: Pty, tx: Sender<PtyMessage>, mut stdin_rx: Receiver<ServerMessage>) {
    macro_rules! pty_error {
        ($tx:expr, $e:expr, $log_message:expr) => {
            error!($log_message);


            // This could error out and if it does then we just assume the controller will deal with it.
            select! {
                _ = $tx.send(PtyMessage::Error($e)) => {},
                _ = tokio::time::sleep(Duration::from_millis(ERROR_TIMEOUT_MS)) => {},
            }
        };

        ($tx:expr, $e:expr) => {
            let e = $e;
            error!(format!(
                "An error occurred in the pty thread. Error description: {:?}",
                &e
            ));

            // This could error out and if it does then we just assume the controller will deal with it.
            select! {
                _ = $tx.send(PtyMessage::Error(e)) => {},
                _ = tokio::time::sleep(Duration::from_millis(ERROR_TIMEOUT_MS)) => {},
            }
        };
    };

    let pfd = poll::PollFd::new(p.as_raw_fd(), poll::PollFlags::POLLIN);

    loop {
        select! {
            res = tokio::spawn(async move {
                // For some reason rust reports that this value is unassigned.
                #[allow(unused_assignments)]
                let mut res = Ok(false);

                loop {
                    match poll::poll(&mut [pfd], POLL_TIMEOUT_MS) {
                        Ok(poll_response) => {
                            // If we get 0, that means the call timed out, a negative value is an error
                            // in my understanding but nix, I believe should handle that as an error
                            if poll_response > 0 {
                                //res = true;
                                res = Ok(true);
                                break;
                            }
                        }
                        Err(e) => {
                            // If we receive an error here, it is a first class (unrecoverable) error.
                            res = Err(e);
                            break;
                        },
                    }
                }

                res
            }) => {
                if res.is_err() {
                    pty_error!(tx, ErrorType::new_failed_read_poll_error(), "Something unexpected went wrong whilst reading the pty poll");
                    return;
                }

                match res.unwrap() {
                    Ok(b) => {
                        if !b {
                            continue;
                        }
                    }
                    Err(e) => {
                        pty_error!(tx, ErrorType::new_failed_read_poll_error(), format!("Failed to poll for available data. Error: {}", e));
                        return;
                    },
                }

                let mut buf = vec![0u8; 4096];
                let res = p.file().read(&mut buf).await;

                if let Ok(count) = res {
                    if count == 0 {
                        if p.running() == Some(false) {
                            pty_error!(tx, ErrorType::new_pty_stopped_running_error());
                            return;
                        }
                    }

                    let mut cpy = vec![0u8; count];
                    cpy.copy_from_slice(&buf[0..count]);

                    // Ignore any errors with communicating data.
                    match tx.send(PtyMessage::Bytes(cpy)).await {
                        Ok(_) => (),
                        Err(_) => {
                            pty_error!(tx, ErrorType::new_failed_to_send_message_error());
                            return;
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                } else {
                    pty_error!(tx, ErrorType::new_failed_to_read_pty_error());
                    return;
                }
            },
            res = stdin_rx.recv() => {
                if let Some(message) = res {
                    match message {
                        ServerMessage::Bytes(bytes) => {
                            select! {
                                res = p.file().write_all(&bytes) => {
                                    match res {
                                        Ok(_) => (),
                                        Err(e) => {
                                            pty_error!(tx, ErrorType::new_pty_write_error(e.to_string()));
                                            return;
                                        },
                                    }
                                },
                                _ = tokio::time::sleep(Duration::from_millis(FILE_TIMEOUT_MS)) => {},
                            }
                        },
                        ServerMessage::Resize(size) => {
                            p.resize(&size).unwrap();
                        },
                        ServerMessage::Shutdown => {
                            break;
                        },
                    }
                } else {
                    pty_error!(tx, ErrorType::new_pty_stdin_receiver_closed_error());
                    return;
                }
            }
        }
    }
}

/// Represents a panel, i.e. the output for a process. It tracks the contents being
/// displayed and assigns an id.
struct Panel {
    parser: Parser,
    id: usize,
    current_scrollback: usize,
}

/// Handles a majority of the overall application logic, i.e. receiving stdin input and the panel
/// outputs, managing the display and executing most commands.
pub struct LogicManager {
    display: Display,
    panels: Vec<Panel>,
    selected_panel: Option<usize>,
    halt_execution: bool,
    single_key_command: bool,
    config: Config,
    connection_manager: ChannelController,
    _input_manager: InputManager,
    close_handles: Vec<(usize, JoinHandle<()>)>,
    ids: BinaryTreeSet<usize>,
    hashed_password: Option<String>,
    password_input: String,
    locked: bool,
    displaying_help: bool,
}

impl LogicManager {
    /// The length of the scrollback history we track for each panel.
    const SCROLLBACK_LEN: usize = 120;

    /// Create a new instance of the logic manager from a config file.
    pub fn new(config: Config, hashed_password: Option<String>) -> Result<Self, MuxideError> {
        // Create a new channel controller with a stdin transmitter which we will use in the input
        // manager to send stdin input to the channel controller
        let (connection_manager, stdin_tx) = ChannelController::new();
        let input_manager = InputManager::start(stdin_tx)?;
        let display = match Display::new(config.clone()).init() {
            Some(d) => d,
            None => return Err(ErrorType::new_display_not_running_error()),
        };

        return Ok(Self {
            config,
            selected_panel: None,
            panels: Vec::new(),
            connection_manager,
            _input_manager: input_manager,
            display,
            ids: BinaryTreeSet::new(),
            halt_execution: false,
            close_handles: Vec::new(),
            single_key_command: false,
            password_input: String::new(),
            hashed_password,
            locked: false,
            displaying_help: false,
        });
    }

    /// Start the main event loop, essentially the main application logic.
    pub async fn start_event_loop(mut self) -> Result<(), String> {
        loop {
            if let Err(e) = self.display.render() {
                if e.should_terminate() {
                    self.shutdown().await;
                    break;
                } else {
                    self.display.set_error_message(e.description());
                }
            }

            let res = self.connection_manager.wait_for_message().await;

            match res {
                Ok(res) => {
                    if let ChannelID::Pty(id) = res.id {
                        self.handle_panel_output(id, res.bytes);
                    } else {
                        let displaying_help = self.displaying_help;

                        if let Err(e) = self.handle_stdin(res.bytes).await {
                            if e.should_terminate() {
                                self.shutdown().await;
                                break;
                            } else {
                                self.display.set_error_message(e.description());
                            }
                        } else {
                            if displaying_help {
                                self.displaying_help = false;
                                self.display.hide_help();
                            } else {
                                self.display.clear_error_message();
                            }
                        }
                    }
                }
                Err(details) => {
                    if let ChannelID::Pty(id) = details.id {
                        if let Err(e) = self.remove_panel(id) {
                            if e.should_terminate() {
                                self.shutdown().await;
                                break;
                            } else {
                                self.display.set_error_message(e.description());
                            }
                        }
                    } else {
                        self.shutdown().await;

                        if let Some(err) = details.error {
                            return Err(format!(
                                "The stdin thread was closed. Error details: {}.",
                                err
                            ));
                        } else {
                            return Err("The stdin thread was closed. An unknown error occurred."
                                .to_string());
                        }
                    }
                }
            }

            if self.halt_execution {
                self.shutdown().await;
                break;
            }
        }

        return Ok(());
    }

    async fn handle_stdin(&mut self, mut bytes: Vec<u8>) -> Result<(), MuxideError> {
        if bytes.is_empty() {
            return Ok(());
        }

        if self.single_key_command {
            let ch = bytes.remove(0) as char;
            self.single_key_command = false;

            let cmd = self.process_single_key_command(ch)?;
            self.execute_command(&cmd)?;
        }

        // If there was a number of bytes built-up deal with them still.
        if bytes.is_empty() {
            return Ok(());
        }

        let event = {
            let first = *bytes.first().unwrap();

            // For some reason, parse_event in termion doesn't support a single escape key.
            // So use this hack until we develop a custom processor.
            if first == 0x1b && bytes.len() == 1 {
                Event::Key(event::Key::Esc)
            } else {
                match event::parse_event(first, &mut bytes[1..bytes.len()].iter().map(|b| Ok(*b))) {
                    Ok(e) => e,
                    Err(e) => return Err(ErrorType::new_event_parsing_error(format!("{}", e))),
                }
            }
        };

        if !self.shortcut(&event)? {
            if self.locked {
                match event {
                    Event::Key(k) => match k {
                        event::Key::Backspace => {
                            self.password_input.pop();
                        }
                        event::Key::Char(ch) => {
                            if ch == '\n' {
                                self.check_password()?;
                            } else {
                                self.password_input.push(ch);
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }

                return Ok(());
            }

            match self.selected_panel {
                Some(id) => {
                    self.connection_manager.write_bytes(id, bytes).await?;
                    self.panel_with_id(id).unwrap().clear_scrollback();
                }
                None => (),
            }
        }

        return Ok(());
    }

    fn shortcut(&mut self, event: &Event) -> Result<bool, MuxideError> {
        if let Event::Key(k) = event {
            if let Some(k) = self
                .config
                .keys_ref()
                .command_for_shortcut(k)
                .map(|cmd| cmd.clone())
            {
                self.execute_command(&k)?;
                return Ok(true);
            } else {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }

    fn handle_panel_output(&mut self, id: usize, bytes: Vec<u8>) {
        let panel = self.panel_with_id(id).unwrap();

        panel.parser.process(&bytes);
        panel.clear_scrollback();

        self.update_panel_output(id);
    }

    fn update_panel_output(&mut self, id: usize) {
        let panel = self.panel_with_id(id).unwrap();

        let content = panel
            .parser
            .screen()
            .rows_formatted(0, panel.parser.screen().size().1)
            .collect();

        let (curs_row, curs_col) = panel.parser.screen().cursor_position();
        let cursor_hidden = panel.parser.screen().hide_cursor() || panel.current_scrollback != 0;

        self.display.update_panel_content(id, content).unwrap();

        self.display
            .update_panel_cursor(id, curs_col, curs_row, cursor_hidden);
    }

    fn open_new_panel(&mut self) -> Result<(), MuxideError> {
        // Checks for an available subdivision
        let (path, size, origin) = self.display.next_panel_details()?;

        let id = self.get_next_id();

        let (tx, stdin_rx) = self.connection_manager.new_channel(id);
        let pty = Pty::open(self.config.environment_ref().panel_init_command_ref())?;

        let new_sizes = self.display.open_new_panel(id, path, size, origin)?;
        let new_panel_size = new_sizes.last().unwrap().1;
        let parser = Parser::new(
            new_panel_size.get_rows(),
            new_panel_size.get_cols(),
            Self::SCROLLBACK_LEN,
        );

        self.display.update_panel_content(
            id,
            parser
                .screen()
                .rows_formatted(0, parser.screen().size().1)
                .collect(),
        )?;

        // Create a separate thread for interfacing with the new pty.
        let handle = tokio::spawn(async move {
            pty_manager(pty, tx, stdin_rx).await;
        });

        self.close_handles.push((id, handle));
        self.panels.push(Panel::new(id, parser));
        self.select_panel(Some(id));
        futures::executor::block_on(self.resize_panels(new_sizes)).unwrap();

        return Ok(());
    }

    fn close_panel(&mut self, id: usize) -> Result<(), MuxideError> {
        if self.panel_with_id(id).is_none() {
            return Err(ErrorType::new_no_panel_with_id_error(id));
        }

        futures::executor::block_on(self.connection_manager.send_shutdown(id));

        return self.remove_panel(id);
    }

    fn scroll_panel(&mut self, id: usize, up: bool) -> Result<(), MuxideError> {
        let lines = self.config.environment_ref().scroll_lines();

        if let Some(panel) = self.panel_with_id(id) {
            if up {
                panel.scroll_up(lines);
            } else {
                panel.scroll_down(lines);
            }

            return Ok(());
        } else {
            return Err(ErrorType::new_no_panel_with_id_error(id));
        }
    }

    /// This method is primarily used when a panel closes unexpectedly
    fn remove_panel(&mut self, id: usize) -> Result<(), MuxideError> {
        self.display.close_panel(id)?;

        for i in 0..self.close_handles.len() {
            if self.close_handles[i].0 == id {
                self.close_handles.remove(i);
                break;
            }
        }

        for i in 0..self.panels.len() {
            if self.panels[i].id == id {
                self.panels.remove(i);
                break;
            }
        }

        if let Some(sel_id) = self.selected_panel {
            if sel_id == id {
                self.select_panel(self.panels.first().map(|p| p.id));
            }
        }

        self.ids.remove(&id);

        return Ok(());
    }

    fn process_single_key_command(&self, character: char) -> Result<Command, MuxideError> {
        return self
            .config
            .keys_ref()
            .command_for_character(&character)
            .map(|cmd| cmd.clone())
            .ok_or(ErrorType::new_command_error(format!(
                "No command mapped to \'{}\'",
                character
            )));
    }

    fn execute_command(&mut self, cmd: &Command) -> Result<(), MuxideError> {
        if self.locked {
            return Err(ErrorType::new_display_locked_error());
        }

        match cmd {
            Command::QuitCommand => {
                self.halt_execution = true;
            }
            Command::OpenPanelCommand => {
                self.open_new_panel()?;
            }
            Command::EnterSingleCharacterCommand => {
                self.single_key_command = true;
            }
            Command::CloseSelectedPanelCommand => {
                if let Some(panel) = self.selected_panel {
                    self.close_panel(panel)?;
                }
            }
            Command::FocusWorkspaceCommand(id) => {
                self.selected_panel = self.display.switch_to_workspace(*id as u8)?;
            }
            Command::SubdivideSelectedVerticalCommand => {
                let new_sizes = self.display.subdivide_selected_panel_vertical()?;

                futures::executor::block_on(self.resize_panels(new_sizes))?;
            }
            Command::SubdivideSelectedHorizontalCommand => {
                let new_sizes = self.display.subdivide_selected_panel_horizontal()?;

                futures::executor::block_on(self.resize_panels(new_sizes))?;
            }
            Command::FocusPanelLeftCommand => {
                if let Some(id) = self.display.focus_direction(Direction::Left) {
                    self.selected_panel = Some(id);
                    self.display.set_selected_panel(Some(id));
                }
            }
            Command::FocusPanelRightCommand => {
                if let Some(id) = self.display.focus_direction(Direction::Right) {
                    self.selected_panel = Some(id);
                    self.display.set_selected_panel(Some(id));
                }
            }
            Command::FocusPanelUpCommand => {
                if let Some(id) = self.display.focus_direction(Direction::Up) {
                    self.selected_panel = Some(id);
                    self.display.set_selected_panel(Some(id));
                }
            }
            Command::FocusPanelDownCommand => {
                if let Some(id) = self.display.focus_direction(Direction::Down) {
                    self.selected_panel = Some(id);
                    self.display.set_selected_panel(Some(id));
                }
            }
            Command::LockCommand => {
                self.lock();
            }
            Command::MergePanelCommand => {
                if let Some(new_sizes) = self.display.merge_selected_panel()? {
                    futures::executor::block_on(self.resize_panels(vec![new_sizes]))?;
                }
            }
            Command::ScrollUpCommand => {
                if let Some(id) = self.selected_panel {
                    self.scroll_panel(id, true)?;
                    self.update_panel_output(id);
                }
            }
            Command::ScrollDownCommand => {
                if let Some(id) = self.selected_panel {
                    self.scroll_panel(id, false)?;
                    self.update_panel_output(id);
                }
            }
            Command::HelpMessageCommand => {
                self.displaying_help = true;
                self.display.show_help();
            }
        }

        return Ok(());
    }

    fn check_password(&mut self) -> Result<(), MuxideError> {
        if let Some(comp) = self.hashed_password.as_ref() {
            if hasher::check_password(
                &self.password_input,
                self.config.password_ref(),
                comp.as_str(),
            )
            .ok_or(ErrorType::new_failed_to_check_password_error())?
            {
                self.unlock();
            } else {
                self.password_input = String::new();
                return Err(ErrorType::new_invalid_password_error());
            }
        } else {
            self.unlock();
        }

        return Ok(());
    }

    fn unlock(&mut self) {
        self.display.unlock();
        self.locked = false;
        self.password_input = String::new();
    }

    fn lock(&mut self) {
        self.display.lock();
        self.locked = true;
    }

    async fn resize_panels(&mut self, panels: Vec<(usize, Size)>) -> Result<(), MuxideError> {
        for (id, size) in panels {
            let mut ok = false;

            for panel in &mut self.panels {
                if panel.id == id {
                    ok = true;

                    panel.parser.set_size(size.get_rows(), size.get_cols());
                    break;
                }
            }

            if !ok {
                return Err(ErrorType::new_no_panel_with_id_error(id));
            }

            self.connection_manager.write_resize(id, size).await?;
        }

        return Ok(());
    }

    async fn shutdown(self) {
        self.connection_manager.shutdown_all().await;
        //self.close_handles.pop().unwrap().await;
    }

    fn select_panel(&mut self, id: Option<usize>) {
        self.selected_panel = id;
        self.display.set_selected_panel(self.selected_panel);
    }

    fn panel_with_id(&mut self, id: usize) -> Option<&mut Panel> {
        for panel in &mut self.panels {
            if panel.id == id {
                return Some(panel);
            }
        }

        return None;
    }

    fn get_next_id(&mut self) -> usize {
        let mut rng = rand::thread_rng();
        let mut next_id: usize = rng.gen();

        while self.ids.contains(&next_id) {
            next_id = rng.gen();
        }

        return next_id;
    }
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
}
