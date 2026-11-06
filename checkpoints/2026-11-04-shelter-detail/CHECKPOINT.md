# Shelter timber and roof detail — started 4 November, verification 5 November 2026

Status: completed and ready for user review, 5 November 2026. No commit, push or live publication.

## Recovery
Starting commit 9d6c6941a8285f279c5c288a9cbf3e8db7978de9. Verified backup: `/Users/marcus/Claude/australian-bbq-before-shelter-detail-2026-11-04.tar.gz`, 1917 files byte-checked before editing. Restore into a new directory, not over newer work. Untracked concept art remains untouched.

## Changes
Blender MCP authored a separate scene. Heavier upper bearers and rafters, softer post edges, grain aligned along each board, exposed endgrain, restrained timber normal detail, and continuous roof oxidation textures in place of random per-face stripes. Existing post footprint (0.24 × 0.24m) and height (3.8m) preserved; bearer underside remains 3.54m. No gameplay or collision-source changes. External scenery, picnic props and the live browser bundle unchanged.

The editable .blend now stores named component objects in the shelter scene only; export joins temporary copies for runtime. Existing arena-depth vertex tint is reapplied to the new shelter only. Prop builder material lookup now tolerates Blender's name suffixes; its runtime asset was not rebuilt.

## Verification
Same-camera native before/after at close, detail and ground views, plus Smoko delivery and Backyard smoke captures. Smoko fixture reports final_score=100, carries=0. 75 real motion frames captured. Browser smoke comparison complete; details below. This is an asset-only pass: source and native/WebAssembly executables from the approved release remain unchanged, so no redundant Rust rebuild was performed.

## Local testing
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Solo game → Bush Servo → Play Smoko.

## Final results and limitations
29,318 triangles and 16 material batches before/after. GLB 1,825,188 → 2,207,724 bytes (+382,536). Embedded images 5 → 12, each 128×512. Estimated RGBA textures including mips +2.45 MB; whole-process image CPU storage in the native capture rose from 89,154,828 to 92,999,680 bytes. No paid assets or generated AI images.

Matched 1280×800 native views, 600 uncapped wall-clock frame intervals each:
- Ground: mean 7.896 → 8.029 ms, p95 10.101 → 10.181 ms.
- Roof/close: mean 7.866 → 8.272 ms, p95 10.213 → 10.529 ms.
- Under-eave detail: mean 7.901 → 8.847 ms, p95 10.204 → 11.697 ms.
- Focused detail repeat after closing the browser test: mean 8.055 → 8.538 ms, p95 11.648 → 10.821 ms. A modest close-up cost remains (~0.48ms, ~6% in the repeat), plausibly from the extra texture/normal shading; not a claim of identical performance. Both native samples remain above 110 FPS on this Mac.

Stationary browser smoke check: default Balanced, 666×864, zero bots; ~75 FPS / 13.3ms average before and after. Before p95 14.6/14.2ms, after 14.7/15.4ms. Spawn cameras differed, and mouse capture overlay remained visible. This is a rendering smoke check, not an exact browser A/B, movement or multiplayer test. Matching-camera comparisons are native.

Smoko delivery +100, zero remaining carries; Backyard loads. 75 moving-camera frames validated and encoded into motion.webp. Final screenshots visually inspected. Triangle/batch counts, post dimensions, original eave clearance and unchanged gameplay/exterior/public files verified by validate.py. Builder scripts parse and whitespace checks pass. Editable Blender file contains one shelter scene and 165 named authoring objects; material lookup for future prop authoring verified. Existing props were not regenerated.

## Exact changed files / recovery
See reviewed-sha256.json for all source/asset hashes, including 13 small source PNGs. The only runtime asset changed is smoko_shelter.glb. Restore build_shelter.py, build_props.py, smoko-shelter.blend and the runtime GLB from the backup to roll back this stage. Remove the newly created shelter_materials.py, shelter-detail-bounds.json and shelter-textures directory only after confirming they still match the reviewed hashes; preserve any later edits. Do not run the earlier arena-depth script's archive-based CLI to rebuild the new shelter: build_shelter.py already applies its finish function to the fresh export correctly.

Next: review the shelter in game before committing, publishing or moving to picnic furniture. The live Cloudflare browser version remains the previously approved release.
