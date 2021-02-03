use crate::channel_controller::{ChannelController, Message};
use crate::command::Command;
use crate::config::Config;
use crate::display::Display;
use crate::error::{ErrorType, MuxideError};
use crate::geometry::Size;
use crate::input_manager::InputManager;
use crate::pty::Pty;
use either::Either;
use nix::poll;
use std::os::unix::io::AsRawFd;
use termion::event;
use termion::event::{Event, Key};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use vt100::Parser;

const POLL_TIMEOUT_MS: i32 = 100;

/// This method runs a pty, handling shutdown messages, stdin and stdout.
/// It should be spawned in a thread.
async fn pty_manager(mut p: Pty, tx: Sender<Vec<u8>>, mut stdin_rx: Receiver<Message>) {
    //TODO: Better error handling
    let pfd = poll::PollFd::new(p.as_raw_fd(), poll::PollFlags::POLLIN);

    loop {
        select! {
            b = tokio::spawn(async move {
                let mut res = false;

                loop {
                    if poll::poll(&mut [pfd], POLL_TIMEOUT_MS).unwrap() != 0 {
                        res = true;
                        break;
                    }
                }

                res
            }) => {
                if !b.unwrap() {
                    continue;
                }

                let mut buf = vec![0u8; 4096];
                let res = p.file().read(&mut buf).await;

                if let Ok(count) = res {
                    if count == 0 {
                        if p.running() == Some(false) {
                            break;
                        }
                    }

                    let mut cpy = vec![0u8; count];
                    cpy.copy_from_slice(&buf[0..count]);

                    tx.send(cpy).await.unwrap();

                    tokio::time::sleep(Duration::from_millis(5)).await;
                } else {
                    panic!("{:?}", res);
                    break;
                }
            },
            res = stdin_rx.recv() => {
                if let Some(message) = res {
                    match message {
                        Message::Bytes(bytes) => {
                            // TODO: This should timeout

                            p.file().write_all(&bytes).await.unwrap();
                        },
                        Message::Resize(size) => {
                            p.resize(&size).unwrap();
                        },
                        Message::Shutdown => {
                            break;
                        }
                    }
                } else {
                    panic!("{:?}", res);
                    break;
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
}

/// Handles a majority of the overall application logic, i.e. receiving stdin input and the panel
/// outputs, managing the display and executing most commands.
pub struct LogicManager {
    config: Config,
    cmd_buffer: Vec<char>,
    selected_panel: Option<usize>,
    panels: Vec<Panel>,
    connection_manager: ChannelController,
    input_manager: InputManager,
    display: Display,
    next_panel_id: usize,
    halt_execution: bool,
    close_handles: Vec<(usize, JoinHandle<()>)>,
}

impl LogicManager {
    /// The length of the scrollback history we track for each panel.
    const SCROLLBACK_LEN: usize = 120;

    /// Create a new instance of the logic manager from a config file.
    pub fn new(config: Config) -> Result<Self, MuxideError> {
        // Create a new channel controller with a stdin transmitter which we will use in the input
        // manager to send stdin input to the channel controller
        let (connection_manager, stdin_tx) = ChannelController::new();
        let input_manager = InputManager::start(stdin_tx)?;
        let display = match Display::new().init() {
            Some(d) => d,
            None => return Err(ErrorType::DisplayNotRunningError.into_error()),
        };

        return Ok(Self {
            config,
            cmd_buffer: Vec::new(),
            selected_panel: None,
            panels: Vec::new(),
            connection_manager,
            input_manager,
            display,
            next_panel_id: 0,
            halt_execution: false,
            close_handles: Vec::new(),
        });
    }

    /// Start the main event loop, essentially the main application logic.
    pub async fn start_event_loop(mut self) {
        loop {
            self.display.render().unwrap();

            let res = self.connection_manager.wait_for_message().await;

            match res {
                Either::Left(res) => {
                    if res.id.is_none() {
                        self.handle_stdin(res.bytes).await;
                    } else {
                        self.handle_panel_output(res.id.unwrap(), res.bytes);
                    }
                }
                Either::Right(id) => {
                    if id.is_none() {
                        panic!("The stdin thread has closed. An unknown error occurred.");
                    } else {
                        self.remove_panel(id.unwrap());
                    }
                }
            }

            if self.halt_execution {
                self.shutdown().await;
                break;
            }
        }
    }

    async fn handle_stdin(&mut self, bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }

        let event = match event::parse_event(
            *bytes.first().unwrap(),
            &mut bytes[1..bytes.len()].iter().map(|b| Ok(*b)),
        ) {
            Ok(e) => e,
            Err(_) => return,
        };

        if self.shortcut(&event) {
            return;
        } else {
            match self.selected_panel {
                Some(id) => {
                    self.connection_manager
                        .write_bytes(id, bytes)
                        .await
                        .unwrap();
                }
                None => self.handle_cmd_input(event),
            }
        }
    }

    fn shortcut(&mut self, event: &Event) -> bool {
        if let Event::Key(k) = event {
            if let Some(k) = self
                .config
                .key_map()
                .command_for_key(k)
                .map(|cmd| cmd.clone())
            {
                self.execute_command(&k);
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    /// Handles input that is intended for the command prompt
    fn handle_cmd_input(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key == Key::Char('\n') {
                //self.process_command();
                self.cmd_buffer.clear();
                self.display.set_cmd_offset(0);
            } else if key == Key::Backspace && !self.cmd_buffer.is_empty() {
                self.cmd_buffer.pop();
                self.display.sub_cmd_offset(1);
            } else if let Key::Char(ch) = key {
                self.cmd_buffer.push(ch);
                self.display.add_cmd_offset(1);
            }

            self.display
                .set_cmd_content(self.cmd_buffer.iter().collect());
        }
    }

    fn handle_panel_output(&mut self, id: usize, bytes: Vec<u8>) {
        let panel = self.panel_with_id(id).unwrap();

        panel.parser.process(&bytes);

        let content = panel
            .parser
            .screen()
            .rows_formatted(0, panel.parser.screen().size().1)
            .collect();

        let (curs_row, curs_col) = panel.parser.screen().cursor_position();
        let cursor_hidden = panel.parser.screen().hide_cursor();

        self.display.update_panel_content(id, content).unwrap();
        self.display
            .update_panel_cursor(id, curs_col, curs_row, cursor_hidden);
    }

    fn open_new_panel(&mut self) -> Result<(), MuxideError> {
        let id = self.get_next_id();
        let (tx, stdin_rx) = self.connection_manager.new_channel(id);
        let pty = Pty::open(self.config.get_panel_init_command())?;

        let mut new_sizes = self.display.open_new_panel(id)?;
        let new_panel_size = new_sizes.pop().unwrap().1;
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
        self.panels.push(Panel { parser, id });
        self.select_panel(Some(id));
        futures::executor::block_on(self.resize_panels(new_sizes)).unwrap();

        return Ok(());
    }

    fn remove_panel(&mut self, id: usize) -> Result<(), MuxideError> {
        let mut new_sizes = self.display.close_panel(id)?;

        for i in 0..self.close_handles.len() {
            if self.close_handles[i].0 == id {
                self.close_handles.remove(i);
                break;
            }
        }

        if let Some(sel_id) = self.selected_panel {
            if sel_id == id {
                self.select_panel(None);
            }
        }

        for i in 0..self.panels.len() {
            if self.panels[i].id == id {
                self.panels.remove(i);
                break;
            }
        }

        futures::executor::block_on(self.resize_panels(new_sizes)).unwrap();

        return Ok(());
    }

    fn execute_command(&mut self, cmd: &Command) {
        match cmd {
            Command::QuitCommand => {
                self.halt_execution = true;
            }
            Command::ToggleInputCommand => {
                if self.selected_panel.is_some() {
                    self.select_panel(None);
                } else {
                    self.select_panel(self.panels.first().map(|p| p.id));
                }
            }
            Command::OpenPanelCommand => {
                self.open_new_panel().unwrap();
            }
            _ => unimplemented!(),
        }
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
                return Err(ErrorType::NoPanelWithIDError { id }.into_error());
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

    fn get_cmd_buffer_string(&self) -> String {
        return self.cmd_buffer.iter().collect();
    }

    fn clear_buffer(&mut self) {
        self.cmd_buffer.clear();
    }

    fn get_next_id(&mut self) -> usize {
        self.next_panel_id += 1;
        return self.next_panel_id - 1;
    }
}
