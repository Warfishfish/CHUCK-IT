#!/bin/sh
# Take a picture with the Rust game and wait for it to finish.
#   rust/tools/rust_shot.sh out.png --gallery
#   rust/tools/rust_shot.sh out.png --bare --cam 0,1.55,10,0,0,85
# Needs a screen (it opens a small window for a moment). A sleeping display gives a black
# picture, so the display is woken first and a black result is tried again.
set -e
export PATH="$HOME/.cargo/bin:$PATH"
out="$1"; shift
here="$(dirname "$0")"
cd "$here/.."
for try in 1 2 3 4; do
  caffeinate -u -t 3 2>/dev/null || true
  sleep 1
  cargo run -q -p bbq_app -- --shot "$out" "$@" >/dev/null 2>&1
  # the corner is sky or scenery in every view, so pure black means the screen was asleep
  if ! python3 tools/pngtool.py probe "$out" 2,2 | grep -q "(0, 0, 0)"; then
    echo "saved $out"
    exit 0
  fi
done
echo "only black pictures came back: is the display on?" >&2
exit 1
