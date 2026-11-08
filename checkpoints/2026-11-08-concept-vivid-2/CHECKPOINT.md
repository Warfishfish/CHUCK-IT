# Smoko concept vividness: late-afternoon light · 8 November 2026

Status: source and recovery checkpoint committed and pushed. The public browser release was not changed.

## Starting state and recovery

- Starting commit: `e36c03d1` (Race route/head gaze browser publication). Branch `main` was clean before this task.
- Baseline source saved at `servo_finish.before.rs`; its SHA-256 is recorded in `starting-state.json`.
- The starting commit is a complete Git recovery point. To restore only this pass, first inspect later edits, then copy `servo_finish.before.rs` back to `rust/crates/bbq_app/src/servo_finish.rs`.
- Public browser files were not changed. Existing protocol-28 online link remains separate from this unreviewed graphics pass.
- Local launch: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`.

## Scope and changes

This is a lighting/colour adjustment scoped to Smoko in Polished Pop and its existing map overview. No gameplay, collision, textures, models, geometry or other maps changed.

`rust/crates/bbq_app/src/servo_finish.rs` now uses a slightly lower, amber sun, more warm ground bounce and hemisphere fill, modestly richer colour, less contrast and less vignette. The intent is to get closer to the warm afternoon concept while preserving detail in shade. The existing display shader/light count and all scene assets are unchanged.

## Visual review

`review.html` shows the picnic and shelter views plus an active browser gameplay capture. The before images are actual captures from the previous Smoko lighting pass at the same camera settings. The after captures were taken from the current native release at those same camera positions. `legacy-baseline-*.png` are extra diagnostic captures using the older neutral rig and are not the current-before images shown in the review.

Concept: `../concept art/exec-6ea4e9b3-70f7-4745-b24d-773f23cbe0ae.png`.

## Verification and performance

- Native release build passed with existing warnings; captured both updated views and 600 wall-clock intervals per capture. Browser WebGPU release build passed, and the private preview loaded into Chrome. The solo Bush Servo overview and active gameplay captures are in evidence/.
- Captures report the same 2,774 meshes, 925 materials, 6,483 mesh entities and 97,194,032 decoded image bytes as the prior pass. No new textures, geometry or render passes were added.
- The native timing samples fluctuated while other rendering apps and a game process were active. Baseline-light runs were 12.954 ms mean (picnic) and 15.192 ms (shelter); updated runs were 12.760 ms and 12.418 ms respectively. These are not a clean performance comparison. Counts remained at 2,774 meshes, 925 materials, 6,483 mesh entities and 97,194,032 image bytes; the light-only changes add no scene objects or passes. Avoid drawing a precise FPS improvement/regression claim from the native samples. Browser sample details are saved in evidence/browser-performance.json.
- No tests were added or run; this pass changes only presentation constants. Private preview: `http://localhost:3044/rust/` (this Mac only; served by `/tmp/bbq-vivid-server.cjs`). It contains the built WebAssembly, matching assets and no multiplayer relay. The live public link was not changed.

## Changed files

- `rust/crates/bbq_app/src/servo_finish.rs`
- `checkpoints/LATEST.md`, `NEXT_UP.md` (progress/status records)
- This checkpoint and its actual review captures.

## Review decision

The source is committed and pushed. The native and private local preview include this lighting pass. The public browser build remains at the previous release until a separate browser update is requested.
