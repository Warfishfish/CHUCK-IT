# Bush Servo Stage 6 — small props
Starting commit bdbe25c; clean tracked working tree. Verified gzip recovery archive: /Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-props.tar.gz. Extract into a NEW empty directory only; never overwrite later work. Tracked source/art/public files included; generated caches excluded. Baseline binary /tmp/bbq-before-servo-props.

Scope: concept-directed eskies, bins, tyre stacks, picnic furniture, cups and signs. Preserve scoring circle, four approaches, road and spawns. Keep shared host/client collision bounds consistent with visible solids. Use existing Blender connection and materials; no paid generation or new services. No commit, push, publication or later stage before review.

In progress: baseline, editable Blender prop kit, shared collision layout, captures/tests/builds/performance comparison.

Launch: cd /Users/marcus/Claude/australian-bbq/rust then ~/.cargo/bin/cargo run -p bbq_app --release -- --menu. Select Smoko Yard.


## Completed 28 October 2026 — ready for review
The directory keeps the original 27 October start date. Stage 6 is implemented locally and remains uncommitted. No public browser update was made. Unrelated untracked `concept art/` appeared after the starting backup and is preserved untouched.

### Content and scope
Four detailed eskies (one replaces the existing simple esky), five bins, six tyre stacks (14 tyres), two picnic tables, four new benches, four enamel mugs, two trays and two roadside signs. Existing rear shelter benches count as the third furniture grouping; existing fences and stone edging already provide the concept's border detail. Picnic groups are placed in the infield, not in the scoring circle. Side props occupy perimeter pockets. Eskies and table dressing are decorative cover, not new pickups/interactions.

Twenty new shared collision boxes, plus the retained existing esky collider. The Blender authoring report verifies each solid's visible bounds within 2.5 cm. Conservative box collision is intentional around tyre corners and under tables; mugs/trays do not block movement. Roadside signs are outside the play fence. Scoring, movement, throw animation, objectives and other map layouts are unchanged.

Protocol increased from 19 to 20 to reject older peers with incompatible map collision. Native and browser clients must be updated together after approval. Public browser remains Stage 5/protocol 19 for now.

### Files changed
- `rust/crates/bbq_core/src/bush_servo.rs`: shared small-prop collision bounds, procedural fallback and three layout tests; replaces the simple runtime esky with the detailed asset.
- `rust/crates/bbq_core/src/net.rs`: compatibility version 20.
- `rust/crates/bbq_app/src/yard_scene.rs`: loads the prop kit only under the Smoko map root.
- `rust/crates/bbq_app/src/servo_finish.rs`: includes prop colour maps in existing distance filtering.
- `rust/crates/bbq_app/assets/models/smoko_props.glb`: runtime art.
- `art/bush-servo/{build_props.py,props-layout.json,props-bounds.json,validate_props.py,smoko-props.blend,PROPS.md}`: editable Blender scene, repeatable authoring, layout and validation.
- This checkpoint directory and `checkpoints/LATEST.md`: review, captures, performance, recovery.

### Verification
- 442 core tests passed, including scoring-circle/spawn/approach clearance, no overlap with existing solids, and no new prop intrusion onto the asphalt. Existing map and gameplay tests pass.
- 239 app tests passed; one existing test ignored. Includes real local relay/link test; no fresh human multiplayer session.
- Native release and browser WebAssembly web-profile builds passed with existing warnings. Public browser bundle not rebuilt or deployed.
- Independent `validate_props.py` passes: twenty new shared boxes match Blender-authored bounds; existing esky bounds match too. Runtime GLB embeds all three reused wood textures; no external URI dependencies.
- Five actual paired native camera views (overview, gameplay, shelter, servo, picnic close-up). Nonblack image checks passed. All after captures and metrics completed before the Mac locked.
- 75 nonblack motion frames packaged as `evidence/moving.webp`. Scripted delivery ends with final_score=100 and carries=0. A duplicate poster screenshot warning did not affect individual motion frames. Backyard smoke capture passed.
- `git diff --check` passes. No graphical work continued after lock.

### Performance
600 wall-clock frame intervals per camera at 1280×800, AutoNoVsync, warmup completed; builds/tests finished before measurement.
- overview: 7.158 → 7.557 ms mean; 8.436 → 12.765 ms p95.
- gameplay: 7.439 → 7.690 ms mean; 8.792 → 11.436 ms p95.
- close: 7.469 → 7.697 ms mean; 8.738 → 12.094 ms p95.
- servo: 7.267 → 7.755 ms mean; 8.848 → 11.050 ms p95.
- props: 7.771 → 7.878 ms mean; 11.160 → 12.592 ms p95.
After averages 127–132 FPS; mean difference +0.107 to +0.488 ms (about +1.4–6.7%). Four original baseline timing runs were on the previous evening; the picnic baseline and all after runs are from the current session. The attempted full baseline refresh was interrupted by Mac lock: overview screenshot refreshed but metrics did not complete, so the earlier completed baseline timings remain the comparison. Do not treat these as a controlled attribution of the entire increase to props. A new timestamp guard in capture.py now rejects stale metrics on interrupted future runs. No GPU or browser frame-time claim.

Asset optimization reduced 74,528 initial triangles to 42,272, mainly by removing bevels from tiny tread blocks. Final GLB 2,191,376 bytes, ten material batches, three reused small wood maps. CPU image bytes 81,465,268 → 82,513,864 (~+1.05 MB). Mesh assets 2,745 → 2,753; materials 847 → 854. Runtime mesh entities 7,032 → 7,078 in measured snapshots, including transient gameplay effects. No new lights or simulation.

### Recovery and limitations
Starting tracked source was clean at bdbe25c. Verified backup: `/Users/marcus/Claude/australian-bbq-backup-2026-10-27-servo-props.tar.gz`; extract into a NEW empty directory, never overwrite later work. Build caches/Git metadata excluded. This backup predates the unrelated concept-art folder; leave that folder alone.
For selective rollback, first save subsequent changes separately. Restore the four modified Rust files from bdbe25c; remove only the new prop GLB and six new prop art files listed above. Preserve review evidence. Restore the protocol together with collision changes; never mix old/new clients. `reviewed-sha256.json` records reviewed implementation hashes.

This stage follows the concept's prop categories and density, not an exact render match. Signs and table details are deliberately simple; existing landscape geometry and foliage remain. Captures are scripted native scenes, not a fresh human multiplayer test or browser visual run. A fully paired same-session five-view timing rerun can be done after unlock if needed, but all planned visual captures, builds, tests and delivery checks are complete.

Next: review `review.html` and test locally. Await user review before commit, push, browser publication or further work.

## Approval and publication — 28 October 2026
User approved commit/push/browser publication, then reported stuttering movement for browser guests. Reviewed Stage 6 hashes were verified before changes. Publication was delayed to correct latency-sensitive guest position reconciliation; see ../2026-10-28-browser-movement/CHECKPOINT.md. Protocol is now 21, superseding the Stage 6-only protocol 20 above. All other reviewed implementation files match the review; new net.rs changes and the movement fix have separate tests/hashes.

Native and browser release builds pass. Combined 445 core and 240 app tests pass, one existing app test ignored. Live public bundle and prop GLB match local hashes; two-way public relay passed. Stage 6 and movement fix published at https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Refresh every client and host a fresh yard. Unrelated concept art/ is not included in this commit.
