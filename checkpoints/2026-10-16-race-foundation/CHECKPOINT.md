# Race foundation — Milestone 1
16 October 2026. Implemented; awaiting user review. Do not commit, push or start Milestone 2 without approval.

## Recovery checkpoint
Starting commit: `89361785455827fb12e050e9e51f3614b362dd2c` (committed Smoko milestones). Starting working tree was clean; see starting-status.txt. No unrelated work changed.

`implementation.patch` contains all race source changes, including new files. `FILES.sha256` lists the 17 source/tool files. The patch was applied to isolated copies of the starting commit and every file matched; see recovery-verification.txt. Recover into a separate checkout at the starting commit, apply the patch, then verify hashes from that checkout root. Never reset or overwrite the active working tree. Checkpoint documents and captures are stored alongside this file. The prior executable remains temporarily at /tmp/bbq-race-m1-before.

## Scope delivered
Selectable Backyard Circuit under Maps / Game modes → Race. Racer and Sideline Thrower role selection is saved in settings. Racer supplies the human plus two bots; Sideline Thrower supplies three racer bots and lets the human move along a bounded sideline strip. This is a solo preview: the menu blocks online race starts. Sideline attacks/scoring/upgrades and complete online synchronization are future milestones, not implemented features.

A separate 190.39-metre oval dirt circuit replaces the standard yard visually while racing. It has eight numbered gates, white edges, direction arrows, a checkered finish, fence, trees and sideline chairs. Existing renderer, character models and movement are reused; no Blender changes, generated assets or paid services. The standard yard is restored on leaving Race, and race geometry despawns. Legacy combat, item errands and ordinary hit scoring do not run in the race foundation.

## Rules and tuning
- Three racers and three laps. Eight checkpoints per lap, passed in order; gate 8 is the start/finish. Constants TRACK, WIDTH, LAPS and LIMIT live in rust/crates/bbq_app/src/race.rs.
- 300-second limit. All racers finishing (or disconnecting) ends the round early. Finished racers stop moving.
- Finish times determine rank. Unfinished racers rank by checkpoints passed, then distance to the next target. Disconnected racers rank last. Exact ties keep deterministic player-ID order.
- Leaving the lane tolerance, moving more than two metres in a tick or falling out of bounds returns a racer to their last passed gate without awarding progress. This also cancels velocity.
- Bots use the actual movement system, with modest lane offsets to avoid all converging on a gate centre. Bot count is automatically filled at round start; arbitrary bot counts are hidden for this preview.
- Sideline preview follows an elliptical near-side arc: angle ±0.85 radians, normalized radius 1.0–1.12 with base radii 42m/28m. Throwing and upgrades are intentionally unavailable until their reviewable milestone.

## Verification
613 tests pass (199 app, 414 core), one existing ignored. Tests include actual bot movement finishing all three laps, role-based bot fill, switching from race to Smoko, ordered lap progression and shortcut/teleport rejection. Previous mode tests remain passing. Final release build passes. See tests.txt and build.txt. All 613 tests and the final release build passed on the triple-length oval source.

Actual menu, racer, sideline, 75-frame motion, completed race results and matched Heist captures saved in evidence/. The completed bot race finished Kev 89.87s, Shazza 91.84s, Davo 94.27s; results display finish times and ranking. Motion frames were checked non-black, inspected and encoded into race.webp. Review page: review.html. Capture runner: rust/tools/race_foundation_review.py. Initial check.txt/captures.txt errors were intermediate diagnostics, superseded by tests.txt/build.txt and final-captures.txt.

Performance at 1280x800, no vsync, fixed 60Hz simulation, 600 wall-clock samples after warmup:

| Run | Mean | P95 | FPS | >16.67ms |
| --- | --- | --- | --- | --- |
| Heist baseline, 11 bots | 8.939ms | 10.375ms | 111.87 | 1/600 |
| Heist after, same setup | 7.938ms | 9.034ms | 125.97 | 0/600 |
| Long oval, 3 bots, overview | 6.014ms | 7.142ms | 166.27 | 0/600 |

Matched Heist asset counts are identical: 2,851 meshes, 893 materials, 7,167 mesh entities, 42,129,816 image CPU bytes. Long oval: 2,841 meshes, 754 materials, 5,889 mesh entities, 40,833,432 image CPU bytes (includes loaded/hidden assets). Small oval before enlargement: 2,840 meshes, same other asset counts, 6.076ms / 164.58 FPS. Camera reframed to fit the enlarged arena, so the oval timings are not strictly identical workloads. No observed regression in matched Heist; differences may reflect run-to-run noise. Frame intervals are not GPU timestamps.

## Launch and next step
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release
```
Solo → Maps / Game modes → Race → choose role → Race preview. Direct racer launch adds `-- --mode race`; sideline preview adds `-- --mode race --sideline`. WASD moves, mouse steers, Space jumps and Shift boosts.

Review this milestone before proceeding. Milestone 2 adds the actual sideline throwing game, bounded racer hit effects, 50/75/100… consecutive-hit scoring, independent thrower standings and thrower bots. Milestone 3 adds upgrades and multiplayer verification. Race camera/feel and fair racing lines still need human playtesting.

Automatic hourly recovery was enabled at the user's request using automation ID resume-smoko-milestone-if-interrupted, renamed Resume race foundation if interrupted. Pause it once this completed milestone is presented for review; no further autonomous implementation is authorized until review.

## Oval revision — 16 October 2026, in progress
User requested a non-square map. Updated to a smooth oval racing surface, elliptical lawn/fence and curved sideline terrace with matching movement bounds. Eight ordered gates and three laps retained. Prior source saved in before-oval/source.tar.gz and prior captures retained there. All 612 tests passed on the small oval; release build and fresh actual captures completed before enlarging. No commit/push.

## Triple-length oval — 16 October 2026, complete
User requested triple physical course length after the oval revision. The previous small oval is preserved in before-long-oval/ (source archive, overview, racer, results and metrics). Centreline grows from ellipse radii 12m/8m (63.46m per lap) to 36m/24m (190.39m), exactly 3×. Three laps cover about 571m. Lane width remains 4.8m. Round limit increases to five minutes.

Yard.race enables 52m/37m movement half-extents and disables the legacy pool/trampoline. Other modes retain existing boundaries/features; a new test covers this isolation. Bots follow the curve with angular look-ahead instead of cutting long chords between gates. Sideline movement follows the enlarged near arc. Outer lawn/fence remain elliptical.

Final release, 613 tests and actual menu/overview/racer/motion/full-race results/matched Heist captures passed. All 75 motion frames are non-black, encoded in evidence/race.webp. Actual results show all racers finished in 89.87–94.27s. Source patch/hash recovery verified for all 17 files. Final capture list: final-captures.txt. Nothing committed or pushed. Await user review; sideline gameplay remains Milestone 2.
