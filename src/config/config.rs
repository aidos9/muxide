use super::BorderColor;
use super::Keys;
use serde::Deserialize;
use std::time::Duration;

fn serde_default_as_true() -> bool {
    true
}

fn default_panel_init_command() -> String {
    return String::from("/bin/sh");
}

fn default_prompt_text() -> String {
    return String::from(">");
}

fn default_border_color() -> Option<BorderColor> {
    return Some(BorderColor::default());
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    environment: Environment,
    #[serde(default)]
    keys: Keys,
    #[serde(default)]
    command_prompt: CommandPrompt,
    #[serde(default)]
    top_border: Border,
    #[serde(default)]
    bottom_border: Border,
    #[serde(default)]
    left_border: Border,
    #[serde(default)]
    right_border: Border,

    /// Potentially can be removed
    thread_delay_period: Option<Duration>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
struct Environment {
    #[serde(default = "default_panel_init_command")]
    panel_init_command: String,
    #[serde(default = "default_border_color")]
    default_border_color: Option<BorderColor>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
struct CommandPrompt {
    #[serde(default = "default_prompt_text")]
    prompt_text: String,
    #[serde(default = "serde_default_as_true")]
    enabled: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct Border {
    #[serde(default)]
    color: BorderColor,
    #[serde(default = "serde_default_as_true")]
    enabled: bool,
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

impl Default for Config {
    fn default() -> Self {
        return Self {
            environment: Environment::default(),
            keys: Keys::default(),
            command_prompt: CommandPrompt::default(),
            top_border: Border::default(),
            bottom_border: Border::default(),
            left_border: Border::default(),
            right_border: Border::default(),

            /// Potentially can be removed
            thread_delay_period: None,
        };
    }
}

impl Default for Environment {
    fn default() -> Self {
        return Self {
            panel_init_command: default_panel_init_command(),
            default_border_color: default_border_color(),
        };
    }
}

impl Default for CommandPrompt {
    fn default() -> Self {
        return Self {
            prompt_text: default_prompt_text(),
            enabled: true,
        };
    }
}

impl Default for Border {
    fn default() -> Self {
        return Self {
            color: BorderColor::default(),
            enabled: true,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{BorderColor, Config};
    use crate::command::Command;
    use termion::event::Key;

    #[test]
    fn basic_toml_test() {
        let input = "
        [environment]\n\
        panel_init_command = \"/usr/local/bin/fish\"\n\
        \n\
        [command_prompt]\n\
        prompt_text = \">\"\n\
        enabled = true\n\
        \
        [top_border]\n\
        color = \"blue\"\n\
        enabled = true\n\
        \n\
        [[keys]]\n\
        key = \"ctrl+a\"\n\
        command = \"OpenPanel\"\n\
        \n\
        [[keys]]\n\
        key = \"ctrl+p\"\n\
        command = \"FocusCommandPrompt\"\n\
        #args = [\"a\"]\n\
        ";

        let conf: Config = toml::from_str(input).unwrap();

        let mut comp = Config::default();
        comp.environment.panel_init_command = String::from("/usr/local/bin/fish");
        comp.command_prompt.prompt_text = String::from(">");
        comp.command_prompt.enabled = true;
        comp.top_border.color = BorderColor::blue();
        comp.top_border.enabled = true;
        comp.keys
            .map_command(Key::Ctrl('a'), Command::OpenPanelCommand);
        comp.keys
            .map_command(Key::Ctrl('p'), Command::FocusCommandPromptCommand);

        assert_eq!(conf, comp);
    }
}
