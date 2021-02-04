use crate::command::Command;
use serde::Deserialize;
use std::collections::HashMap;
use termion::event::Key;

#[derive(Clone, PartialEq, Debug)]
pub struct Keys {
    map: HashMap<Key, Command>,
}

fn key_from_string(string: String) -> Result<Key, &'static str> {
    let mut first_half = String::new();
    let mut string: Vec<char> = string.chars().collect();

    while string.len() > 0 {
        if string[0] == '+' {
            if first_half.len() == 0 {
                return Err("A single character is required to follow a '+'");
            }

            string.remove(0);
            break;
        } else {
            first_half.push(string.remove(0));
        }
    }

    if string.len() > 0 {
        let lowered = first_half.to_lowercase();

        if lowered == "ctrl" {
            if string.len() != 1 {
                return Err("Expected a single character to follow '+'.");
            } else {
                return Ok(Key::Ctrl(string[0]));
            }
        } else if lowered == "alt" {
            if string.len() != 1 {
                return Err("Expected a single character to follow '+'.");
            } else {
                return Ok(Key::Alt(string[0]));
            }
        } else {
            return Err("Only the \"Alt\" and \"Ctrl\" modifiers are supported.");
        }
    } else {
        if first_half.len() != 1 {
            return Err("A single character key or modifier '+' single character is expected.");
        } else {
            return Ok(Key::Char(first_half.remove(0)));
        }
    }
}

impl Keys {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn command_for_key(&self, key: &Key) -> Option<&Command> {
        return self.map.get(key);
    }

    pub fn map_command(&mut self, key: Key, cmd: Command) {
        self.map.insert(key, cmd);
    }

    pub fn unmap_key(&mut self, key: &Key) {
        self.map.remove(key);
    }
}

impl Default for Keys {
    fn default() -> Self {
        let mut n = Self {
            map: HashMap::new(),
        };

        n.map.insert(Key::Ctrl('a'), Command::ToggleInputCommand);
        n.map.insert(Key::Ctrl('q'), Command::QuitCommand);
        n.map.insert(Key::Ctrl('o'), Command::OpenPanelCommand);

        return n;
    }
}

impl<'de> Deserialize<'de> for Keys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct KeyPair {
            key: String,
            command: String,
            args: Option<Vec<String>>,
        };
        let keys: Vec<KeyPair> = Deserialize::deserialize(deserializer)?;
        let mut res = Self::default();

        for key_pair in keys {
            let (key, command, args) = (
                key_pair.key,
                key_pair.command,
                key_pair.args.unwrap_or(Vec::new()),
            );

            let key = key_from_string(key).map_err(|e| serde::de::Error::custom(e))?;
            let cmd =
                Command::try_from_string(command, args).map_err(|e| serde::de::Error::custom(e))?;

            res.map.insert(key, cmd);
        }

        return Ok(res);
    }
}
