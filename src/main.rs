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
