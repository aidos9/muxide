use crate::config::{Command, Config};
use crate::error::{Error, ErrorType};
use crate::pty::Pty;
use crate::{ChannelController, Display, InputManager};
use termion::event;
use termion::event::{Event, Key};
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
        });
    }

    pub fn process_config_file(&mut self, file: &str) {
        unimplemented!();
    }

    pub async fn start_event_loop(mut self) {
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
        }
    }
    fn handle_stdin(&mut self, bytes: Vec<u8>) {}

    fn handle_panel_output(&mut self, id: usize, bytes: Vec<u8>) {}

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
                .rows_formatted(0, parser_1.screen().size().1)
                .collect(),
        )?;

        tokio::spawn(async move {
            pty_manager(pty, tx, shutdown_rx).await;
        });

        self.panels.push(Panel { parser, id });

        return Ok(());
    }

    fn get_cmd_buffer_string(&self) -> String {
        return self.cmd_buffer.iter().collect();
    }

    fn clear_buffer(&mut self) {
        self.cmd_buffer.clear();
    }

    fn process_bytes(&mut self, mut bytes: Vec<u8>) -> Option<Command> {
        if bytes.len() == 0 {
            return None;
        }

        let first = bytes.remove(0);
        let event = event::parse_event(first, &mut bytes.into_iter().map(|b| Ok(b)));

        // Ignore any errors with parsing
        return match event {
            Ok(Event::Key(key)) => {
                self.redraw = true;

                match self.handle_key(&key) {
                    Some(c) => Some(c),
                    None => {
                        if self.command_mode {
                            self.handle_cmd_key(&key)
                        } else {
                            None
                        }
                    }
                }
            }
            _ => None,
        };
    }

    fn key_to_char(key: &Key) -> Option<Vec<char>> {
        return match key {
            Key::Char(ch) => Some(vec![*ch]),
            Key::Ctrl(ch) => Some(vec!['^', *ch]),
            Key::Alt(ch) => Some(vec!['A', '-', *ch]),
            _ => None,
        };
    }

    fn handle_key(&mut self, key: &Key) -> Option<Command> {
        let cmd = self
            .config
            .key_map()
            .command_for_key(key)
            .map(|v| v.clone());

        match cmd {
            Some(Command::UnMapKey(k)) => self.config.mut_key_map().unmap_key(&k),
            Some(Command::MapCommand(k, cmd)) => self.config.mut_key_map().map_command(*cmd, k),
            Some(cmd) => return Some(cmd.clone()),
            None => (),
        }

        return None;
    }

    fn handle_cmd_key(&mut self, key: &Key) -> Option<Command> {
        if key == &Key::Char('\n') {
            //TODO: Process command
            self.clear_buffer();
        }

        self.cmd_buffer.append(&mut Self::key_to_char(&key)?);

        return None;
    }

    fn get_next_id(&mut self) -> usize {
        self.next_panel_id += 1;
        return self.next_panel_id - 1;
    }
}
