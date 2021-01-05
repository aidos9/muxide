use crate::{Error, ErrorType};
use mio::{Events, Poll};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use termion::get_tty;
use termion::input::TermReadEventsAndRaw;
use termion::raw::{IntoRawMode, RawTerminal};

/// The input manager controls all input received from the TTY passing it to the display
pub struct InputManager {
    read_content: Arc<Mutex<Vec<Vec<u8>>>>,
    running: Arc<AtomicBool>,
}

impl InputManager {
    /// Attempt to create a new instance of an InputManager.
    pub fn new() -> Self {
        return Self {
            read_content: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        };
    }

    pub fn start(&mut self) -> bool {
        if self.is_running() {
            return false;
        }

        let mut tty_input = get_tty().unwrap().into_raw_mode().unwrap();
        let mut storage = self.read_content.clone();
        let mut running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            loop {
                let mut buffer = [0u8; 1024];
                let end = tty_input.read(&mut buffer).unwrap();
                let mut content = buffer[0..end].to_vec();

                match storage.lock() {
                    Ok(mut storage) => {
                        storage.push(content);
                    }
                    Err(_) => break,
                }
            }

            running.store(false, Ordering::SeqCst);
        });

        return true;
    }

    pub fn is_running(&self) -> bool {
        return self.running.load(Ordering::SeqCst);
    }

    pub fn take_buffer(&mut self) -> Vec<Vec<u8>> {
        let mut lock = self.read_content.lock().unwrap();
        let res = lock.clone();
        lock.clear();

        return res;
    }
}
