# Shelter-side small props — review pending

6 November 2026. One green esky (5.6,15.9) and adjacent bin (7.2,14.8). This first group does not extend to mugs/thermos/crates elsewhere. No commit or publication.

## Changes

- Reused the completed picnic esky geometry for the shelter esky, retaining its original green colour: lid channels, hinges, latch insets, moulded ribs and grip details.
- Existing bin retains its body and lid; adds a recessed lid seam, protective lower rim, framed front opening, lower flap, rear hinges, rounded rivets and pressed side seams.
- Existing wear textures and materials reused. All new geometry inside existing collider bounds. Scoring area, carrying routes, gameplay, shelter structure, picnic settings and other scenery unchanged.
- `art/bush-servo/refine_shelter_props.py`: new Blender refinement/export script.
- `art/bush-servo/shelter-props-detail.blend`: editable authoring scene.
- `art/bush-servo/shelter-prop-bounds.json`: footprint checks.
- `rust/crates/bbq_app/assets/models/smoko_props.glb`: final runtime asset.
- This checkpoint and LATEST.md: evidence and recovery.

Authoring pipeline: build_props → refine_picnic → extend_picnic → refine_shelter_props. Each stage reads its predecessor's saved scene and preserves other objects. The latest script compacts the export and applies arena depth tint to fresh bytes. Never run the old archive-based finish_arena_depth CLI over new assets.

## Validation

590 previous authoring objects unchanged. Replaced only 8 original esky pieces; 39 detailed pieces in the edited group. Export confirms 54,632 outside triangles unchanged within 0.11mm (float32 re-export rounding due to changed joined origin); authoring coordinates exactly preserved. Existing collision/gameplay code untouched. Syntax/whitespace checks pass.

Native release asset load, Backyard smoke and Smoko carry/delivery fixture pass (score100, carries0). 75-frame moving capture validated. Asset-only change: reused native/browser release code; no redundant builds or code tests.

Matched 1280x800 native captures, 300 warmup + 600 uncapped wall-clock intervals:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Close | 8.485 ms | 8.365 ms | 11.094 ms | 10.818 ms |
| Gameplay | 8.599 ms | 8.588 ms | 11.189 ms | 10.809 ms |

No meaningful regression observed; small decreases are measurement variation, not optimisation claims.

Asset: 56,144 → 59,540 triangles (+3,396); 3,582,756 → 3,861,700 bytes (+278,944). Still 14 material batches/13 embedded images. Full-scene image CPU memory unchanged at95,446,404 bytes. No paid resources or new textures.

## Recovery

Starting commit9d6c6941a8285f279c5c288a9cbf3e8db7978de9 with prior graphics work uncommitted.
Verified backup: `/Users/marcus/Claude/australian-bbq-before-shelter-props-2026-11-06.tar.gz` — 2,018 files individually hash-verified. starting-state.json lists every hash. Restore into a new directory, never overwrite later work.

Selective rollback: verify current runtime asset against reviewed-sha256.json; restore only smoko_props.glb from this archive or before-props.glb in this checkpoint. Preserve earlier picnic/shelter files. New refinement script/blend/bounds may remain archived. Never reset unrelated changes.

## Test

```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Solo game → Bush Servo → Play Smoko. Inspect the green esky and bin at the shelter edge. Review before extending to another group, committing or publishing.

Browser smoke: 1280x720 Balanced, no bots, stationary behind capture overlay. Before ~75FPS /13.3ms mean /15.5ms p95; after ~74FPS /13.5ms mean /15.1ms p95. Same apparent spawn view, independently started rounds. Not a multiplayer or active movement benchmark. Public release files unchanged.
