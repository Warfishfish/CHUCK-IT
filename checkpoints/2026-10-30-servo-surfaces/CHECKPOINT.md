# Smoko shelter surfaces — 30 October 2026

Ready for local review. No commit, push or browser publication.

## Scope and result
A focused first slice towards the Bush Servo reference: wider uneven dusty road shoulders and restrained wear patches, chipped sandstone edges, fuller dry planting and tiny gravel at the shelter border. Blender MCP rebuilt the existing surroundings; reuse of existing palette and textures kept nine material batches. Existing imported Poly Haven bark/timber and Sketchfab pumps stay unchanged; no new download or generation was necessary. This remains a modest improvement, not a match for the concept image. Broad bare terrain, tree silhouettes and distant mesas remain future work.

## Preserved starting state
HEAD `952efd2`; pending atmospheric haze in servo_finish.rs and its checkpoint preserved. Untracked concept art preserved. Backup `/Users/marcus/Claude/australian-bbq-before-surfaces-2026-10-30.tar.gz` contains 1,763 nonignored project files including uncommitted work. Every archived file was verified against the original SHA-256 before edits. Build caches and Git metadata excluded.

## Changes
- art/bush-servo/build_surroundings.py: named tuning constants, inward chipped bevels, restored exact stone bounds, gravel and planting; saves only its own Blender scene; removes unused UVs on export.
- art/bush-servo/smoko-surroundings.blend and runtime smoko_surroundings.glb: rebuilt asset.
- rust/tools/make_servo_ground.py and assets/textures/servo_road.png: broader nonuniform dust and aggregate variation; resolution unchanged.
- art/bush-servo/SURROUNDINGS.md, this checkpoint and LATEST.md.

## Verification
Actual native release loaded the revised assets. Two identical cameras, 300-frame warmup and 600 measured intervals each at 1280×800. Overview mean 7.499 → 7.551 ms; p95 8.782 → 9.118 ms. Close mean 7.657 → 7.690 ms; p95 9.422 → 9.774 ms. Differences small in these samples, not a browser performance claim. Same decoded image memory (87,407,208 bytes), scene entities and materials.

Surroundings: 4,978 → 6,904 triangles, 424,856 → 553,696 bytes, nine batches unchanged. Road PNG +82,363 bytes; same 1024px resolution. All 22 stone bounds match existing shared collision layout. No collision, scoring, movement or Rust code modified by this stage. Builder syntax and git whitespace checks pass. Native Smoko motion fixture ends with +100 score and zero carries; Backyard smoke capture passes. Actual 75-frame moving-camera clip included. Fine planting still needs browser-device shimmer review. Other modes were not individually retested in this asset-only pass.

No new native/browser compilation: this pass changes loaded assets only, and used the existing release binary from the preceding haze pass. Browser bundle remains unchanged pending review.

## Recovery
Restore the five changed art/builder/runtime files listed in reviewed-sha256.json from the backup into a separate staging directory, inspect them, then copy only those files back. Restore SURROUNDINGS.md from backup if reverting its documentation. Never extract the whole archive over later work. This preserves the already pending haze adjustment. Backup can also recreate the complete nonignored starting source state in a new directory. reviewed-sha256.json guards against overwriting later changes.

## Test
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Maps / Game modes → Smoko Yard. Compare the shelter stones, planting and adjacent road; carry a knocked-down opponent through each approach.

## Publication approval — 30 October 2026
User approved commit and push, including the pending atmospheric adjustment shown in these captures. Reviewed hashes reverified. Browser bundle and live page remain unchanged.
