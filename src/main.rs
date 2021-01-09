use muxide::pty::Pty;
use muxide::{ChannelController, InputManager};
use std::process::Command;
use tokio::io::{self, AsyncReadExt};
use tokio::select;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
/*
use crossterm::{execute, terminal};
use muxide::{Config, Display, InputManager};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
use tab_pty_process::{AsyncPtyMaster, CommandExt, PtyMaster};
use tokio::io::AsyncReadExt;
*/

const RENDER_LIMIT: u64 = 100;

async fn read_output(mut r: Pty, tx: Sender<Vec<u8>>) {
    let mut buf = vec![0u8; 4096];

    loop {
        select! {
            res = r.read(&mut buf) => {
                if let Ok(count) = res {
                    if count == 0 {
                        if r.running() == Some(false) {
                            break;
                        }
                    }

                    let mut cpy = vec![0u8; count];
                    cpy.copy_from_slice(&buf[0..count]);

                    tx.send(cpy).await;

                    tokio::time::sleep(Duration::from_millis(5)).await;
                } else {
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // let runtime = tokio::runtime::Builder::new_multi_thread()
    //     .enable_io()
    //     .enable_time()
    //     .build()
    //     .unwrap();

    let (mut controller, stdin_sender) = ChannelController::new();
    let tx = controller.new_pair(0);
    let manager = InputManager::start(stdin_sender).unwrap();

    let mut pty = Pty::open().unwrap();
    let mut buf = vec![0; 4096];

    let handle = tokio::spawn(async move {
        read_output(pty, tx).await;
    });

    loop {
        let res = controller.wait_for_message().await;
        if let Some(bytes) = res.bytes {
            if !res.id.is_some() {
                println!("{:?}", std::str::from_utf8(&bytes));
            }
        } else {
            break;
        }
    }

    return Ok(());
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
