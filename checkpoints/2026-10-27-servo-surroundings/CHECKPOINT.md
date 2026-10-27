# Bush Servo surroundings — 27 October 2026
Starting commit: 7d1fab5; clean worktree. Backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-surroundings.tar.gz (tracked source/assets, archive listing verified; build caches excluded). Recover into a NEW directory, not over active work.
Scope: stage 2 only: low sandstone edging, dry grass, small native shrubs and open approaches. Preserve scoring, movement tuning and the existing shelter. Solid stones use shared host/guest collision dimensions. Plants are decorative. No commit/push/publication until review.
Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko --look polished --style pop
Status: complete and ready for review; not committed or published.

## Implemented
- Blender-authored 22 beveled sandstone blocks, dry-grass tufts and eight saltbush clusters. Four broad cardinal approaches remain open; scoring circle untouched.
- `art/bush-servo/build_surroundings.py`, `make_sandstone.py`, `surroundings-layout.json`, `surroundings-bounds.json`, `sandstone-grain-{0,1,2,3}.png`, `smoko-surroundings.blend`, `SURROUNDINGS.md`.
- `rust/crates/bbq_app/assets/models/smoko_surroundings.glb` (424,856 bytes; 4,978 triangles; nine material batches; four embedded 128px images).
- `rust/crates/bbq_app/src/yard_scene.rs`: loads surroundings only with the Smoko environment.
- `rust/crates/bbq_core/src/bush_servo.rs`: matching stone collider table, procedural fallback, clearance tests.
- `rust/crates/bbq_core/src/net.rs`: protocol 18; prevents mixing old/new solid map boundaries online. Both peers must update together.
- This checkpoint folder and `checkpoints/LATEST.md` contain review evidence and recovery information.

## Verification
- 438 core tests passed; includes four Bush Servo tests covering solid dimensions, four approach lanes and delivery-circle clearance.
- Authoring script verifies Blender rock dimensions against layout before export; independent JSON/Rust comparison passed. Beveled corners lie inside rectangular collision bounds (maximum 4.5cm corner approximation).
- Native release passed with existing warnings.
- Three matched before/after 1280×800 views; all valid nonblack images. 600 measured frame intervals per view; no build running during measurements.
- Mean milliseconds: overview 8.279 → 8.195 (-1.0%); gameplay 8.465 → 8.275 (-2.2%); close 8.239 → 8.401 (+2.0%). p95 respectively 13.121→14.431, 14.034→12.815, 12.851→13.959. Small session variation; no meaningful average regression, no claim of a speed improvement.
- Smoko moving-camera delivery fixture: final_score=100, carries=0. 75 actual frames packaged as moving.webp. Backyard smoke capture passed. No fresh human multiplayer playtest; other modes covered by core regression tests, not new visual captures.
- Browser wasm web-profile compilation passed (existing warnings only). Public browser bundle and tunnel unchanged.

## Recovery
Do not restore over unrelated new work. Extract the verified archive into a new empty directory to run the exact starting tracked project. Starting commit 7d1fab5 is also available in Git. Build caches were excluded from the backup.
To selectively revert this stage, first save the current worktree elsewhere, then restore the three Rust source files from 7d1fab5 and remove only the new surroundings GLB and art files listed above. Preserve checkpoint evidence. Rebuild native/browser together before playing online because of protocol differences.

## Next step
Review `review.html` and test the local release. Do not commit, push, publish or start trees/distant scenery until user approval. Remaining concept detail (trees, building, lighting and small props) is outside this stage. No claim that the concept finish is already matched.

## Approval — 27 October 2026
User approved committing and pushing Stage 2 and starting Stage 3. Reviewed source and asset hashes verified unchanged. Browser publication not included in this request.
