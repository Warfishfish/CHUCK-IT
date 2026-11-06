# Both picnic settings — review pending

6 November 2026. User authorised next step: extend the completed east picnic-table and esky treatment to the west setting. No commit, push or publication authorised.

## Implementation

`art/bush-servo/extend_picnic.py` reads `picnic-detail.blend`, replaces only the old west table/bench and esky pieces with translated copies of the refined east pieces. Copies share meshes and materials in the authoring scene; runtime is batched on export. East setting, mugs/crates, other props, shelter, outside scenery and game rules preserved.

Changes: aligned weathered timber and endgrain; split bench seats; stronger legs, underside bearers, stretcher and knee braces; esky hinge/latch/grip/lid details. All inside original collision footprints. No change to routes or shared collision code.

New files: `art/bush-servo/extend_picnic.py`, `picnic-pair.blend`, `picnic-pair-bounds.json`; this checkpoint/evidence. Existing file changed: `rust/crates/bbq_app/assets/models/smoko_props.glb`, plus checkpoint index. All prior pending shelter and east-setting work retained.

Authoring pipeline: `build_props.py` → `refine_picnic.py` → `extend_picnic.py`. The last step exports the final props GLB, compacts unused UVs and applies the current arena vertex tint to the fresh export. Do not run the archive-based finish_arena_depth CLI, which would restore old assets.

## Verification

- Blender: 550 objects untouched; 27 west objects replaced with 48 shared detail objects. New detail inside original table/bench/esky bounds.
- Export: 43,960 triangles outside the west setting unchanged at 0.1mm precision, including the entire east setting.
- Syntax and whitespace checks pass. Only props GLB and checkpoint index differ from starting file hashes.
- Native release loads the final asset. Backyard launch/capture passes; actual Smoko carry-and-deliver fixture ends at score 100, carries 0. 75-frame moving-camera capture checked nonblack.
- No Rust source changes, so reused the existing native and browser release code. No redundant rebuild. Private browser preview uses identical release JS/WASM and only differs in the props asset; public files untouched.

Matched native camera, 1280x800, 300 warmup frames and 600 uncapped intervals:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Close | 8.401 ms | 8.396 ms | 10.633 ms | 10.624 ms |
| Gameplay | 8.901 ms | 8.757 ms | 11.471 ms | 11.124 ms |

No meaningful regression in these samples; small decreases are measurement variation, not an optimisation claim. Browser measurements recorded separately; stationary behind mouse-capture overlay, not movement/multiplayer validation. Different spawn views prevent an exact browser A/B.

Prop kit: 53,876 → 56,144 triangles; 3,406,692 → 3,582,756 bytes (+176,064). Still 14 batches and 13 images. Full-scene reported texture CPU memory stays 95,446,404 bytes. No new textures or materials.

## Recovery

Starting commit: 9d6c6941a8285f279c5c288a9cbf3e8db7978de9, with prior shelter/east-setting work uncommitted.
Verified archive: `/Users/marcus/Claude/australian-bbq-before-picnic-pair-2026-11-06.tar.gz` — 1,992 files individually hash-verified. Starting-state.json records every hash. Restore into a new directory, never over later work.

Selective rollback: first verify the current props asset matches reviewed-sha256.json, then restore only `rust/crates/bbq_app/assets/models/smoko_props.glb` from this archive or before-props.glb here. Keep prior shelter/east files. New pair authoring files can remain archived. Never git-reset unrelated work.

## Local test

```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Solo game → Bush Servo → Play Smoko. Inspect both picnic tables and nearby blue eskies. Review before any commit/publication or further expansion.

Browser smoke samples (1280x720, Balanced): both ~75 FPS and 13.3 ms mean; p95 15.0 → 14.9 ms. Zero >33ms frames/backlog in displayed rolling samples. Cameras differ; no claim about multiplayer or moving-camera browser performance.
