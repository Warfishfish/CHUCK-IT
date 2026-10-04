#!/bin/sh
# Take a picture with the Rust game and wait for it to finish.
#   rust/tools/rust_shot.sh out.png --gallery
#   rust/tools/rust_shot.sh out.png --cam 0,1.55,10,0,0,85
# Needs a screen (it opens a small window for a moment).
set -e
export PATH="$HOME/.cargo/bin:$PATH"
out="$1"; shift
cd "$(dirname "$0")/.."
cargo run -q -p bbq_app -- --shot "$out" "$@" >/dev/null 2>&1
test -s "$out" && echo "saved $out"
