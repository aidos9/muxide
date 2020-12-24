use crate::geometry::Size;
use nix::fcntl::{self, FcntlArg, OFlag};
use nix::unistd;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
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
    pub fn new(program: &str, size: &Size) -> Self {
        let (master, slave) = Self::open_pty(size).unwrap();

        let mut handle = unsafe {
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
                .unwrap()
        };

        let (tx, rx) = mpsc::channel();
        let open = Arc::new(AtomicBool::new(true));
        let cp = open.clone();

        let handle = thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = handle.kill();
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => match handle.try_wait() {
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
            handle: Some(handle),
            tx,
        };

        pty.resize(size).unwrap();

        return pty;
    }

    pub fn is_running(&self) -> bool {
        return self.open.load(Ordering::Relaxed);
    }

    pub fn kill(mut self) {
        if self.is_running() {
            self.tx.send(());
            self.handle.take().unwrap().join().unwrap();
        }
    }

    pub fn resize(&self, size: &Size) -> Result<(), ()> {
        unsafe {
            if libc::ioctl(self.fd, libc::TIOCSWINSZ, &size.to_winsize()) != 0 {
                return Err(());
            }
        }

        return Ok(());
    }

    fn open_pty(size: &Size) -> Result<(RawFd, RawFd), ()> {
        let res = nix::pty::openpty(Some(&size.to_winsize()), None).unwrap();
        let (master, slave) = (res.master, res.slave);

        let res = OFlag::from_bits(fcntl::fcntl(master, FcntlArg::F_GETFL).unwrap()).unwrap();
        fcntl::fcntl(master, FcntlArg::F_SETFL(res)).unwrap();

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
        if self.is_running() {
            let _ = self.tx.send(());
            let _ = self.handle.take().unwrap().join();
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_read_some() {
        let mut pty = PTY::new("/usr/local/bin/fish", &Size::new(300, 300));

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
