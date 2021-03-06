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

use crate::util::{find_program, find_sock_file, ExitError, ExitValue};
use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn log(sub_m: &ArgMatches) -> Result<(), ExitValue> {
    let sock_file = find_sock_file(sub_m)?;
    let log_file = find_log_file(&sock_file)?;

    let follow = sub_m.is_present("TAIL");
    let lines = match sub_m.value_of("LINES") {
        Some(l) => l
            .parse::<i32>()
            .conv("Failed to parse command line argument")?,
        None => {
            eprintln!("No value provided for --lines argument");
            return Err(ExitValue::Code(1));
        }
    };

    return tail(log_file, lines, follow);
}

pub fn find_log_file<P: AsRef<Path>>(sock_file: P) -> Result<PathBuf, ExitValue> {
    let sock_file = sock_file.as_ref();
    return match sock_file.parent().map(|p| p.join("logs/latest.log")) {
        Some(f) => Ok(f),
        None => {
            eprintln!("Failed to find log file in logs/latest.log");
            Err(ExitValue::Code(1))
        }
    };
}

pub fn tail<P: AsRef<Path>>(path: P, lines: i32, follow: bool) -> Result<(), ExitValue> {
    let path = path.as_ref();
    if !path.is_file() {
        eprintln!("file could not be found: {}", path.to_string_lossy());
        return Err(ExitValue::Code(1));
    }

    let tail_prog = match find_program(&[("PATH", "tail")]) {
        Some(t) => t,
        None => {
            eprintln!("Failed to find 'tail' program on the PATH");
            return Err(ExitValue::Code(1));
        }
    };

    let line_string = lines.to_string();
    let mut args = Vec::<&str>::new();
    args.push("-n");
    args.push(line_string.as_str());
    if follow {
        args.push("-F");
    }

    let result = Command::new(&tail_prog).args(args).arg(&path).spawn();

    let mut child = match result {
        Ok(c) => c,
        Err(err) => {
            eprintln!(
                "Failed to tail log file {}: {}",
                path.to_string_lossy(),
                err
            );
            return Err(ExitValue::Code(1));
        }
    };

    return match child.wait().map(|status| status.code().unwrap_or(1)) {
        Ok(status) => {
            if status == 0 {
                Ok(())
            } else {
                Err(ExitValue::Code(status))
            }
        }
        Err(err) => {
            eprintln!(
                "Error while tailing log file {}: {}",
                path.to_string_lossy(),
                err
            );
            Err(ExitValue::Code(1))
        }
    };
}
