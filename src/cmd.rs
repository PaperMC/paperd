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

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};

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

    return App::new("paperd")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .version("0.1.0")
        .author("PaperMC (papermc.io)")
        .about("PaperMC server daemon for running and controlling daemonized PaperMC servers.")
        .subcommand(
            SubCommand::with_name("status")
                .about("Get the status of the currently running server.")
                .arg(&pid_arg)
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Send a command to the running MC server.")
                .arg(&pid_arg)
                .arg(tail_arg(
                    "Tail the server log after sending the command to the \
                     server, useful for viewing the response. Press C-c to quit.",
                ))
                .arg(
                    Arg::with_name("COMMAND")
                        .help(
                            "The command to send to the MC server. Note the whole \
                             command should be one argument, so whitespace will need to be quoted \
                             or escaped. The string will be passed directly to the server to \
                             parse itself.",
                        )
                        .required(true),
                )
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Print recent log messages from the running MC server.")
                .arg(&pid_arg)
                .arg(
                    Arg::with_name("LINES")
                        .help("The number of log messages to print.")
                        .short("l")
                        .long("lines")
                        .default_value("10"),
                )
                .arg(tail_arg(
                    "Tail the server log rather than just printing recent \
                     messages. Press C-c.",
                ))
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Start the MC server in the background.")
                .arg(tail_arg(
                    "Tail the server log after starting the server. Press C-c to \
                     quit (will NOT stop the server).",
                ))
                .java_run()
                .display_order(2),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Start the MC server in the foreground.")
                .java_run()
                .display_order(2),
        )
        .subcommand(
            SubCommand::with_name("stop")
                .about(
                    "Stop the MC server gracefully. This is functionally \
                     equivalent to sending the 'stop' command to the server.",
                )
                .arg(&pid_arg)
                .arg(
                    Arg::with_name("FORCE")
                        .help(
                            "Forcefully kill the server if it does not respond \
                             within a timeout. This will first attempt to stop the server \
                             gracefully as normal before forcefully killing the server. \
                             Forcefully killing the server can result in loss of data and \
                             is not recommended. Only do so if the server is not responding.",
                        )
                        .short("-f")
                        .long("--force"),
                )
                .arg(
                    Arg::with_name("KILL")
                        .help(
                            "Immediately forcefully kill the server. This will \
                             NOT attempt to gracefully shutdown the server. This can result \
                             in loss of data, it is not recommended unless the server is \
                             not responding.",
                        )
                        .short("-k")
                        .long("--kill"),
                )
                .group(ArgGroup::with_name("FORCE_ARGS").args(&["FORCE", "KILL"]))
                .display_order(3),
        )
        .get_matches();
}

trait JavaArg {
    fn java_run(self) -> Self;
}

impl<'a, 'b> JavaArg for App<'a, 'b> {
    fn java_run(self) -> Self {
        return self
            .arg(
                Arg::with_name("JVM")
                    .help(
                        "The java binary to use to execute the paperclip jar. By default \
                         paperd will search the PATH. If there is no java binary on the PATH, \
                         paperd will use the JAVA_HOME environment variable instead. If neither of \
                         these finds a JVM, this argument must be supplied.",
                    )
                    .long("jvm")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("JAR")
                    .help("The jar to run.")
                    .long("jar")
                    .takes_value(true)
                    .default_value("paperclip.jar"),
            )
            .arg(
                Arg::with_name("CWD")
                    .help(
                        "The working directory of the server. Default is the parent \
                         directory of the jar.",
                    )
                    .short("w")
                    .long("--working-dir")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("DEFAULT_ARGS")
                    .help(
                        "Use a default set of recommended JVM arguments (Aikar's flags) \
                         with the specified amount of memory. The format should be something \
                         like 500m or 10G. It's recommended to provide as much memory as possible \
                         up to 10G. You may not provide custom arguments if defaults are used.",
                    )
                    .short("d")
                    .long("default-args")
                    .value_name("MEMORY")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("CUSTOM_ARGS")
                    .help(
                        "Provide a custom set of JVM arguments to be used when running \
                         the jar. This argument specifies all JVM arguments which will be \
                         passed, there are no defaults when using this argument. You may not pass \
                         custom arguments while also using -d or --default-args.",
                    )
                    .takes_value(true)
                    .allow_hyphen_values(true)
                    .multiple(true),
            )
            .group(
                ArgGroup::with_name("JVM_ARGS")
                    .arg("DEFAULT_ARGS")
                    .arg("CUSTOM_ARGS"),
            )
            .after_help(
                "EXAMPLES:\n    The --default-args argument or the 'CUSTOM_ARGS' \
                 arguments are mutually exclusive. That is, you can either use --default-args \
                 OR specify custom arguments, but not both.\n\n    \
                 Examples:\n        \
                 $ paperd run -d 10G\n    \
                 OR\n        \
                 $ paperd run --default-args 2G\n    \
                 OR\n        \
                 $ paperd run -- -Xmx5G -Xms5G",
            );
    }
}

fn tail_arg(message: &str) -> Arg {
    return Arg::with_name("TAIL").help(message).short("t").long("tail");
}
