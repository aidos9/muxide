use futures::FutureExt;
use tokio::sync::mpsc::{self, Receiver, Sender};

struct RxPair {
    id: usize,
    rx: Receiver<Vec<u8>>,
}

pub struct ControllerResponse {
    pub bytes: Option<Vec<u8>>,
    pub id: Option<usize>,
}

pub struct ChannelController {
    stdin_rx: Receiver<Vec<u8>>,
    pty_rx: Vec<RxPair>,
}

impl ChannelController {
    const BUFFER_SIZE: usize = 100;

    pub fn new() -> (Self, Sender<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(Self::BUFFER_SIZE);

        return (
            Self {
                stdin_rx: rx,
                pty_rx: Vec::new(),
            },
            tx,
        );
    }

    pub fn new_pair(&mut self, id: usize) -> Sender<Vec<u8>> {
        let (tx, rx) = mpsc::channel(Self::BUFFER_SIZE);

        self.pty_rx.push(RxPair { id, rx });

        return tx;
    }

    pub async fn wait_for_message(&mut self) -> ControllerResponse {
        let bytes;
        let mut id = None;

        tokio::select! {
            b = self.stdin_rx.recv() => {
                bytes = b;
            }

            (b, i, _) = futures::future::select_all(
            self.pty_rx
                .iter_mut()
                .map(|mut pair| pair.rx.recv().boxed())) => {
                    bytes = b;
                    id = Some(i);
               }
        }

        if let Some(i) = id {
            id = Some(self.pty_rx[i].id)
        }

        if bytes.is_none() {
            panic!("{:?}", id);
        }

        return ControllerResponse { bytes, id };
    }
}
