/*
This code was heavily based and inspired by https://github.com/pkgw/stund/blob/master/tokio-pty-process/
*/

use crate::error::{Error, ErrorType};
use nix::fcntl::{FcntlArg, OFlag};
use nix::pty::Winsize;
use nix::{fcntl, unistd};
use std::io;
use std::os::unix::io::{FromRawFd, RawFd};
use std::process::Stdio;
use std::task::Context;
use tokio::fs::File;
use tokio::io::{AsyncRead, ReadBuf};
use tokio::macros::support::{Pin, Poll};
use tokio::process::Command;

pub struct Pty {
    file: File,
    handle: tokio::process::Child,
}

// pub struct PtyRead(BiLock<File>);
// pub struct PtyWrite(BiLock<File>);

impl Pty {
    pub fn open(cmd: &str) -> Result<Self, Error> {
        // Comment taken directly from: https://github.com/pkgw/stund/blob/master/tokio-pty-process/src/lib.rs
        // On MacOS, O_NONBLOCK is not documented as an allowed option to
        // posix_openpt(), but it is in fact allowed and functional, and
        // trying to add it later with fcntl() is forbidden. Meanwhile, on
        // FreeBSD, O_NONBLOCK is *not* an allowed option to
        // posix_openpt(), and the only way to get a nonblocking PTY
        // master is to add the nonblocking flag with fcntl() later. So,
        // we have to jump through some #[cfg()] hoops.
        const APPLY_NONBLOCK_LATER: bool = cfg!(target_os = "freebsd");

        let (file_descriptor, slave) = Self::open_pty().unwrap();

        let mut pty_command_handle = match unsafe {
            Command::new(cmd)
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
                .kill_on_drop(true)
                .spawn()
        } {
            Ok(h) => h,
            Err(e) => {
                return Err(ErrorType::PTYSpawnError {
                    description: e.to_string(),
                }
                .into_error());
            }
        };

        if APPLY_NONBLOCK_LATER {
            let flags = unsafe { libc::fcntl(file_descriptor, libc::F_GETFL, 0) };
            if flags < 0 {
                return Err(ErrorType::FCNTLError {
                    reason: io::Error::last_os_error().to_string(),
                }
                .into_error());
            }

            let res =
                unsafe { libc::fcntl(file_descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) };

            if res == -1 {
                return Err(ErrorType::FCNTLError {
                    reason: io::Error::last_os_error().to_string(),
                }
                .into_error());
            }
        }

        return Ok(Self {
            file: unsafe { File::from_raw_fd(file_descriptor) },
            handle: pty_command_handle,
        });
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

    fn open_pty() -> Result<(RawFd, RawFd), ()> {
        let res = nix::pty::openpty(
            Some(&Winsize {
                ws_row: 24,
                ws_col: 80,
                ws_xpixel: 0,
                ws_ypixel: 0,
            }),
            None,
        )
        .map_err(|e| ())?;

        let (master, slave) = (res.master, res.slave);

        let res =
            OFlag::from_bits_truncate(fcntl::fcntl(master, FcntlArg::F_GETFL).map_err(|e| ())?);

        fcntl::fcntl(master, FcntlArg::F_SETFL(res)).map_err(|e| ())?;

        return Ok((master, slave));
    }

    pub fn running(&mut self) -> Option<bool> {
        match self.handle.try_wait() {
            Ok(Some(_)) => return Some(false),
            Ok(None) => return Some(true),
            Err(_) => return None,
        }
    }
}

// impl AsyncRead for PtyRead {
//     fn poll_read(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &mut ReadBuf<'_>,
//     ) -> Poll<io::Result<()>> {
//         let mut l = ready!(self.0.poll_lock(cx));
//         return l.as_pin_mut().poll_read(cx, buf);
//     }
// }

impl AsyncRead for Pty {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        return Pin::new(&mut self.file).poll_read(cx, buf);
    }
}
