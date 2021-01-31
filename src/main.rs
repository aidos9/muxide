use crossterm::{execute, terminal};
use muxide::{Config, LogicManager};
use std::fs::File;
use std::io::{stdout, Read};
use std::path::Path;
use std::process::exit;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.enter();
    rt.block_on(async { muxide_start().await });
}

async fn muxide_start() {
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

    // We don't care about errors that happen with this function, if it fails that's ok.
    let _ = execute!(stdout(), terminal::EnterAlternateScreen);

    let logic_manager = LogicManager::new(config).unwrap();
    logic_manager.start_event_loop().await;

    // We don't care about errors that happen with this function, if it fails that's ok.
    let _ = execute!(stdout(), terminal::LeaveAlternateScreen);
}
