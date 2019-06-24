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

extern crate clap;
extern crate nix;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate sys_info;

mod cmd;
mod daemon;
mod messaging;
mod runner;
mod status;

use crate::cmd::handle_cmd_line;
use crate::messaging::{SendCommandMessage, StopMessage};
use crate::runner::{run_cmd, start};
use crate::status::status;
use clap::ArgMatches;
use std::env;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    exit(run());
}

fn run() -> i32 {
    let matches = handle_cmd_line();

    let ret = match matches.subcommand() {
        ("status", Some(sub_m)) => status(sub_m),
        ("send", Some(sub_m)) => send(sub_m),
        ("log", Some(sub_m)) => log(sub_m),
        ("start", Some(sub_m)) => start(sub_m),
        ("run", Some(sub_m)) => run_cmd(sub_m),
        ("stop", Some(sub_m)) => stop(sub_m),
        _ => {
            // This shouldn't happen, clap will error if no command is provided
            eprint!("Unknown command");
            Err(1)
        }
    };

    return match ret {
        Ok(()) => 0,
        Err(exit) => exit,
    };
}

fn stop(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = get_pid(sub_m)?;

    let message = StopMessage {};

    let chan = messaging::open_message_channel(pid_file)?;
    chan.send_message::<StopMessage, ()>(message)?;
    // TODO wait for server to stop / timeout
    return Ok(());
}

fn send(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = get_pid(sub_m)?;

    let command = match sub_m.value_of("COMMAND") {
        Some(s) => s,
        None => {
            eprintln!("No command given.");
            return Err(1);
        }
    };

    let message = SendCommandMessage {
        message: command.to_string(),
    };

    let chan = messaging::open_message_channel(pid_file)?;
    chan.send_message::<SendCommandMessage, ()>(message)?;
    // TODO support tailing
    return Ok(());
}

fn log(_sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    unimplemented!();
}

pub fn get_pid(sub_m: &ArgMatches) -> Result<PathBuf, i32> {
    let pid_file = sub_m
        .value_of("PID")
        .map(PathBuf::from)
        .or_else(|| env::var_os("PAPERMC_PID").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(runner::PID_FILE_NAME));

    if !pid_file.is_file() {
        eprintln!("No PID file found to send commands to");
        return Err(1);
    }

    return Ok(pid_file);
}
