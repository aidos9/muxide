use crossterm::{execute, terminal};
use muxide::{Config, Display, InputManager};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

const RENDER_LIMIT: u64 = 100;

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
