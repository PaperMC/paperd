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
use serde::{Deserialize, Serialize};

pub fn status(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = get_pid(sub_m)?;

    let message = StatusMessage {};

    let chan = messaging::open_message_channel(&pid_file)?;
    let response_chan = chan
        .send_message::<StatusMessage>(message)?
        .expect("Failed to create response channel");

    let res = response_chan.receive_message::<StatusMessageResponse>()?;
    response_chan.close()?;

    output_status(&res)?;

    return Ok(());
}

fn output_status(status: &StatusMessageResponse) -> Result<(), i32> {
    let text = match serde_json::to_string_pretty(status) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to generate response: {}", e);
            return Err(1);
        }
    };

    println!("{}", text);

    return Ok(());
}

// Request
#[derive(Serialize)]
struct StatusMessage {}

impl MessageHandler for StatusMessage {
    fn type_id() -> i16 {
        return 2;
    }

    fn expect_response() -> bool {
        return true;
    }
}

// Response
#[derive(Serialize, Deserialize, Default)]
struct StatusMessageResponse {
    #[serde(rename = "motd")]
    motd: String,
    #[serde(rename = "serverName")]
    server_name: String,
    #[serde(rename = "serverVersion")]
    server_version: String,
    #[serde(rename = "apiVersion")]
    api_version: String,
    #[serde(rename = "numPlayers")]
    num_players: i32,
    #[serde(rename = "worlds")]
    worlds: Vec<WorldStatus>,
    #[serde(rename = "tps")]
    tps: TpsStatus,
    #[serde(rename = "memoryUsage")]
    memory_usage: MemoryStatus,
}

#[derive(Serialize, Deserialize, Default)]
struct WorldStatus {
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "dimension")]
    dimension: String,
    #[serde(rename = "seed")]
    seed: i64,
    #[serde(rename = "difficulty")]
    difficulty: String,
    #[serde(rename = "players")]
    players: Vec<String>,
    #[serde(rename = "time")]
    time: String,
}

#[derive(Serialize, Deserialize, Default)]
struct TpsStatus {
    #[serde(rename = "oneMin")]
    one_min: f64,
    #[serde(rename = "fiveMin")]
    five_min: f64,
    #[serde(rename = "fifteenMin")]
    fifteen_min: f64,
}

#[derive(Serialize, Deserialize, Default)]
struct MemoryStatus {
    #[serde(rename = "usedMemory")]
    used_memory: String,
    #[serde(rename = "totalMemory")]
    total_memory: String,
    #[serde(rename = "maxMemory")]
    max_memory: String,
}
