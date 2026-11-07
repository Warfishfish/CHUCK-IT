# Arena tyre and timber — ready for review

7 November 2026. Starting commit a9b001bab68d2ffaa1608390ebe8613063a2351d. No commit, push or publication for this change.

## Implemented

One Blender-built prop group beside the eastern arena gum: two worn tyres at (13,-0.3), footprint1.1×1.1m and height0.72m, plus a three-plank timber barrier at (14,1.2), footprint2×0.28m and height0.8m. Existing timber, rubber and metal materials reused. Tyres are static low cover, not throwable items. Shared collision appended to PROP_SOLIDS; protocol25 rejects older layouts. Update host and guests together when publishing.

All 713 original Blender prop objects preserved by geometry/material/transform signatures; all68,612 original runtime triangles preserved within float32 export tolerance. Outside scenery and other models unchanged. Carrying routes, spawns, road and scoring circle checks pass. No scoring, controls or movement changes.

## Verification

446 core +246 app tests passed;1 ignored. Native release and browser web builds passed with existing warnings. Actual Backyard smoke and Smoko carry/delivery motion check passed (+100, no player left carried). git diff --check passed.

Matched native cameras,1280×800,300warmup frames,600wall-clock samples. Mean/p95 milliseconds:

| View | Before | After |
|---|---|---|
| close | 8.382 / 10.771 | 8.732 / 11.007 |
| gameplay | 8.690 / 11.085 | 8.690 / 10.794 |

Browser stationary results are in browser-metrics.json with actual screenshots. Same size and zero bots, but random spawn views differ; mouse-capture overlay remains visible. These are rendering/load checks, not a full multiplayer or browser movement playtest. Native same-camera measurements provide the controlled comparison. No claim of identical speed on every device.

Runtime props68,612→71,504triangles (+2,892),4,750,752→4,889,840bytes (+139,088). Same14material batches,13images; no new materials/textures. Local edit only, not an exact reproduction of the concept. Gameplay screenshots and75-frame motion capture in review.html.

## Changed files

- art/bush-servo/build_tyre_timber.py
- art/bush-servo/arena-tyre-timber.blend
- art/bush-servo/tyre-timber-bounds.json
- rust/crates/bbq_app/assets/models/smoko_props.glb
- rust/crates/bbq_core/src/bush_servo.rs
- rust/crates/bbq_core/src/net.rs
- checkpoints/LATEST.md and this directory

Run build_tyre_timber.py after refine_entrance.py. It loads arena-crates.blend, preserves existing props, saves a separate author scene, exports and applies the existing fresh-byte depth finish. No paid resources; Blender library integrations report off, so existing materials reused.

## Recovery

Verified pre-edit archive: /Users/marcus/Claude/australian-bbq-before-tyre-timber-20261107.tar.gz. All2083file hashes verified; starting-state.json lists them. Extract to a NEW empty directory, never over ongoing work. For selective rollback compare the three changed existing source/runtime files above with the extraction before restoring; preserve subsequent edits. Restore both collision code and GLB together, then rebuild. New Blender/script files may remain unused. Unrelated concept art preserved.

Public browser remains protocol24 and unchanged. Private staged protocol25 build used localhost:3018. Review before committing, pushing or publishing.

Local test:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Solo game → Bush Servo → Play Smoko. Find the group beside the eastern gum.

The first native executable failed its launch check. After clearing a regenerable debug build cache and rebuilding, the ARM64 executable launched and completed both native captures and the delivery check. The browser build used the rust workspace configuration. Final native comparisons used the same executable with separate before/after asset roots. The browser averaged 13.3 ms in both stationary samples.
