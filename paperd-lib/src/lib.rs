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

#![no_std]

use nix::libc::c_long;

pub mod libc {
    use nix::libc::{c_int, c_long, c_void, key_t, size_t, ssize_t};

    pub const IPC_CREAT: c_int = 0o1000;
    pub const IPC_RMID: c_int = 0;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    extern "C" {
        pub fn msgctl(msqid: c_int, cmd: c_int, buf: *mut c_void) -> c_int;
        pub fn msgget(key: key_t, msgflg: c_int) -> c_int;
        pub fn msgrcv(msqid: c_int, msgp: *mut c_void, msgsz: size_t, msgtyp: c_long, msgflg: c_int) -> ssize_t;
        pub fn msgsnd(msqid: c_int, msgp: *const c_void, msgsz: size_t, msgflg: c_int) -> c_int;
    }
}

pub const MESSAGE_TYPE: c_long = 0x7654;
pub const MESSAGE_LENGTH: usize = 100;

#[repr(C)]
pub struct Message {
    pub m_type: c_long,
    pub data: Data,
}

#[repr(C)]
pub struct Data {
    pub response_chan: i32,
    pub response_pid: u32,
    pub message_type: i16,
    pub message_length: u8,
    pub message: [u8; MESSAGE_LENGTH],
}
