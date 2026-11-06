# Arena entrance and supply crates — ready for review

7 November 2026. Starting commit: 1bbd6c3e9e3831c676b4259b30a26956014c3a8d. No commit, push or publication for this task.

## Changes

- Refined five existing south-east shelter sandstone blocks, preserving their collision extents.
- Added 30 embedded gravel pieces and a subtle feathered dust wash at the south entrance. Scoring pad and carrying approach remain clear.
- Built three closed timber supply crates in Blender, using existing wood/metal materials. Positions (x,z): (-15,3), (15,3), (-10,12.5); each 1.1 × 0.8 × 0.65 m. These are static low obstacles, not new throwable pickups.
- Added matching crate collision to shared PROP_SOLIDS. Protocol 24 prevents older clients with different obstacles joining. Native and browser players must all update together when publishing.
- Existing props, outside landscape, scoring and movement preserved. Original 59,540 prop triangles unchanged; 5,488 surroundings triangles outside the local edit region unchanged within float32 export tolerance.

## Verification

446 core tests and 246 app tests passed; 1 ignored. Native release and browser web builds passed with existing warnings. Shared road/spawn/scoring/carrying-lane clearance checks passed. Actual Smoko motion review delivered +100 and ended with no carried player. Backyard smoke capture passed. git diff --check passed.

Actual identical native cameras, 1280×800, 300 warmup frames and 600 measured wall-clock intervals:

| View | Before mean / p95 | After mean / p95 |
|---|---|---|
| Close | 8.594 / 10.948 ms | 8.547 / 10.857 ms |
| Gameplay | 8.506 / 11.055 ms | 8.630 / 10.862 ms |

Initial gameplay sample was 10.115 ms during surrounding packaging activity; retained as after-gameplay-initial. Repeat without that activity returned to 8.630 ms. These are local samples, not GPU timings or guarantees on other devices.

Browser stationary solo smoke samples: before ~75 FPS /13.3 ms mean /14.9 ms p95; after ~75 FPS /13.4 ms mean /14.7 ms p95. Both 1280×720, no bots, but spawn cameras differ. Mouse-capture overlay remained visible; this is a rendering/load check, not a full movement or multiplayer playtest. Matched native views are the stronger performance comparison. Actual browser screenshots retained.

Asset cost: props 59,540→68,612 triangles, 3,861,700→4,750,752 bytes; same 14 batches and 13 images. Surroundings 6,904→7,984 triangles, 697,512→820,844 bytes; 9→10 batches, same 4 images. Total new geometry 10,152 triangles, no new textures; full-scene image CPU bytes unchanged at 95,446,404. One added blended dust material. Native close/gameplay and 75-frame moving capture visually inspected for clipping, dust edges and readability. This remains a modest local detail pass, not a match to the concept's overall finish.

## Files

- art/bush-servo/refine_entrance.py
- art/bush-servo/arena-crates.blend
- art/bush-servo/arena-entrance.blend
- art/bush-servo/arena-entrance-bounds.json
- rust/crates/bbq_app/assets/models/smoko_props.glb
- rust/crates/bbq_app/assets/models/smoko_surroundings.glb
- rust/crates/bbq_core/src/bush_servo.rs
- rust/crates/bbq_core/src/net.rs
- checkpoints/LATEST.md and this checkpoint directory

Blender pipeline: run refine_entrance.py after refine_shelter_props.py. Saved author scenes are separate; originals untouched. Use finish_arena_depth.py's fresh-byte finish function as the script does, not its old archive-based CLI.

## Recovery and testing

Verified pre-edit archive: /Users/marcus/Claude/australian-bbq-before-arena-entrance-2026-11-07.tar.gz (2047 files byte-verified). Manifest: starting-state.json. Extract into a NEW empty directory to recover; do not extract over ongoing work. To revert selectively, compare the four modified existing runtime/source files listed above against that extraction, restoring only after checking for subsequent edits. Keep code and assets together because the crates change collision. Rebuild native/browser after restoring. New author files can be retained as unused references. No changes were made to concept art/.

Local launch:

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Solo game → Bush Servo → Play Smoko.

Private browser comparison was staged at /tmp/bbq-entrance-review-20261107, served on localhost:3017. Public browser bundle is unchanged (protocol23); this work is protocol24. Do not publish until reviewed. Next: user review of entrance treatment and crate placement before expansion or publication.
