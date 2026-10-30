# Two gums inside the arena — 30 October 2026

## Scope
User resumed the arena-focused graphics work. Added one second gum to the west side of the inner road loop, preserving the first tree and all exterior scenery. This is a local review stage, with no commit, push or live browser publication.

## Changes
Second gum at (-11.5, 2), height 7.7m, alongside the existing 8.4m specimen at (12, 1.5). Independent seeded branching creates a different canopy rather than a mirrored copy. Nine compact grass/saltbush clusters at its base. Existing Poly Haven bark reused in Blender MCP; no new textures, downloaded assets or paid generation. Shared 0.7 × 0.7m lower trunk collision to 4m on both trees. High branches/foliage are visual only. Both stay inside the arena, clear of road and delivery approaches. Protocol 23 prevents peers using older collision layouts from joining.

## Verification
446 core tests and 244 app tests pass (one existing app test ignored). Native release and WebAssembly release-profile compilation pass with existing warnings. Actual native Backyard smoke capture and Smoko motion fixture pass; delivery awards +100 and leaves zero carries. No live two-device playtest was performed. Current public browser bundle was not changed.

The asset validator compares previous vertex positions with 0.02mm tolerance; original scenery and first tree remain intact. All new geometry is in the western arena slice. Both trunk definitions match the shared collider table. Landscape asset grows 8,733,372 → 9,036,288 bytes (+302,916), 131,380 → 136,744 triangles (+5,364), with 11 material batches unchanged and no new texture memory. Cumulative cost of both arena trees relative to the pre-tree landscape: +10,728 triangles and +604,716 bytes.

## Recovery
Starting commit remains 4b188db; the first tree was already uncommitted and has been preserved. Verified pre-edit archive: `/Users/marcus/Claude/australian-bbq-before-arena-gums-2026-10-30.tar.gz`. All 1,799 nonignored files byte-verified, including first-tree work and unrelated concept art. Ignored build caches and Git metadata excluded. Restore into a new directory for full recovery. To undo only this second tree, compare current files with reviewed-sha256.json, extract original copies into a staging directory, then restore only those source/art files. Rebuild binaries after rollback because collision/protocol changed. Never overwrite later edits or the unrelated concept art folder.

## Changed files
- art/bush-servo/arena-trees.json: second trunk and per-tree heights.
- art/bush-servo/build_landscape.py: per-tree height selection; existing seeded geometry retained.
- art/bush-servo/smoko-landscape.blend and rust/crates/bbq_app/assets/models/smoko_landscape.glb.
- rust/crates/bbq_core/src/bush_servo.rs: second shared trunk.
- rust/crates/bbq_core/src/net.rs: protocol 23.
- art/bush-servo/LANDSCAPE.md, checkpoints/LATEST.md and this checkpoint.

## Testing
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Smoko Yard. Both gums are inside the road loop, near the picnic areas. Test walking and throwing around their bases, and carrying an opponent to the shelter. All peers will need the same version when published. Further arena decoration awaits review.

## Matched performance comparison
1280×800 native AutoNoVsync, 300-frame warmup and 600 wall-clock intervals per view. No build running during measurement. Initial matched means: overview 7.649 → 7.880 ms, tree 7.955 → 8.225 ms. Initial p95: overview 9.344 → 11.440 ms, tree 9.734 → 11.469 ms.

Because of the tail-frame increase, repeated the comparison with the same final binary, changing only the GLB and restoring the new file in a finally block. Repeat means: overview 8.001 → 7.939 ms, tree 8.172 → 8.497 ms. Repeat p95: overview 12.696 → 12.441 ms, tree 11.950 → 12.628 ms. Close-view cost is consistently about 0.27–0.33 ms (3–4%); overview varies with timing noise. This is a modest cost, not a zero-cost claim. After samples correspond to roughly 118–127 FPS on this Mac. Tail latency also varied for the original asset. Browser/device performance and distant foliage shimmer still require device testing before broad expansion. Complete initial and repeat measurements are retained in evidence/comparison.json.

## Browser publication — 30 October 2026
User approved commit and push including browser publication. Rebuilt WebAssembly bundle installed with all current assets, including previously committed surfaces and haze. Verified six public file hashes against local files (evidence/live-verification.json), successful two-way public relay, and actual in-app browser menu/Smoko scene loading. This is a smoke check, not a multi-device performance test. Old Cloudflare tunnel reported Tunnel not found; replaced it with https://planners-stripes-proved-suspension.trycloudflare.com/rust/ . Previous bundle preserved at /Users/marcus/Claude/australian-bbq-browser-before-arena-gums-20261030. All peers must refresh onto protocol 23 and create a fresh room. Temporary link requires Mac/server/tunnel running.
