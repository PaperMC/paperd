#!/usr/bin/env bash

set -e

echo "Building paperd"

if [[ "$INCLUDE_CONSOLE_BUILD" == "true" ]] ; then
  EXTRA_ARGS="--features console"
fi
# shellcheck disable=SC2086
cargo build -j 1 --target-dir=/usr/src/target --color always --release $EXTRA_ARGS

paperd_path="/usr/src/target/release/"
paperd_file="${paperd_path}paperd"

echo "Stripping unneeded symbols from paperd"
strip "$paperd_file"

echo "Packaging paperd"
XZ_OPT=-9 tar -Jcf /usr/src/target/paperd.tar.xz -C "$paperd_path" "paperd"

echo "Build complete, output file: /usr/src/target/paperd.tar.xz"
