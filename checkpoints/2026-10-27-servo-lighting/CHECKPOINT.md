# Bush Servo Stage 5 — lighting and final materials

Started 27 October 2026 at commit 676d888. Working source tree was clean. Verified pre-edit backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-lighting.tar.gz; gzip integrity passed, 1,788 tracked entries listed. Extract into a NEW directory for recovery, never overwrite later work.

Scope: warm sun, open shade, material consistency and reduced texture shimmer in Smoko Yard. Preserve other modes, physics, game rules, scoring, all Stage 1–4 assets and current character/throw animations. No new obstacles or Stage 6 props. No commit, push or publication before review.

Baseline: four actual native captures and paired performance measurements in evidence/before-*. Starting binary preserved at /tmp/bbq-before-servo-lighting. Capture script uses fixed scene/camera, 1280×800, 600 wall-clock samples following warmup and screenshot completion. Mac lock checks required before capture.

Implemented: scoped lighting transition with restoration of original world lights and camera settings; distance filtering for imported Smoko textures including sRGB colour maps previously skipped by legacy texture pipeline. Existing art resources reused; no paid generation or new downloads.

Launch:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Smoko Yard. Live Cloudflare remains published Stage 4 until review/approval.


## Completed — ready for review

Stage 5 is implemented locally. No commit, push or public browser update performed. Starting full commit: `676d888f6132aa0bf6436dcfc57a59940551be88`.

### Exact source changes
- `rust/crates/bbq_app/src/servo_finish.rs`: new scoped Smoko polished/Pop light rig; warm sun angle/colour, open shade, mild grade and distant haze. Saves original world lights and EyeCamera values on activation and restores them when leaving the mode. Imported Smoko GLB colour/data texture mipmaps and anisotropic filtering; alpha-weighted linear-light colour averaging. Two focused tests. Named tuning constants at file top.
- `rust/crates/bbq_app/src/lighting.rs`: exposes existing hemisphere calculation for a supplied rig; default calculation unchanged.
- `rust/crates/bbq_app/src/yard_scene.rs`: marks the main world sun and fill so preview/lobby lights are not changed.
- `rust/crates/bbq_app/src/main.rs`: registers the new plugin.
- `checkpoints/LATEST.md` and this checkpoint directory: recovery, capture scripts, review and evidence.

No art assets, geometry, collision, core rules, network protocol or animation files changed. Existing material colour maps are retained; this pass improves their lighting and distance sampling rather than repainting them.

### Verification
- 239 app tests passed; 1 existing test ignored. Includes two new targeted filtering tests.
- 439 core tests passed.
- Native release and browser WebAssembly web-profile builds passed; existing compiler warnings remain. No public bundle regeneration, deployment or tunnel changes.
- Four identical-camera before/after actual native views at 1280×800; all nonblack. Compared overview, gameplay, shelter close-up and servo material close-up.
- Runtime logs confirm 14 imported textures received mip chains. Base texture pixels and wrapping remain unchanged.
- Backyard capture passed; scripted Smoko motion/delivery returned final_score=100, carries=0. 75 nonblack native frames packaged as animated WebP. A duplicate poster screenshot warning is benign; the individual motion frames and delivery result were captured.
- `git diff --check` passed.

### Performance and asset costs
Builds and tests had finished before measurements. 600 measured wall-clock intervals per view after warmup, same camera/scene/settings, AutoNoVsync:
- overview: mean 7.751 → 7.204 ms; p95 10.832 → 8.478 ms.
- gameplay: mean 7.927 → 7.444 ms; p95 11.027 → 8.887 ms.
- close: mean 7.877 → 7.648 ms; p95 11.737 → 9.043 ms.
- servo: mean 8.075 → 7.308 ms; p95 12.415 → 8.722 ms.
No measured regression; lower averages in this run are not a controlled claim of a speedup. After averages approximately 131–139 FPS. Texture CPU bytes 78,144,756 → 81,465,268 (+3,320,512 bytes, ~3.3 MB), expected additional mip levels. Mesh assets remain 2,745 and materials 847; no GLB download size increase. Runtime mesh entities varied from 7,037 to 7,032 with transient scene effects; no geometry was deleted. These are wall-clock measurements, not GPU timings or browser FPS.

### Limits
This is a modest cohesive lighting/filtering improvement, not an exact match to the concept. Existing low-poly foliage silhouettes and shadow-map edges still show some aliasing. No new small props, detail models or obstacles; those remain Stage 6. No fresh online multiplayer playtest or browser graphical/performance capture. Other mode gameplay is covered by core tests and Backyard visual smoke check, not fresh captures of every mode. Lighting restoration is implemented but was not separately exercised through an interactive mid-session menu switch.

### Recovery and next step
Verified full tracked starting backup: `/Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-lighting.tar.gz`. Extract only into a NEW empty directory. It excludes build caches and Git metadata. The source tree was clean at backup time.
For a selective rollback, first save any later work outside the repo, inspect differences, then restore only main.rs, lighting.rs and yard_scene.rs from starting commit 676d888 and remove the newly added servo_finish.rs. Do not reset the whole repository or overwrite newer unrelated edits. Keep checkpoint evidence. `reviewed-sha256.json` records reviewed source hashes to detect later edits.
Review `review.html`, test locally, and await user approval before commit, push, browser publication or Stage 6.

## Commit approval
User approved committing and pushing Stage 5. Reviewed source hashes reverified unchanged before commit. Public browser deployment is not part of this request; browser remains Stage 4. Stage 6 has not started.
