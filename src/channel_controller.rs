use crate::error::{ErrorType, MuxideError};
use futures::FutureExt;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::watch;
use tokio::time::{self, Duration};

struct Channel {
    id: usize,
    rx: Receiver<Vec<u8>>,
    tx: Sender<Vec<u8>>,
    shutdown_tx: watch::Sender<bool>,
}

pub struct ControllerResponse {
    pub bytes: Option<Vec<u8>>,
    pub id: Option<usize>,
}

pub struct ChannelController {
    stdin_rx: Receiver<Vec<u8>>,
    ptys: Vec<Channel>,
}

impl ChannelController {
    const BUFFER_SIZE: usize = 100;
    const SHUTDOWN_TIMEOUT_MS: u64 = 200;
    const SEND_TIMEOUT_MS: u64 = 200;

    /// Returns self, and the stdin receiver
    pub fn new() -> (Self, Sender<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(Self::BUFFER_SIZE);

        return (
            Self {
                stdin_rx: rx,
                ptys: Vec::new(),
            },
            tx,
        );
    }

    pub fn new_channel(
        &mut self,
        id: usize,
    ) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>, watch::Receiver<bool>) {
        let (stdout_tx, stdout_rx) = mpsc::channel(Self::BUFFER_SIZE);
        let (stdin_tx, stdin_rx) = mpsc::channel(Self::BUFFER_SIZE);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        self.ptys.push(Channel {
            id,
            rx: stdout_rx,
            tx: stdin_tx,
            shutdown_tx,
        });

        return (stdout_tx, stdin_rx, shutdown_rx);
    }

    /// Shutdown a pty thread and remove it from the channel controller.
    pub async fn send_shutdown(&mut self, id: usize) {
        for i in 0..self.ptys.len() {
            if self.ptys[i].id == id {
                // Try to shutdown, if this fails then we just exit.
                if self.ptys[i].shutdown_tx.send(true).is_ok() {
                    // Give the thread a chance to shutdown.
                    std::thread::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));
                }

                self.ptys.remove(i);
                return;
            }
        }
    }

    pub async fn shutdown_all(mut self) {
        while self.ptys.len() > 0 {
            // Try to shutdown, if this fails then we just exit.
            if self.ptys[0].shutdown_tx.send(true).is_ok() {
                // Give the thread a chance to shutdown.
                std::thread::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));
            }

            self.ptys.remove(0);
        }
    }

    pub async fn wait_for_message(&mut self) -> ControllerResponse {
        let bytes;
        let mut id = None;

        if self.ptys.is_empty() {
            bytes = self.stdin_rx.recv().await;
        } else {
            tokio::select! {
                b = self.stdin_rx.recv() => {
                    bytes = b;
                }

                (b, i, _) = futures::future::select_all(
                self.ptys
                    .iter_mut()
                    .map(|pair| pair.rx.recv().boxed())) => {
                        bytes = b;
                        id = Some(i);
                   }
            }
        }

        if let Some(i) = id {
            id = Some(self.ptys[i].id)
        }

        if bytes.is_none() {
            panic!("{:?}", id);
        }

        return ControllerResponse { bytes, id };
    }

    pub async fn write_bytes(&mut self, id: usize, bytes: Vec<u8>) -> Result<(), MuxideError> {
        for channel in &mut self.ptys {
            if channel.id == id {
                let slp = time::sleep(Duration::from_millis(Self::SEND_TIMEOUT_MS));

                select! {
                    res = channel.tx.send(bytes) => {
                        if let Err(e) = res {
                            return Err(ErrorType::PTYWriteError { description: format!("Error while sending stdin. Error: {}", e)}.into_error());
                        }
                    }
                    _ = slp => {
                        return Err(ErrorType::PTYWriteError { description: String::from("Timout while sending stdin.")}.into_error());
                    }
                }

                return Ok(());
            }
        }

        return Err(ErrorType::PTYWriteError {
            description: format!("No panel with the id: {}", id),
        }
        .into_error());
    }
}
