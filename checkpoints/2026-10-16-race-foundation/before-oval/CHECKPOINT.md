# Race foundation — Milestone 1
16 October 2026. Implemented; awaiting user review. Do not commit, push or start Milestone 2 without approval.

## Recovery checkpoint
Starting commit: `89361785455827fb12e050e9e51f3614b362dd2c` (committed Smoko milestones). Starting working tree was clean; see starting-status.txt. No unrelated work changed.

`implementation.patch` contains all race source changes, including new files. `FILES.sha256` lists the 15 source/tool files. The patch was applied to isolated copies of the starting commit and every file matched; see recovery-verification.txt. Recover into a separate checkout at the starting commit, apply the patch, then verify hashes from that checkout root. Never reset or overwrite the active working tree. Checkpoint documents and captures are stored alongside this file. The prior executable remains temporarily at /tmp/bbq-race-m1-before.

## Scope delivered
Selectable Backyard Circuit under Maps / Game modes → Race. Racer and Sideline Thrower role selection is saved in settings. Racer supplies the human plus two bots; Sideline Thrower supplies three racer bots and lets the human move along a bounded sideline strip. This is a solo preview: the menu blocks online race starts. Sideline attacks/scoring/upgrades and complete online synchronization are future milestones, not implemented features.

A separate procedural dirt circuit replaces the standard yard visually while racing. It has eight numbered gates, white edges, direction arrows, a checkered finish, fence, trees and sideline chairs. Existing renderer, character models and movement are reused; no Blender changes, generated assets or paid services. The standard yard is restored on leaving Race, and race geometry despawns. Legacy combat, item errands and ordinary hit scoring do not run in the race foundation.

## Rules and tuning
- Three racers and three laps. Eight checkpoints per lap, passed in order; gate 8 is the start/finish. Constants TRACK, WIDTH, LAPS and LIMIT live in rust/crates/bbq_app/src/race.rs.
- 180-second limit. All racers finishing (or disconnecting) ends the round early. Finished racers stop moving.
- Finish times determine rank. Unfinished racers rank by checkpoints passed, then distance to the next target. Disconnected racers rank last. Exact ties keep deterministic player-ID order.
- Leaving the lane tolerance, moving more than two metres in a tick or falling out of bounds returns a racer to their last passed gate without awarding progress. This also cancels velocity.
- Bots use the actual movement system, with modest lane offsets to avoid all converging on a gate centre. Bot count is automatically filled at round start; arbitrary bot counts are hidden for this preview.
- Sideline preview is bounded to x [-13,13], z [-17,-12]. Throwing and upgrades are intentionally unavailable until their reviewable milestone.

## Verification
612 tests pass (199 app, 413 core), one existing ignored. Tests include actual bot movement finishing all three laps, role-based bot fill, switching from race to Smoko, ordered lap progression and shortcut/teleport rejection. Previous mode tests remain passing. Final release build passes. See tests.txt and build.txt. The final change after the full suite was a display-only correction to the round announcement's bot count; release rebuilt and racer capture checked afterward.

Actual menu, racer, sideline, 75-frame motion, completed race results and matched Heist captures saved in evidence/. The completed bot race finished Kev 30.48s, Shazza 31.23s, Davo 32.62s; results display finish times and ranking. Motion frames were checked non-black, inspected and encoded into race.webp. Review page: review.html. Capture runner: rust/tools/race_foundation_review.py. Initial check.txt/captures.txt errors were intermediate diagnostics, superseded by tests.txt/build.txt and final-captures.txt.

Performance at 1280x800, no vsync, fixed 60Hz simulation, 600 wall-clock samples after warmup:

| Run | Mean | P95 | FPS | >16.67ms |
| --- | --- | --- | --- | --- |
| Heist baseline, 11 bots | 8.939ms | 10.375ms | 111.87 | 1/600 |
| Heist after, same setup | 9.244ms | 11.289ms | 108.18 | 1/600 |
| Race, 3 bots, overview | 6.948ms | 8.288ms | 143.93 | 0/600 |

Matched Heist asset counts are identical: 2,851 meshes, 893 materials, 7,167 mesh entities, 42,129,816 image CPU bytes. Race: 2,549 meshes, 754 materials, 5,496 mesh entities, 40,833,432 image CPU bytes (includes loaded/hidden assets). Heist difference is 0.305ms / 3.4%; no material regression observed, but a single pair cannot distinguish small changes from run-to-run noise. Race is a different, lighter workload. These are frame intervals, not GPU timestamps.

## Launch and next step
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release
```
Solo → Maps / Game modes → Race → choose role → Race preview. Direct racer launch adds `-- --mode race`; sideline preview adds `-- --mode race --sideline`. WASD moves, mouse steers, Space jumps and Shift boosts.

Review this milestone before proceeding. Milestone 2 adds the actual sideline throwing game, bounded racer hit effects, 50/75/100… consecutive-hit scoring, independent thrower standings and thrower bots. Milestone 3 adds upgrades and multiplayer verification. Race camera/feel and fair racing lines still need human playtesting.

Automatic hourly recovery was enabled at the user's request using automation ID resume-smoko-milestone-if-interrupted, renamed Resume race foundation if interrupted. Pause it once this completed milestone is presented for review; no further autonomous implementation is authorized until review.
