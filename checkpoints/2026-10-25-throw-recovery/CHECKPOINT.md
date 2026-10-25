# Charge trajectory and smooth throw recovery — 25 October 2026

Starting commit: c720bf2. Clean starting tree. Exact original versions of all six changed source files are saved under before/; recover selectively after comparing later changes, or copy this checkout elsewhere and restore those files there. The full project backup from the preceding checkpoint is also preserved. No commit or push authorized for this follow-up.

## Changes

- Charge adds up to 0.21 radians (about 12 degrees) of launch elevation, retaining the existing speed curve, base lift, inherited movement and overhold fall. Tap throws retain their old direction. The trajectory preview calls the same launch function.
- Character charge pose and hand retain a damped spring on release, giving a small overshoot and natural settling instead of an abrupt reset.
- Wind-up squash fades out while release stretch fades in; existing weight scaling and hit responses remain.
- First-person view leans back by up to 0.07 radians (about 4 degrees) while charging and springs back on release. This is a presentation offset, not an extra alteration of mouse aim; charge lift is applied once in the launch function.
- Host charge/time and velocity validation remain in place. Guests send the resulting trajectory through the existing authoritative throw path. No new per-frame network animation data.
- Existing close-range scoring test fixtures now aim lower to compensate for the deliberately higher charged arc. Scoring assertions remain unchanged.

Tuning: CHARGE_ARC_RADIANS in core/hands.rs; RETURN_STIFFNESS, RETURN_DAMPING and CAMERA_CHARGE_LEAN in core/feel.rs.

## Verification

669 tests pass (237 app + 432 core, 1 historical ignored). Includes monotonically increasing launch angle, capped charge, retained launch speed and spring continuity/overshoot/settling. Native release and browser builds pass. Actual first-person input playback and third-person scripted charge/release clips are included in review.html. Native release gameplay code is verified; the subsequent capture-only correction points the staged third-person throw toward the observer, and is included in debug capture/browser builds. No live two-computer playtest or new performance benchmark was run for this follow-up. Existing warnings remain.

Launch:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

## Publication approval — 25 October 2026

User approved committing and pushing this work, including rebuilt browser artifacts. The Backyard graphics preview remains optional (`--backyard-finish` / `?backyard_finish=1`). Earlier review statuses above are historical. Live Cloudflare availability has not been reverified for this publication.
