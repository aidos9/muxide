use crossterm::{execute, terminal};
use muxide::{config_file_path, Config, LogicManager};
use std::fs::File;
use std::io::{stdout, Read};
use std::path::Path;
use std::process::exit;

/*
use muxide::{Config, Display, InputManager};
use std::thread;
use std::time::Duration;
use tab_pty_process::{AsyncPtyMaster, CommandExt, PtyMaster};
use tokio::io::AsyncReadExt;
*/

#[tokio::main]
async fn main() {
    // We don't care about errors that happen with this function, if it fails that's ok.
    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_io()
    //     .enable_time()
    //     .build()
    //     .unwrap();

    let path_string = match config_file_path() {
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

    let _ = execute!(stdout(), terminal::EnterAlternateScreen);

    let logic_manager = LogicManager::new(config).unwrap();
    logic_manager.start_event_loop().await;

    // We don't care about errors that happen with this function, if it fails that's ok.
    let _ = execute!(stdout(), terminal::LeaveAlternateScreen);
}

/*
fn main() {
    let config = Config::new();
    let mut manager = InputManager::new();

    if !manager.start() || !manager.is_running() {
        panic!("Fail?");
    }

    execute!(stdout(), terminal::EnterAlternateScreen);

    //let mut display = Display::new("/usr/bin/vim").init().unwrap();
    let mut display = Display::new("/usr/local/bin/fish", config.clone())
        .init()
        .unwrap();

    display.open_new_panel().unwrap();
    display.render();

    while !display.quit() {
        let content = manager.take_buffer();

        for vc in content {
            display.receive_input(vc).unwrap();
        }

        if display.pre_render().unwrap() {
            display.render();
        }

        thread::sleep(config.get_thread_time());
    }

    execute!(stdout(), terminal::LeaveAlternateScreen);
}
*/
/*
    let signal = notify(&[Signal::WINCH]);

    let mut tty_output = get_tty().unwrap().into_raw_mode().unwrap();
    let mut tty_input = tty_output.try_clone().unwrap();

    let pty_resize = PTY::new("/usr/bin/vim", &Size::new(40, 40));


    let mut pty_output = pty_resize.try_clone().unwrap();
    let mut pty_input = pty_output.try_clone().unwrap();
    let mut state_machine = Parser::new(20, 80, 40);

    let handle = thread::spawn(move || {
        loop {
            let mut packet = [0; 4096];

            let count = pty_input.read(&mut packet).unwrap();

            let read = &packet[..count];
            state_machine.process(read);

            tty_output.write_all(&read).unwrap();
            tty_output.flush().unwrap();
        }
    });

    let second = thread::spawn(move || loop {
        match pipe(&mut tty_input, &mut pty_output) {
            Err(_) => return,
            _ => (),
        }
    });

    // thread::spawn(move || {
    //     loop {
    //         signal.recv().unwrap();
    //         //pty_resize.resize(&get_terminal_size().unwrap());
    //     }
    // });

    thread::sleep(Duration::new(5, 0));

    if pty_resize.is_running() {
        println!("running");
    }
    pty_resize.kill();
}

/// Sends the content of input into output
fn pipe(input: &mut File, output: &mut File) -> Result<()> {
    let mut packet = [0; 4096];

    let count = input.read(&mut packet)?;

    let read = &packet[..count];
    output.write_all(&read)?;
    output.flush()?;

    Ok(())
}
*/
