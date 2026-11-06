# East picnic table and esky — ready for review

Started 5 November; verification completed 6 November 2026. One east picnic table (8.5,-2.5), its benches and nearby blue esky (11,-1.5). No extension to other settings. Pending shelter work preserved. No commit, push or publication.

## Changes

- Aligned weathered timber grain, exposed endgrain, split bench planks and stronger table legs, bearers, stretcher and knee braces.
- Esky rear hinges, latch insets/catches, recessed grip shadows/bridges, moulded corner ribs and subtle lid channels. Existing worn blue/cream finish retained.
- All detail stays within existing table/bench/esky footprints. No new obstacles or changes to movement, carrying routes, collisions, rules or exterior scenery.
- `art/bush-servo/refine_picnic.py`: reproducible Blender refinement, reads untouched `smoko-props.blend` and current shelter timber. Run after the base props builder when regenerating. Never run the archive-based finish_arena_depth CLI; this script calls its fresh-export function correctly.
- `art/bush-servo/picnic-detail.blend`: separate editable authoring scene.
- `art/bush-servo/picnic-detail-bounds.json`: footprint and unchanged-object checks.
- `rust/crates/bbq_app/assets/models/smoko_props.glb`: updated game asset.
- This checkpoint and `checkpoints/LATEST.md`: review/evidence/recovery.

## Verification

Blender MCP used directly. 529 non-target authoring objects unchanged. Export comparison confirms 41,692 triangles outside the east setting unchanged at 0.1mm precision. Original shared collision code and all other starting files preserved (except checkpoint index). Python syntax and git diff whitespace checks pass.

Native release loads the asset; Backyard smoke and actual Smoko carry/delivery fixture pass (`final_score=100 carries=0`). Motion clip is 75 actual frames; screenshot outputs checked nonblack. No new Rust code, so existing native and browser release binaries reused rather than redundant compilation. Private browser page loads current assets; public bundle untouched.

Identical native cameras, 1280x800, 300 warmup + 600 uncapped wall-clock frame intervals:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Close | 8.332 ms | 8.567 ms | 10.418 ms | 10.807 ms |
| Gameplay | 8.502 ms | 8.552 ms | 11.345 ms | 10.842 ms |

Browser Balanced, 666x864, solo/no bots, stationary behind pointer-capture overlay: before 74–75 FPS, mean 13.3/13.5 ms, p95 14.5/14.4; after 74–75 FPS, mean 13.5/13.3 ms, p95 14.6/14.2. Camera/spawn differs between samples: this is a browser loading/performance smoke comparison, NOT matched-camera proof or multiplayer/movement testing. Native comparison is exactly matched. No broad browser performance claim.

Prop kit: 51,608 → 53,876 triangles (+2,268), 10 → 14 batches, 8 → 13 embedded images, 2,958,016 → 3,406,692 bytes (+448,676). Full native image CPU storage +2,446,724 bytes. Extra wood materials reuse shelter artwork but load separately in glTF. Do not expand globally without considering shared materials.

## Recovery

Starting commit: `9d6c6941a8285f279c5c288a9cbf3e8db7978de9`.
Verified full pre-edit archive: `/Users/marcus/Claude/australian-bbq-before-picnic-detail-2026-11-05.tar.gz`, 1,964 files individually byte-verified. Includes pending shelter work and concept art. See starting-state.json for hashes/status.

Restore the archive into a NEW directory to avoid overwriting later changes. To undo only this pass, after confirming current asset matches reviewed-sha256.json, restore `rust/crates/bbq_app/assets/models/smoko_props.glb` from the archive (also copied as before-props.glb here). Keep pending shelter files. New picnic authoring script/blend/bounds may be archived separately. Do not reset Git or overwrite unrelated work. Both graphics passes await review/publication.

## Test and next step

```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Solo game → Bush Servo → Play Smoko. Inspect the table and blue esky near the eastern gum. Private browser preview while server remains running: http://localhost:3014/after/?perf=1 (only this Mac, not public multiplayer).

Next: user reviews this setting; if approved, extend the finish to the second picnic setting, keeping materials shared where possible. No next-stage implementation or commit authorised yet.
