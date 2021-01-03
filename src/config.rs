use std::collections::HashMap;
use termion::event::Key;

#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    key_map: KeyMap,
}

#[derive(Clone, Eq, PartialEq)]
pub struct KeyMap {
    map: HashMap<Key, Command>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Command {
    EnterInputCommand,
    StopInputCommand,
    ArrowLeftCommand,
    ArrowRightCommand,
    ArrowUpCommand,
    ArrowDownCommand,
    ClosePanelCommand(usize),
    OpenPanelCommand,
    SwapPanelsCommand(usize, usize),
    FocusPanelCommand(usize),
    IdentifyPanelsCommand(usize),
    MapCommand(Key, Box<Command>),
    UnMapKey(Key),
    CustomCommand(String),
    /*
    ChangeLayout(String),
     */
    QuitCommand,
}

impl Config {
    pub fn new() -> Self {
        return Self {
            key_map: KeyMap::default(),
        };
    }

    pub fn key_map(&self) -> &KeyMap {
        return &self.key_map;
    }

    pub fn mut_key_map(&mut self) -> &mut KeyMap {
        return &mut self.key_map;
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self::new();
    }
}

impl KeyMap {
    pub fn new() -> Self {
        let mut n = Self {
            map: HashMap::new(),
        };

        n.map.insert(Key::Ctrl('p'), Command::EnterInputCommand);
        n.map.insert(Key::Ctrl('q'), Command::QuitCommand);
        n.map.insert(Key::Ctrl('o'), Command::OpenPanelCommand);

        return n;
    }

    pub fn command_for_key(&self, key: &Key) -> Option<&Command> {
        return self.map.get(key);
    }

    pub fn map_command(&mut self, cmd: Command, key: Key) {
        self.map.insert(key, cmd);
    }

    pub fn unmap_key(&mut self, key: &Key) {
        self.map.remove(key);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        return Self::new();
    }
}

impl Command {
    pub fn get_name(&self) -> &str {
        return match self {
            Self::EnterInputCommand => "EnterInput",
            Self::StopInputCommand => "StopInput",
            Self::ArrowLeftCommand => "ArrowLeft",
            Self::ArrowRightCommand => "ArrowRight",
            Self::ArrowUpCommand => "ArrowUp",
            Self::ArrowDownCommand => "ArrowDown",
            Self::ClosePanelCommand(_) => "ClosePanel",
            Self::OpenPanelCommand => "OpenPanel",
            Self::SwapPanelsCommand(_, _) => "SwapPanels",
            Self::FocusPanelCommand(_) => "FocusPanel",
            Self::IdentifyPanelsCommand(_) => "Identify",
            Self::MapCommand(_, _) => "Map",
            Self::UnMapKey(_) => "UnMap",
            Self::CustomCommand(cmd) => cmd,
            Self::QuitCommand => "Quit",
        };
    }
}
