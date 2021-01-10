use futures::FutureExt;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{self, Duration};

struct ChannelTriple {
    id: usize,
    rx: Receiver<Vec<u8>>,
    tx: Sender<()>,
}

pub struct ControllerResponse {
    pub bytes: Option<Vec<u8>>,
    pub id: Option<usize>,
}

pub struct ChannelController {
    stdin_rx: Receiver<Vec<u8>>,
    pty_triples: Vec<ChannelTriple>,
}

impl ChannelController {
    const SHUTDOWN_BUFFER_SIZE: usize = 5;
    const BUFFER_SIZE: usize = 100;
    const SHUTDOWN_TIMEOUT_MS: u64 = 200;

    /// Returns self, and the stdin receiver
    pub fn new() -> (Self, Sender<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(Self::BUFFER_SIZE);

        return (
            Self {
                stdin_rx: rx,
                pty_triples: Vec::new(),
            },
            tx,
        );
    }

    pub fn new_pair(&mut self, id: usize) -> (Sender<Vec<u8>>, Receiver<()>) {
        let (tx, rx) = mpsc::channel(Self::BUFFER_SIZE);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(Self::SHUTDOWN_BUFFER_SIZE);

        self.pty_triples.push(ChannelTriple {
            id,
            rx,
            tx: shutdown_tx,
        });

        return (tx, shutdown_rx);
    }

    /// Shutdown a pty thread and remove it from the channel controller.
    pub async fn send_shutdown(&mut self, id: usize) {
        for i in 0..self.pty_triples.len() {
            if self.pty_triples[i].id == id {
                let slp = time::sleep(Duration::from_millis(Self::SHUTDOWN_TIMEOUT_MS));

                select! {
                    _ = self.pty_triples[i].tx.send(()) => {}
                    _ = slp => {}
                }

                self.pty_triples.remove(i);
                return;
            }
        }
    }

    pub async fn wait_for_message(&mut self) -> ControllerResponse {
        let bytes;
        let mut id = None;

        tokio::select! {
            b = self.stdin_rx.recv() => {
                bytes = b;
            }

            (b, i, _) = futures::future::select_all(
            self.pty_triples
                .iter_mut()
                .map(|mut pair| pair.rx.recv().boxed())) => {
                    bytes = b;
                    id = Some(i);
               }
        }

        if let Some(i) = id {
            id = Some(self.pty_triples[i].id)
        }

        if bytes.is_none() {
            panic!("{:?}", id);
        }

        return ControllerResponse { bytes, id };
    }
}
