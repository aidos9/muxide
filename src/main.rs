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
                .value_name("FILE")
                .required(false)
                .help("Sets the file to write logging output to."),
        )
        .arg(
            Arg::with_name("log_level")
                .short("l")
                .long("log_level")
                .requires("log_file")
                .takes_value(true)
                .value_name("LEVEL")
                .max_values(1)
                .possible_values(&["1", "2", "3"])
                .help("Sets the level of logging to enable."),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .takes_value(true)
                .value_name("FILE")
                .max_values(1)
                .help("Specify a config file."),
        )
        .arg(
            Arg::with_name("print-config")
                .long("print-config")
                .takes_value(false)
                .help("Print the default config to stdout."),
        )
        .arg(
            Arg::with_name("config-format")
                .long("config-format")
                .takes_value(true)
                .max_values(1)
                .value_name("FORMAT")
                .possible_values(&["JSON", "TOML"])
                .default_value("TOML")
                .help("Specify the format of the config file."),
        )
        .get_matches();

    if matches.is_present("print-config") {
        print_default_config(matches.value_of("config-format").unwrap_or("TOML"));
        return;
    }

    let mut config = load_config(
        matches.value_of("config").map(|s| s.to_string()),
        matches.value_of("config-format").unwrap_or("TOML"),
    );

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

fn load_config(path: Option<String>, format: &str) -> Config {
    let path_string;

    if let Some(path) = path {
        path_string = path;
    } else {
        path_string = match Config::default_path(format) {
            Some(p) => p,
            None => {
                eprintln!("Could not determine a suitable path for the config file.");
                exit(1);
            }
        };
    }

    let path = Path::new(&path_string);
    let config;

    if !path.exists() {
        config = Config::default();
    } else {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to read config file at path: {}. Error: {}",
                    path_string, e
                );
                exit(1);
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "Failed to read config file at path: {}. Error: {}",
                    path_string, e
                );
                exit(1);
            }
        }

        config = match format.to_lowercase().as_str() {
            "toml" => match Config::from_toml_string(&contents) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "Failed to parse config file at path: {}, due to error: {}",
                        path_string, e
                    );
                    exit(1);
                }
            },
            "json" => match Config::from_json_string(&contents) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "Failed to parse config file at path: {}, due to error: {}",
                        path_string, e
                    );
                    exit(1);
                }
            },
            _ => {
                eprintln!("Invalid format specified. Choose either 'TOML' or 'JSON'.");
                exit(1);
            }
        };
    }

    return config;
}

fn print_default_config(config_format: &str) {
    if config_format == "TOML" {
        println!("{}", toml::to_string(&Config::default()).unwrap());
    } else if config_format == "JSON" {
        println!(
            "{}",
            serde_json::to_string_pretty(&Config::default()).unwrap()
        );
    } else {
        eprintln!("Unknown format: {}", config_format);
    }
}
