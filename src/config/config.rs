use super::Color;
use super::Keys;
use serde::Deserialize;
use std::time::Duration;

#[inline]
const fn serde_default_as_true() -> bool {
    true
}

fn default_panel_init_command() -> String {
    return String::from("/bin/sh");
}

fn default_prompt_text() -> String {
    return String::from(">");
}

#[inline]
const fn default_vertical_character() -> char {
    return '|';
}

#[inline]
const fn default_horizontal_character() -> char {
    return '-';
}

#[inline]
const fn default_intersection_character() -> char {
    return '+';
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    environment: Environment,
    #[serde(default)]
    keys: Keys,
    #[serde(default)]
    borders: Borders,

    /// Potentially can be removed
    thread_delay_period: Option<Duration>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Environment {
    #[serde(default = "default_panel_init_command")]
    panel_init_command: String,
    #[serde(default = "default_prompt_text")]
    prompt_text: String,
    #[serde(default)]
    selected_panel_color: Color,
    #[serde(default)]
    selected_workspace_color: Color,
    #[serde(default = "serde_default_as_true")]
    show_workspaces: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
pub struct Borders {
    #[serde(default = "default_vertical_character")]
    vertical_character: char,
    #[serde(default = "default_horizontal_character")]
    horizontal_character: char,
    #[serde(default = "default_intersection_character")]
    intersection_character: char,
    #[serde(default)]
    color: Color,
}

impl Config {
    const DEFAULT_THREAD_DELAY_TIME: Duration = Duration::from_micros(500);

    pub fn new() -> Self {
        return Self::default();
    }

    pub fn get_thread_time(&self) -> Duration {
        return self
            .thread_delay_period
            .unwrap_or(Self::DEFAULT_THREAD_DELAY_TIME);
    }

    pub fn key_map(&self) -> &Keys {
        return &self.keys;
    }

    pub fn mut_key_map(&mut self) -> &mut Keys {
        return &mut self.keys;
    }

    pub fn get_borders_ref(&self) -> &Borders {
        return &self.borders;
    }

    pub fn get_environment_ref(&self) -> &Environment {
        return &self.environment;
    }

    pub fn get_panel_init_command(&self) -> &String {
        return &self.environment.panel_init_command;
    }

    pub fn from_toml_string(toml: &str) -> Result<Self, String> {
        return toml::from_str(toml).map_err(|e| e.to_string());
    }

    pub fn default_path() -> Option<String> {
        let mut path = dirs::home_dir()?;
        path.push(".config/muxide/config.toml");

        return path.to_str().map(|s| s.to_string());
    }
}

impl Borders {
    #[inline]
    pub fn get_intersection_char(&self) -> char {
        return self.intersection_character;
    }

    #[inline]
    pub fn get_vertical_char(&self) -> char {
        return self.vertical_character;
    }

    #[inline]
    pub fn get_horizontal_char(&self) -> char {
        return self.horizontal_character;
    }
}

impl Environment {
    pub fn show_workspaces(&self) -> bool {
        return self.show_workspaces;
    }

    pub fn selected_workspace_color(&self) -> Color {
        return self.selected_workspace_color;
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            environment: Environment::default(),
            keys: Keys::default(),
            borders: Borders::default(),

            /// Potentially can be removed
            thread_delay_period: None,
        };
    }
}

impl Default for Environment {
    fn default() -> Self {
        return Self {
            panel_init_command: default_panel_init_command(),
            prompt_text: default_prompt_text(),
            selected_panel_color: Color::default(),
            selected_workspace_color: Color::default(),
            show_workspaces: true,
        };
    }
}

impl Default for Borders {
    fn default() -> Self {
        return Self {
            vertical_character: default_vertical_character(),
            horizontal_character: default_horizontal_character(),
            intersection_character: default_intersection_character(),
            color: Color::default(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Config};
    use crate::command::Command;
    use termion::event::Key;

    #[test]
    fn basic_toml_test() {
        let input = "
        [environment]\n\
        panel_init_command = \"/usr/local/bin/fish\"\n\
        show_workspaces = true\n\
        prompt_text = \">\"\n\
        \n\
        [borders]\n\
        vertical_character = \"|\"\n\
        horizontal_character = \" \"\n\
        intersection_character = \"~\"\n\
        color = \"blue\"\n\
        \n\
        [command_prompt]\n\
        enabled = true\n\
        \n\
        [[keys]]\n\
        shortcut = \"ctrl+a\"\n\
        command = \"OpenPanel\"\n\
        \n\
        [[keys]]\n\
        shortcut = \"ctrl+p\"\n\
        key = \"f\"\n\
        command = \"FocusCommandPrompt\"\n\
        #args = [\"a\"]\n\
        ";

        let conf: Config = toml::from_str(input).unwrap();

        let mut comp = Config::default();
        comp.environment.panel_init_command = String::from("/usr/local/bin/fish");
        comp.borders.color = Color::blue();
        comp.borders.horizontal_character = ' ';
        comp.borders.intersection_character = '~';
        comp.keys
            .map_shortcut(Key::Ctrl('a'), Command::OpenPanelCommand);
        comp.keys
            .map_shortcut(Key::Ctrl('p'), Command::FocusCommandPromptCommand);
        comp.keys
            .map_character('f', Command::FocusCommandPromptCommand);

        assert_eq!(conf, comp);
    }
}
