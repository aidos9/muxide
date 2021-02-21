use crate::command::Command;
use serde::Deserialize;
use std::collections::HashMap;
use termion::event::Key;

#[derive(Clone, PartialEq, Debug)]
pub struct Keys {
    single_key_map: HashMap<char, Command>,
    shortcut_map: HashMap<Key, Command>,
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

    pub fn command_for_shortcut(&self, key: &Key) -> Option<&Command> {
        return self.shortcut_map.get(key);
    }

    pub fn map_shortcut(&mut self, key: Key, cmd: Command) {
        self.shortcut_map.insert(key, cmd);
    }

    pub fn unmap_shortcut(&mut self, key: &Key) {
        self.shortcut_map.remove(key);
    }

    pub fn command_for_character(&self, ch: &char) -> Option<&Command> {
        return self.single_key_map.get(ch);
    }

    pub fn map_character(&mut self, key: char, cmd: Command) {
        self.single_key_map.insert(key, cmd);
    }

    pub fn unmap_character(&mut self, key: &char) {
        self.single_key_map.remove(key);
    }

    #[inline]
    const fn is_permitted_char(ch: char) -> bool {
        return (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || ch == '!'
            || ch == '@'
            || ch == '#'
            || ch == '$'
            || ch == '%'
            || ch == '^'
            || ch == '&'
            || ch == '*'
            || ch == '('
            || ch == ')'
            || ch == '{'
            || ch == '}'
            || ch == '['
            || ch == ']'
            || ch == '\\'
            || ch == '|'
            || ch == ':'
            || ch == ';'
            || ch == '"'
            || ch == '\''
            || ch == '<'
            || ch == '>'
            || ch == ','
            || ch == '.'
            || ch == '?'
            || ch == '/'
            || ch == '~'
            || ch == '`'
            || ch == '_'
            || ch == '-'
            || ch == '+'
            || ch == '=';
    }
}

impl Default for Keys {
    fn default() -> Self {
        let mut n = Self {
            single_key_map: HashMap::new(),
            shortcut_map: HashMap::new(),
        };

        n.shortcut_map
            .insert(Key::Ctrl('a'), Command::EnterSingleCharacterCommand);
        n.shortcut_map
            .insert(Key::Ctrl('p'), Command::ToggleInputCommand);
        n.shortcut_map.insert(Key::Ctrl('q'), Command::QuitCommand);

        n.single_key_map.insert('n', Command::OpenPanelCommand);
        n.single_key_map
            .insert('q', Command::CloseMostRecentPanelCommand);
        n.single_key_map.insert('s', Command::SwapLayout);

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
            shortcut: Option<String>,
            key: Option<String>,
            command: String,
            args: Option<Vec<String>>,
        };
        let keys: Vec<KeyPair> = Deserialize::deserialize(deserializer)?;
        let mut res = Self::default();

        for key_pair in keys {
            let (shortcut, key, command, args) = (
                key_pair.shortcut,
                key_pair.key,
                key_pair.command,
                key_pair.args.unwrap_or(Vec::new()),
            );

            let cmd =
                Command::try_from_string(command, args).map_err(|e| serde::de::Error::custom(e))?;

            if let Some(shortcut) = shortcut {
                let shortcut =
                    key_from_string(shortcut).map_err(|e| serde::de::Error::custom(e))?;

                res.shortcut_map.insert(shortcut, cmd.clone());
            }

            if let Some(key) = key {
                let key: Vec<char> = key.chars().collect();

                if key.len() != 1 {
                    return Err(serde::de::Error::custom(
                        "Expected a single character 'key'.",
                    ));
                } else if !Self::is_permitted_char(*key.first().unwrap()) {
                    return Err(serde::de::Error::custom(format!(
                        "Unsupported 'key': {}",
                        key.first().unwrap()
                    )));
                }

                res.single_key_map.insert(*key.first().unwrap(), cmd);
            }
        }

        return Ok(res);
    }
}
