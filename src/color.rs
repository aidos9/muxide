use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

lazy_static! {
    /// Contains information about the terminal if available.
    static ref TERMINFO_DATABASE: Option<terminfo::Database> = terminfo::Database::from_env().ok();
}

/// Helper macro for defining a new named color.
macro_rules! define_new_color {
    ($name:tt, $r:literal, $g:literal, $b:literal) => {
        pub const $name: Self = Self {
            red: $r,
            green: $g,
            blue: $b,
        };
    };
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// Represents a Color using an RGB representation.
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    // Color constants
    define_new_color!(RED, 255, 0, 0);
    define_new_color!(GREEN, 0, 255, 0);
    define_new_color!(ORANGE, 255, 165, 0);
    define_new_color!(BLUE, 0, 0, 255);
    define_new_color!(MAGENTA, 128, 0, 128);
    define_new_color!(CYAN, 0, 255, 255);
    define_new_color!(TEAL, 0, 128, 128);
    define_new_color!(YELLOW, 255, 255, 0);
    define_new_color!(GREY, 128, 128, 128);
    define_new_color!(WHITE, 255, 255, 255);
    define_new_color!(BLACK, 0, 0, 0);

    /// Create a new color from the specified RGB values.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        return Self { red, green, blue };
    }

    /// Returns a Crossterm [Color](crossterm::style::Color) representation.
    pub fn crossterm_color(&self, default: crossterm::style::Color) -> crossterm::style::Color {
        use crossterm::style::Color as cColor;

        if TERMINFO_DATABASE.is_some() {
            if let Some(b) = TERMINFO_DATABASE
                .as_ref()
                .unwrap()
                .get::<terminfo::capability::TrueColor>()
            {
                if b.0 {
                    return cColor::Rgb {
                        r: self.red,
                        g: self.green,
                        b: self.blue,
                    };
                }
            }
        }

        if self == &Self::RED {
            return cColor::Red;
        } else if self == &Self::GREEN {
            return cColor::Green;
        } else if self == &Self::BLUE {
            return cColor::Blue;
        } else if self == &Self::MAGENTA {
            return cColor::Magenta;
        } else if self == &Self::CYAN {
            return cColor::Cyan;
        } else if self == &Self::TEAL {
            return cColor::DarkCyan;
        } else if self == &Self::YELLOW {
            return cColor::Yellow;
        } else if self == &Self::GREY {
            return cColor::Grey;
        } else if self == &Self::WHITE {
            return cColor::White;
        } else if self == &Self::BLACK {
            return cColor::Black;
        } else {
            return default;
        }
    }

    /// Constructs a new color from an RGB string.
    ///
    /// # Expected format:
    /// No spaces and commas separating the red, green and blue values.
    /// r,g,b - 123,123,123
    ///
    /// # Errors
    /// Errors occur from invalid formatting
    pub fn from_rgb_string(string: String) -> Result<Self, String> {
        let characters: Vec<char> = string.chars().collect();

        let (mut r, mut g, mut b) = (None, None, None);

        let mut i = 0;

        while i < characters.len() {
            let mut current_value = String::new();

            while i < characters.len() && characters[i] != ',' {
                if characters[i].is_whitespace() {
                    i += 1;
                    continue;
                } else if !characters[i].is_numeric() {
                    return Err(format!(
                        "Unexpected non whitespace character: {}",
                        characters[i]
                    ));
                }

                current_value.push(characters[i]);
                i += 1;

                if current_value.len() > 3 {
                    return Err(format!(
                        "Invalid integer value: {}, must be < 255",
                        current_value
                    ));
                }
            }

            if r.is_none() {
                r = match current_value.parse() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(format!("Invalid red value: {}", current_value)),
                };
            } else if g.is_none() {
                g = match current_value.parse() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(format!("Invalid green value: {}", current_value)),
                };
            } else if b.is_none() {
                b = match current_value.parse() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(format!("Invalid blue value: {}", current_value)),
                };
            } else if i < characters.len() {
                return Err(format!("Unexpected extra character: {}", characters[i]));
            }

            i += 1;
        }

        if r.is_none() {
            return Err(String::from("No value for red supplied"));
        } else if g.is_none() {
            return Err(String::from("No value for green supplied"));
        } else if b.is_none() {
            return Err(String::from("No value for blue supplied"));
        }

        return Ok(Self {
            red: r.unwrap(),
            green: g.unwrap(),
            blue: b.unwrap(),
        });
    }
}

impl TryFrom<String> for Color {
    type Error = String;

    /// Converts [String] values to a [Color]. This will accept words or RGB representations. It is not case sensitive.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lowered = value.to_lowercase();

        return Ok(match lowered.as_str() {
            "default" => Self::default(),
            "red" => Self::RED,
            "green" => Self::GREEN,
            "orange" => Self::ORANGE,
            "blue" => Self::BLUE,
            "magenta" => Self::MAGENTA,
            "cyan" => Self::CYAN,
            "teal" => Self::TEAL,
            "yellow" => Self::YELLOW,
            "gray" | "grey" => Self::GREY,
            "white" => Self::WHITE,
            "black" => Self::BLACK,
            _ => Self::from_rgb_string(lowered)?,
        });
    }
}

impl Default for Color {
    /// Creates a new white Color. Same as [WHITE](Color::WHITE).
    fn default() -> Self {
        return Self::WHITE;
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match Self::try_from(s) {
            Ok(c) => return Ok(c),
            Err(e) => return Err(serde::de::Error::custom(e)),
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return Serialize::serialize(
            &format!("{}, {}, {}", self.red, self.green, self.blue),
            serializer,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Color;
    use std::convert::TryFrom;

    #[test]
    fn test_from_string_red() {
        let input = "red".to_string();
        assert_eq!(Color::RED, Color::try_from(input).unwrap());
    }

    #[test]
    fn test_from_string_fail() {
        let input = "reds".to_string();
        assert!(Color::try_from(input).is_err());
    }

    #[test]
    fn test_from_string_fail_2() {
        let input = "1288, 0, 88".to_string();
        assert!(Color::try_from(input).is_err());
    }

    #[test]
    fn test_from_string_rgb() {
        let input = "128, 0, 88".to_string();
        assert_eq!(Color::new(128, 0, 88), Color::try_from(input).unwrap());
    }
}
