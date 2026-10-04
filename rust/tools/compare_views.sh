#!/bin/sh
# Take the standard comparison views with the Rust game and print how far each is from the browser picture.
#   rust/tools/compare_views.sh <folder with b_over.png b_eye.png ...>
# (the browser pictures come from running the __shot lines in compare.md)
set -e
dir="$1"
cd "$(dirname "$0")/.."
run() {
  name="$1"; shift
  tools/rust_shot.sh "$dir/r_$name.png" --bare --cam "$@" >/dev/null
  printf "%-6s " "$name"
  python3 tools/pngtool.py diff "$dir/b_$name.png" "$dir/r_$name.png" "$dir/d_$name.png"
  python3 tools/pngtool.py side "$dir/b_$name.png" "$dir/r_$name.png" "$dir/s_$name.png"
}
run over  0,45,40,0,-50,60
run eye   0,1.55,19.5,0,0,85
run pool  -20,3,20,0,-20,70
run back  -10,2.2,-8,0,-5,85
run smoko -16,2.2,-3,90,-8,85
run shed  12,2.2,-6,-45,-5,85
