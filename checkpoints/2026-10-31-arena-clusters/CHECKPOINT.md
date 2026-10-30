# Eastern arena planting — 31 October 2026

User authorized implementation followed by commit and push. This stage and the pending ground/prop stages are included in the approved commit. No browser publication requested in this step.

## Implemented
Three compact groups of saltbush, dry-grass fans and low sandstone gravel around the eastern arena gum and nearby existing rock cover. Reuses existing foliage/stone materials and textures; adds one warm dry-grass material. 33 small gravel stones, 27 grass fans and three low saltbush groups. Gravel remains below 6cm and is decorative; no additional collision, physics, scoring, network or outside scenery changes. Existing road, fighting space and carry routes remain open. Blender MCP rebuilt the landscape in a separate scene.

## Recovery
Starting HEAD e47700a plus pending reviewed arena ground and prop-detail work. Verified archive `/Users/marcus/Claude/australian-bbq-before-arena-clusters-2026-10-31.tar.gz` includes 1,870 nonignored files, each checked against its original bytes before editing. Restore to a new directory for full source recovery; Git metadata/build caches excluded. Selectively restore files in reviewed-sha256.json from the archive, preserving later edits. arena-cluster-bounds.json is new. Do not overwrite the pending ground/prop files when reverting only this planting stage. Unrelated concept art remains untracked and excluded from the commit.

## Verification
Builder checks every new vertex against the arena slice and inner road ellipse. Asset comparison confirms all previous scenery is preserved within 0.02mm export tolerance. Shared tree trunks still match layout; no collider edits. Reviewed ground and prop file hashes remain unchanged. Native Smoko delivery fixture awards +100 with zero remaining carries; Backyard smoke capture passes. 75 real motion frames validated and encoded. Python validation and git whitespace checks pass. This is an asset-only addition using the already verified native/browser builds from the ground pass (245 app tests passed there); no redundant recompilation. Public browser files unchanged, no browser/device performance claim.

Landscape cost: 136,744 → 138,475 triangles (+1,731), 9,036,288 → 9,138,480 bytes (+102,192), material batches 11 → 12. No new images or texture-memory allocation. Matched 1280×800, 300 warmup frames then 600 wall-clock intervals, AutoNoVsync:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Ground | 7.885 ms | 7.819 ms | 9.514 ms | 9.654 ms |
| Overview | 7.700 ms | 7.616 ms | 9.449 ms | 9.229 ms |

Differences small and within timing variation; no measurable regression in these samples. This is a small local slice, not a full-arena landscape makeover.

## Changed files
- art/bush-servo/build_landscape.py
- art/bush-servo/smoko-landscape.blend
- art/bush-servo/arena-cluster-bounds.json
- art/bush-servo/LANDSCAPE.md
- rust/crates/bbq_app/assets/models/smoko_landscape.glb
- checkpoints/LATEST.md and this checkpoint
Pending ground/props files are listed in their own checkpoints and are committed together under the user's instruction.

## Local test
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Bush Servo / Smoko; inspect the eastern gum and adjacent picnic area.
