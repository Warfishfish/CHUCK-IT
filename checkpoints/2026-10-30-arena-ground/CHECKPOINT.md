# Picnic ground variation — 30 October 2026

First reviewable ground-only slice inside Smoko: east picnic area and the footpath towards the south shelter approach. No new trees, shrubs, props or exterior changes. Gameplay, ground elevation, collision and protocol unchanged.

## Implementation
The existing Smoko dirt surface is tessellated only around a 14 × 12m local paint region. Neutral coarse strips cover the remainder of the original 260m plane. Existing UVs and repeating dirt texture are preserved. Vertex colours add softly compacted earth under the table and along an irregular path, a restrained dusty fringe, and quiet gravel-like tonal flecks. These are colour details, not raised or colliding gravel objects. No transparency, layered planes or z-fighting; no new texture, material, light or draw call. Blender was not needed for this renderer-only ground paint.

Art tuning values are grouped in shapes.rs: SERVO_GROUND_STEP, SERVO_PATH_WIDTH, SERVO_WEAR_STRENGTH, SERVO_DUST_STRENGTH, SERVO_GRIT_STRENGTH and SERVO_FOOTPATH. Paint is neutral outside x=-2..12, z=-5..7, protecting the scoring pad and exterior. Models.rs selects this mesh only for ServoDirt.

## Recovery
Starting commit e47700a. Tracked worktree clean; unrelated concept art preserved. Verified archive `/Users/marcus/Claude/australian-bbq-before-arena-ground-2026-10-30.tar.gz`: all 1,828 nonignored files byte-verified before editing. Git metadata and ignored build caches excluded. Restore into a fresh directory for complete source recovery. To selectively undo, compare current hashes against reviewed-sha256.json, then restore only shapes.rs and models.rs from the archive or starting commit and rebuild. Do not overwrite later edits. No public browser files have been changed.

## Changed files
- rust/crates/bbq_app/src/shapes.rs: local ground mesh/paint and bounded-scope/flatness/UV test.
- rust/crates/bbq_app/src/models.rs: Smoko dirt mesh selection.
- checkpoints/LATEST.md and this checkpoint's evidence/review files.

## Testing
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Bush Servo / Smoko. Inspect the east picnic table and the approach to the shelter. No commit, push or publication until review.

## Verification and cost
245 app tests pass (one existing test ignored), including a new ground-scope, flatness, finite-colour and UV preservation test. Native release build passes with existing warnings. Actual Backyard smoke capture and Smoko motion fixture pass; delivery awards +100 with zero remaining carries. No core, collision, network, assets or public browser files changed. Other modes were not individually captured; app tests include race and Heist simulations.

Matched native 1280×800, AutoNoVsync, 300 warmup frames then 600 wall-clock intervals. Close view mean 8.594 → 8.650 ms, p95 12.628 → 13.593 ms. Overview mean 8.690 → 8.487 ms, p95 13.367 → 13.383 ms. Average differences small relative to timing variation; close-view p95 rose 0.965 ms in this sample. No browser/device performance claim. Added mesh detail: 4,599 vertices and 8,928 triangles instead of four vertices and two triangles, approximately 320 KiB additional vertex/index data. Same material, texture memory and draw count. The current cost is confined to one ground mesh; do not extend a fine grid across the whole world.

Visual limitations: this is restrained colour wear using the existing dirt texture, not new high-resolution gravel or physically modelled stones. Worn colour is partly under tree shade. Review the matching close view and motion clip before increasing strength or expanding to other picnic areas.

## Final check
WebAssembly release-profile compilation passed with existing warnings. Reviewed source hashes and whitespace checks pass. The public browser bundle remains unchanged pending review.
