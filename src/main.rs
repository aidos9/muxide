use chan_signal::{notify, Signal};
use muxide::pty::PTY;
use muxide::geometry::Size;
use muxide::vte_handler::VTEHandler;
use std::fs::File;
use std::io::{Read, Result, Write};
use std::thread;
use std::time::Duration;
use termion::get_tty;
use termion::raw::IntoRawMode;
use vte::Parser;
use muxide::terminal_screen::TerminalScreen;

fn main() {
    let signal = notify(&[Signal::WINCH]);

    let mut tty_output = get_tty().unwrap().into_raw_mode().unwrap();
    let mut tty_input = tty_output.try_clone().unwrap();

    let pty_resize = PTY::new("/usr/bin/vim", &Size::new(40, 40));


    let mut pty_output = pty_resize.try_clone().unwrap();
    let mut pty_input = pty_output.try_clone().unwrap();
    let mut handler = TerminalScreen::new(Size::new(20, 20));
    let mut state_machine = Parser::new();

    let handle = thread::spawn(move || {
        loop {
            let mut packet = [0; 4096];

            let count = pty_input.read(&mut packet).unwrap();

            let read = &packet[..count];

            for b in read {
                //println!("{}", (*b));
                state_machine.advance(&mut handler, *b);
            }

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
