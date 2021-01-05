use crate::config::{Command, Config};
use termion::event;
use termion::event::{Event, Key};

/// Handles a majority of command parsing and config logic
pub struct LogicManager {
    config: Config,
    cmd_buffer: Vec<char>,
    redraw: bool,
    command_mode: bool,
}

impl LogicManager {
    pub fn new() -> Self {
        return Self {
            config: Config::new(),
            cmd_buffer: Vec::new(),
            redraw: false,
            command_mode: true,
        };
    }

    pub fn process_config_file(&mut self, file: &str) {
        unimplemented!();
    }

    pub fn get_cmd_buffer_string(&self) -> String {
        return self.cmd_buffer.iter().collect();
    }

    pub fn clear_buffer(&mut self) {
        self.cmd_buffer.clear();
    }

    pub fn redraw_required(&mut self) -> bool {
        let v = self.redraw;
        self.redraw = false;

        return v;
    }

    pub fn process_bytes(&mut self, mut bytes: Vec<u8>) -> Option<Command> {
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

    pub fn set_command_mode(&mut self, cmd_mode: bool) {
        self.command_mode = cmd_mode;
    }

    pub fn get_command_mode(&self) -> bool {
        return self.command_mode;
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
}
