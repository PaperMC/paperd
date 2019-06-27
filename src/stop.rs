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

use crate::messaging;
use crate::messaging::MessageHandler;
use crate::util::{get_pid, ExitError};
use clap::ArgMatches;
use nix::errno::Errno::ESRCH;
use nix::sys::signal::{kill, SIGKILL};
use nix::unistd::Pid;
use nix::Error;
use serde::Serialize;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io};

pub fn stop(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = get_pid(sub_m)?;
    let pid = fs::read_to_string(&pid_file).conv()?;
    let pid = Pid::from_raw(pid.parse::<i32>().conv()?);

    if sub_m.is_present("KILL") {
        force_kill(pid);
        println!("Server killed");
        return Ok(());
    }

    let message = StopMessage {};

    println!("Sending stop command to the server..");
    let chan = messaging::open_message_channel(&pid_file)?;
    chan.send_message::<StopMessage, ()>(message)?;

    print!("Waiting for server to exit.");
    let _ = io::stdout().flush();
    // If -f is set then we need to wait to see if it fails
    for _ in 0..30 {
        if let Err(_) = kill(pid, None) {
            break;
        }
        sleep(Duration::from_millis(500));
        print!(".");
        let _ = io::stdout().flush();
    }
    println!();

    if let Err(Error::Sys(e)) = kill(pid, None) {
        if e == ESRCH {
            println!("Server exited successfully");
            return Ok(());
        } else {
            println!("Unknown error occurred (stop): {}", e);
            return Err(1);
        }
    }

    if !sub_m.is_present("FORCE") {
        println!("Server failed to exit cleanly");
        return Err(1);
    }

    println!("Server failed to exit cleanly, killing now");
    force_kill(pid);
    println!("Server killed");

    return Ok(());
}

fn force_kill(pid: Pid) {
    let _ = kill(pid, SIGKILL);
}

#[derive(Serialize)]
struct StopMessage {}

impl MessageHandler for StopMessage {
    fn type_id() -> i16 {
        return 0;
    }

    fn expect_response() -> bool {
        return false;
    }
}
