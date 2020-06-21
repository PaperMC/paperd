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
use crate::protocol::check_protocol;
use crate::util::{get_sock, ExitValue};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};

pub fn timings(sub_m: &ArgMatches) -> Result<(), ExitValue> {
    let (sock, _) = get_sock(sub_m)?;
    check_protocol(&sock)?;

    let message = TimingsMessage {};

    sock.send_message(&message)?;

    loop {
        let res = sock.receive_message::<TimingsMessageResponse>()?;
        if res.done {
            break;
        }
        if let Some(msg) = res.message {
            #[cfg(feature = "console")]
            println!(
                "{}",
                ansi::StyledMessage::parse(msg.as_str()).get_string()
            );

            #[cfg(not(feature = "console"))]
            println!("{}", mc_colors(msg.as_str()));
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
pub struct TimingsMessage {}

// Response
#[derive(Serialize, Deserialize)]
struct TimingsMessageResponse {
    #[serde(rename = "message")]
    message: Option<String>,
    #[serde(rename = "done")]
    done: bool,
}
