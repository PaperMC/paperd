#!/bin/sh

# This file is part of paperd, the PaperMC server daemon
# Copyright (C) 2019 Kyle Wood (DemonWav)
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Lesser General Public License as published by
# the Free Software Foundation, version 3 only.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Lesser General Public License for more details.
#
# You should have received a copy of the GNU Lesser General Public License
# along with this program. If not, see <https://www.gnu.org/licenses/>.

set -e

if [ "$1" = "__build" ]; then
    echo "Building paperd"

    # Force static linking of ncurses
    NCURSES_RS_RUSTC_LINK_LIB="static=ncursesw" \
        NCURSES_RS_RUSTC_FLAGS="-l static=tinfo -L native=/usr/lib/x86_64-linux-gnu" \
        C_INCLUDE_PATH="/usr/include" \
        cargo build --color always --release --features console

    paperd_path="target/release/"
    paperd_file="${paperd_path}paperd"

    echo "Stripping unneeded symbols from paperd"
    strip "$paperd_file"

    echo "Packaging paperd"
    XZ_OPT=-9 tar -Jcf paperd.tar.xz --transform="s|$paperd_path||g" "$paperd_file"

    echo "Build complete, output file: paperd.tar.xz"
else
    docker build -t paperd .
    docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/paperd paperd
fi
