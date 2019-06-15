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

mod cmd;
mod daemon;

use crate::cmd::handle_cmd_line;
use crate::daemon::{run_daemon, Status};
use clap::ArgMatches;
use std::process::exit;

fn main() {
    let res = match run() {
        Ok(_) => 0,
        Err(err) => err,
    };
    exit(res);
}

fn run() -> Result<(), i32> {
    let matches = handle_cmd_line();

    return match matches.subcommand() {
        ("status", Some(sub_m)) => status(sub_m),
        ("send", Some(sub_m)) => send(sub_m),
        ("log", Some(sub_m)) => log(sub_m),
        ("start", Some(sub_m)) => start(sub_m),
        ("run", Some(sub_m)) => run_cmd(sub_m),
        _ => {
            // This shouldn't happen, clap will error if no command is provided
            eprint!("Unknown command");
            Ok(())
        }
    };
}

fn status(sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    return Ok(());
}

fn send(sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    return Ok(());
}

fn log(sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    return Ok(());
}

fn start(sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    match run_daemon() {
        Ok(Status::CONTINUE) => {}
        Ok(Status::QUIT) => return Ok(()),
        Err(err) => return Err(err),
    }
    return Ok(());
}

fn run_cmd(sub_m: &ArgMatches) -> Result<(), i32> {
    // TODO
    return Ok(());
}
