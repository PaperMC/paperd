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
extern crate signal_hook;
extern crate sys_info;

mod cmd;
mod daemon;
mod messaging;
mod runner;

use crate::cmd::handle_cmd_line;
use crate::runner::{run_cmd, start, stop};
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

fn status(_sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    unimplemented!();
}

fn send(sub_m: &ArgMatches) -> Result<(), i32> {
    let pid_file = sub_m
        .value_of("PID")
        .map(PathBuf::from)
        .or_else(|| env::var_os("PAPERMC_PID").map(PathBuf::from))
        .map(PathBuf::from);
    let pid_file = match pid_file {
        Some(p) => p,
        None => {
            eprintln!("Could not find a PID file for a running server.");
            return Err(1);
        }
    };

    let chan = messaging::open_message_channel(pid_file)?;
    return chan.send_message("Hello World!");
}

fn log(_sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    unimplemented!();
}
