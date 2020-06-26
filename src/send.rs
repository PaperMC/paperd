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

use crate::log::{find_log_file, tail};
use crate::messaging::MessageSocket;
use crate::protocol::check_protocol;
use crate::util::{get_sock, ExitValue};
use clap::ArgMatches;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

pub fn send(sub_m: &ArgMatches) -> Result<(), ExitValue> {
    let (sock, sock_file) = get_sock(sub_m)?;
    check_protocol(&sock)?;

    let command: String = match sub_m.values_of("COMMAND") {
        Some(s) => s.map(|a| a.to_string()).collect::<Vec<String>>().join(" "),
        None => {
            eprintln!("No command given.");
            return Err(ExitValue::Code(1));
        }
    };

    return if sub_m.is_present("TAIL") {
        let lock: Arc<Mutex<Option<ExitValue>>> = Arc::new(Mutex::new(None));
        let thread_lock = lock.clone();

        // Start tailing in a separate thread so it won't be too late and miss the response
        spawn(move || {
            let mut exit_value = thread_lock.lock().unwrap();
            let log_file = match find_log_file(&sock_file) {
                Ok(f) => f,
                Err(e) => {
                    *exit_value = Some(e.clone());
                    return;
                }
            };
            if let Err(e) = tail(log_file, 0, true) {
                *exit_value = Some(e.clone());
            }
        });

        // Wait long enough to acquire the lock
        sleep(Duration::from_millis(1));

        send_command(&sock, command.as_str())?;

        // Wait for tail to complete before returning
        let exit_value = lock.lock().unwrap();
        if let Some(e) = &*exit_value {
            Err(e.clone())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    };
}

pub fn send_command(sock: &MessageSocket, cmd: &str) -> Result<(), ExitValue> {
    let message = SendCommandMessage {
        message: cmd.to_string(),
    };

    sock.send_message(&message)?;

    return Ok(());
}

#[derive(Serialize)]
pub struct SendCommandMessage {
    #[serde(rename = "message")]
    message: String,
}
