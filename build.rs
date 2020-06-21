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

use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::OpenOptions;
use std::io::copy;
use std::io::{BufReader, Read};
use std::process::{Command, Stdio};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let cargo_loc = env::var("CARGO").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let is_release = profile == "release";
    let is_mac = target_os == "macos";
    let extension = if is_mac { "dylib" } else { "so" };

    build_jni(&cargo_loc, is_release, &out_dir);

    let lib_file_name = if is_release {
        "release/libpaperd_jni"
    } else {
        "debug/libpaperd_jni"
    };
    let lib_file = format!("{}/{}.{}", out_dir, lib_file_name, extension);

    if is_release {
        strip(lib_file.as_str(), is_mac);
    }
    compress(lib_file.as_str());

    println!("cargo:rustc-env=PAPERD_JNI_LIB={}.gz", lib_file);
}

fn build_jni(cargo_loc: &str, is_release: bool, out_dir: &str) {
    let mut command = Command::new(cargo_loc);
    let mut command = command
        .current_dir("paperd-jni")
        .args(&["build", "--target-dir", out_dir]);

    if is_release {
        command.arg("--release");
    }

    execute(&mut command);
}

fn strip(lib_file: &str, is_mac: bool) {
    if is_mac {
        let mut command = Command::new("strip");
        let mut command = command.args(&["-x", lib_file]);
        execute(&mut command);
    } else {
        let nm_process = Command::new("nm")
            .args(&["--extern-only", lib_file])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut output = String::new();
        nm_process
            .stdout
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();

        let symbols: Vec<&str> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 3 {
                    return None;
                }
                let part = parts[2];
                return if part.starts_with("Java_com_destroystokyo_paper") {
                    Some(part)
                } else {
                    None
                };
            })
            .collect();

        let mut command = Command::new("strip");
        for symbol in symbols {
            command.args(&["-K", symbol]);
        }
        command.arg(lib_file);

        execute(&mut command);
    }
}

fn compress(lib_file: &str) {
    let output_file = format!("{}.gz", lib_file);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .unwrap();

    let mut encoder = GzEncoder::new(file, Compression::best());

    let source_file = OpenOptions::new().read(true).open(lib_file).unwrap();
    let mut input = BufReader::new(source_file);

    copy(&mut input, &mut encoder).unwrap();
}

fn execute(cmd: &mut Command) {
    if cmd.spawn().unwrap().wait().unwrap().code().unwrap() != 0 {
        panic!("Failed to execute command");
    }
}
