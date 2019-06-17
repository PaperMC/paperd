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

use crate::daemon::{run_daemon, Status};
use clap::ArgMatches;
use nix::sys::signal;
use nix::unistd::Pid;
use regex::Regex;
use signal_hook::iterator::Signals;
use signal_hook::{SIGABRT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTRAP};
use std::cmp::max;
use std::env;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use std::{fs, thread};
use sys_info::mem_info;

const PID_FILE_NAME: &str = "papermc.pid";

pub fn start(sub_m: &ArgMatches) -> i32 {
    let env = match setup_java_env(sub_m) {
        Ok(env) => env,
        Err(exit) => {
            return exit;
        }
    };

    match run_daemon() {
        Ok(Status::CONTINUE) => {}
        Ok(Status::QUIT) => {
            println!("Server starting in background, waiting for server to start...");

            let pid_file = env.working_dir.join(PID_FILE_NAME);
            let dur = Duration::from_secs(5);
            let start = Instant::now();

            // wait for pid file to be created, until timeout
            while Instant::now().duration_since(start) < dur && !pid_file.exists() {
                thread::yield_now();
            }

            if pid_file.exists() {
                match fs::read_to_string(pid_file) {
                    Ok(pid) => {
                        println!("Server started in the background. PID: {}", pid);
                        return 0;
                    }
                    Err(err) => {
                        eprintln!(
                            "Server started in the background. Failed to retrieve PID. \
                             Error: {}",
                            err
                        );
                        return 1;
                    }
                }
            } else {
                eprintln!("Timeout while waiting for server to start.");
                return 1;
            }
        }
        Err(err) => return err,
    }

    let process = start_process(env.clone());
    let child = match process {
        Ok(child) => child,
        Err(err) => {
            // TODO write to a log somewhere
            return err;
        }
    };

    let pid = child.id();

    // Write pid file
    let pid_file = env.working_dir.join(PID_FILE_NAME);
    if let Err(_) = fs::write(pid_file, pid.to_string()) {
        return 1;
    }

    let signals = match forward_signals(pid) {
        Ok(sig) => sig,
        Err(exit) => {
            return exit;
        }
    };

    let result = wait_for_child(child);

    signals.close();

    return result;
}

pub fn run_cmd(sub_m: &ArgMatches) -> i32 {
    let child = match setup_java_env(sub_m).and_then(|env| start_process(env)) {
        Ok(c) => c,
        Err(exit) => return exit,
    };

    let pid = child.id();
    let signals = match forward_signals(pid) {
        Ok(s) => s,
        Err(exit) => return exit,
    };

    let result = wait_for_child(child);

    signals.close();

    return result;
}

#[derive(Clone)]
struct JavaEnv {
    java_file: PathBuf,
    jar_file: PathBuf,
    working_dir: PathBuf,
    args: Vec<String>,
}

fn start_process(env: JavaEnv) -> Result<Child, i32> {
    let result = Command::new(env.java_file)
        .args(env.args)
        .arg("-jar")
        .arg(env.jar_file)
        .current_dir(env.working_dir)
        .spawn();
    return match result {
        Ok(c) => Ok(c),
        Err(err) => {
            eprintln!("Failed to start server: {}", err);
            Err(1)
        }
    };
}

fn setup_java_env(sub_m: &ArgMatches) -> Result<JavaEnv, i32> {
    // Find Java executable
    let java_path = sub_m.value_of("JVM").map(PathBuf::from).or_else(find_java);
    let java_path = match java_path {
        Some(path) => path,
        None => {
            eprintln!(
                "Could not find a JVM executable. Either make sure it's present on the PATH, or \
                 there's a valid JAVA_HOME, or specify it with -j. See --help for more details."
            );
            return Err(1);
        }
    };

    // Find target jar file
    let jar_path = match sub_m.value_of("JAR") {
        Some(path) => match canonicalize(PathBuf::from(path)) {
            Ok(canonical) => canonical,
            _ => {
                eprintln!("Failed to get full path to jar {}", path);
                return Err(1);
            }
        },
        None => {
            eprintln!("Failed to resolve jar file path");
            return Err(1);
        }
    };
    if !jar_path.is_file() {
        eprintln!("Could not find jar {}", jar_path.to_string_lossy());
        return Err(1);
    }

    // Get the jar's parent directory
    let parent_path = sub_m
        .value_of("CWD")
        .map(|s| PathBuf::from(s))
        .or_else(|| jar_path.parent().map(|p| p.to_path_buf()));
    let parent_path = match parent_path {
        Some(path) => path,
        None => {
            eprintln!(
                "Failed to find parent directory for jar {}",
                jar_path.to_string_lossy()
            );
            return Err(1);
        }
    };

    let jvm_args = match get_jvm_args(sub_m) {
        Ok(vec) => vec,
        Err(exit) => {
            return Err(exit);
        }
    };

    return Ok(JavaEnv {
        java_file: java_path,
        jar_file: jar_path,
        working_dir: parent_path,
        args: jvm_args,
    });
}

