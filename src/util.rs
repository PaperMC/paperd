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

use crate::runner;
use clap::ArgMatches;
use std::path::PathBuf;
use std::{env, io};

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

pub fn find_prog(searches: &[(&str, &str)]) -> Option<PathBuf> {
    return searches
        .iter()
        .filter_map(|(var, file)| {
            env::var_os(var).and_then(|paths| {
                env::split_paths(&paths)
                    .filter_map(|dir| {
                        let full_path = dir.join(file);
                        if full_path.is_file() {
                            Some(full_path)
                        } else {
                            None
                        }
                    })
                    .next()
            })
        })
        .next();
}

pub trait ExitError<T> {
    fn conv(self) -> Result<T, i32>;
}

impl<T> ExitError<T> for io::Result<T> {
    fn conv(self) -> Result<T, i32> {
        return match self {
            Ok(t) => Ok(t),
            Err(e) => {
                eprintln!("IO Error: {}", e);
                return Err(1);
            }
        };
    }
}
