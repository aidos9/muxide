use crate::channel_controller::ChannelController;
use crate::config::{Command, Config};
use crate::display::Display;
use crate::error::{Error, ErrorType};
use crate::input_manager::InputManager;
use crate::pty::Pty;
use termion::event;
use termion::event::Event;
use tokio::io::AsyncReadExt;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use vt100::Parser;

async fn pty_manager(mut r: Pty, tx: Sender<Vec<u8>>, mut shutdown_rx: Receiver<()>) {
    let mut buf = vec![0u8; 4096];

    loop {
        select! {
            res = r.read(&mut buf) => {
                if let Ok(count) = res {
                    if count == 0 {
                        if r.running() == Some(false) {
                            break;
                        }
                    }

                    let mut cpy = vec![0u8; count];
                    cpy.copy_from_slice(&buf[0..count]);

                    tx.send(cpy).await;

                    tokio::time::sleep(Duration::from_millis(5)).await;
                } else {
                    panic!("{:?}", res);
                    break;
                }
            }

            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
}

struct Panel {
    parser: Parser,
    id: usize,
}

/// Handles a majority of command parsing and config logic
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
}

impl LogicManager {
    const SCROLLBACK_LEN: usize = 120;

    pub fn new() -> Result<Self, Error> {
        let (connection_manager, stdin_tx) = ChannelController::new();
        let input_manager = InputManager::start(stdin_tx)?;
        let display = match Display::new().init() {
            Some(d) => d,
            None => return Err(ErrorType::DisplayNotRunningError.into_error()),
        };

        return Ok(Self {
            config: Config::new(),
            cmd_buffer: Vec::new(),
            selected_panel: None,
            panels: Vec::new(),
            connection_manager,
            input_manager,
            display,
            next_panel_id: 0,
            halt_execution: false,
        });
    }

    pub fn process_config_file(&mut self, file: &str) {
        unimplemented!();
    }

    pub async fn start_event_loop(mut self) {
        self.open_new_panel().unwrap();

        loop {
            self.display.render().unwrap();

            let res = self.connection_manager.wait_for_message().await;
            if let Some(bytes) = res.bytes {
                if res.id.is_none() {
                    self.handle_stdin(bytes);
                } else {
                    self.handle_panel_output(res.id.unwrap(), bytes);
                }
            } else {
                break;
            }

            if self.halt_execution {
                self.shutdown();
                break;
            }
        }
    }
    fn handle_stdin(&mut self, bytes: Vec<u8>) {
        if self.shortcut(&bytes) {
            return;
        } else {
            match self.selected_panel {
                Some(id) => self.panel_with_id(id).unwrap().parser.process(&bytes),
                None => self.handle_cmd_input(bytes),
            }
        }
    }

    fn shortcut(&mut self, bytes: &Vec<u8>) -> bool {
        if bytes.len() == 0 {
            return false;
        }

        let event = match event::parse_event(
            *bytes.first().unwrap(),
            &mut bytes[1..bytes.len()].iter().map(|b| Ok(*b)),
        ) {
            Ok(e) => e,
            Err(_) => return false,
        };

        if let Event::Key(k) = event {
            if let Some(k) = self
                .config
                .key_map()
                .command_for_key(&k)
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

    fn handle_cmd_input(&mut self, bytes: Vec<u8>) {}

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

    fn open_new_panel(&mut self) -> Result<(), Error> {
        let id = self.get_next_id();
        let (tx, shutdown_rx) = self.connection_manager.new_pair(id);
        let pty = Pty::open(self.config.get_panel_init_command())?;

        let new_sizes = self.display.open_new_panel(id)?;
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

        tokio::spawn(async move {
            pty_manager(pty, tx, shutdown_rx).await;
        });

        self.panels.push(Panel { parser, id });
        self.select_panel(Some(id));

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
            _ => unimplemented!(),
        }
    }

    async fn shutdown(self) {
        self.connection_manager.shutdown_all().await;
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
