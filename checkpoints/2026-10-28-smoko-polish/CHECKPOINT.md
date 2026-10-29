# Smoko performance and gameplay polish — first pass

Smoko / Bush Servo is the primary development map. Validate improvements here before intentionally transferring suitable improvements to other maps. Keep each map's terrain, objective and balance distinct.

Starting commit: ce5d5da. Tracked source tree was clean; unrelated untracked `concept art/` was preserved. Verified full tracked recovery archive: `/Users/marcus/Claude/australian-bbq-before-smoko-polish.tar`. Original native renderer saved at `/tmp/bbq-before-smoko-polish`.

## First pass

- Stop unnecessary hidden Backyard pool material animation and perched-head animation while Smoko is active; cloud motion remains.
- Avoid redundant repeated visibility writes for shared yard and supply roots.
- In Smoko, provide contextual pickup instructions, shelter distance, host-side carry time remaining, and a guest carrying hint that does not invent a timer.
- Reuse the existing hint text entity during updates and hide the unrelated Backyard tutorial prompt in Smoko.
- No assets, collisions, movement, scoring, objective, network protocol or map layout changes.

## Verification

- 244 app tests passed; one existing test ignored. Targeted Smoko tests cover renderer map switching, pickup eligibility, carry countdown and guest state. Existing delivery tests cover host attribution, +100 exactly once, protection, respawns, bots, throws, disconnects and round cleanup.
- Native release and browser WebAssembly builds passed (existing warnings only).
- Matched actual native gameplay and overview screenshots: `evidence/comparison.jpg`. Actual delivery sequence: `evidence/smoko-delivery.webp`; automated motion log reports `final_score=100 carries=0`.
- Same native Mac, 1280x800, 600 frame intervals: gameplay mean 7.588 ms before and 7.514 ms final after; median 7.425→7.365 ms; p95 9.017→9.004 ms; frames over 16.67 ms 3→3. Repeated runs vary by more than this difference, so no FPS gain is claimed. Meshes/materials/entities and CPU image bytes unchanged. These timings do not measure GPU time, browser rendering or multiplayer peer frame pacing.
- One automated Bevy screenshot run logged a duplicate-render-target warning while saving multiple sequence images; image timestamps and outputs were verified, score result was logged, and the game exited successfully.

## Recovery and next step

Starting commit remains recoverable with the archive above. To undo only this pass, preserve any later work and restore `rust/crates/bbq_app/src/hud.rs`, `smoko_mode.rs`, and `yard_scene.rs` from commit ce5d5da. No reset or broad restore is needed.

Next, profile browser frame pacing on host and guest devices, separately from the native baseline, and playtest Smoko pickup/carry routes, interruption and bot delivery. Only after review should proven shared UI/performance improvements move to other maps.

Local run: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`
