use crate::error::{ErrorType, MuxideError};
use crate::geometry::Size;
use either::{Either, Left, Right};
use futures::FutureExt;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{self, Duration};

#[derive(Clone, Debug)]
pub enum Message {
    Bytes(Vec<u8>),
    Resize(Size),
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct ControllerResponse {
    pub bytes: Vec<u8>,
    pub id: Option<usize>,
}

pub struct ChannelController {
    stdin_rx: Receiver<Vec<u8>>,
    ptys: Vec<Channel>,
}

/// Represents a pty, storing the id of the channels and two for communication with the channel and
/// 1 to signal a shutdown.
struct Channel {
    id: usize,
    rx: Receiver<Vec<u8>>,
    tx: Sender<Message>,
}

impl ChannelController {
    /// The size of the buffer for the mpsc channels
    const BUFFER_SIZE: usize = 100;
    /// The amount of time allowed for each pty to shutdown
    const SHUTDOWN_TIMEOUT_MS: u64 = 200;
    /// The amount of time to delay when writing
    const SEND_TIMEOUT_MS: u64 = 200;

    /// Creates a new instance of the channel controller, it returns an instance and the stdin
    /// sender that should send any stdin input..
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

    /// Open a new channel the necessary components are kept and tracked in the controller whilst,
    /// the send stdout sender, input receiver and shutdown receiver are returned.
    pub fn new_channel(&mut self, id: usize) -> (Sender<Vec<u8>>, Receiver<Message>) {
        let (stdout_tx, stdout_rx) = mpsc::channel(Self::BUFFER_SIZE);
        let (stdin_tx, stdin_rx) = mpsc::channel(Self::BUFFER_SIZE);

        self.ptys.push(Channel {
            id,
            rx: stdout_rx,
            tx: stdin_tx,
        });

        return (stdout_tx, stdin_rx);
    }

    pub fn remove_panel(&mut self, id: usize) -> Result<(), MuxideError> {
        for i in 0..self.ptys.len() {
            let pty = &self.ptys[i];
            if pty.id == id {
                self.ptys.remove(i);
                return Ok(());
            }
        }

        return Err(ErrorType::NoPanelWithIDError { id }.into_error());
    }

    /// Shutdown a pty thread and remove it from the channel controller.
    pub async fn send_shutdown(&mut self, id: usize) {
        for i in 0..self.ptys.len() {
            if self.ptys[i].id == id {
                let timer = tokio::time::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));

                select! {
                    result = self.ptys[i].tx.send(Message::Shutdown) => {
                         // Try to shutdown, if this fails then we just exit.
                        if result.is_ok() {
                            // Give the thread a chance to shutdown.
                            std::thread::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));
                        }
                    }
                    _ = timer => {}
                }

                self.ptys.remove(i);
                return;
            }
        }
    }

    /// Shutdown all open pty's.
    pub async fn shutdown_all(mut self) {
        while self.ptys.len() > 0 {
            let timer = tokio::time::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));

            select! {
                result = self.ptys[0].tx.send(Message::Shutdown) => {
                     // Try to shutdown, if this fails then we just exit.
                    if result.is_ok() {
                        // Give the thread a chance to shutdown.
                        std::thread::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));
                    }
                }
                _ = timer => {}
            }

            self.ptys.remove(0);
        }
    }

    /// Wait until a receiver, from the pty's or the stdin receiver receives a message and return
    /// information about what source the data came from and what the message was or the id of a pty
    /// that has shutdown.
    pub async fn wait_for_message(&mut self) -> Either<ControllerResponse, Option<usize>> {
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
            return Right(id);
        }

        return Left(ControllerResponse {
            bytes: bytes.unwrap(),
            id,
        });
    }

    /// Send bytes to a channel with the specified id. Returns an error if something failed when
    /// sending the data or if no panel exists with the specified id.
    pub async fn write_bytes(&mut self, id: usize, bytes: Vec<u8>) -> Result<(), MuxideError> {
        return self.write_message(id, Message::Bytes(bytes)).await;
    }

    /// Send a resize message to a channel with the specified id. Returns an error if something
    /// failed when sending the data or if no panel exists with the specified id.
    pub async fn write_resize(&mut self, id: usize, size: Size) -> Result<(), MuxideError> {
        return self.write_message(id, Message::Resize(size)).await;
    }

    /// Send a message to a channel with the specified id. Returns an error if something
    /// failed when sending the data or if no panel exists with the specified id.
    pub async fn write_message(&mut self, id: usize, message: Message) -> Result<(), MuxideError> {
        for channel in &mut self.ptys {
            if channel.id == id {
                let slp = time::sleep(Duration::from_millis(Self::SEND_TIMEOUT_MS));

                select! {
                    res = channel.tx.send(message) => {
                        if let Err(e) = res {
                            return Err(ErrorType::PTYWriteError { description: format!("Error while sending message. Error: {}", e)}.into_error());
                        }
                    }
                    _ = slp => {
                        return Err(ErrorType::PTYWriteError { description: String::from("Timeout while sending message.")}.into_error());
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
