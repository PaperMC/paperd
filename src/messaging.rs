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
use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::{c_long, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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

#[repr(C)]
struct Message {
    m_type: c_long,
    data: Data,
}

#[repr(C)]
struct Data {
    response_queue: i32,
    message_type: i32,
    message_length: i32,
    message: [u8; 1000],
}

impl MessageChannel {
    pub fn send_message(&self, msg: &str) -> Result<(), i32> {
        let mut message = Message {
            m_type: MESSAGE_TYPE,
            data: Data {
                response_queue: 0,
                message_type: 0,
                message_length: 0,
                message: [0; 1000],
            },
        };

        let b = msg.as_bytes();
        let mut len = 0;
        for (place, data) in message.data.message.iter_mut().zip(b) {
            *place = *data;
            len += 1;
        }
        message.data.message_length = len;

        let res = unsafe {
            msgsnd(
                self.msq_id,
                &mut message as *mut _ as *mut c_void,
                size_of::<Data>(),
                0,
            )
        };

        return Err(res);
    }
}
