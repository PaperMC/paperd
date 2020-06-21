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
use crate::protocol::check_protocol;
use crate::util::{get_sock, ExitValue};
use clap::ArgMatches;
use serde::Serialize;

pub fn restart(sub_m: &ArgMatches) -> Result<(), ExitValue> {
    let (sock, sock_file) = get_sock(sub_m)?;
    check_protocol(&sock)?;

    let message = RestartMessage {};

    println!("Sending restart request...");

    sock.send_message(&message)?;

    if sub_m.is_present("TAIL") {
        let log_file = find_log_file(&sock_file)?;
        return tail(log_file, 0, true);
    }

    return Ok(());
}

// Request
#[derive(Serialize)]
pub struct RestartMessage {}
