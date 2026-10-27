# Bush Servo building — 27 October 2026
Start: aba683019ddf9222f349dd479b7022b516275e83; clean source tree, only this capture checkpoint pending from interrupted start.
Verified backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-building.tar.gz (1,751 tracked entries, archive listed successfully). Extract into a NEW directory for rollback; never overwrite later work.
Scope: Stage 4 servo façade, veranda, pumps, signs, freezer and parked ute silhouette; improve material detail and overall visual consistency toward the Bush Servo concept. Keep existing modes and rules, arena boundaries and scoring unchanged. Props beyond the northern fence are decorative.
Connected libraries: Poly Haven and Sketchfab. User approved choosing pale weathered timber and a compact red vintage pump. Record creators/licences/modifications of imported assets. No paid generation.
Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu
Status: baseline and implementation in progress. No commit/push/publication until review.

## Completed verification and current handoff

Stage 4 implementation is complete. Earlier “in progress” text above records the start, not the current status. User subsequently authorised commit, push and browser publication when the account reaches 5% remaining. Last check after resume showed the short window had reset (100% remaining; weekly 37% remaining), so that conditional publication has not been performed. No further stage is authorised by this checkpoint.

Actual before/after views and 75-frame moving-camera delivery are in `review.html` and `evidence/`. Native scripted delivery ends at +100 with zero carries. Backyard smoke capture passes. All 439 core tests pass; native release and browser WebAssembly builds pass (existing compiler warnings). Final captures contain no asset-loading errors. JPEG export incompatibility was corrected by embedding PNG images. Bevy currently ignores glTF normalTexture.scale, so normal strengths are baked into the maps.

Matched measurements, mean milliseconds before → after: overview 7.182 → 7.470; gameplay 7.497 → 7.603; close 7.397 → 7.621; servo 7.377 → 7.663. Cost 1.4–4.0%, 130–134 FPS average on this Mac. 600 wall-clock samples per view, same camera/settings, 1280×800, builds stopped. These are not GPU timestamps or browser performance measurements.

Runtime GLB: 8,183,164 bytes; 47,662 triangles; 16 material batches; 11 embedded PNG images. Entire new model is outside the northern fence, game Z ≤ −24.02. Scene image CPU bytes 51,930,364 → 78,144,756 (+26.21 MB). Runtime mesh entities 7062 → 7037; materials 840 → 847. No collision, game rule, scoring, protocol or player-animation changes.

Library resources: Poly Haven Distressed Painted Planks by Rob Tuytel (CC0), adapted with cream glaze, broader 3.2m tiling, 512px maps and subdued baked normals. Sketchfab Vintage Gas Pump by Rescue3D Assets (CC BY 4.0), scaled, capped at 1024px / 512px overlays, subdued normals and no emissive glow. Creator/source/licence/modification credits are included in assets/CREDITS.txt and shown in How to play. Offline rebuild sources are packed in art/bush-servo/library/servo-library.blend. Original higher-resolution import is additionally preserved outside Git at /Users/marcus/Claude/australian-bbq-stage4-library-original-2026-10-27.blend.

Changed files: art/bush-servo/build_servo.py; art/bush-servo/library/{README.md,servo-library.blend}; art/bush-servo/smoko-servo.blend; rust/crates/bbq_app/assets/{CREDITS.txt,models/smoko_servo.glb}; rust/crates/bbq_app/src/{yard_scene.rs,menu.rs}; rust/crates/bbq_core/src/bush_servo.rs; checkpoints/LATEST.md; this checkpoint directory. No public browser files changed yet. Staging was already present on resume and has been preserved.

Limits: approximate concept composition, not an exact concept render. Ute and forecourt are decorative. Fine distant-texture filtering, vegetation finish, final lighting and Stage 6 prop density remain future work. No new human online session or browser frame-time run. Stage 5 is the planned lighting/final-material pass, but do not begin it without review.

Recovery: verified start archive above contains all tracked starting files; the source tree was clean at backup time. Extract to a NEW directory and compare against this checkpoint's SHA256SUMS before restoring any file. Never reset/clean the working tree or overwrite subsequent changes. For a selective rollback, restore only the three edited Rust files from starting commit aba6830 and remove the newly added runtime model and credits from a separate copy first. Keep the art and captures as references. No credentials are included in the asset library.

Local launch:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Choose Maps / Game modes → Smoko Yard.

Existing public browser (Stage 3 until publication): https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Verified HTTP 200 on 27 October. Cloudflare quick tunnel depends on this Mac/server staying online.

A quiet 15-minute heartbeat checks for the user’s 5%-remaining publication trigger. It must validate SHA256SUMS, publish only this verified scope, and pause after success. Polling is not an exact real-time threshold guarantee.

## Publication approved — 27 October 2026

User explicitly requested “commit and push to browser as well”, superseding the earlier 5%-remaining condition. The threshold automation is now PAUSED. Browser bundle rebuilt successfully with rust/tools/build_web.sh. Live HTML, JavaScript, WebAssembly, servo GLB and credits were downloaded over verified HTTPS and matched byte-for-byte with public/rust; hashes are in evidence/publication-http.json. Two-way public WebSocket relay passed in an isolated temporary room. This is transport verification, not a new human multiplayer playtest.

Current browser: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Refresh existing browser tabs to load Stage 4; choose Smoko Yard. This is a temporary Cloudflare tunnel and requires the Mac, local server and tunnel to stay running. Native command above runs the same source update. No Stage 5 work has started.

The publication commit includes the previously verified Stage 4 source, art, credits, recovery evidence, and generated browser files. Find its exact commit with git log -1 -- checkpoints/2026-10-27-servo-building. Earlier statuses describe the work at that time; this section supersedes them.
