# Backyard finish preview — 25 October 2026

Starting commit c720bf2; uncommitted charge-trajectory and throw-recovery work preserved. Full pre-edit project backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-25-backyard-finish.tar.gz. All 4,867 archived files verified against original hashes. Excludes .git, target, node_modules, .venv and symlinks/runtime links; includes source, assets, built browser game, checkpoints and pending work. Extract into a NEW directory, never over this working copy.

## Scope

First reviewable lighting/materials/scenery pass only, exposed as --backyard-finish (native), or ?backyard_finish=1 (browser). Default look is unchanged. Use Backyard for this preview; the opt-in renderer look is session-wide, so do not treat other maps selected during that preview session as approved. Remove the switch/query and restart for the original look.

- Warmer softened sun, slightly brighter ambient fill, gentler grading and smoother sky/horizon gradient.
- Restrained lawn texture contrast, warmer muted blades with broad height/tint variation and shorter growth along the central worn route. Same grass geometry budget; no extra grass shadows or simulation.
- Quieter timber, roof, fence and painted weatherboard detail, using existing assets/mipmaps.
- Softer character sheen preserves the current palette and models. Existing outlines, ambient occlusion and contact shading remain.
- 15 small gutter/ridge/window trim parts, against the existing house outside the playable fence. No collisions, props, objectives or obstacles added.
- Existing clouds, foliage geometry and throwable designs reused. No new concept art or generated assets.

Art tuning: rust/crates/bbq_app/src/backyard_finish.rs; lighting preview rig in lighting.rs. All earlier throw, camera, magpie and menu changes preserved. 669 automated tests pass (237 app, 432 core; one historical ignored). Native release and browser builds pass. Two matched camera comparisons and a five-second moving-camera clip are saved under evidence/ and shown in review.html. No new physics tests were added for this visual-only change; existing gameplay coverage passes.

Performance, release builds, same scenes/cameras at 1280×800, 300 warmup frames and 600 wall-clock frame intervals, AutoNoVsync (not GPU timings):

| View | Before mean | Preview mean | Before p95 | Preview p95 |
|---|---:|---:|---:|---:|
| Gameplay | 8.861 ms | 8.896 ms (+0.4%) | 12.610 ms | 12.695 ms |
| Close | 8.121 ms | 8.347 ms (+2.8%) | 13.552 ms | 13.949 ms |

No meaningful regression in this short local sample. Gameplay frames above 16.67ms: 0 → 2; close: 5 → 5. Assets: 2,964 → 2,969 meshes, 788 → 791 materials, 6,851 → 6,866 mesh entities. CPU image allocation unchanged within each view. Grass triangle budget unchanged. Broader-device and multiplayer stress performance were not rebenchmarked.

Current constraints: this is deliberately a modest first pass; cloud/tree shapes and prop models remain the existing art. Shadow filtering/contact AO are reused, not replaced with expensive new effects. The browser preview and normal game share builds; enabling the query only changes the local presentation for that session. No commit or push.

Launch preview:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu --backyard-finish
```
Browser preview after npm start: http://localhost:3000/rust/?backyard_finish=1

Browser preview query verified: menu, character preview and new house trim render; no console errors observed during this menu check. Screenshot: evidence/browser-preview.png. Mouse-lock and a live multiplayer session were not retested for this visual pass.

## Publication approval — 25 October 2026

User approved committing and pushing this work, including rebuilt browser artifacts. The Backyard graphics preview remains optional (`--backyard-finish` / `?backyard_finish=1`). Earlier review statuses above are historical. Live Cloudflare availability has not been reverified for this publication.
