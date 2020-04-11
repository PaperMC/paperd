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
use nix::errno::Errno::ESRCH;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use nix::Error;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::{env, fs, io};

pub fn get_pid(sub_m: &ArgMatches) -> Result<(PathBuf, Pid), i32> {
    let pid_file = sub_m
        .value_of("PID")
        .map(PathBuf::from)
        .or_else(|| env::var_os("PAPERMC_PID").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(runner::PID_FILE_NAME));

    if !pid_file.is_file() {
        eprintln!("No PID file found to send commands to");
        return Err(1);
    }

    let text = fs::read_to_string(&pid_file).conv("Failed to read PID file")?;
    let pid = Pid::from_raw(match text.parse::<i32>() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse PID file: {}", e);
            fs::remove_file(&pid_file).conv("Failed to delete PID file")?;
            eprintln!("No server found to send commands to");
            return Err(1);
        }
    });

    match kill(pid, None) {
        Err(Error::Sys(e)) => {
            if e == ESRCH {
                println!("Found stale PID file, removing");
                fs::remove_file(&pid_file).conv("Failed to delete PID file")?;
                eprintln!("No server found to send commands to");
                return Err(1);
            }
        }
        _ => {}
    }

    return Ok((pid_file, pid));
}

#[cfg(feature = "console")]
pub fn is_pid_running(pid: Pid) -> bool {
    return kill(pid, None).is_ok();
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

pub fn tps_cap(tps: f64) -> f64 {
    return tps.min(20.0);
}

pub trait ExitError<T> {
    fn conv(self, context: &str) -> Result<T, i32>;
}

impl<T> ExitError<T> for io::Result<T> {
    fn conv(self, context: &str) -> Result<T, i32> {
        return match self {
            Ok(t) => Ok(t),
            Err(e) => {
                eprintln!("{}", context);
                eprintln!("  Caused by: IO Error: {}", e);
                return Err(1);
            }
        };
    }
}

impl<T> ExitError<T> for Result<T, ParseIntError> {
    fn conv(self, context: &str) -> Result<T, i32> {
        return match self {
            Ok(t) => Ok(t),
            Err(e) => {
                eprintln!("{}", context);
                eprintln!("  Caused by: Failed to parse int: {}", e);
                return Err(1);
            }
        };
    }
}
