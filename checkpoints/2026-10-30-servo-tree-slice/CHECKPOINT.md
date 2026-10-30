# First in-arena gum — 30 October 2026

## Scope correction
The user prioritised the playable arena and requested gum trees inside it. This first reviewable slice adds one specimen and its planted base; exterior trees, surrounding landscape and mesas remain unchanged. Original landscape vertex positions were compared against the backup: all remain present, and every added vertex is within the arena slice. Do not resume work outside the arena under the previous broader plan.

## Implemented
- One 8.4m gum at (12, 1.5), inside the inner road loop near the picnic area. Seven asymmetric boughs with 21 hanging foliage sprays, pale bark and small root flares.
- Nine compact dry grass/saltbush clusters around its base. No new texture or material batches; existing CC0 Poly Haven eucalyptus bark reused. Blender MCP generated and exported the new geometry. No download, paid generation or new library dependency.
- Shared 0.7 × 0.7m lower trunk collision to 4m, marked no-top. High branches and foliage are visual only. Existing road, delivery circle, spawn positions and carry routes remain clear.
- Lightweight procedural fallback includes a visible trunk/crown. Multiplayer protocol 22 prevents mixed versions with different collision layouts from joining. All peers must update together on eventual publication.

## Verification
446 core tests pass, including tree clearance, shared collision presence, spawns and non-overlap checks. 244 app tests pass; one ignored existing test. Native release build passes. Existing compiler warnings retained. Actual native Backyard smoke capture passes; the Smoko motion fixture awards +100 once with no remaining carry. No live multi-device session was captured; existing network/relay tests pass, including version checks.

Matched 1280×800 cameras, 300 warmup frames then 600 wall-clock frame intervals, AutoNoVsync:

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Arena overview | 7.608 ms | 7.529 ms | 9.502 ms | 9.466 ms |
| Tree view | 7.753 ms | 7.782 ms | 9.569 ms | 9.764 ms |

Small sample differences, no meaningful measured regression; not GPU timings or browser/device claims. Texture CPU bytes remain 87,407,208. Landscape 126,016 → 131,380 triangles (+5,364), 8,431,572 → 8,733,372 bytes (+301,800), 11 material batches unchanged. Actual before/after images and 75-frame moving camera clip are in review.html. Fine foliage still needs browser/device shimmer review before broad rollout. Other modes beyond Backyard/Smoko were not individually captured.

## Starting state and recovery
Starting commit 4b188db1b640b102d0c683a0c4f0019571dffde2. Tracked tree was clean; unrelated untracked concept art preserved. Verified backup: `/Users/marcus/Claude/australian-bbq-before-tree-slice-2026-10-30.tar.gz`. All 1,780 nonignored files verified against original hashes before edits. Git metadata and ignored build caches excluded.

Restore into a new directory for complete source recovery. To selectively undo this pass, compare current files against reviewed-sha256.json, stage original files from the archive separately, then restore only listed files; arena-trees.json is new and should be removed when reverting. Rebuild native/browser from restored source because collision and protocol changed. Do not overwrite later edits or unrelated concept art. LATEST.md and this checkpoint document the work.

## Changed files
- art/bush-servo/build_landscape.py
- art/bush-servo/arena-trees.json (new)
- art/bush-servo/smoko-landscape.blend
- art/bush-servo/LANDSCAPE.md
- rust/crates/bbq_app/assets/models/smoko_landscape.glb
- rust/crates/bbq_core/src/bush_servo.rs
- rust/crates/bbq_core/src/net.rs
- checkpoints/LATEST.md and this checkpoint

## Local testing
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Maps / Game modes → Smoko Yard. Find the new gum beside the picnic area within the road loop. Walk around its trunk, throw towards it, and carry a downed opponent into the shelter. Review this first placement before adding further trees inside the arena. No commit, push or browser publication yet.

## Final build verification
WebAssembly release-profile compilation passed. Public browser files were not modified or published; the live version remains unchanged. Native/browser builds, reviewed hashes and whitespace checks pass.
