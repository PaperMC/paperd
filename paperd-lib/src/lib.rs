// This file is part of paperd, the PaperMC server daemon
// Copyright (C) 2019 Kyle Wood (DemonWav)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

extern crate nix;

use nix::errno::Errno;
use nix::sys::socket::sockopt::{ReceiveTimeout, SendTimeout};
use nix::sys::socket::{
    accept, bind, connect, listen, recv, send, setsockopt, socket, AddressFamily, MsgFlags,
    SockAddr, SockFlag, SockType, UnixAddr,
};
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::{close, unlink};
use nix::NixPath;
use std::cmp::min;
use std::fmt;
use std::fmt::Display;
use std::os::unix::io::RawFd;
use std::string::FromUtf8Error;
use std::time::{Duration, Instant};

macro_rules! syscall {
    ($syscall:ident($( $args:expr ),*)) => {
        $syscall($($args,)*).map_err(|e| crate::Error::from(e).for_syscall(stringify!($syscall)))
    };
}

pub struct MessageHeader {
    pub message_type: i64,
    pub message_length: i64,
}

pub struct Message {
    pub header: MessageHeader,
    pub message_text: String,
}

pub type Socket = RawFd;

const META_SIZE: usize = 8;
const MESSAGE_SIZE: usize = 1000;
const TIMEOUT_MILLIS: u64 = 500;

pub fn create_socket() -> Result<Socket, Error> {
    let sock = syscall!(socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None
    ))?;

    let time_val = TimeVal::milliseconds((TIMEOUT_MILLIS / 2) as i64);
    syscall!(setsockopt(sock, ReceiveTimeout, &time_val))?;
    syscall!(setsockopt(sock, SendTimeout, &time_val))?;

    return Ok(sock);
}

pub fn close_socket(sock: Socket) -> Result<(), Error> {
    return syscall!(close(sock));
}

pub fn bind_socket(sock: Socket, file_path: &str) -> Result<(), Error> {
    match syscall!(unlink(file_path)) {
        Ok(()) => Ok(()),
        // ENOENT == no such file or directory, we don't care if it doesn't exist
        Err(Error::Nix(nix::Error::Sys(Errno::ENOENT), _)) => Ok(()),
        Err(e) => Err(e),
    }?;

    let addr = UnixAddr::new(file_path)?;
    let sock_addr = SockAddr::Unix(addr);

    syscall!(bind(sock, &sock_addr))?;

    syscall!(listen(sock, 128))?;

    return Ok(());
}

pub fn connect_socket<P: ?Sized + NixPath>(sock_file: &P) -> Result<Socket, Error> {
    let sock = create_socket()?;

    let addr = UnixAddr::new(sock_file)?;
    let socket_addr = SockAddr::Unix(addr);

    loop {
        match syscall!(connect(sock, &socket_addr)) {
            Ok(_) => break,
            Err(Error::Nix(nix::Error::Sys(Errno::EINPROGRESS), _)) => continue,
            Err(e) => return Err(e),
        }
    }

    return Ok(sock);
}

macro_rules! handle_timeout {
    ($res:ident, $timeout:ident, $start:ident, $has_data:expr) => {
        match $res {
            Ok(amt) => {
                $start = std::time::Instant::now();
                Ok(amt)
            }
            Err(Error::Nix(nix::Error::Sys(Errno::EAGAIN), s)) => {
                if $start.elapsed() > $timeout {
                    if ($has_data) {
                        // If we've received data and we have a timeout, we can't keep listening
                        Err((Error::Nix(nix::Error::Sys(Errno::UnknownErrno), s)))
                    } else {
                        Err((Error::Nix(nix::Error::Sys(Errno::EAGAIN), s)))
                    }
                } else {
                    continue;
                }
            }
            Err(e) => Err(e),
        }
    };
}

pub fn receive_message(sock: Socket) -> Result<Option<Message>, Error> {
    let message_header = read_meta(sock)?;
    let message_length = message_header.message_length as usize;

    let timeout = Duration::from_millis(TIMEOUT_MILLIS);
    let mut start = Instant::now();

    let mut message_buffer: [u8; MESSAGE_SIZE] = [0; MESSAGE_SIZE];

    let mut output_buffer = Vec::<u8>::new();

    let mut total_received: usize = 0;
    while total_received < message_length {
        let amount_left = message_length - total_received;
        let buffer_size = min(MESSAGE_SIZE, amount_left);

        let res = syscall!(recv(
            sock,
            &mut message_buffer[..buffer_size],
            MsgFlags::empty()
        ));
        let amount_received = handle_timeout!(res, timeout, start, true)?;
        if amount_received == 0 {
            return Ok(None);
        }
        total_received += amount_received;

        output_buffer.extend_from_slice(&message_buffer[..amount_received]);
    }

    let message_string = String::from_utf8(output_buffer)?;

    return Ok(Some(Message {
        header: message_header,
        message_text: message_string,
    }));
}

