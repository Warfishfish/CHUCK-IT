# Smoko concept vividness · 8 November 2026

This pass strengthens the playable Smoko palette and light. It builds on the existing restrained vivid-pop rig while preserving the other pending graphics and inventory work.

## Recovery

Backup /Users/marcus/Claude/australian-bbq-before-concept-vivid-20261108 includes source.tar for commit b6d4b1f and working.tar for the pending state. All 187 pending files were compared byte-for-byte to the archive, using the saved servo_finish.rs for the one file edited by this task. An original native executable is retained. Extract both archives into a fresh directory to reconstruct the start. For a selective revert, compare subsequent changes then restore only servo_finish.rs from working.tar or the same-named saved copy. That preserves the earlier global lighting and Smoko colour work.

## Implementation

Only rust/crates/bbq_app/src/servo_finish.rs is edited by this pass. Polished Pop Smoko and its menu overview use a cooler, lower fill at 0.70, a warm sun at 1.12, saturation 1.12 and contrast 1.075. Warmth is kept restrained at 0.010. The same light direction keeps the existing filtered canopy shade aligned. Geometry, materials, texture resolutions and gameplay are as found.

A capture-only --servo-baseline option requires both --shot and --review. It selects the exact starting Smoko rig in the same executable, allowing camera-, asset- and code-matched comparisons. Normal play uses the new rig.

## Verification

The first captures exposed a mismatch between the earlier executable and some pending scenery changes. The final comparison uses the capture-only rig switch in one fresh release build. Images and timing reports in evidence belong to the final matched run. A paused picnic capture was discarded and replaced with a valid gameplay capture.

Native and WebGPU release builds passed. The final native picnic pair measured 8.758 → 8.590 ms mean and 10.993 → 10.769 ms p95. Shelter measured 8.702 → 8.393 ms mean and 10.788 → 10.648 ms p95. All final runs report 2,774 meshes, 925 materials, 6,483 mesh entities and 97,194,032 decoded image bytes. These small single-run differences establish no performance gain. The colour pass adds no geometry, material or render pass. Actual Chrome gameplay and a short walk through the arena were captured at balanced browser quality, solo with zero bots. This is a single-Mac visual check. No gameplay code or test fixtures were edited. Other pending code is included in the current build.

The concept still has finer material detail and different tree and prop silhouettes. This pass addresses colour and light for review. Further asset work should be judged against the same playable views.

## Test

Native launch from /Users/marcus/Claude/australian-bbq/rust uses ~/.cargo/bin/cargo run -p bbq_app --release -- --menu, then Solo game → Bush Servo → Play Smoko. Private browser preview http://localhost:3042/?perf=1 while its server runs. Public browser publication follows review.
