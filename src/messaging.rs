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

use crate::messages::{MessageHandler, ServerErrorMessage};
use crate::util::{ExitError, ExitValue};
use paperd_lib::{close_socket, receive_message, send_message, Message, MessageHeader, Socket};
use serde::de::DeserializeOwned;
use serde::Serialize;
use {nix::errno::Errno, paperd_lib::Error};

pub struct MessageSocket {
    sock: Socket,
    pub print_err: bool,
}

macro_rules! message_resp {
    ($msg:ident, $self:ident) => {
        match $msg {
            Some(m) => m,
            None => {
                if $self.print_err {
                    eprintln!("The Paper server closed the socket");
                }
                return Err(ExitValue::Code(1));
            }
        }
    };
}

impl MessageSocket {
    pub fn new(sock: Socket) -> Self {
        return MessageSocket {
            sock,
            print_err: true,
        };
    }

    pub fn send_message<T>(&self, message: &T) -> Result<(), ExitValue>
    where
        T: MessageHandler + Serialize,
    {
        let msg = match serde_json::to_string(message) {
            Ok(s) => s,
            Err(e) => {
                if self.print_err {
                    eprintln!("Failed to serialize JSON: {}", e);
                }
                return Err(ExitValue::Code(1));
            }
        };

        let message = Message {
            header: MessageHeader {
                message_type: T::type_id(),
                message_length: msg.len() as i64,
            },
            message_text: msg,
        };

        let res = send_message(self.sock, &message);
        match res {
            Err(Error::Nix(nix::Error::Sys(Errno::EPIPE), _)) => {
                if self.print_err {
                    eprintln!("Socket closed");
                }
                return Err(ExitValue::Shutdown);
            }
            Err(_) => {
                if self.print_err {
                    res.conv("Error attempting to send message to Paper server")?;
                } else {
                    res.map_err(|_| ExitValue::Code(1))?;
                };
            }
            _ => {}
        }

        return Ok(());
    }

    pub fn receive_message<R: DeserializeOwned>(&self) -> Result<R, ExitValue> {
        return self.receive_loop(|| true);
    }

    pub fn receive_loop<R, F>(&self, keep_waiting_filter: F) -> Result<R, ExitValue>
    where
        R: DeserializeOwned,
        F: Fn() -> bool,
    {
        let msg = loop {
            match receive_message(self.sock) {
                Ok(m) => break m,
                Err(Error::Nix(nix::Error::Sys(Errno::EAGAIN), _)) => {
                    if keep_waiting_filter() {
                        continue;
                    } else {
                        return Err(ExitValue::Code(1));
                    }
                }
                Err(Error::Nix(nix::Error::Sys(Errno::UnknownErrno), s)) => {
                    return Err(Error::Nix(nix::Error::Sys(Errno::EAGAIN), s))
                        .conv(format!("Timeout occurred during the transfer of a message"));
                }
                Err(e) => {
                    return Err(e).conv("Error attempting to receive message from Paper server")
                }
            }
        };

        let msg = message_resp!(msg, self);
        return self.handle_message(&msg);
    }

    fn handle_message<R: DeserializeOwned>(&self, msg: &Message) -> Result<R, ExitValue> {
        let msg_text = msg.message_text.as_str();

        return match serde_json::from_str::<R>(msg_text) {
            Ok(r) => Ok(r),
            Err(e) => match serde_json::from_str::<ServerErrorMessage>(msg_text) {
                Ok(message) => {
                    if message.is_shutdown {
                        Err(ExitValue::Shutdown)
                    } else {
                        if self.print_err && message.error.is_some() {
                            eprintln!("{}", message.error.unwrap());
                        }
                        Err(ExitValue::Code(1))
                    }
                }
                Err(_) => {
                    if self.print_err {
                        eprintln!("Failed to parse response from server: {}", e);
                    }
                    Err(ExitValue::Code(1))
                }
            },
        };
    }
}

impl Drop for MessageSocket {
    fn drop(&mut self) {
        self.print_err = false;
        let _ = close_socket(self.sock);
    }
}
