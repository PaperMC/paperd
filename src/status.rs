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

use crate::protocol::check_protocol;
use crate::util;
use crate::util::{get_sock, ExitValue};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};

pub fn status(sub_m: &ArgMatches) -> Result<(), ExitValue> {
    let (sock, _) = get_sock(sub_m)?;
    check_protocol(&sock)?;

    let message = StatusMessage {};

    sock.send_message(&message)?;

    let res = sock.receive_message::<StatusMessageResponse>()?;

    output_status(&res);

    return Ok(());
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn output_status(status: &StatusMessageResponse) {
    let line_length = 60;

    println!("======================= Server Info =======================");
    println!("   Server | {}", status.server_name);
    println!("     MOTD | {}", status.motd);
    print_players(&status.players, "  Players", line_length);
    println!();
    println!("------------------------- Version --------------------------");
    println!("  PaperMC Server Version | {}", status.server_version);
    println!("  Bukkit API Version     | {}", status.api_version);
    println!();
    println!("------------------------  Worlds  --------------------------");

    for world in &status.worlds {
        println!("************************************************************");
        println!("       Name | {}", world.name);
        println!("  Dimension | {}", world.dimension);
        println!("       Seed | {}", world.seed);
        println!(" Difficulty | {}", world.difficulty);
        print_players(&world.players, "    Players", line_length);
        println!("       Time | {}", format_time(world.time.as_str()));
        println!("************************************************************");
    }

    println!();
    println!("-------------------- Server Performance --------------------");
    println!("  TPS");
    println!("    Past 1 Minute   | {:.2}", util::tps_cap(status.tps.one_min));
    println!("    Past 5 Minutes  | {:.2}", util::tps_cap(status.tps.five_min));
    println!("    Past 15 Minutes | {:.2}", util::tps_cap(status.tps.fifteen_min));
    println!();
    println!("  Memory Usage");
    println!("    Memory Currently Used   | {}", status.memory_usage.used_memory);
    println!("    Total Memory Allocated  | {}", status.memory_usage.total_memory);
    println!("    Maximum Possible Memory | {}", status.memory_usage.max_memory);
    println!();
}

fn print_players(players: &Vec<String>, prefix: &str, length: usize) {
    let mut current_line = String::with_capacity(length);
    current_line.push_str(prefix);
    current_line.push_str(" | (");
    current_line.push_str(players.len().to_string().as_str());
    current_line.push_str(") ");

    for (i, player) in players.iter().enumerate() {
        if current_line.len() + player.len() + 1 > length {
            println!("{}", current_line);
            current_line = String::with_capacity(length);
            let indent = " ".repeat(prefix.len());
            current_line.push_str(indent.as_str());
            current_line.push_str(" | ")
        }

        current_line.push_str(player.as_str());
        if i != players.len() - 1 {
            current_line.push_str(", ");
        }
    }

    if !current_line.ends_with(" | ") {
        println!("{}", current_line);
    }
}

fn format_time(time: &str) -> String {
    let mut res = String::with_capacity(5);
    res.push_str(&time[..time.len() / 2]);
    res.push(':');
    res.push_str(&time[time.len() / 2..]);
    return res;
}

// Request
#[derive(Serialize)]
pub struct StatusMessage {}

// Response
#[derive(Deserialize)]
pub(crate) struct StatusMessageResponse {
    #[serde(rename = "motd")]
    motd: String,
    #[serde(rename = "serverName")]
    server_name: String,
    #[serde(rename = "serverVersion")]
    server_version: String,
    #[serde(rename = "apiVersion")]
    api_version: String,
    #[serde(rename = "players")]
    players: Vec<String>,
    #[serde(rename = "worlds")]
    worlds: Vec<WorldStatus>,
    #[serde(rename = "tps")]
    tps: TpsStatus,
    #[serde(rename = "memoryUsage")]
    memory_usage: MemoryStatus,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct TpsStatus {
    #[serde(rename = "oneMin")]
    one_min: f64,
    #[serde(rename = "fiveMin")]
    five_min: f64,
    #[serde(rename = "fifteenMin")]
    fifteen_min: f64,
}

#[derive(Deserialize)]
struct MemoryStatus {
    #[serde(rename = "usedMemory")]
    used_memory: String,
    #[serde(rename = "totalMemory")]
    total_memory: String,
    #[serde(rename = "maxMemory")]
    max_memory: String,
}
