# Smoko separation — Milestone 1

15 October 2026. Milestone 1 complete; ready for review. Starting commit: 6dd9397c5ed109a920ebe5a401d72e162b5a968d (main, pushed). Working tree clean at start; previous ignored captures remain local. Recover using a separate checkout at this commit. Never reset or overwrite later active work.

Scope: remove Smoko props, activation/settings/help and Naughty Corner rules from standard FFA/Teams and Heist, including menu background. Preserve reusable core assets, carry/throw, knockout and seating code for the separate future Smoko mode. Restore open lawn and remove obsolete approach path. Do not begin Milestone 2, commit or push before user review.

Inspection: Features.smoko defaults on; F4/menu/saved settings can re-enable it. Yard renderer spawns pad/chairs/umbrella/signs; Heist moves them beside the pool. Runtime interactions/bots already use feature flags. Naughty Corner awards 100 to the local player for drag/drop/throw, so it needs redesign for future authoritative multiplayer delivery scoring. Net snapshots carry feature flags, seat state and zone centre. Existing carry/throw and bot errand systems can be reused but the new objective is not implemented in this milestone.

Plan: disable current map feature at construction and settings boundaries; stop spawning Smoko visuals; remove obsolete UI and F4 toggle; retain shared primitives with explicit opt-in test fixtures. Add regressions for no sitting/scoring in former zones, stale settings and Heist. Capture same before/after views and performance, run app/core tests and release build.

Launch:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky
```

Assets: procedural yard in rust/crates/bbq_core/src/looks_yard.rs, renderer in rust/crates/bbq_app/src/yard_scene.rs. Existing lawn textures remain. No new Blender assets needed. Starting release executable copied to /tmp/bbq-smoko-before for comparison; durable baseline is the starting Git commit.

Prior validation: 182 app tests passed, 1 ignored; 411 core tests passed in earlier milestone; prior release passed. Final results below.

## Implementation and checks

Current changes: standard/Heist yard constructors ignore legacy Smoko flags, defaults are off, round/menu startup disables Naughty Corner, saved settings cannot re-enable either. No Smoko render entities are spawned. F4/F8 toggles and related settings/help are removed; pause title is now Paused. The old approach path ends near the woodpile and natural leaf scatter can fill the former clearing. Shared procedural prop builders, knockout/carry/throw/seating primitives and wire fields remain available for the future mode.

Guest snapshots ignore legacy Smoko feature/seat flags while continuing to accept authoritative scores. Mixed-version sessions are not a supported compatibility target; all players should use this build.

185 app tests passed, 1 existing test ignored; 413 core tests passed. Five new regression tests cover constructor flags, current-map bot errands, sitting/delivery in both old locations across FFA/Teams/Heist, stale settings in the menu yard, and legacy snapshot flags while preserving host scores. Shared legacy seating/scoring tests explicitly opt in through a test fixture. Existing normal hits, carrying, throwing, Heist banking/winners and long bot simulation tests pass. Initial test-fixture compile/placement/score-ID errors were corrected; final run is tests.txt.

Release build and after captures passed. Before captures completed: evidence/before-standard.png, before-heist.png, before-stress.png with individual frame timing files. No human multiplayer playtest yet.

Recovery of this implementation: create a separate checkout at the starting commit, then apply implementation.patch from this folder and run shasum -a 256 -c FILES.sha256 from the checkout root (pass the full checkpoint path). The patch includes the new capture script. Keep this checkpoint folder with the recovered checkout. No commit or push has been made.

## Final verification and performance

Release build passed in 2m 34s with 16 existing app warnings. Six captures (three before, three after) completed without errors. Matching standard and Heist before/after images inspected: no pad, chairs, umbrella, Smoko signs or dedicated esky; continuous lawn remains and Heist base/pool/teddies are preserved. No new physics motion capture was needed for this removal milestone.

| Scene | Mean ms, before → after | FPS, before → after | P95 ms, before → after | Mesh entities, before → after |
|---|---:|---:|---:|---:|
| standard | 7.759 → 7.810 | 128.88 → 128.04 | 8.525 → 9.090 | 5825 → 5602 |
| heist | 8.098 → 8.234 | 123.49 → 121.45 | 9.457 → 10.546 | 6250 → 6028 |
| stress | 8.708 → 8.732 | 114.84 → 114.52 | 10.000 → 9.983 | 7389 → 7167 |

Same Mac, 1280×800, fixed 60 Hz simulation, uncapped presentation, 300 warmup frames + 600 wall-clock intervals; screenshot writing completes before measurement. No meaningful mean-frame-time regression (all under 2%). The 11-bot stress scene stayed near 115 FPS, with no frames above 16.67 ms in either sample. Thirty fewer mesh assets and twelve fewer material assets in each scene; image CPU bytes unchanged. Asset-count reductions are real; small timing differences are run-to-run noise. No GPU timestamp profiling or human multiplayer session was performed.

`git diff --check` passed. All implementation hashes checked. The recovery patch was applied against starting-commit source in a temporary directory and all reconstructed bytes matched. See recovery-verification.txt.

Review: open review.html for actual before/after screenshots. Next: user review before beginning the separate Smoko map/mode, committing or pushing. No commit or push performed. The temporary hourly credit-interruption follow-up is disabled now that this milestone is awaiting review.
