use clap::{App, Arg};
use crossterm::{execute, terminal};
use muxide::{Config, LogicManager};
use muxide_logging::log::LogLevel;
use muxide_logging::{error, info, warning};
use std::fs::File;
use std::io::{stdout, Read};
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = App::new("muxide")
        .about("A basic terminal multiplexer for Linux and MacOS.")
        .arg(
            Arg::with_name("log_file")
                .short("f")
                .long("log_file")
                .takes_value(true)
                .max_values(1)
                .required(false)
                .help("Sets the file to write logging output to."),
        )
        .arg(
            Arg::with_name("log_level")
                .short("l")
                .long("log_level")
                .requires("log_file")
                .takes_value(true)
                .max_values(1)
                .possible_values(&["1", "2", "3"])
                .help("Sets the level of logging to enable."),
        )
        .get_matches();

    let mut config = load_config();

    if let Some(log_file) = matches.value_of("log_file") {
        config
            .get_environment_mut_ref()
            .set_log_file(log_file.to_string());
    }

    if let Some(log_level) = matches.value_of("log_level") {
        if let Ok(log_level) = log_level.parse() {
            config.get_environment_mut_ref().set_log_level(log_level);
        } else {
            eprintln!("Expected a value of 1, 2 or 3 for the log level.");
            exit(1);
        }
    }

    if let Some(f) = config.get_environment_ref().log_file() {
        if let Err(e) = muxide_logging::set_output_file(f) {
            eprintln!(
                "Failed to open '{}' for logging. Error description: {}",
                f, e
            );
            exit(1);
        }

        match config.get_environment_ref().log_level() {
            0 | 1 => {
                if let Err(e) = muxide_logging::restrict_log_levels(&[
                    LogLevel::StateChange,
                    LogLevel::Information,
                    LogLevel::Warning,
                ]) {
                    eprintln!("Failed to set log level. Error description: {}", e);
                    exit(1);
                }
            }
            2 => {
                if let Err(e) = muxide_logging::restrict_log_levels(&[
                    LogLevel::StateChange,
                    LogLevel::Information,
                ]) {
                    eprintln!("Failed to set log level. Error description: {}", e);
                    exit(1);
                }
            }
            _ => (),
        }
    }

    info!("Completed config load.");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.enter();
    if let Some(err) = rt.block_on(async { muxide_start(config).await }) {
        eprintln!("Terminating with error: {}", err);
        error!(format!("Terminated with error: {}", err));
    }
}

async fn muxide_start(config: Config) -> Option<String> {
    // We don't care about errors that happen with this function, if it fails that's ok.
    if let Err(e) = execute!(stdout(), terminal::EnterAlternateScreen) {
        warning!(format!(
            "Failed to enter alternate tty screen. Reason: {}",
            e
        ));
    }

    let logic_manager = LogicManager::new(config).unwrap();
    let err = logic_manager.start_event_loop().await.err();

    // We don't care about errors that happen with this function, if it fails that's ok.
    if let Err(e) = execute!(
        stdout(),
        crossterm::cursor::Show,
        crossterm::style::ResetColor,
        terminal::LeaveAlternateScreen
    ) {
        warning!(format!(
            "Failed to leave alternate tty screen. Reason: {}",
            e
        ));
    }

    return err;
}

fn load_config() -> Config {
    let path_string = match Config::default_path() {
        Some(p) => p,
        None => {
            eprintln!("Could not determine a suitable path for the config file.");
            exit(1);
        }
    };

    let path = Path::new(&path_string);
    let config;

    if !path.exists() {
        config = Config::default();
    } else {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to read config file. Error: {}", e);
                exit(1);
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to read config file. Error: {}", e);
                exit(1);
            }
        }

        config = match Config::from_toml_string(&contents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to parse config file: {}", e);
                exit(1);
            }
        };
    }

    return config;
}
