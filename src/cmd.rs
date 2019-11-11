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

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, Shell, SubCommand};
use std::io;

pub fn get_cmd_line_matches<'a>() -> ArgMatches<'a> {
    let start_text = run_after_text("start");
    let run_text = run_after_text("run");
    return handle_cmd_line(start_text.as_str(), run_text.as_str()).get_matches();
}

pub fn gen_completions(shell: &str) {
    let start_text = run_after_text("start");
    let run_text = run_after_text("run");
    handle_cmd_line(start_text.as_str(), run_text.as_str()).gen_completions_to(
        "paperd",
        shell.parse::<Shell>().unwrap(),
        &mut io::stdout(),
    );
}

fn handle_cmd_line<'a, 'b>(start_after: &'b str, run_after: &'b str) -> App<'a, 'b> {
    let pid_arg = Arg::<'a, 'b>::with_name("PID")
        .help(
            "Custom PID file to send commands to a running server. If not set, the \
             PAPERMC_PID environment variable will be checked. If neither are set, the default \
             value is ./paper.pid.",
        )
        .short("p")
        .long("pid")
        .takes_value(true);

    let license_text = r"ISSUES:
    Please submit any bugs or issues with paperd to the paperd issue tracker:
    https://github.com/PaperMC/paperd/issues

SOURCE:
    The source code for this program is available on GitHub:
    https://github.com/PaperMC/paperd

LICENSE:
    GNU LGPLv3 only (no future versions)
    https://www.gnu.org/licenses/lgpl-3.0.en.html";

    return App::new("paperd")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .version(crate_version!())
        .author("PaperMC (papermc.io)")
        .about("PaperMC daemon for running and controlling daemonized PaperMC servers.")
        .subcommand(
            SubCommand::with_name("status")
                .about("Get the status of the currently running server.")
                .arg(&pid_arg)
                .display_order(1)
                .after_help(license_text),
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
        .console(&pid_arg)
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
            SubCommand::with_name("timings")
                .about("If timings is enabled, generate a report and return the URL.")
                .arg(&pid_arg)
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Start the MC server in the background.")
                .arg(tail_arg(
                    "Tail the server log after starting the server. Press C-c to \
                     quit (will NOT stop the server).",
                ))
                .java_run(start_after)
                .arg(
                    Arg::with_name("KEEP_ALIVE")
                        .help(
                            "Restart the server when it crashes. If the server stops gracefully \
                             from a shutdown command it will not restart, but if the server \
                             shutdowns down due to a crash, paperd will restart it automatically.",
                        )
                        .short("k")
                        .long("keep-alive"),
                )
                .display_order(2),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Start the MC server in the foreground.")
                .java_run(run_after)
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
        .subcommand(
            SubCommand::with_name("restart")
                .about(
                    "Tell the server to shutdown with an exit code telling paperd to restart. \
                     This will reuse the same command-line the original command invocation \
                     used, but if the jar has been replaced it will be used instead of the \
                     original jar. The paperd instance will not be changed if it has been updated, \
                     however, as it does not restart.",
                )
                .arg(&pid_arg)
                .arg(tail_arg(
                    "Tail the server log after asking the server to restart. Press \
                     C-c to quit.",
                ))
                .display_order(3),
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate completion scripts for your shell")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::with_name("SHELL")
                        .help("The shell to generate the completion script for")
                        .possible_values(&["bash", "zsh", "fish"]),
                )
                .after_help(COMPLETIONS_HELP)
                .display_order(4),
        )
        .after_help(license_text);
}

trait PaperArg<'a, 'b> {
    fn java_run(self, after_text: &'b str) -> Self;
    fn console(self, arg: &Arg<'a, 'b>) -> Self;
}

impl<'a, 'b> PaperArg<'a, 'b> for App<'a, 'b> {
    fn java_run(self, after_text: &'b str) -> Self {
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
            .arg(
                Arg::with_name("CONFIG_FILE")
                    .help(
                        "Define a JSON configuration file which specifies all other arguments. \
                         This allows defining complex or large startup commands permanently for \
                         using them again for each server startup. The JSON configuration file can \
                         define more configuration past what is possible with just command line \
                         arguments. See documentation for the JSON configuration file below in the \
                         CONFIG FILE section.",
                    )
                    .long("config-file")
                    .takes_value(true),
            )
            .group(
                ArgGroup::with_name("JVM_ARGS")
                    .arg("DEFAULT_ARGS")
                    .arg("CUSTOM_ARGS"),
            )
            .after_help(after_text);
    }

