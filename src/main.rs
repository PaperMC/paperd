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

#[macro_use]
extern crate clap;
extern crate nix;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate sys_info;

mod cmd;
mod daemon;
mod log;
mod messaging;
mod restart;
mod runner;
mod send;
mod status;
mod stop;
mod util;

use crate::cmd::handle_cmd_line;
use crate::log::log;
use crate::restart::restart;
use crate::runner::{run_cmd, start};
use crate::send::send;
use crate::status::status;
use crate::stop::stop;
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
        ("restart", Some(sub_m)) => restart(sub_m),
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
