# Soft silicone design

Started 14 October 2026; completed 15 October 2026 — ready for review. User requested improved dildo design and a more natural soft-material flop. Starting HEAD dc75e0bb7089d2358954022aa642dc536921a829. Preserve prior graphics, Q, noodle and physics work. No commit/push. Starting Rust implementation archived with hashes.

Plan: continuous Blender-built surface with gentle molded texture and satin silicone material; shared GPU-skinned mesh using existing seven joints; firmer base with delayed tip flex. Compare actual held motion and performance. Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky

## Completed implementation

- `rust/tools/make_silicone.py`: creates six continuous Blender mesh surfaces (Smooth, Studded, Ribbed, Rilled, Spiral, Veiny) with rounded base/tip and shallow molded detail. Exports game-native Y-up positions, normals and triangles. Does not change scene objects or selection; temporary export mesh datablocks are removed.
- `rust/crates/bbq_app/assets/models/silicone.json`: compact exported geometry, 1,387,481 bytes; 2736 vertices and 5424 triangles per skin. Variant colours and sizes remain the existing eight game variants.
- `rust/crates/bbq_app/src/silicone.rs`: shares meshes/materials by existing model key; adds normalized skin weights and seven-joint deformation with dynamic bounds. Satin material, rounded silhouette, less harsh shine, no separate outlined link seams. Each instance has seven inverse bind matrices. No per-frame CPU vertex uploads or paid generation.
- `rust/crates/bbq_app/src/models.rs`, `main.rs`: integrate silicone rendering into the existing model cache and spawning path, covering in-hand, yard, chest and gallery items.
- `rust/crates/bbq_app/src/items_view.rs`: silicone-specific held spring; firm base, progressively softer links and delayed tip response. Ground/flight contact physics, catch/hit radii, aiming and Q remain unchanged. Pool noodle visuals and dynamics are preserved.
- `rust/tools/silicone_review.py`: captures held motion, model gallery, drop and separate stress scene; optional --encode generates actual-frame WebP previews and contact sheets.

Blender connection: scene inspection and Python export succeeded. Add-on status endpoint returned `No module named blender_mcp.config`, but scene and execute endpoints worked. No plugin reinstall or external service was required. Original Cube/Light/Camera scene objects were preserved.

## Verification and actual evidence

`tests.txt`: 182 app + 411 core tests passed; one existing ignored app test. New tests check finite bounded geometry, normalized positive weights, valid joint indices, satin material parameters, bounded mesh vertex cost, firm grip and spring settling at 30/60/120 Hz. `build.txt`: release passed (2m35s), existing 16 app warnings remain. `git diff --check` passed.

`before-captures.txt` and `after-captures.txt`: baseline gallery/stress plus after held-rubber/gallery/drop/stress completed without logged ERROR. Before held sequence reused from the preceding comic-flex checkpoint's after evidence; scripted inputs and camera are unchanged. Reviewed full-size held frames, motion contact sheet, all-model gallery and settled drop. New shape is visible in all reviewed paths, has a continuous bend and no persistent lawn penetration in reviewed settled frames. Gallery is a technical model lineup with some overlap; the first-person comparison is the primary design evidence.

`review.html`: actual before/after stills, looping held-motion clips, matched-frame playback and drop capture. 75 frames per sequence at 15 fps. Original PNGs 1280×800; WebP previews 960×600. These are implemented game renders, not generated concepts. No human live gameplay or multiplayer session was performed.

Fresh stress measurements: mean 7.825 → 7.903 ms (+1.0%), 127.79 → 126.53 FPS; P95 8.979 → 8.812 ms. Zero of 600 sampled intervals over 16.67 ms in both runs. Fixed 60 Hz simulation, 300 warmup frames then 600 wall-clock samples, AutoNoVsync, 1280×800. Capture/encoding/build jobs were not run during measurement. Not GPU timestamps; small difference does not establish a meaningful regression. Mesh assets 2897 → 2881; materials 911 → 905; mesh entities 7737 → 7389. CPU image memory unchanged at 42,129,816 bytes. New geometry asset/skin buffers replace many individual part meshes; total GPU memory was not measured.

Limits: stylized skeletal deformation approximates soft silicone; no volume-preserving soft-body solver or mesh-shaped collision. Surface textures are intentionally shallow. Final subjective comic feel needs user playtesting, including fast turns, charging/slapping and variant sizes.

## Recovery

Restore only in a separate clean checkout at the starting HEAD. Extract starting-work.tar.gz at the repository root to recover prior uncommitted implementation, then apply implementation.patch to recover this milestone. Verify from that root with `shasum -a 256 -c /path/to/checkpoint/FILES.sha256`. The seven-file delta, including embedded mesh JSON, was applied to the starting files in an isolated directory and verified byte-for-byte (`recovery-verification.txt`). Keep this entire checkpoint folder separately; it is not part of the starting archive. Do not overwrite later working changes.

To regenerate geometry, execute `rust/tools/make_silicone.py` in connected Blender (or a Blender background process). Normal game builds only need the saved JSON; they do not need Blender running.

To reproduce captures from repository root: `python3 rust/tools/silicone_review.py --label after`. Encode after captures finish with `/Users/marcus/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 rust/tools/silicone_review.py --label after --encode`. Baseline executable currently lives at /tmp/bbq-silicone-before; it is temporary. Durable baseline recovery is HEAD plus this starting archive.

No commit or push. Next step: user review/playtest of the design and soft-material motion. Await review before starting another milestone.
