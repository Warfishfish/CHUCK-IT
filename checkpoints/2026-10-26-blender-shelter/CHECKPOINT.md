# Blender shelter — 26 October 2026
Starting commit: 3d63c1e36f36872006d93fc38d41bf46291430f5; clean worktree.
Recovery: /Users/marcus/Claude/australian-bbq-backup-2026-10-26-blender-shelter.tar.gz (git archive, successfully listed). Extract to a NEW directory, never over the active project. Includes tracked source and assets; generated build caches excluded.
Scope: concept-inspired shelter authored in live Blender and imported into Smoko. Preserve collisions, scoring and other modes. No commit/push.
Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko --look polished --style pop
Status: baseline and asset work in progress.

## Implemented
Live Blender model: art/bush-servo/smoko-shelter.blend. Repeatable authoring script: art/bush-servo/build_shelter.py. Original default Blender scene backed up in pre-shelter.blend and retained in the working file. Reference: original Bush Servo concept exec-6ea4e9b3-70f7-4745-b24d-773f23cbe0ae.png.

GLB at rust/crates/bbq_app/assets/models/smoko_shelter.glb: 29,318 triangles, 16 primitives, 1,672,392 bytes, five embedded grain maps. Export has one scene/node/mesh and no external image dependencies. Only Smoko loads the asset. Shared collider dimensions, scoring and other maps are unchanged. The old procedural shelter remains available through parts(); app uses parts_with_shelter(false) plus the imported visual.

Native release build passed (existing warnings remain). Both focused bush_servo tests passed. Before/after overview, gameplay and close views captured with identical cameras. Controlled delivery sequence: final_score=100, carries=0. Captures produced no runtime ERROR entries. Performance details in performance.json and review.html: about 3–9% mean frame cost increase, p95 below 15ms in this local sample; no broad device claim.

Known limits: roof/sign remain visual overhead geometry, as in starting version. Bench feet have visible gaps within existing simple box colliders. Central picnic tables omitted to preserve the clear delivery zone and existing physics. This is the shelter only, not the full concept scene; foliage, ground, servo frontage and lighting remain existing assets.

Rollback: restore the two Rust source files and remove this newly added GLB from a COPY of the project using the starting archive; keep the active project safe. The art and checkpoint folders can remain as references. Do not use a hard reset over later work. Browser files will need rebuilding from restored source if reverting the browser version.

Browser release build passed, and its copied GLB exactly matches the native asset. Browser runtime has not been independently playtested in this pass. Review is ready in review.html. No commit/push/publish.

## Publication approval — 26 October 2026
User approved commit/push including browser. All nine reviewed source/art/build hashes verified unchanged. Delivery frames packaged as delivery.webp so the committed review works without ignored raw frames. Native/browser builds and prior checks remain valid.

Browser publication: https://luther-bali-amsterdam-plans.trycloudflare.com/rust/ . Previous tunnel expired (Tunnel not found); replacement PID 40406, log /tmp/australian-bbq-cloudflare-2026-10-26.log. Public page HTTP 200; public shelter GLB and bbq.js byte-for-byte match the reviewed local browser assets. Local server on port 3000 remains running. Temporary link depends on this Mac and tunnel staying running. No new rendered-browser playtest performed during publication.
