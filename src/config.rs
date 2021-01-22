use std::collections::HashMap;
use std::time::Duration;
use termion::event::Key;

#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    key_map: KeyMap,
    panel_init_command: String,
    thread_delay_period: Option<Duration>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct KeyMap {
    map: HashMap<Key, Command>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Command {
    EnterInputCommand,
    StopInputCommand,
    ToggleInputCommand,
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
    CustomCommandCall(String),
    /*
    ChangeLayout(String),
     */
    QuitCommand,
}

impl Config {
    const DEFAULT_THREAD_DELAY_TIME: Duration = Duration::from_micros(500);

    pub fn new() -> Self {
        return Self {
            key_map: KeyMap::default(),
            panel_init_command: "/usr/bin/fish".to_string(),
            /// NOTE: Change this
            thread_delay_period: None,
        };
    }

    pub fn get_thread_time(&self) -> Duration {
        return self
            .thread_delay_period
            .unwrap_or(Self::DEFAULT_THREAD_DELAY_TIME);
    }

    pub fn key_map(&self) -> &KeyMap {
        return &self.key_map;
    }

    pub fn mut_key_map(&mut self) -> &mut KeyMap {
        return &mut self.key_map;
    }

    pub fn get_panel_init_command(&self) -> &String {
        return &self.panel_init_command;
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

        n.map.insert(Key::Ctrl('a'), Command::ToggleInputCommand);
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
            Self::ToggleInputCommand => "ToggleInput",
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
            Self::CustomCommandCall(cmd) => cmd,
            Self::QuitCommand => "Quit",
        };
    }
}
