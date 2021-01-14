paperd [![Current Release](https://img.shields.io/badge/release-1.0.0-orange.svg)](https://papermc.io/ci/job/paperd/)
======

| Build          | Status |
|----------------|--------|
| Latest Commit  | [![Build Status](https://travis-ci.org/PaperMC/paperd.svg?branch=master)](https://travis-ci.org/PaperMC/paperd/branches) |
| Latest Release | [![Build Status](https://papermc.io/ci/job/paperd/badge/icon)](https://papermc.io/ci/job/paperd/) |

paperd is a wrapper application which enables the PaperMC Minecraft server to be run more properly in the background as
a daemon, rather than simply backgrounded using `screen` or `tmux`. This is accomplished both by the `paperd`
application and custom changes in the Paper server.

**Support and Project Discussion:**
 - [IRC](https://webchat.esper.net/?channels=paper) or [Discord](https://discord.gg/papermc)

Building
--------

`paperd` is strictly Unix / POSIX compatible. Windows is not supported.

A 64 bit JDK is required to build `paperd` and a 64 bit JVM is required to use `paperd`.

This project requires the [Rust](https://www.rust-lang.org/) toolchain. `paperd` is built on the latest release of Rust,
currently version `1.44.1`. 

To build for your own system with console support:
```sh
cargo build --release --features console
```

If you don't want the console feature, or don't have `ncurses` installed, simply omit the feature to build without it:
```sh
cargo build --release
```

To build a release binary like the pre-built binaries Jenkins provides, you'll need Docker installed:
```sh
./build_release.sh
```

This will produce a `paperd.tar.xz` file in the current working directory, which is the file available for download
from Jenkins.

Documentation
-------------

[For general usage instructions, please click here.](usage.md)

[For technical info on how `paperd` works and communicates with the Paper server, please click here.](protocol.md)

Contributing
------------

PRs are greatly appreciated, but when a change requires modifications to both this project and to the
[Paper](https://github.com/PaperMC/Paper) server itself, please link both PRs together in the PR description.

For this project in particular, please run `rustfmt` with all default settings across the whole project before
committing.

License
-------

This project is licensed under LGPLv3 only, no future versions.
