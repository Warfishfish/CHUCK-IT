#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
out="$PWD/checkpoints/2026-11-07-road-reference/evidence"
export BEVY_ASSET_ROOT="${CAPTURE_ASSET_ROOT:-$PWD/rust/crates/bbq_app}"
if ioreg -n Root -d1 | grep -q '"CGSSessionScreenIsLocked"=Yes'; then exit 1; fi
rust/target/release/bbq_app --mode smoko --shot "$out/$1.png" --review --big --look polished --style pop --cam '13,2.45,-7.8,141,-13,64' --capture-frame 360 >"$out/$1.log" 2>&1
