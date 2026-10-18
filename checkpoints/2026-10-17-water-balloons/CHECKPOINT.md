# Water balloons — recovery checkpoint

Status: completed, awaiting user review. Implementation and captures: 17 October 2026; final artifact verification: 18 October 2026. No commit or push.

## Starting state and recovery

Starting commit: `89361785455827fb12e050e9e51f3614b362dd2c`. Existing uncommitted grass, lighting, Race and Smoko work is preserved. `starting-status.txt`, `starting-tracked.patch`, `starting-untracked.tar.gz` and `before-source/` preserve the original pending source state. `previous-LATEST.md` records the preceding checkpoint. Do not reset Git or apply old patches over later edits.

`changed-files.json` lists the 27 changed Rust sources with before/after SHA-256 hashes. `water-balloons.patch` contains only this update relative to the preserved starting sources. `recovery-check.txt` records successful isolated rollback verification, including rejection of later edits.

From the repository root, preview rollback with:

```sh
python3 checkpoints/2026-10-17-water-balloons/revert.py
```

Only if reverting is desired, add `--apply`, then rebuild. The script checks every file before writing and refuses to overwrite later edits. It restores the pre-balloon sources while retaining earlier pending graphics/Race/Smoko changes. Checkpoint evidence stays on disk; update LATEST manually after a deliberate rollback.

## Implemented rules

Throwable gnomes are now tied, colourful water balloons in pickups, inventory, bots and throwing. Decorative gnome geometry remains. A valid hit consumes the balloon once, creates a splash and applies four seconds of wetness at 80% movement speed. Repeated hits refresh the timer without stacking. Walking and sprinting use the same multiplier. Existing acceleration still applies during speed transitions.

Misses and scenery impacts burst without wetting a player; catches preserve the balloon. Expiry, respawning and round transitions clear wetness. Balloons apply no knockdown or stun; other weapons retain their existing behaviour. Wet droplets preserve faces and team colours.

Tuning: `rust/crates/bbq_core/src/stun.rs` defines WET_DURATION and WET_SPEED. Core movement, hitting, flight and itemworld implement the status and single-hit consumption. `looks.rs` supplies balloon geometry; app `wet.rs` pools droplets, with HUD/inventory feedback. `visual_review.rs` supplies an opt-in actual-collision replay fixture.

Host-owned wet duration and positions replicate through protocol 10 snapshots, including explicit recovery. Everyone in an online session needs the updated build. Client-supplied wet duration is not trusted. Wet remote movement is bounded by host movement rules within the existing client-predicted architecture; this is not a general anti-cheat rewrite.

## Verification and actual captures

- Final tests: 621 passed (203 app, 418 core), one existing ignored; `tests.log`.
- Final optimized release build passed; `release-final.log`. Existing warnings remain. Earlier release-build.log is superseded.
- Focused tests cover the 20% speed multiplier, expiry, refreshing, one-hit consumption, catches/misses, round/respawn cleanup and host/guest wire replication through actual hit and recovery.
- Actual release replay: dry 6.200 m/s, wet 4.960 m/s, recovered 6.200 m/s. The fixture throws a real projectile; wetness and speed are not injected. See `motion/capture.log`.
- `water-balloon.webp`: seven-second actual gameplay animation, 105 captured frames; all checked for black output. `motion/splash.png`, `wet.png`, `dry.png` retain stills.
- Before/after held-item and Heist arena captures; current Smoko and Race captures. Backyard is exercised by the held-item and hit replay scenes. All capture logs completed without ERROR. Automated mode cleanup tests cover all five modes; captures are smoke checks, not exhaustive playtesting.
- `review.html` presents the comparisons and motion capture. Capture and encoding scripts are retained for reproducibility.

## Performance and limitations

Same Mac, 1280×800, fixed scene/camera, uncapped rendering, 300 warmup frames and 600 measured wall-clock intervals; compiler finished before sampling. These are CPU wall-clock observations, not GPU timestamps.

| Scene | Before mean | After mean | Change |
| --- | --- | --- | --- |
| Held item / Backyard | 6.702 ms / 149.22 FPS | 7.437 ms / 134.46 FPS | +11.0% frame time |
| Heist arena | 8.438 ms / 118.51 FPS | 8.924 ms / 112.06 FPS | +5.8% frame time |

Held-item p95 rose from 7.321 to 8.166 ms with zero frames over 16.67 ms. Heist p95 rose from 9.506 to 12.844 ms, with 18/600 frames over 16.67 ms versus 1/600 before. This is an observed cost, not a claim of unchanged performance. Scene activity varies; a single sample cannot isolate GPU cost or prove a stable regression.

Cost inspection: wet visuals allocate a bounded pool of 216 drops sharing one mesh/material and no shadows; hidden when dry. Held scene mesh entities rose from 5822 to 6219 (including that pool and other scene variability), while the active Heist scene fell from 7369 to 6710. No unbounded particle creation or expensive fluid simulation was introduced. Further repeated profiling can establish the remaining variability; the Mac was locked at final documentation, so no new graphical run was attempted.

Live two-computer multiplayer playtesting remains unverified; headless replication tests passed. Existing Race/Heist online restrictions remain. Race is still the foundation milestone: sideline scoring/upgrades are not implemented by this change.

## Launch and next step

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --give balloon
```

Omit `-- --give balloon` for normal startup. Review the saved captures and playtest. Await user review before committing, pushing or starting another milestone. The automatic resume follow-up is to be paused now that this work is awaiting review.

## Publication approval — 18 October 2026

User approved commit and push of the current integrated game state. Earlier awaiting-review/no-push statements describe the checkpoint at capture time. Actual captures, limitations and selective recovery files are preserved.
