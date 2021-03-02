use crate::error::{ErrorType, MuxideError};
use crate::geometry::Size;
use futures::FutureExt;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{self, Duration};

#[derive(Clone, Debug, Hash)]
pub enum ServerMessage {
    Bytes(Vec<u8>),
    Resize(Size),
    Shutdown,
}

#[derive(Clone, Debug, Hash)]
pub enum PtyMessage {
    Bytes(Vec<u8>),
    Error(MuxideError),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ChannelID {
    Pty(usize),
    Stdin,
}

#[derive(Clone, Debug)]
pub struct ControllerResponse {
    pub bytes: Vec<u8>,
    pub id: ChannelID,
}

#[derive(Clone, Debug)]
pub struct ChannelWaitFail {
    pub id: ChannelID,
    pub error: Option<MuxideError>,
}

/// Represents a pty, storing the id of the channels and two for communication with the channel and
/// 1 to signal a shutdown.
struct Channel {
    id: usize,
    rx: Receiver<PtyMessage>,
    tx: Sender<ServerMessage>,
}

pub struct ChannelController {
    stdin_rx: Receiver<Vec<u8>>,
    ptys: Vec<Channel>,
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
    pub fn new_channel(&mut self, id: usize) -> (Sender<PtyMessage>, Receiver<ServerMessage>) {
        let (stdout_tx, stdout_rx) = mpsc::channel(Self::BUFFER_SIZE);
        let (stdin_tx, stdin_rx) = mpsc::channel(Self::BUFFER_SIZE);

        self.ptys.push(Channel {
            id,
            rx: stdout_rx,
            tx: stdin_tx,
        });

        return (stdout_tx, stdin_rx);
    }

    /// Shutdown a pty thread and remove it from the channel controller.
    pub async fn send_shutdown(&mut self, id: usize) {
        for i in 0..self.ptys.len() {
            if self.ptys[i].id == id {
                let timer = tokio::time::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));

                select! {
                    result = self.ptys[i].tx.send(ServerMessage::Shutdown) => {
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
                result = self.ptys[0].tx.send(ServerMessage::Shutdown) => {
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
    pub async fn wait_for_message(&mut self) -> Result<ControllerResponse, ChannelWaitFail> {
        let bytes;
        let channel_id: ChannelID;
        let mut error = None;
        let mut index = None;

        if self.ptys.is_empty() {
            bytes = self.stdin_rx.recv().await;
            channel_id = ChannelID::Stdin;
        } else {
            tokio::select! {
                b = self.stdin_rx.recv() => {
                    bytes = b;
                }

                (message, i, _) = futures::future::select_all(
                self.ptys
                    .iter_mut()
                    .map(|pair| pair.rx.recv().boxed())) => {
                        match message {
                            Some(PtyMessage::Bytes(b)) => {
                                bytes = Some(b);
                                error = None;
                            },
                            Some(PtyMessage::Error(e)) => {
                                bytes = None;
                                error = Some(e);
                            },
                            None => {
                                bytes = None;
                            }
                        }

                        index = Some(i);
                   }
            }

            if let Some(i) = index {
                channel_id = ChannelID::Pty(self.ptys[i].id);
            } else {
                channel_id = ChannelID::Stdin;
            }
        }

        if let Some(bytes) = bytes {
            return Ok(ControllerResponse {
                bytes,
                id: channel_id,
            });
        } else {
            if channel_id != ChannelID::Stdin {
                self.ptys.remove(index.unwrap());
            }

            return Err(ChannelWaitFail {
                id: channel_id,
                error,
            });
        }
    }

    pub fn remove_panel(&mut self, id: usize) {
        for i in 0..self.ptys.len() {
            if self.ptys[i].id == id {
                self.ptys.remove(i);
                return;
            }
        }
    }

    /// Send bytes to a channel with the specified id. Returns an error if something failed when
    /// sending the data or if no panel exists with the specified id.
    pub async fn write_bytes(&mut self, id: usize, bytes: Vec<u8>) -> Result<(), MuxideError> {
        return self.write_message(id, ServerMessage::Bytes(bytes)).await;
    }

    /// Send a resize message to a channel with the specified id. Returns an error if something
    /// failed when sending the data or if no panel exists with the specified id.
    pub async fn write_resize(&mut self, id: usize, size: Size) -> Result<(), MuxideError> {
        return self.write_message(id, ServerMessage::Resize(size)).await;
    }

    /// Send a message to a channel with the specified id. Returns an error if something
    /// failed when sending the data or if no panel exists with the specified id.
    pub async fn write_message(
        &mut self,
        id: usize,
        message: ServerMessage,
    ) -> Result<(), MuxideError> {
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
