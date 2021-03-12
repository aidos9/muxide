use super::{Borders, Environment, Keys, PasswordSettings};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    environment: Environment,
    #[serde(default)]
    borders: Borders,
    #[serde(default)]
    keys: Keys,
    #[serde(default)]
    password: PasswordSettings,
}

impl Config {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn keys_ref(&self) -> &Keys {
        return &self.keys;
    }

    pub fn keys_mut(&mut self) -> &mut Keys {
        return &mut self.keys;
    }

    pub fn borders_ref(&self) -> &Borders {
        return &self.borders;
    }

    pub fn environment_ref(&self) -> &Environment {
        return &self.environment;
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        return &mut self.environment;
    }

    pub fn password_ref(&self) -> &PasswordSettings {
        return &self.password;
    }

    pub fn from_toml_string(toml: &str) -> Result<Self, String> {
        return toml::from_str(toml).map_err(|e| e.to_string());
    }

    pub fn from_json_string(json: &str) -> Result<Self, String> {
        return serde_json::from_str(json).map_err(|e| e.to_string());
    }

    pub fn default_path(format: &str) -> Option<String> {
        let mut path = dirs::home_dir()?;

        if format.to_lowercase() == "toml" {
            path.push(".config/muxide/config.toml");
        } else if format.to_lowercase() == "json" {
            path.push(".config/muxide/config.json");
        } else {
            return None;
        }

        return path.to_str().map(|s| s.to_string());
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            environment: Environment::default(),
            keys: Keys::default(),
            borders: Borders::default(),
            password: PasswordSettings::default(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Borders, Config, Environment};
    use crate::command::Command;
    use crate::Color;
    use termion::event::Key;

    #[test]
    fn basic_toml_test() {
        let input = "
        [environment]\n\
        panel_init_command = \"/usr/local/bin/fish\"\n\
        prompt_text = \"-> \"\n\
        selected_panel_color = \"blue\"\n\
        selected_workspace_color = \"green\"\n\
        show_workspaces = false\n\
        log_level = 3\n\
        log_file = \"/usr/log_file\"\n\
        scroll_lines = 120\n\
        \n\
        [borders]\n\
        vertical_character = \"|\"\n\
        horizontal_character = \" \"\n\
        intersection_character = \"~\"\n\
        color = \"red\"\n\
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
        #key = \"f\"\n\
        command = \"SubdivideSelectedVertical\"\n\
        #args = [\"a\"]\n\
        ";

        let conf: Config = toml::from_str(input).unwrap();

        let mut comp = Config::default();
        comp.environment = Environment::new(
            String::from("/usr/local/bin/fish"),
            String::from("-> "),
            Color::BLUE,
            Color::GREEN,
            false,
            3,
            Some(String::from("/usr/log_file")),
            120,
        );

        comp.borders = Borders::new('|', ' ', '~', Color::RED);

        comp.keys
            .map_shortcut(Key::Ctrl('a'), Command::OpenPanelCommand);
        comp.keys
            .map_shortcut(Key::Ctrl('p'), Command::SubdivideSelectedVerticalCommand);

        assert_eq!(conf, comp);
    }
}
