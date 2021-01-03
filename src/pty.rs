use crate::error::{Error, ErrorType};
use crate::geometry::Size;
use mio::unix::SourceFd;
use mio::{Interest, Registry, Token};
use nix::fcntl::{self, FcntlArg, OFlag};
use nix::unistd;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct PTY {
    fd: RawFd,
    file: File,
    open: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    tx: Sender<()>,
}

impl PTY {
    pub fn new(program: &str, size: &Size) -> Result<Self, Error> {
        let (master, slave) = Self::open_pty(size)?;

        let mut pty_command_handle = match unsafe {
            Command::new(program)
                .stdin(
                    Stdio::from_raw_fd(slave), // Unsafe
                )
                .stdout(
                    Stdio::from_raw_fd(slave), // Unsafe
                )
                .stderr(
                    Stdio::from_raw_fd(slave), // Unsafe
                )
                .pre_exec(Self::in_between) // Unsafe
                .spawn()
        } {
            Ok(h) => h,
            Err(e) => {
                return Err(ErrorType::PTYSpawnError {
                    description: format!("{}", e),
                }
                .into_error());
            }
        };

        let (tx, rx) = mpsc::channel();
        let open = Arc::new(AtomicBool::new(true));
        let cp = open.clone();

        let manager_handle = thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = pty_command_handle.kill();
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => match pty_command_handle.try_wait() {
                        Ok(Some(_)) => {
                            break;
                        }
                        Ok(None) => (),
                        Err(e) => panic!(e),
                    },
                }
            }

            cp.store(false, Ordering::Relaxed);
        });

        let pty = Self {
            fd: master,
            file: unsafe { File::from_raw_fd(master) },
            open,
            handle: Some(manager_handle),
            tx,
        };

        pty.resize(size)?;

        return Ok(pty);
    }

    pub fn is_running(&self) -> bool {
        return self.open.load(Ordering::Relaxed);
    }

    pub fn resize(&self, size: &Size) -> Result<(), Error> {
        let res = unsafe { libc::ioctl(self.fd, libc::TIOCSWINSZ, &size.to_winsize()) };

        if res != 0 {
            return Err(ErrorType::IOCTLError {
                code: res,
                outcome: "Failed to resize the PTY.".to_string(),
            }
            .into_error());
        }

        return Ok(());
    }

    fn open_pty(size: &Size) -> Result<(RawFd, RawFd), Error> {
        let res = nix::pty::openpty(Some(&size.to_winsize()), None).map_err(|e| {
            ErrorType::OpenPTYError {
                reason: format!("{}", e),
            }
            .into_error()
        })?;

        let (master, slave) = (res.master, res.slave);

        let res = OFlag::from_bits(fcntl::fcntl(master, FcntlArg::F_GETFL).map_err(|e| {
            ErrorType::FCNTLError {
                reason: format!("{}", e),
            }
            .into_error()
        })?)
        .unwrap();
        fcntl::fcntl(master, FcntlArg::F_SETFL(res)).map_err(|e| {
            ErrorType::FCNTLError {
                reason: format!("{}", e),
            }
            .into_error()
        })?;

        return Ok((master, slave));
    }

    fn in_between() -> std::io::Result<()> {
        unistd::setsid()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let res = unsafe { libc::ioctl(0, libc::TIOCSCTTY as u64, 1) };

        if res != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to make process the controlling process: {}", res),
            ));
        }

        return Ok(());
    }
}

impl Drop for PTY {
    fn drop(&mut self) {
        // We try to gracefully close the process ignoring any errors in the process
        if self.is_running() {
            // If we successfully send the stop message then join the handle
            if let Ok(_) = self.tx.send(()) {
                match self.handle.take() {
                    Some(handle) => {
                        let _ = handle.join();
                    }
                    None => (),
                }
            }
        }

        // Try to close, may fail if the file descriptor was already closed but we ignore that error.
        let _ = unistd::close(self.fd);
    }
}

impl Read for PTY {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return self.file.read(buf);
    }
}

impl Write for PTY {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.file.write(buf);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.file.flush();
    }
}

impl std::ops::Deref for PTY {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl std::ops::DerefMut for PTY {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl mio::event::Source for PTY {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        return SourceFd(&self.fd).register(registry, token, interests);
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        return SourceFd(&self.fd).reregister(registry, token, interests);
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        return SourceFd(&self.fd).deregister(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_read_some() {
        let mut pty = PTY::new("/usr/local/bin/fish", &Size::new(300, 300)).unwrap();

        let mut bytes = [0u8; 4096];
        let count = pty.read(&mut bytes).unwrap();
        assert!(count > 1);

        let cmd = b"exit\n";
        pty.write_all(cmd).unwrap();
        pty.flush().unwrap();

        let count = pty.read(&mut bytes).unwrap();
        assert!(String::from_utf8_lossy(&bytes[..count])
            .to_string()
            .starts_with("exit"));
    }
}
