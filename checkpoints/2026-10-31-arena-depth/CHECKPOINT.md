# Shelter/picnic depth — 31 October 2026

Status: reviewed pass approved by the user for commit and push on 4 November 2026. Browser publication not requested.

## Starting state and recovery
Starting commit d2a91fd956f06d39d8137fdfe5f32087ba9b539e. Tracked tree clean; untracked concept art preserved. Verified archive `/Users/marcus/Claude/australian-bbq-before-arena-depth-2026-10-31.tar.gz` contains 1,890 nonignored project files, checked byte-for-byte before editing. Git metadata and build caches excluded. Restore archive into a NEW directory for recovery; do not extract over later work. Selective rollback: restore the three changed GLBs, main.rs and shapes.rs from that archive; remove arena_depth.rs only if unchanged since this pass. Preserve all other work.

## Scope
Smoko polished Pop only. Three local shadowless bounce lights at shelter and picnic tables, soft radial contact patches under nearby props and ground-level characters, neutralised warm timber/sandstone colours and a restrained dirt tint beside the east picnic table. Existing global sun, haze, shadow map size, SSAO quality and exterior geometry/textures unchanged. No gameplay, collision, networking, sound or scoring changes.

The canopy receives softer-looking contrast from local reflected light; this does not introduce physically accurate area-light penumbrae. Existing Gaussian shadow filtering remains. Contact patches supplement existing SSAO; ground-level only, hidden for airborne/carried characters. First-person local character has no separate visible body/contact patch.

Material finish uses existing Blender-exported GLBs, without new downloaded/generated textures. `art/bush-servo/finish_arena_depth.py` applies an auditable local vertex tint to the verified original inputs. It preserves original binary geometry/UV/image data and material values; white vertex colours outside selected zones. The original editable Blender files remain untouched. Reapply this final runtime finish after re-exporting assets; the function accepts fresh GLB bytes, while its command-line entry intentionally reads the verified backup to avoid compounded tint.

## Verification
246 app tests pass, one ignored. Native release and wasm web-profile builds pass (existing warnings). Identical before/after screenshots captured at ground and overview cameras with 600 wall-clock samples. Mac briefly locked before after-captures, then user resumed/unlocked. Browser verification used private localhost:3012 before/after bundles, not public/rust. Embedded browser blocks pointer lock; browser timing is a rendering comparison, not multiplayer/input validation.

## Files
- rust/crates/bbq_app/src/arena_depth.rs
- rust/crates/bbq_app/src/main.rs
- rust/crates/bbq_app/src/shapes.rs
- rust/crates/bbq_app/assets/models/smoko_{shelter,props,surroundings}.glb
- art/bush-servo/finish_arena_depth.py
- checkpoints/LATEST.md and this directory

## Local launch
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Solo game → Bush Servo → Play Smoko. Live browser remains unchanged.

## Results
Actual comparison: review.html. Native picnic mean 8.009 → 8.127 ms, p95 12.254 → 10.199; overview mean 7.917 → 7.791, p95 12.537 → 9.285. Same 1280×800 views, 600 wall-clock intervals each. No meaningful regression in these samples.

Browser stationary rendering: same 666×864 view, Balanced, zero bots; two 5-second samples. Before mean 13.4/13.3 ms, p95 14.6/14.3; after mean 13.4/13.4, p95 14.6/14.6. Approximately 74–75 FPS. Pointer-lock overlay present and simulation active; this does not verify movement/multiplayer or uncapped GPU headroom. Saved four actual screenshot samples and browser-performance.json.

75 native moving-camera frames validated and encoded to motion.webp. Smoko delivery fixture final score=100, carries=0. Backyard smoke capture passed. No observed new flicker in the moving sequence. Existing physical shadow-edge filtering is unchanged: this is gentler shade contrast, not a new soft-shadow renderer.

Added runtime cost: 552,800 bytes across three GLBs, no new image payloads. One shared 129-vertex/224-triangle radial mesh; nine static contacts plus one per remote/bot character, two shared materials, three shadowless point lights. With three bots: +12 mesh entities, +1 mesh, +2 materials, unchanged image CPU bytes (89,154,828). Asset original binary payloads retained exactly; local vertex colours remain within [0,1] and neutral outside the slice.

Current source hashes: reviewed-sha256.json. Build/test logs saved under evidence. Python finish script and git whitespace checks passed. Next: user visual review before expanding this treatment, committing or publishing. Public browser files unchanged.
