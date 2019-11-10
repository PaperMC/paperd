#!/usr/bin/env bash

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

function __strip() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        strip -x "$1"
    else
        strip "$1"
    fi
}

function strip_paperd() {
    paperd_lib="$1"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        strip -x "$paperd_lib"
    else
        mapfile -t symbols < <(nm "$paperd_lib" | grep Java_com_destroystokyo_paper | awk '{print $3}')

        for symbol in "${symbols[@]}"; do
            out_args+=("-K")
            out_args+=("$symbol")
        done

        strip "${out_args[@]}" "$paperd_lib"
    fi
}

function help() {
    echo "Expected 'clean', 'build', or 'package'"
}

if [[ -z "$1" ]]; then
    help
    exit
fi

while [[ -n "$1" ]]; do
    case "$1" in
    "clean")
        rm -f paperd.tar.xz
        cargo clean
        (
            cd paperd-jni
            cargo clean
        )
        ;;
    "build")
        exten="$(if [[ "$OSTYPE" == "darwin"* ]]; then echo "dylib"; else echo "so"; fi)"
        if [[ "$2" == "--release" ]]; then
            rel="true"
            lib_file="target/release/libpaperd_jni.$exten"
        else
            lib_file="target/debug/libpaperd_jni.$exten"
        fi

        (
            echo "Building paperd-jni"
            cd paperd-jni
            if [[ -n "$rel" ]] ; then
                cargo build --color always --release
            else
                cargo build --color always
            fi

            if [[ -n "$rel" ]]; then
                echo "Stripping unneeded symbols from libpaperd_jni.$exten"
                strip_paperd "$lib_file"
            fi

            echo "Compressing libpaperd_jni"
            gzip -fk -9 "$lib_file"
        )

        echo "Building paperd"
        (
            export PAPERD_JNI_LIB="../paperd-jni/$lib_file.gz"
            cargo build --color always "${@:2}"
        )

        if [[ -n "$rel" ]]; then
            echo "Stripping unneeded symbols from paperd"
            if [[ "$OSTYPE" == "darwin"* ]]; then
                strip -x "target/release/paperd"
            else
                strip "target/release/paperd"
            fi
        fi

        echo "Packaging paperd binary"
        if [[ -n "$rel" ]]; then
            pream="target/release/"
        else
            pream="target/debug/"
        fi
        XZ_OPT=-9 tar -Jcf paperd.tar.xz --transform="s|$pream||g" "${pream}paperd"
        exit
        ;;
    "package")
        docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/paperd -w /usr/src/paperd rust:1.36.0 ./build.sh build --release --features console
        ;;
    *)
        help
        exit
        ;;
    esac
    shift
done
