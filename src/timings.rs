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

#[cfg(feature = "console")]
use crate::console::ansi;
use crate::messaging;
use crate::messaging::MessageHandler;
use crate::protocol::check_protocol;
use crate::util::get_pid;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};

pub fn timings(sub_m: &ArgMatches) -> Result<(), i32> {
    let (pid_file, _) = get_pid(sub_m)?;
    check_protocol(&pid_file)?;

    let message = TimingsMessage {};

    let chan = messaging::open_message_channel(&pid_file)?;
    let response_chan = chan
        .send_message::<TimingsMessage>(message)?
        .expect("Failed to create response channel");

    loop {
        let res = response_chan.receive_message::<TimingsMessageResponse>()?;
        if res.done {
            break;
        }
        if res.message.is_some() {
            #[cfg(feature = "console")]
            println!(
                "{}",
                ansi::StyledMessage::parse(res.message.unwrap().as_str()).get_string()
            );

            #[cfg(not(feature = "console"))]
            println!("{}", mc_colors(res.message.unwrap().as_str()));
        }
    }

    return Ok(());
}

#[cfg(not(feature = "console"))]
fn mc_colors(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut skip = false;
    for ch in s.chars() {
        if skip {
            skip = false;
            continue;
        }
        if ch == '§' {
            skip = true;
            continue;
        }
        out.push(ch);
    }
    return out;
}

// Request
#[derive(Serialize)]
struct TimingsMessage {}

impl MessageHandler for TimingsMessage {
    fn type_id() -> i16 {
        return 5;
    }

    fn expect_response() -> bool {
        return true;
    }
}

// Response
#[derive(Serialize, Deserialize)]
struct TimingsMessageResponse {
    #[serde(rename = "message")]
    message: Option<String>,
    #[serde(rename = "done")]
    done: bool,
}
