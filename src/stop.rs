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
use crate::util::get_pid;
use clap::ArgMatches;
use serde::Serialize;

pub fn stop(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = get_pid(sub_m)?;

    let message = StopMessage {};

    let chan = messaging::open_message_channel(pid_file)?;
    chan.send_message::<StopMessage, ()>(message)?;
    // TODO wait for server to stop / timeout
    return Ok(());
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
