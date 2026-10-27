# Bush Servo landscape — 27 October 2026
Starting commit: 058ed37 (Stage 2 committed and pushed to origin/main). Clean starting tree. Recovery archive: /Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-landscape.tar.gz; git archive with full tracked source/assets, listing verified. Extract into a NEW empty directory for recovery; do not overwrite later changes.
Scope: Stage 3 gum trees, distant ochre ridges, scrub and perimeter boulder cover, directed by the Bush Servo concept. Match its irregular silhouettes and concentrated edge detail, preserve clear combat/delivery routes. Solid added cover must have shared host/guest collision. Small prop/furniture density is tracked in CONCEPT-PLAN.md for its later stage. No Stage 3 commit/push/publication before review.
Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu
Status: Stage 3 ready for review; no Stage 3 commit, push or publication.

## Implemented changes
- `art/bush-servo/build_landscape.py`: deterministic Blender scene, 40 pale-bark branching gums, soft irregular crowns, opaque tapered leaves, ten terraced ochre ridges and distant scrub.
- `art/bush-servo/landscape-layout.json`, `landscape-bounds.json`, `validate_landscape.py`: eight solid perimeter boulder dimensions and independent authored-bounds/Rust-table verification.
- `art/bush-servo/smoko-landscape.blend`, `LANDSCAPE.md`: editable source and tuning/rebuild instructions.
- `rust/crates/bbq_app/assets/models/smoko_landscape.glb`: 5,036,596 bytes; 98,036 triangles; 11 material batches; zero textures or alpha materials. Replaces the old procedural trees/hills in the native/browser application scene rather than layering both.
- `rust/crates/bbq_app/src/yard_scene.rs`: Smoko-only landscape import.
- `rust/crates/bbq_core/src/bush_servo.rs`: shared boulder colliders, matching fallback meshes, clearance tests. Other modes and gameplay tuning unchanged. Boulders provide the requested extra cover outside the asphalt route.
- `rust/crates/bbq_core/src/net.rs`: protocol 19 prevents mixing clients with differing physical cover. Stage 2 is protocol 18; both online peers need the same build.
- This checkpoint folder and `checkpoints/LATEST.md`: comparisons, concept density/remaining prop plan and recovery.

## Verification
- 439 core tests pass. Tests cover collision/visual dimensions, broad delivery approaches, scoring circle, full boulder bounds within fence and outside road shoulder. The four existing respawn points at ±20/-10 and ±20/18 remain well clear of new boulders.
- Native release passed (existing warnings only). Browser wasm web-profile compilation passed (existing warnings only).
- Initial baseline PNG was black and rejected. Repeated baseline plus final captures are valid and visually inspected; no black capture or its timings used in the comparison.
- Same camera/settings at 1280×800, 600 wall-clock intervals per view after warmup, no concurrent build. Overview mean 8.205→8.558ms (+4.3%); gameplay 8.302→8.779ms (+5.7%); close 8.581→8.614ms (+0.4%). Corresponding p95 10.226→10.442, 10.627→11.197, 10.261→10.489ms. Modest expected geometry/shadow cost, final average 114–117 FPS. No claim of zero cost or browser FPS guarantee.
- Smoko moving-camera fixture still awards exactly +100 and releases carry (final_score=100, carries=0); 75 valid frames in moving.webp. Backyard visual smoke capture passed. No fresh human multiplayer session; no claim to have tested every mode visually this stage.
- All trees and scrub are beyond the fence; solid boulders have matched bounding boxes. Rounded corners approximate boxes by at most 9cm. Edging, scoring circle and prior shelter retained.

## Recovery
Stage 2 is committed and pushed as 058ed37. Verified backup contains 1,717 tracked entries. Extract archive into a new empty directory; rebuild there to restore exact Stage 2 without disturbing active work. To revert just Stage 3, first preserve new work, restore the three listed Rust sources from 058ed37 and remove only the new landscape asset/art files listed above. Preserve evidence. Rebuild both online peers together after a protocol change.

## Limits and next step
This is the Stage 3 landscape review, not completion of the concept. Building detail, final lighting/materials and small prop/furniture density remain in Stages 4–6; see CONCEPT-PLAN.md for explicit targets. The concept cannot provide an exact full-map obstacle count because objects are cropped/occluded. No new concept images or paid services used.
Do not commit, push, publish or begin Stage 4 before review. Current public browser remains the earlier ground/road publication; Stage 2 source was pushed but no browser publication requested in this turn.

## Publication approval — 27 October 2026
User approved committing and pushing Stage 3 including browser publication. All reviewed source/asset hashes verified unchanged. Browser rebuilt using rust/tools/build_web.sh (browser-specific flags); live delivery checks recorded below. Stage 4 has not started.

## Published browser verification
Browser package built successfully with rust/tools/build_web.sh. Existing URL https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ serves byte-identical HTML, JavaScript, compressed WASM, shelter, surroundings and landscape GLBs. Two temporary public WebSocket peers joined and broadcast successfully. Evidence: evidence/publication-verification.json. No new rendered browser playtest; native review/captures and 439 tests remain the verified gameplay evidence. Refresh all peers for protocol 19. This temporary link requires the Mac, server and Cloudflare tunnel to stay running.
