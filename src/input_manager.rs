use crate::{ErrorType, MuxideError};
use std::io::{ErrorKind, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use termion::get_tty;
use termion::raw::IntoRawMode;
use tokio::sync::mpsc::Sender;

/// The input manager controls all input received from the TTY passing it to the display
pub struct InputManager {
    running: Arc<AtomicBool>,
}

impl InputManager {
    /// The buffer size for stdin.
    const BUFFER_SIZE: usize = 2048;

    /// Attempt to create a new IOManager instance. This will start a new thread that will read
    /// from the Stdin and send the information through the sender instance supplied.
    pub fn start(sender: Sender<Vec<u8>>) -> Result<Self, MuxideError> {
        let mut val = Self {
            running: Arc::new(AtomicBool::new(false)),
        };

        return val.start_internal(sender).map(|_| val);
    }

    fn start_internal(&mut self, sender: Sender<Vec<u8>>) -> Result<(), MuxideError> {
        // Ensure this method hasn't been called more than once
        if self.is_running() {
            return Err(ErrorType::new_input_manager_running_error());
        }

        // Put the tty into raw mode
        let mut tty_input = get_tty()
            .map_err(|e| ErrorType::new_failed_tty_acquisition_error(format!("{}", e)))?
            .into_raw_mode()
            .map_err(|e| ErrorType::new_enter_raw_mode_error(format!("{}", e)))?;
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let mut buffer = [0u8; Self::BUFFER_SIZE];

            loop {
                // Read bytes into the buffer
                let size = match tty_input.read(&mut buffer) {
                    Ok(s) => s,
                    Err(e) => match e.kind() {
                        ErrorKind::TimedOut | ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                            continue
                        }
                        _ => break,
                    },
                };

                // Copy them into a vector
                let content = buffer[0..size].to_vec();

                if sender.blocking_send(content).is_err() {
                    break;
                }
            }

            running.store(false, Ordering::SeqCst);
        });

        return Ok(());
    }

    /// Returns the status of the input thread, if it is still running or not.
    pub fn is_running(&self) -> bool {
        return self.running.load(Ordering::SeqCst);
    }
}