pub fn send_message(sock: Socket, message: &Message) -> Result<(), Error> {
    send_meta(sock, &message.header)?;

    let message_data = message.message_text.as_bytes();

    let timeout = Duration::from_millis(TIMEOUT_MILLIS);
    let mut start = Instant::now();

    let mut total_sent: usize = 0;
    let message_size = message_data.len();

    while total_sent < message_size {
        let res = syscall!(send(sock, &message_data[total_sent..], MsgFlags::empty()));
        let amount_sent = handle_timeout!(res, timeout, start, true)?;
        total_sent += amount_sent;
    }

    return Ok(());
}

pub fn accept_connection(sock: Socket) -> Result<Option<Socket>, Error> {
    let res = syscall!(accept(sock));
    return match res {
        Ok(client_sock) => Ok(Some(client_sock)),
        Err(Error::Nix(nix::Error::Sys(Errno::EAGAIN), _)) => Ok(None),
        Err(e) => Err(e),
    };
}

fn read_meta(sock: Socket) -> Result<MessageHeader, Error> {
    // meta_buffer will contain:
    //  * message_type (first 8 bytes)
    //  * message_length (last 8 bytes)
    // 8 bytes for each is overkill by an enormous margin, but 16 bytes is cheap and easy to do so
    // there's not really much downside
    //
    // Both numbers are big endian
    let mut meta_buffer: [u8; META_SIZE] = [0; META_SIZE];

    let message_type = read_i64(sock, &mut meta_buffer, true)?;
    let message_length = read_i64(sock, &mut meta_buffer, false)?;

    return Ok(MessageHeader {
        message_type,
        message_length,
    });
}

fn send_meta(sock: Socket, message_header: &MessageHeader) -> Result<(), Error> {
    write_i64(sock, message_header.message_type, true)?;
    write_i64(sock, message_header.message_length, false)?;

    return Ok(());
}

fn read_i64(sock: Socket, buffer: &mut [u8; META_SIZE], is_start: bool) -> Result<i64, Error> {
    let timeout = Duration::from_millis(TIMEOUT_MILLIS);
    let mut start = Instant::now();

    let mut total_received: usize = 0;
    while total_received < META_SIZE {
        let res = syscall!(recv(sock, &mut buffer[total_received..], MsgFlags::empty()));
        let amount_received =
            handle_timeout!(res, timeout, start, !is_start || total_received > 0)?;
        total_received += amount_received;
    }

    return Ok(i64::from_be_bytes(*buffer));
}

fn write_i64(sock: Socket, value: i64, is_start: bool) -> Result<(), Error> {
    let buffer: [u8; META_SIZE] = value.to_be_bytes();

    let timeout = Duration::from_millis(TIMEOUT_MILLIS);
    let mut start = Instant::now();

    let mut total_sent: usize = 0;
    while total_sent < META_SIZE {
        let res = syscall!(send(sock, &buffer[total_sent..], MsgFlags::empty()));
        let amount_sent = handle_timeout!(res, timeout, start, !is_start || total_sent > 0)?;
        total_sent += amount_sent;
    }

    return Ok(());
}

pub enum Error {
    Nix(nix::Error, Option<String>),
    Internal(String),
}

impl Error {
    pub fn with_message(msg: &str) -> Error {
        return Error::Internal(msg.to_string());
    }

    pub fn for_syscall(&self, syscall: &str) -> Self {
        return match &self {
            Error::Nix(e, _) => Error::Nix(e.clone(), Some(syscall.to_string())),
            Error::Internal(s) => Error::Internal(s.clone()),
        };
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match &self {
            Error::Nix(e, Some(syscall)) => {
                write!(f, "In syscall: {}, ", syscall)?;
                e.fmt(f)
            }
            Error::Nix(e, None) => e.fmt(f),
            Error::Internal(s) => write!(f, "{}", s),
        };
    }
}

impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        return Error::Nix(e, None);
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        return Error::Internal(e.to_string());
    }
}
