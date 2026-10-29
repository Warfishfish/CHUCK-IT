# Bush Servo atmospheric depth — ready for review (29 October 2026)

A focused Smoko-only lighting pass brings the distant mesas closer to the dusty
horizon colour while preserving their silhouettes and keeping the playable yard
clear. The change applies only to polished Pop Smoko and restores the camera's
original fog when leaving that mode. No other map, gameplay, collision, geometry
or network settings change.

## Review

[Actual before/after native game captures](review.html). Both images use the same
1280×800 overview camera and landscape assets. The after image uses a slightly
warmer haze colour (`#d1d2cc`) and moves the linear fog range from 60–160 m to
55–150 m. This is a restrained adjustment after a first trial faded the mesas
too heavily; that overly strong trial is not included in the review.

## Build record

`cargo build -p bbq_app --release` completed successfully with the repository's
existing warnings. I compared the published 60–160 m / `#c9cfd0` haze against
55–150 m / `#d1d2cc` using release-native builds with the same ground-cover
assets. Three alternating runs per setting used the same 1280×800 overview
camera; each run warmed up for 300 frames and measured 600 wall-clock intervals
with AutoNoVsync.

| Haze | Mean frame time across runs | Median of run p95 values | Mean FPS | Frames over 16.67 ms |
| --- | ---: | ---: | ---: | ---: |
| Before | 7.670 ms | 9.178 ms | 130.40 | 5 / 1,800 |
| After | 7.595 ms | 9.404 ms | 131.66 | 4 / 1,800 |

The 0.075 ms lower mean and 0.226 ms higher p95 are within run-to-run variation;
no measurable frame-time regression appeared. These are CPU/GPU-present wall
clock intervals, not GPU timestamps, and do not establish browser or guest-device
performance. Full per-run logs and metrics are in `evidence/performance/`.
Browser build/publication and device testing remain pending review.

## Changed files

- `rust/crates/bbq_app/src/servo_finish.rs`: Smoko-only haze colour/range.
- This checkpoint's comparison captures and review page.
- `checkpoints/LATEST.md`.

## Recovery

No commit, push or browser update. To return to the published state, restore
`rust/crates/bbq_app/src/servo_finish.rs` from commit `952efd2`. Preserve the
unrelated `concept art/` directory.
