mod channel_controller;
mod command;
mod config;
mod display;
mod error;
mod geometry;
mod input_manager;
mod logic_manager;
mod pty;

pub use config::Config;
pub use error::{Error, ErrorType};
pub use logic_manager::LogicManager;

pub fn config_file_path() -> Option<String> {
    let mut path = dirs::home_dir()?;
    path.push(".config/muxide/config.toml");

    return path.to_str().map(|s| s.to_string());
}
