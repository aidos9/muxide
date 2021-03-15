use crate::pty_controller::{PtyMessage, ServerMessage};
use crate::pty::Pty;
use crate::ErrorType;
use muxide_logging::error;
use nix::poll;
use std::os::unix::io::AsRawFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;

/// The timeout used when we poll the PTY for if it is available.
const POLL_TIMEOUT_MS: i32 = 100;
/// The timeout used when reporting an error.
const ERROR_TIMEOUT_MS: u64 = 100;
/// The timeout used when writing to a file.
const FILE_TIMEOUT_MS: u64 = 750;

/// This method runs a pty, handling shutdown messages, stdin and stdout.
/// It should be spawned in a thread.
pub async fn pty_manager(
    mut p: Pty,
    tx: Sender<PtyMessage>,
    mut stdin_rx: Receiver<ServerMessage>,
) {
    macro_rules! pty_error {
        ($tx:expr, $e:expr, $log_message:expr) => {
            error!($log_message);


            // This could error out and if it does then we just assume the controller will deal with it.
            select! {
                _ = $tx.send(PtyMessage::Error($e)) => {},
                _ = tokio::time::sleep(Duration::from_millis(ERROR_TIMEOUT_MS)) => {},
            }
        };

        ($tx:expr, $e:expr) => {
            let e = $e;
            error!(format!(
                "An error occurred in the pty thread. Error description: {:?}",
                &e
            ));

            // This could error out and if it does then we just assume the controller will deal with it.
            select! {
                _ = $tx.send(PtyMessage::Error(e)) => {},
                _ = tokio::time::sleep(Duration::from_millis(ERROR_TIMEOUT_MS)) => {},
            }
        };
    };

    let pfd = poll::PollFd::new(p.as_raw_fd(), poll::PollFlags::POLLIN);

    loop {
        select! {
            res = tokio::spawn(async move {
                // For some reason rust reports that this value is unassigned.
                #[allow(unused_assignments)]
                let mut res = Ok(false);

                loop {
                    match poll::poll(&mut [pfd], POLL_TIMEOUT_MS) {
                        Ok(poll_response) => {
                            // If we get 0, that means the call timed out, a negative value is an error
                            // in my understanding but nix, I believe should handle that as an error
                            if poll_response > 0 {
                                //res = true;
                                res = Ok(true);
                                break;
                            }
                        }
                        Err(e) => {
                            // If we receive an error here, it is a first class (unrecoverable) error.
                            res = Err(e);
                            break;
                        },
                    }
                }

                res
            }) => {
                if res.is_err() {
                    pty_error!(tx, ErrorType::new_failed_read_poll_error(), "Something unexpected went wrong whilst reading the pty poll");
                    return;
                }

                match res.unwrap() {
                    Ok(b) => {
                        if !b {
                            continue;
                        }
                    }
                    Err(e) => {
                        pty_error!(tx, ErrorType::new_failed_read_poll_error(), format!("Failed to poll for available data. Error: {}", e));
                        return;
                    },
                }

                let mut buf = vec![0u8; 4096];
                let res = p.file().read(&mut buf).await;

                if let Ok(count) = res {
                    if count == 0 {
                        if p.running() == Some(false) {
                            pty_error!(tx, ErrorType::new_pty_stopped_running_error());
                            return;
                        }
                    }

                    let mut cpy = vec![0u8; count];
                    cpy.copy_from_slice(&buf[0..count]);

                    // Ignore any errors with communicating data.
                    match tx.send(PtyMessage::Bytes(cpy)).await {
                        Ok(_) => (),
                        Err(_) => {
                            pty_error!(tx, ErrorType::new_failed_to_send_message_error());
                            return;
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                } else {
                    pty_error!(tx, ErrorType::new_failed_to_read_pty_error());
                    return;
                }
            },
            res = stdin_rx.recv() => {
                if let Some(message) = res {
                    match message {
                        ServerMessage::Bytes(bytes) => {
                            select! {
                                res = p.file().write_all(&bytes) => {
                                    match res {
                                        Ok(_) => (),
                                        Err(e) => {
                                            pty_error!(tx, ErrorType::new_pty_write_error(e.to_string()));
                                            return;
                                        },
                                    }
                                },
                                _ = tokio::time::sleep(Duration::from_millis(FILE_TIMEOUT_MS)) => {},
                            }
                        },
                        ServerMessage::Resize(size) => {
                            p.resize(&size).unwrap();
                        },
                        ServerMessage::Shutdown => {
                            break;
                        },
                    }
                } else {
                    pty_error!(tx, ErrorType::new_pty_stdin_receiver_closed_error());
                    return;
                }
            }
        }
    }
}
