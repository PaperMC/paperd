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

extern crate alloc;
extern crate jni;
extern crate nix;
extern crate paperd_lib;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jobject};
use jni::JNIEnv;
use nix::errno::Errno;
use paperd_lib::{accept_connection, bind_socket, Error};

use paperd_lib::{
    close_socket, create_socket, receive_message, send_message, Message, MessageHeader,
};

use crate::util::{
    get_path_string, throw, throw_timeout, throw_with_cause, JAVA_STRING_TYPE, NPE_CLASS,
};

#[macro_use]
mod macros;
mod util;

const BUFFER_CLASS: &str = "com/destroystokyo/paper/daemon/PaperDaemonMessageBuffer";
const BUFFER_CONST: &str = "(JLjava/lang/String;)V";

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_createSocket(
    env: JNIEnv,
    _: JClass,
    sock_file: JObject,
) -> jint {
    let sock_file_path = match get_path_string(&env, sock_file) {
        Ok(str) => str,
        _ => {
            const MESSAGE: &'static str = "Failed to get absolute path to PID file";
            match env.exception_occurred() {
                Ok(thrown) => {
                    if thrown.is_null() {
                        throw(&env, MESSAGE);
                    }
                    let _ = env.exception_clear();
                    throw_with_cause(&env, MESSAGE, &thrown);
                }
                Err(_) => {
                    throw(&env, MESSAGE);
                }
            }
            return -1;
        }
    };

    let sock = handle_syscall!(env, create_socket(), -1);
    handle_syscall!(env, bind_socket(sock, sock_file_path.as_str()), -1);

    return sock;
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_acceptConnection(
    env: JNIEnv,
    _: JClass,
    sock: jint,
) -> jint {
    let client_sock = handle_syscall!(env, accept_connection(sock), 0);

    return if let Some(value) = client_sock {
        value
    } else {
        throw_timeout(&env);
        0
    };
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_receiveMessage(
    env: JNIEnv,
    _: JClass,
    client_sock: jint,
) -> jobject {
    let message = match receive_message(client_sock) {
        Ok(opt) => match opt {
            Some(m) => m,
            None => return jnull!(),
        },
        Err(Error::Nix(nix::Error::Sys(Errno::EAGAIN), _)) => {
            // timeout
            throw_timeout(&env);
            return jnull!();
        }
        Err(e) => {
            let error_msg = format!("Error attempting system call: {}", e);
            throw(&env, error_msg.as_str());
            return jnull!();
        }
    };

    let result_string = match env.new_string(message.message_text) {
        Ok(s) => s,
        Err(_) => return jnull!(),
    };

    let result_obj = env.new_object(
        BUFFER_CLASS,
        BUFFER_CONST,
        &[
            JValue::Long(message.header.message_type),
            JValue::Object(JObject::from(result_string)),
        ],
    );

    return match result_obj {
        Ok(o) => o.into_inner(),
        Err(_) => jnull!(),
    };
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_sendMessage(
    env: JNIEnv,
    _: JClass,
    client_sock: jint,
    message: jobject,
) {
    if message.is_null() {
        let _ = env.throw_new(NPE_CLASS, "message must not be null");
        return;
    }

    let message_type = get_field!(env, message, "messageType", Long);
    let message_data = get_field!(env, message, "messageData", Object(JAVA_STRING_TYPE));

    let java_string = env.get_string(JString::from(message_data));
    let java_string = match java_string {
        Ok(s) => String::from(s),
        Err(e) => {
            let error_msg = format!("Failed to retrieve string from message: {}", e);
            throw(&env, error_msg.as_str());
            return;
        }
    };

    let message = Message {
        header: MessageHeader {
            message_type,
            message_length: java_string.len() as i64,
        },
        message_text: java_string,
    };

    if let Err(e) = send_message(client_sock, &message) {
        let error_msg = format!("Failed to send message to {}: {}", client_sock, e);
        throw(&env, error_msg.as_str());
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_destroystokyo_paper_daemon_PaperDaemonJni_closeSocket(
    env: JNIEnv,
    _: JClass,
    sock: jint,
) {
    if let Err(e) = close_socket(sock) {
        let error_msg = format!("Error while closing socket {}: {}", sock, e);
        throw(&env, error_msg.as_str());
    }
}
