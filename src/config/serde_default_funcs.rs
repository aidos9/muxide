pub const fn serde_default_as_true() -> bool {
    return true;
}

pub const fn serde_default_as_false() -> bool {
    return false;
}

pub fn serde_default_panel_init_command() -> String {
    return String::from("/bin/sh");
}

pub fn serde_default_prompt_text() -> String {
    return String::from(">");
}

pub const fn serde_default_vertical_character() -> char {
    return '|';
}

pub const fn serde_default_horizontal_character() -> char {
    return '-';
}

pub const fn serde_default_intersection_character() -> char {
    return '+';
}

pub const fn serde_default_1() -> usize {
    return 1;
}

pub const fn serde_default_5() -> usize {
    return 5;
}

pub fn serde_default_password_file_location() -> String {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".config/muxide/password");

        return path.to_str().map(|s| s.to_string()).unwrap();
    } else {
        return String::from("~/.config/muxide/password");
    }
}

#[cfg(feature = "pbkdf2")]
pub fn serde_default_pbkdf2_iterations() -> usize {
    return pbkdf2::Params::default().rounds as usize;
}