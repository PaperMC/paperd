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

extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn handle_cmd_line<'a>() -> ArgMatches<'a> {
    let pid_arg = Arg::with_name("PID")
        .help(
            "Custom PID file to send commands to a running server. If not set, the \
             PAPERMC_PID environment variable will be checked. If neither are set, the default \
             value is ./paper.pid.",
        )
        .short("p")
        .long("pid")
        .takes_value(true);

    let java_arg = Arg::with_name("JVM")
        .help(
            "The java binary to use to execute the paperclip jar. By default paperd will \
             search the PATH. If there is no java binary on the PATH, paperd will use the \
             JAVA_HOME environment variable instead. If neither of these finds a JVM, this \
             argument must be supplied.",
        )
        .long("jvm")
        .takes_value(true);

    let jar_arg = Arg::with_name("JAR")
        .help("The jar to run.")
        .long("jar")
        .takes_value(true)
        .default_value("paperclip.jar");

    return App::new("paperd")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .version("0.1.0")
        .author("PaperMC (papermc.io)")
        .about("PaperMC server daemon for running and controlling daemonized PaperMC servers.")
        .subcommand(
            SubCommand::with_name("status")
                .about("Get the status of the currently running server.")
                .arg(pid_arg.clone())
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Send a command to the running MC server.")
                .arg(pid_arg.clone())
                .arg(tail_arg(
                    "Tail the server log after sending the command to the \
                     server, useful for viewing the response. Press q to quit.",
                ))
                .arg(
                    Arg::with_name("COMMAND")
                        .help("The command to send to the MC server.")
                        .required(true),
                )
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Print recent log messages from the running MC server.")
                .arg(pid_arg.clone())
                .arg(
                    Arg::with_name("LINES")
                        .help("The number of log messages to print.")
                        .short("l")
                        .long("lines")
                        .default_value("10"),
                )
                .arg(tail_arg(
                    "Tail the server log rather than just printing recent \
                     messages. Press q to quit.",
                ))
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Start the MC server in the background.")
                .arg(java_arg.clone())
                .arg(jar_arg.clone())
                .arg(tail_arg(
                    "Tail the server log after starting the server. Press q to \
                     quit (will NOT stop the server).",
                ))
                .display_order(2),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Start the MC server in the foreground.")
                .arg(java_arg.clone())
                .arg(jar_arg.clone())
                .display_order(2),
        )
        .get_matches();
}

fn tail_arg<'a, 'b>(message: &'a str) -> Arg<'a, 'b> {
    return Arg::with_name("TAIL").help(message).short("t").long("tail");
}
