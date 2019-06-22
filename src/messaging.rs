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

use nix::libc::{ftok, msgget, msgsnd, IPC_CREAT};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::{c_long, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process;

pub fn open_message_channel<P: AsRef<Path>>(pid_file: P) -> Result<MessageChannel, i32> {
    let pid_file = pid_file.as_ref();
    let pid_file = match pid_file.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Failed to canonicalize {}", pid_file.to_string_lossy());
            return Err(1);
        }
    };

    let file_name = match CString::new(pid_file.as_os_str().to_os_string().as_bytes()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "Failed to initialize message channel for {}",
                pid_file.to_string_lossy()
            );
            return Err(1);
        }
    };

    let msq_id: i32 = unsafe {
        let msg_key = ftok(file_name.as_ptr(), 'P' as i32);
        msgget(msg_key, 0o666 | IPC_CREAT)
    };

    return Ok(MessageChannel { msq_id });
}

pub struct MessageChannel {
    msq_id: i32,
}

const MESSAGE_TYPE: c_long = 0x7654;
const MESSAGE_LENGTH: usize = 100;

#[repr(C)]
struct Message {
    m_type: c_long,
    data: Data,
}

#[repr(C)]
struct Data {
    response_pid: u32,
    message_type: i16,
    message_length: u8,
    message: [u8; MESSAGE_LENGTH],
}

pub trait MessageHandler {
    fn type_id(&self) -> i16;
}

#[derive(Serialize, Deserialize)]
pub struct StopMessage {}

impl MessageHandler for StopMessage {
    fn type_id(&self) -> i16 {
        return 0;
    }
}

#[derive(Serialize, Deserialize)]
pub struct SendCommandMessage {
    pub message: String,
}

impl MessageHandler for SendCommandMessage {
    fn type_id(&self) -> i16 {
        return 1;
    }
}

impl MessageChannel {
    pub fn send_message<T: Serialize + MessageHandler>(&self, message: T) -> Result<(), i32> {
        let msg = match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize JSON: {}", e);
                return Err(1);
            }
        };

        let mut data = msg.as_bytes();

        while data.len() > 0 {
            let size = min(data.len(), MESSAGE_LENGTH);
            let ret = self.send_paged_message(message.type_id(), &data[..size], size == data.len());
            if ret == -1 {
                return Err(ret);
            }
            data = &data[size..];
        }

        return Ok(());
    }

    fn send_paged_message(&self, type_id: i16, msg: &[u8], fin: bool) -> i32 {
        let mut message = Message {
            m_type: MESSAGE_TYPE,
            data: Data {
                response_pid: process::id(),
                message_type: type_id,
                message_length: 0,
                message: [0; MESSAGE_LENGTH],
            },
        };

        let len = msg.len();
        {
            let message_slice = &mut message.data.message[..len];
            message_slice.copy_from_slice(msg);
        }

        let mut len = len as u8;
        if fin {
            // Set the far left bit to denote this is the end of a message
            len |= 0x80;
        }
        message.data.message_length = len;

        return unsafe {
            msgsnd(
                self.msq_id,
                &mut message as *mut _ as *mut c_void,
                size_of::<Data>(),
                0,
            )
        };
    }
}
