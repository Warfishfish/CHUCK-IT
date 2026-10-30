# Arena props and weathering — completed 31 October 2026

Work began 30 October; checkpoint folder retains that date. Ready for local review. No commit, push or browser publication.

## Scope and changes
User requested step 3 (prop detail) and more small props from the Bush Servo concept. Blender MCP refined the existing kit and added ten recognisable objects across two picnic areas: two wooden supply crates, two galvanised buckets, two thermoses, two enamel plates and two extra mugs. Added small tabletop fasteners. Four existing eskies, five bins and six tyre stacks gain restrained scuffs/dust using shared 256px colour maps. Timber reuses the existing shelter materials with face-projected UVs. No new downloads, paid generators, animations or lights. The existing roadside signs share some kit materials and consequently receive the same finish; their placement/geometry was not expanded. New objects are all inside the arena.

Five baked colour maps add age without new runtime procedural shaders. WEAR_SIZE, DUST_AMOUNT and SCUFF_AMOUNT are tunable near the top of build_props.py. Authoring .blend keeps separately named editable objects; runtime export copies are joined and unused UV streams removed.

## Gameplay preservation
All 21 existing recorded solid bounds still match their shared geometry, including the existing shelter esky and 20 prop colliders. No new collider or protocol changes. Crates/buckets are entirely within existing table solids; prop-decoration-bounds.json records verified bounds. Thermoses/plates/mugs are small non-blocking tabletop decoration, not new throwable pickups. Ground pass sources retain their reviewed hashes; unrelated concept art preserved.

## Verification
Builder syntax and git whitespace checks pass. validate_props.py passes; its historical new_colliders label is corrected in this checkpoint's evidence to existing_shared_prop_colliders=20 and new_colliders_added=0. Real native release loads the updated GLB. Backyard smoke capture passes; actual Smoko motion fixture awards +100 with zero remaining carries. All 75 motion frames validated and encoded as WebP. No Rust code changed by this prop pass, so the already-built native/browser binaries from the preceding ground pass were reused rather than rebuilt. No full browser playtest or multi-device performance test performed for the new assets; public bundle remains unchanged.

Same 1280×800 camera, 300 warmup and 600 measured intervals per view, AutoNoVsync:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Overview | 7.921 ms | 7.821 ms | 11.358 ms | 12.308 ms |
| Picnic close-up | 7.844 ms | 7.757 ms | 11.600 ms | 12.136 ms |

Means remain close to baseline; p95 increased 0.536–0.950 ms in these samples. These are wall-clock intervals, not GPU timestamps or browser results. Runtime kit: 42,272 → 51,608 triangles (+9,336), 2,191,376 → 2,701,828 bytes (+510,452); ten material batches unchanged. Five new 256px maps add approximately 1.75 MB decoded texture/mip memory. Small clutter is most visible in close gameplay views; this is not a claim to match the full concept's finish.

## Recovery
Starting HEAD e47700a with pending arena-ground work. Verified pre-edit archive `/Users/marcus/Claude/australian-bbq-before-arena-props-2026-10-30.tar.gz`: all 1,845 nonignored files byte-verified, including pending source and concept art. Build caches/Git metadata excluded. Restore to a new directory for complete source recovery. For a selective rollback, compare current files with reviewed-sha256.json, extract original props source/.blend/GLB/PROPS.md into a staging directory, and restore only those files. Remove newly added prop-textures and prop-decoration-bounds.json only if they still match this reviewed state. Keep pending ground sources and later work intact.

## Changed files
- art/bush-servo/build_props.py
- art/bush-servo/smoko-props.blend
- art/bush-servo/prop-textures/weather-3010.png through weather-3014.png
- art/bush-servo/prop-decoration-bounds.json
- art/bush-servo/PROPS.md
- rust/crates/bbq_app/assets/models/smoko_props.glb
- checkpoints/LATEST.md and this checkpoint

## Local testing
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Bush Servo / Smoko and visit either picnic table. Review before committing or publishing. No additional stage started.
