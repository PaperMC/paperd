use crate::daemon::{run_daemon, Status};
use clap::ArgMatches;
use nix::sys::signal;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use signal_hook::iterator::Signals;
use signal_hook::{SIGABRT, SIGHUP, SIGILL, SIGINT, SIGQUIT, SIGTERM, SIGTRAP};
use std::env;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

pub fn start(sub_m: &ArgMatches) -> i32 {
    // TODO
    match run_daemon() {
        Ok(Status::CONTINUE) => {}
        Ok(Status::QUIT) => return 0,
        Err(err) => return err,
    }
    return 0;
}

pub fn run_cmd(sub_m: &ArgMatches) -> i32 {
    // Find Java executable
    let java_path = sub_m.value_of("JVM").map(PathBuf::from).or_else(find_java);
    let java_path = match java_path {
        Some(path) => path,
        None => {
            java_not_found();
            return 1;
        }
    };

    // Find target jar file
    let jar_file = match sub_m.value_of("JAR") {
        Some(path) => match canonicalize(PathBuf::from(path)) {
            Ok(canonical) => canonical,
            _ => {
                eprintln!("Failed to get full path to jar {}", path);
                return 1;
            }
        },
        None => {
            eprintln!("Failed to resolve jar file path");
            return 1;
        }
    };
    if !jar_file.is_file() {
        jar_not_found(jar_file);
        return 1;
    }

    // Get the jar's parent directory
    let parent_path = jar_file.clone();
    let parent_path = match parent_path.parent() {
        Some(path) => path,
        None => {
            eprintln!(
                "Failed to find parent directory for jar {}",
                jar_file.to_string_lossy()
            );
            return 1;
        }
    };

    // TODO support JVM args somehow
    let process = Command::new(java_path)
        .arg("-jar")
        .arg(jar_file)
        .current_dir(parent_path)
        .spawn();

    let mut child = match process {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to start server: {}", err);
            return 1;
        }
    };

    // While the server is running we'll redirect some signals to it
    let signals = Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGTRAP, SIGABRT, SIGTERM]);
    let signals = match signals {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to register signal handlers: {}", err);
            return 1;
        }
    };

    let pid = child.id();

    let signals_bg = signals.clone();
    thread::spawn(move || {
        for sig_int in signals_bg.forever() {
            if let Ok(sig) = signal::Signal::from_c_int(sig_int) {
                let _ = kill(Pid::from_raw(pid as i32), sig);
            }
        }
    });

    let result = match child.wait() {
        Ok(status) => match status.code() {
            Some(code) => code,
            None => 1,
        },
        Err(err) => {
            eprintln!("Error while running server: {}", err);
            return 1;
        }
    };

    signals.close();

    return result;
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

fn java_not_found() {
    eprintln!(
        "Could not find a JVM executable. Either make sure it's present on the PATH, or \
         there's a valid JAVA_HOME, or specify it with -j. See --help for more details."
    )
}

fn jar_not_found(path: PathBuf) {
    eprintln!("Could not find jar {}", path.to_string_lossy())
}
