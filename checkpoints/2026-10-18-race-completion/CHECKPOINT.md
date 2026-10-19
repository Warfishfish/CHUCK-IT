# Race completion and dramatic balloon splash

Completed 19 October 2026; awaiting user review. No commit or push.

Starting point: `39fad79476fc4f2bed240d2c50dc33d6f580b4c0` on main, clean working tree. The previously published water balloons, Race foundation, Smoko timing and grass/lighting are preserved. Sources/legacy conversations were treated as references.

[Actual game captures and clips](review.html) · [Source manifest](changed-files.json) · [Patch](implementation.patch)

## Implemented rules

- The existing 190 m oval has eight ordered gates and three laps. Three racer seats; bots fill vacancies. Host racer preference takes priority, then connected players in stable order; overflow plays as throwers. Two thrower bots provide opposition. Choose Racer or Sideline Thrower in Maps / Game modes before starting.
- Throwers stay on the near curved terrace and target active racers. Hold/release left click to throw. Supplies replenish automatically. Q changes held items when the second choice is unlocked.
- Each thrown item can score once. Consecutive hits pay 50, 75, 100… points. Attempts resolve in launch order, including when collisions arrive out of order. A resolved miss resets the streak without removing points. Maximum three outstanding throws; six-second resolution timeout. Finished racers are excluded.
- Balloon hits slow racers to 80% for four seconds; teddy hits for 0.8 seconds. No Race knockdown or stun. A 5.5-second protection window prevents repeated slowdown refreshes; hits during protection may still score. Off-course recovery returns to the last passed gate without advancing progress, with two seconds of protection.
- Unlocks: 225 points reduces refill delay from 1.4 to 1.0 seconds; 500 reduces throw recovery from 0.55 to 0.35 seconds; 900 adds a second slot and teddy choice. Bots use the same supply, scoring and upgrade rules.
- Round ends when all racers finish/disconnect or after 300 seconds. Finishers rank by time; unfinished racers by passed gates then distance to the next gate, disconnected racers last. Throwers have separate standings/winner; tied positive leaders share the win. Zero points does not produce a throwing winner.
- Host validates roles, throws, item ownership, launch position/speed, cooldowns, progress, scores and upgrades. Protocol 11 requires matching builds. Role changes are accepted only between rounds. Guest snapshots reproduce host-owned progress, wet state and scores.
- Balloon burst now has 140 radial water droplets and 18 brief mist particles. The existing pool splash is unchanged. Standard modes retain the four-second 20% wet slowdown.

## Verification

- `cargo test --workspace`: 626 passed (208 app + 418 core), one existing ignored, zero failures; see tests.log.
- `cargo build -p bbq_app --release`: passed, existing warnings retained; see release-build.log.
- Tests cover ordered/duplicate scoring, misses, bounded supplies, unlocks, role/throw validation, authoritative snapshots, wet expiry/protection and full bot race completion. Existing mode and water-balloon tests passed.
- Actual full-round capture: all three bots finished (91.64, 93.92, 97.59 seconds), separate throwing results shown. Race motion, both role views and setup UI captured.
- Actual water replay: real balloon collision, wet effect, movement 6.20 → 4.96 → 6.20 m/s. Compressed clips are water-balloon.webp and race.webp; selected stills retained. Raw frame sequences remain local and ignored by Git.
- Real local relay with two final release instances: host Racer and guest Thrower joined and entered Play; guest submitted an actual throw. Screenshots/logs in network/. The first capture batch exited early before the network assertions; the standalone retry passed and supersedes those logs. No separate-machine/WAN latency or packet-loss verification was performed.
- Actual Heist and Smoko smoke captures; standard Backyard exercised in the water replay. Teams remains covered by existing tests, without a new dedicated visual capture.
- `git diff --check` passed. Guarded recovery verified in an isolated copy; see recovery-check.txt.

## Performance

Same 1280×800 scene/camera, 600 wall-clock frame intervals, fixed simulation step, no compiler running. Not GPU timestamp measurements.

| Scene | Before mean | After mean | Before / after p95 |
| --- | --- | --- | --- |
| Race sideline | 6.887 ms / 145.19 FPS | 6.845 ms / 146.10 FPS | 8.648 / 8.744 ms |
| Held balloon | 8.202 ms / 121.93 FPS | 8.066 ms / 123.98 FPS | 11.933 / 11.935 ms |

No meaningful measured regression. The first held-balloon sample was noisy (8.780 ms), so both baseline and changed scene were repeated back-to-back; first sample retained as after/held-first.metrics.txt. Race includes two extra throwing bots and active projectiles/effects. These measurements are not an isolated peak-splash GPU benchmark.

Verification uncovered excessive fallback font atlases in the hidden developer HUD (~1.46 GB retained image data). It now skips hidden text updates and uses the loaded UI font. Final Race image data was 39.8 MB and 41.9 MB after the sustained full-round run. Mesh/material/entity costs are saved with each metrics file. First-person wet droplets can briefly cover the view; review the racer capture when judging effect intensity. Human balance/comfort playtesting remains appropriate, particularly long bot streaks and upgrade pacing.

## Test and recovery

Launch the menu:

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Choose Maps / Game modes → Race → Racer or Sideline Thrower. Direct thrower launch: append `-- --mode race --sideline` instead of `-- --menu`; omit `--sideline` to race.

The 16 changed Rust source files are listed with before/after SHA-256 hashes in changed-files.json. Exact starting versions are saved in before-source/. implementation.patch records this update. Capture/checkpoint scripts and evidence are in this directory; checkpoints/LATEST.md points here.

To check selective rollback without changing files, from the repository root:

```sh
python3 checkpoints/2026-10-18-race-completion/revert.py
```

Only if reverting is requested, add `--apply`, then rebuild. The script checks every source before writing and refuses if later edits exist; request a selective merge in that case. It restores only this update's Rust sources, preserves prior published features, and retains checkpoint evidence. Do not use a destructive reset. The real repository was only dry-run checked; actual restoration was tested in a disposable copy.

Next: user playtest/review. Do not commit, push or start another milestone without review.