    #[cfg(feature = "console")]
    fn console(self, arg: &Arg<'a, 'b>) -> Self {
        return self.subcommand(
            SubCommand::with_name("console")
                .about("Attach to the console of the running MC server.")
                .arg(arg)
                .display_order(1),
        );
    }

    #[cfg(not(feature = "console"))]
    fn console(self, _: &Arg<'a, 'b>) -> Self {
        return self;
    }
}

fn tail_arg(message: &str) -> Arg {
    return Arg::with_name("TAIL").help(message).short("t").long("tail");
}

fn run_after_text(command_text: &str) -> String {
    return format!(
        r#"EXAMPLES:
    The --default-args argument or the 'CUSTOM_ARGS' arguments are mutually exclusive. That is, you
    can either use --default-args OR specify custom arguments, but not both.

    Examples:
        $ paperd {cmd} -d 10G
    OR
        $ paperd {cmd} --default-args 2G
    OR
        $ paperd {cmd} -- -Xmx5G -Xms5G

CONFIG FILE:
    You may pass options to this command using a JSON configuration file instead of command line
    arguments using the --config-file argument. When using this argument the config file values
    have lower precedence than the other command line arguments, so any other arguments specified
    will effectively override any configuration values present in the file. The config file must be
    a valid JSON file with the following keys. All keys are optional.

    * jvm        | This is equivalent to the --jvm argument.
    * jarFile    | This is equivalent to the --jar argument.
    * workingDir | This is equivalent to the -w or --working-dir argument.
    * jvmArgs    | This is equivalent to the CUSTOM_ARGS argument.
    * serverArgs | This has no equivalent argument. This has the same format as the jvmArgs or
                   CUSTOM_ARGS configuration, but specifies server arguments instead of JVM
                   arguments such as --world-dir or --port.

    Example JSON file:
    {{
        "jarFile": "../some/global/paperclip.jar",
        "workingDir": "/minecraft/servers/paper",
        "jvmArgs": ["-Xmx5G", "-Xms5G"],
        "serverArgs": ["--port", "22222"]
    }}"#,
        cmd = command_text
    );
}

// This excellent description was taken from rustup
// https://github.com/rust-lang/rustup.rs/blob/256488923d3fb2637b7d706002b3e6d2db917590/src/cli/help.rs#L156
pub static COMPLETIONS_HELP: &str = r"DISCUSSION:
    One can generate a completion script for `paperd` that is
    compatible with a given shell. The script is output on `stdout`
    allowing one to re-direct the output to the file of their
    choosing. Where you place the file will depend on which shell, and
    which operating system you are using. Your particular
    configuration may also determine where these scripts need to be
    placed.

    Here are some common set ups for the three supported shells under
    Unix and similar operating systems (such as GNU/Linux).

    BASH:

    Completion files are commonly stored in `/etc/bash_completion.d/` for
    system-wide commands, but can be stored in
    `~/.local/share/bash-completion/completions` for user-specific commands.
    Run the command:

        $ mkdir -p ~/.local/share/bash-completion/completions
        $ paperd completions bash >> ~/.local/share/bash-completion/completions/paperd

    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take affect.

    BASH (macOS/Homebrew):

    Homebrew stores bash completion files within the Homebrew directory.
    With the `bash-completion` brew formula installed, run the command:

        $ mkdir -p $(brew --prefix)/etc/bash_completion.d
        $ paperd completions bash > $(brew --prefix)/etc/bash_completion.d/paperd.bash-completion

    FISH:

    Fish completion files are commonly stored in
    `$HOME/.config/fish/completions`. Run the command:

        $ mkdir -p ~/.config/fish/completions
        $ paperd completions fish > ~/.config/fish/completions/paperd.fish

    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take affect.

    ZSH:

    ZSH completions are commonly stored in any directory listed in
    your `$fpath` variable. To use these completions, you must either
    add the generated script to one of those directories, or add your
    own to this list.

    Adding a custom directory is often the safest bet if you are
    unsure of which directory to use. First create the directory; for
    this example we'll create a hidden directory inside our `$HOME`
    directory:

        $ mkdir ~/.zfunc

    Then add the following lines to your `.zshrc` just before
    `compinit`:

        fpath+=~/.zfunc

    Now you can install the completions script using the following
    command:

        $ paperd completions zsh > ~/.zfunc/_paperd

    You must then either log out and log back in, or simply run

        $ exec zsh

    for the new completions to take affect.

    CUSTOM LOCATIONS:

    Alternatively, you could save these files to the place of your
    choosing, such as a custom directory inside your $HOME. Doing so
    will require you to add the proper directives, such as `source`ing
    inside your login script. Consult your shells documentation for
    how to add such directives.";
