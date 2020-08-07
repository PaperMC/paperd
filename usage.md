Using paperd
============

`paperd` require Paper build #<TODO> and up to work correctly. Attempting to use `paperd` with an older version of Paper
will not work.

The first step in using `paperd` is to read the `--help` documentation on the command itself. The documentation is
pretty complete and will not be repeated here.

Installation
------------

To install `paperd` simply [download the latest release from Jenkins](https://papermc.io/ci/view/all/job/paperd/) on
your server:

```sh
curl https://papermc.io/ci/view/all/job/paperd/lastSuccessfulBuild/artifact/paperd.tar.xz -o paperd.tar.xz
```

Unpack the `.tar.xz` file to get the `paperd` binary:

```sh
tar fxv paperd.tar.xz
```

This will result in the executable `paperd` binary being extracted. Place this anywhere you like, it can be next to your
`paperclip.jar` file in your server directory, or you can place it in a directory somewhere on your `PATH` if you want
to use it more like a typical command.

Repeat this process any time you are updating `paperd`.

Installing as a systemd service
-------------------------------

### TODO

General Usage
-------------

This document will not go into significant detail on how to use the `paperd` tool, as stated above reading the `--help`
documentation is the easiest method of learning how to use it. But here is a quick breakdown of the commands `paperd`
makes available:

 * Commands for general server administration:
   * `log`: View the latest log messages, or follow the log file.
   * `send`: Send a command to the server.
   * `status`: View the current status of the server.
   * `timings`: Generate a Timings report and get a URL to view it.
   * `console`: Attach to an emulated console for the server. 
 * Commands for running the server:
   * `run`: Run the server in the foreground (not as a daemon, really only useful for testing)
   * `start`: Start the server in the background as a daemon
   * `restart`: Restart the server. While in daemon mode, this is the same as the `/restart` command in-game. This is
                a much cleaner system than the old "restart" script system. Instead, The server fully shuts down with
                an exit code telling `paperd` to restart it.
   * `stop`: Stop the server, optionally killing it if it does not respond.
