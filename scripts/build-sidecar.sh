#!/usr/bin/env bash
#
# Builds the translation worker and places it where Tauri expects a sidecar.
#
# The worker is a separate binary because whisper.cpp and llama.cpp cannot share
# a process - see translator/Cargo.toml.

set -euo pipefail

cd "$(dirname "$0")/.."

TRIPLE=$(rustc -vV | awk '/^host:/ {print $2}')

echo "==> building the translation worker for $TRIPLE"
cargo build --release --manifest-path translator/Cargo.toml

mkdir -p src-tauri/binaries
cp translator/target/release/marswind-translator \
	"src-tauri/binaries/marswind-translator-$TRIPLE"

echo "==> src-tauri/binaries/marswind-translator-$TRIPLE"
ls -lh "src-tauri/binaries/marswind-translator-$TRIPLE" | awk '{print "    " $5}'