fn forward_signals(pid: u32) -> Result<Signals, i32> {
    // While the server is running we'll redirect some signals to it
    let signals = Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGTRAP, SIGABRT, SIGTERM]);
    let signals = match signals {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to register signal handlers: {}", err);
            return Err(1);
        }
    };

    let signals_bg = signals.clone();
    thread::spawn(move || {
        for sig_int in signals_bg.forever() {
            if let Ok(sig) = signal::Signal::from_c_int(sig_int) {
                let _ = signal::kill(Pid::from_raw(pid as i32), sig);
            }
        }
    });

    return Ok(signals);
}

fn wait_for_child(mut child: Child) -> i32 {
    return match child.wait().map(|status| status.code().unwrap_or(1)) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("Error while running server: {}", err);
            1
        }
    };
}

/// Searches the PATH for java. If that fails, JAVA_HOME is searched as well.
fn find_java() -> Option<PathBuf> {
    return vec![("PATH", "java"), ("JAVA_HOME", "bin/java")]
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

fn get_jvm_args(sub_m: &ArgMatches) -> Result<Vec<String>, i32> {
    if let Some(vals) = sub_m.values_of("CUSTOM_ARGS") {
        return Ok(vals.map(|s| s.to_owned()).collect());
    }

    // When all else fails, use 500m
    // This should hopefully be small enough to not cause problems for anyone
    let mut heap: String = "500m".to_owned();

    if let Some(value) = sub_m.value_of("DEFAULT_ARGS") {
        let reg = Regex::new(r"\d+[mG]").unwrap();
        if !reg.is_match(value) {
            eprintln!("Invalid format for JVM heap size. Should be something like 500m or 2G.");
            return Err(1);
        }

        heap = value.to_owned();
    } else {
        // If no arguments are provided, use 1/2 of the current available memory with default flags
        if let Ok(info) = mem_info() {
            // info.avail should always be greater than free, but it seems there may be a bug
            // for macOS. Assuming most users are using linux this doesn't really affect much
            let mem = max(info.avail, info.free);
            // mem is in kb, so convert to mb by dividing by 1000
            // Then we take half of it
            let mut mb = ((mem / 1000) / 2).to_string();

            println!(
                "Warning: No memory argument provided, automatically determining to use {} MB \
                 instead. This is not recommended, please specify an amount of memory with -d or \
                 --default-args",
                mb
            );

            mb.push_str("m");
            heap = mb;
        }
    }

    let mut xms = "-Xms".to_owned();
    let mut xmx = "-Xmx".to_owned();
    xms.push_str(heap.as_str());
    xmx.push_str(heap.as_str());

    return Ok(vec![
        xms,
        xmx,
        "-XX:+UseG1GC".to_owned(),
        "-XX:+UnlockExperimentalVMOptions".to_owned(),
        "-XX:MaxGCPauseMillis=100".to_owned(),
        "-XX:+DisableExplicitGC".to_owned(),
        "-XX:TargetSurvivorRatio=90".to_owned(),
        "-XX:G1NewSizePercent=50".to_owned(),
        "-XX:G1MaxNewSizePercent=80".to_owned(),
        "-XX:G1MixedGCLiveThresholdPercent=35".to_owned(),
        "-XX:+AlwaysPreTouch".to_owned(),
        "-XX:+ParallelRefProcEnabled".to_owned(),
        "-Dusing.aikars.flags=mcflags.emc.gs".to_owned(),
    ]);
}
