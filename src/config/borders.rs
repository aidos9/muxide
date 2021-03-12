use super::serde_default_funcs::{
    serde_default_horizontal_character, serde_default_intersection_character,
    serde_default_vertical_character,
};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Borders {
    #[serde(default = "serde_default_vertical_character")]
    vertical_character: char,
    #[serde(default = "serde_default_horizontal_character")]
    horizontal_character: char,
    #[serde(default = "serde_default_intersection_character")]
    intersection_character: char,
    #[serde(default)]
    color: Color,
}

impl Borders {
    #[allow(dead_code)]
    pub fn new(
        vertical_character: char,
        horizontal_character: char,
        intersection_character: char,
        color: Color,
    ) -> Self {
        return Self {
            vertical_character,
            horizontal_character,
            intersection_character,
            color,
        };
    }

    pub fn get_intersection_char(&self) -> char {
        return self.intersection_character;
    }

    pub fn get_vertical_char(&self) -> char {
        return self.vertical_character;
    }

    pub fn get_horizontal_char(&self) -> char {
        return self.horizontal_character;
    }
}

impl Default for Borders {
    fn default() -> Self {
        return Self {
            vertical_character: serde_default_vertical_character(),
            horizontal_character: serde_default_horizontal_character(),
            intersection_character: serde_default_intersection_character(),
            color: Color::default(),
        };
    }
}
