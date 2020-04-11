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

extern crate alloc;
extern crate jni;
extern crate nix;

#[macro_use]
mod util;

use crate::util::{get_path_string, throw};
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::null_mut;
use jni::objects::JValue::{Byte, Int, Long, Object, Short};
use jni::objects::{JClass, JObject};
use jni::sys::{jboolean, jbyte, jint, jlong, jobject, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use nix::errno::Errno;
use nix::libc::{c_char, ftok};
use nix::sys::signal::kill;
use nix::unistd::Pid;
use nix::Error::Sys;
use paperd_lib::libc::{msgctl, msgget, msgrcv, msgsnd, IPC_CREAT, IPC_RMID};
use paperd_lib::{Data, Message, MESSAGE_TYPE};

// These macros allow getting around the really dumb limitation of only allowing literals (not even
// static const strings and integers) in the concat! macro
macro_rules! data_class {
    () => {
        "com/destroystokyo/paper/daemon/Data"
    };
}

macro_rules! message_class {
    () => {
        "com/destroystokyo/paper/daemon/MsgBuf"
    };
}

macro_rules! message_field_name {
    () => {
        "com.destroystokyo.paper.daemon.Data.message"
    };
}

macro_rules! message_length {
    () => {
        100
    };
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_createQueue(
    env: JNIEnv,
    _: JClass,
    pid_file: JObject,
) -> jint {
    let mut file_path = match get_path_string(&env, pid_file) {
        Ok(str) => str,
        _ => {
            throw(&env, "Failed to get absolute path to PID file");
            return -1;
        }
    };

    file_path.push('\0'); // end C-string with null char
    let file_name = file_path.as_str().as_ptr();

    return unsafe {
        let msg_key = ftok(file_name as *const c_char, 'P' as i32);
        check_err!(env, msg_key, -1);
        let ret = msgget(msg_key, 0o666 | IPC_CREAT);
        check_err!(env, ret, -1)
    };
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_sendMessage(
    env: JNIEnv,
    _: JClass,
    queue_id: jint,
    message: JObject,
) {
    let m_type = get_field!(env, message, "mType", Long);
    let data = get_field!(
        env,
        message,
        "data",
        Object(concat!("L", data_class!(), ";"))
    );

    let response_chan = get_field!(env, data, "responseChan", Int);
    let response_pid = get_field!(env, data, "responsePid", Int);
    let message_type = get_field!(env, data, "messageType", Short);
    let message_length = get_field!(env, data, "messageLength", Byte);
    let message = get_field!(env, data, "message", Object("[B"));

    let len = match env.get_array_length(message.into_inner()) {
        Ok(siz) => siz,
        _ => {
            throw(
                &env,
                concat!("Failed to get array length for ", message_field_name!()),
            );
            return;
        }
    };

    if len != message_length!() as i32 {
        throw(
            &env,
            concat!(
                "Length of {} is not {}",
                message_field_name!(),
                message_length!()
            ),
        );
        return;
    }

    let mut message_data: [jbyte; message_length!()] = [0; message_length!()];
    if let Err(_) = env.get_byte_array_region(message.into_inner(), 0, &mut message_data) {
        throw(&env, concat!("Failed to copy ", message_field_name!()));
        return;
    }

    let mut msg = Message {
        m_type,
        data: Data {
            response_chan,
            response_pid: response_pid as u32,
            message_type,
            message_length: message_length as u8,
            message: [0; message_length!()],
        },
    };
    message_data
        .iter()
        .map(|b| *b as u8)
        .enumerate()
        .for_each(|(i, b)| msg.data.message[i] = b);

    let ret = unsafe {
        msgsnd(
            queue_id,
            &mut msg as *mut _ as *mut c_void,
            size_of::<Data>(),
            0,
        )
    };
    check_err!(env, ret);
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_receiveMessage(
    env: JNIEnv,
    _: JClass,
    queue_id: jint,
) -> jobject {
    let mut message = Message {
        m_type: MESSAGE_TYPE,
        data: Data {
            response_chan: 0,
            response_pid: 0,
            message_type: 0,
            message_length: 0,
            message: [0; message_length!()],
        },
    };

    let ret = unsafe {
        msgrcv(
            queue_id,
            &mut message as *mut _ as *mut c_void,
            size_of::<Data>(),
            MESSAGE_TYPE,
            0,
        )
    };
    check_err!(env, ret, JObject::null().into_inner());

    let message_data = match env.byte_array_from_slice(&message.data.message) {
        Ok(array) => JObject::from(array),
        _ => {
            throw(&env, "Failed to create Java byte array from message data");
            return JObject::null().into_inner();
        }
    };

    let d = &message.data;
    let obj = env.new_object(
        data_class!(),
        "(IISB[B)V",
        &[
            Int(d.response_chan),
            Int(d.response_pid as jint),
            Short(d.message_type),
            Byte(d.message_length as jbyte),
            Object(message_data),
        ],
    );
    let obj = match obj {
        Ok(obj) => obj,
        _ => {
            throw(&env, concat!("Could not create class ", data_class!()));
            return JObject::null().into_inner();
        }
    };

    let msg_obj = env.new_object(
        message_class!(),
        concat!("(JL", data_class!(), ";)V"),
        &[Long(message.m_type as jlong), Object(obj)],
    );
    let msg_obj = match msg_obj {
        Ok(msg_obj) => msg_obj,
        _ => {
            throw(&env, concat!("Could not create class ", message_class!()));
            return JObject::null().into_inner();
        }
    };

    return msg_obj.into_inner();
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_deleteQueue(
    env: JNIEnv,
    _: JClass,
    queue_id: jint,
) {
    let res = unsafe { msgctl(queue_id, IPC_RMID, null_mut()) };
    check_err!(env, res);
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_pidExists(
    env: JNIEnv,
    _: JClass,
    pid: jint,
) -> jboolean {
    return if let Err(Sys(e)) = kill(Pid::from_raw(pid), None) {
        if e != Errno::ESRCH {
            throw(&env, e.desc());
        }
        JNI_FALSE
    } else {
        JNI_TRUE
    };
}
