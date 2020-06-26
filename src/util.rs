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

use crate::messaging::MessageSocket;
use crate::runner;
use crate::runner::PID_FILE_NAME;
use clap::ArgMatches;
use nix::unistd::Pid;
use paperd_lib::connect_socket;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub fn get_sock(sub_m: &ArgMatches) -> Result<(MessageSocket, PathBuf), ExitValue> {
    let sock_file = find_sock_file(sub_m)?;
    let sock = get_sock_from_file(&sock_file)?;

    return Ok((sock, sock_file));
}

pub fn find_sock_file(sub_m: &ArgMatches) -> Result<PathBuf, ExitValue> {
    let sock_file = sub_m
        .value_of("SOCK")
        .map(PathBuf::from)
        .or_else(|| env::var_os("PAPERD_SOCK").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(runner::SOCK_FILE_NAME));

    if !sock_file.exists() {
        eprintln!("No socket file found to send commands to");
        return Err(ExitValue::Code(1));
    }

    return Ok(sock_file);
}

pub fn get_sock_from_file<P: AsRef<Path>>(sock_file: P) -> Result<MessageSocket, ExitValue> {
    let msg = format!(
        "Failed to connect to socket {}",
        sock_file.as_ref().display()
    );
    let sock = connect_socket(sock_file.as_ref()).conv(msg)?;

    return Ok(MessageSocket::new(sock));
}

pub fn find_program(searches: &[(&str, &str)]) -> Option<PathBuf> {
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

pub fn get_pid<P: AsRef<Path>>(sock_file: P) -> Result<(PathBuf, Pid), ExitValue> {
    let pid_file = match sock_file.as_ref().parent().map(|p| p.join(PID_FILE_NAME)) {
        Some(path) => path,
        None => {
            eprintln!("Failed to find PID file {}", PID_FILE_NAME);
            return Err(ExitValue::Code(1));
        }
    };

    let pid_text = fs::read_to_string(&pid_file).conv("Failed to read PID file")?;
    let pid = Pid::from_raw(match pid_text.parse::<i32>() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse PID file: {}", e);
            fs::remove_file(&pid_file).conv("Failed to delete PID file")?;
            eprintln!("No server found to send commands to");
            return Err(ExitValue::Code(1));
        }
    });

    return Ok((pid_file, pid));
}

pub fn tps_cap(tps: f64) -> f64 {
    return tps.min(20.0);
}

#[derive(Clone)]
pub enum ExitValue {
    Code(i32),
    Shutdown,
}

pub trait ExitError<T> {
    fn conv<S: AsRef<str>>(self, context: S) -> Result<T, ExitValue>;
}

impl<T> ExitError<T> for io::Result<T> {
    fn conv<S: AsRef<str>>(self, context: S) -> Result<T, ExitValue> {
        return self.map_err(|e| {
            eprintln!("{}", context.as_ref());
            eprintln!("  Caused by: IO Error: {}", e);
            return ExitValue::Code(1);
        });
    }
}

impl<T> ExitError<T> for Result<T, ParseIntError> {
    fn conv<S: AsRef<str>>(self, context: S) -> Result<T, ExitValue> {
        return self.map_err(|e| {
            eprintln!("{}", context.as_ref());
            eprintln!("  Caused by: Failed to parse int: {}", e);
            return ExitValue::Code(1);
        });
    }
}

impl<T> ExitError<T> for Result<T, paperd_lib::Error> {
    fn conv<S: AsRef<str>>(self, context: S) -> Result<T, ExitValue> {
        return self.map_err(|e| {
            eprintln!("{}", context.as_ref());
            eprintln!("  Caused by: Error during system call: {}", e);
            return ExitValue::Code(1);
        });
    }
}
