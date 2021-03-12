use super::serde_default_funcs::*;
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Environment {
    #[serde(default = "serde_default_panel_init_command")]
    panel_init_command: String,
    #[serde(default = "serde_default_prompt_text")]
    prompt_text: String,
    #[serde(default)]
    selected_panel_color: Color,
    #[serde(default)]
    selected_workspace_color: Color,
    #[serde(default = "serde_default_as_true")]
    show_workspaces: bool,
    #[serde(default = "serde_default_1")]
    log_level: usize,
    log_file: Option<String>,
    #[serde(default = "serde_default_5")]
    scroll_lines: usize,
}

impl Environment {
    #[allow(dead_code)]
    pub fn new(
        panel_init_command: String,
        prompt_text: String,
        selected_panel_color: Color,
        selected_workspace_color: Color,
        show_workspaces: bool,
        log_level: usize,
        log_file: Option<String>,
        scroll_lines: usize,
    ) -> Self {
        return Self {
            panel_init_command,
            prompt_text,
            selected_panel_color,
            selected_workspace_color,
            show_workspaces,
            log_level,
            log_file,
            scroll_lines,
        };
    }

    pub fn show_workspaces(&self) -> bool {
        return self.show_workspaces;
    }

    pub fn selected_workspace_color(&self) -> Color {
        return self.selected_workspace_color;
    }

    pub fn set_log_file(&mut self, file: String) {
        self.log_file = Some(file);
    }

    pub fn log_file_ref(&self) -> &Option<String> {
        return &self.log_file;
    }

    pub fn set_log_level(&mut self, level: usize) {
        self.log_level = level;
    }

    pub fn log_level(&self) -> usize {
        return self.log_level;
    }

    pub fn scroll_lines(&self) -> usize {
        return self.scroll_lines;
    }

    pub fn panel_init_command_ref(&self) -> &String {
        return &self.panel_init_command;
    }
}

impl Default for Environment {
    fn default() -> Self {
        return Self {
            panel_init_command: serde_default_panel_init_command(),
            prompt_text: serde_default_prompt_text(),
            selected_panel_color: Color::default(),
            selected_workspace_color: Color::default(),
            show_workspaces: true,
            log_level: 1,
            log_file: None,
            scroll_lines: 5,
        };
    }
}
