# Race sideline and visible gaze · 8 November 2026

Status: complete, awaiting user review. No commit, push or public browser update.

Starting commit 963e1f8c70f8b207a82be4146559de05a2dc7f5d. Verified source archive and previous release executable in /Users/marcus/Claude/australian-bbq-before-race-gaze-20261108-203358. Git was clean at backup time.

Scope: continuous full-lap sideline route, then bounded smooth cosmetic head pitch from existing camera controls. Preserve all modes, rules and graphics. No commit, push or publication authorised.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust` then `~/.cargo/bin/cargo run -p bbq_app --release -- --menu`.

Recovery: extract source.tar to a separate directory, apply pending.tar there, then compare selected files before restoring. Do not reset the working tree or overwrite later work. Asset root for saved executable is the existing rust/crates/bbq_app directory.

## Implementation

- Race: shared continuous ellipse band from inner axes 42/28 metres to radial multiplier 1.12; all angles allowed. Host clamp and guest prediction call the same projection. Route edges match the legal band. One vertex-coloured mesh replaces the short starting terrace. Existing spectator props move beyond the new route. Racer lane, gates, lap counts, grass, scoring, supplies and upgrades retained.
- Gaze: one pitch byte in protocol 28, separate cosmetic update even while race/knockback movement is host controlled; bounded smoothed existing head parent. See GAZE-DECISION.md.
- Two previously stale assertions updated for the already-implemented 19-item gallery and variable cloud clusters; production graphics untouched by these assertion corrections.

## Verification complete

- `cargo test -p bbq_core -p bbq_app`: **707 passed**, one existing ignored. Full logs in tests.log. Includes existing mode, inventory, wet, carrying, scoring and animation tests.
- Native release and browser WebGPU release both passed after the final grass cleanup. Logs saved here. Existing warnings remain.
- Native same camera `(0,72,58,0,-56,70)`, Polished Pop, 1280×800, fixed 60 Hz simulation, capture at frame 720 and 600 later wall-clock intervals. Average **6.257 → 6.467 ms** (+0.210 ms / 3.4%); p95 **7.225 → 7.508 ms**. 0 → 1 intervals over 16.67 ms. No meaningful frame-budget regression observed on this Mac. GPU timestamps were not measured.
- Browser same starting view, 1100×713, 10-second samples: average **13.352 → 13.335 ms**, p95 **14.3 → 14.3 ms**, no intervals above 33.3 ms. About 75 FPS, refresh-limited. Before uses the saved pre-change browser artifact with the same Race code; the final browser test is a one-person hosted round. These figures do not establish performance on other devices or internet connections.
- Whole route is one vertex-coloured mesh (582 vertices, 576 triangles), zero extra textures. Native mesh entities **7047 → 7034**, material count unchanged at 900, image CPU bytes unchanged at 49,829,628. Head gaze adds no meshes. Grass may produce different tile counts after clearing the complete band.
- Real browser guest completed a clockwise full lap: 6.294 radians, 168 samples, radius 1.002–1.061. Both browser clients were kept active for this final guest run. Host completed a counterclockwise lap: −6.314 radians, 168 samples, radius 1.000–1.062. Sampled speeds stayed within normal movement; the largest reporting gaps came from video encoding, without a spatial reset. Both full logs and short excerpts saved.
- Racer bots finished three ordered laps and displayed separate race and sideline results. A host lap attempt initially ran into normal round completion; it was restarted and completed. Results evidence retained.
- Focused fixed-scene test at the east bend `(44,0)` → racer `(36,0)` verifies a valid host throw and genuine guest action validation each award **50 points exactly once**, then next value 75. A live moving-target aiming recording missed three shots; it is labelled aiming, not a successful scoring demonstration. Bot scoring and race results were observed live.
- Gaze in both relay directions. Guest → host in Race: sent/received pitches **−1.153, 0, +1.153 radians** match exactly; actual down/level/up screenshots and 10.6-second motion clip checked. Turning and throwing composed with gaze. Host → guest in Backyard: same three values matched; a magpie event and too-close camera obscured that extra visual test, so those images are not presented in the review.
- Focused tests cover pitch quantisation and wire truncation, stale packets, a downed Race mirror, smoothing/reversal/neutral recovery and stable head/neck pivot at short/default/long face scales. Existing leg-length/throw/hit tests pass. Slider extremes and all poses were not individually filmed in every mode.
- Final grass cleanup was rebuilt in both targets, captured natively, and checked through a fresh browser load with cache disabled. Final browser movement film shows the cleared path. Earlier peer/lap recordings precede only this visual grass cleanup; their movement and gaze code is unchanged.
- Screenshot/video QA used actual captures and decoded sample frames. No concept images or Blender assets generated.

## Changed files

- `DEVELOPMENT-PLAN.md`
- `NEXT_UP.md`
- `checkpoints/LATEST.md`
- `rust/crates/bbq_app/src/face.rs`
- `rust/crates/bbq_app/src/game.rs`
- `rust/crates/bbq_app/src/grass.rs`
- `rust/crates/bbq_app/src/online.rs`
- `rust/crates/bbq_app/src/race.rs`
- `rust/crates/bbq_app/src/shot.rs`
- `rust/crates/bbq_app/src/yard_scene.rs`
- `rust/crates/bbq_core/src/looks_yard.rs`
- `rust/crates/bbq_core/src/net.rs`

Checkpoint files, logs, review.html and evidence are new. Source change boundaries and final hashes are in recovery-manifest.json. No runtime assets or public files changed.

## Local browser review

http://localhost:3043/after/?perf=1 — private staged build, protocol **28**. Host and guest should both use this build. The relay is running from /tmp/bbq-race-gaze-server.cjs with staged content in /tmp/bbq-race-gaze-preview. This address works on this Mac; public browser deployment is unchanged.

## Recovery and resumption

1. Keep the verified backup above. It contains the starting tracked source archive, pending.tar (empty at this clean starting point) and the previous release executable.
2. Run `python3 checkpoints/2026-11-08-race-gaze/restore.py` from the repository to check the guard without changing files. `--restore` restores only listed tracked source/planning files from source.tar, and refuses if later edits differ. It preserves evidence, other files, Git history, assets and public content. Rebuild afterward.
3. To compare without restoring anything, launch the saved `bbq_app-before` with `BEVY_ASSET_ROOT=/Users/marcus/Claude/australian-bbq/rust/crates/bbq_app`. The current assets match the backup; this milestone changed none.
4. If interrupted now, implementation and required verification are finished. Start by reading this document and Git status. Await user review before committing, pushing, publishing or beginning another feature.

## Limits and next decision

Browser peer tests used a local relay on this Mac. Internet latency and a second physical device still need ordinary friend playtesting. Pitch is cosmetic and bounded; gaze is not separate free look. Public clients keep the previous build until publication is approved. Review the complete Race route and head poses before any commit or public update.
