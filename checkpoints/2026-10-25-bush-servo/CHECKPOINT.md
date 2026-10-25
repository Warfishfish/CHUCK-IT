# Bush Servo — first playable environment section

Starting commit: bc68c8a. Clean starting worktree. Backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-25-bush-servo.tar.gz; 4,969 files verified by SHA256. Excludes Git/build caches and symlinks.

Scope: separate Smoko scenery, curved road and central shelter, servo frontage. Preserve delivery scoring, timing, carry/throw and supply rules. Other maps unchanged. Future Tradie Yard/Surf Club variants deferred.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust` then `~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko`. Menu: With mates or solo setup → Maps / Game modes → Smoko.

Recovery: extract archive into a NEW directory; never extract over active work. Starting committed version also available as bc68c8a. Do not reset unrelated work.

Status: implementation in progress; native baseline captures saved. No commit/push/publication authorized for this stage.

## Completed first section — awaiting review

- Smoko now selects Bush Servo through the existing Maps / Game modes selector. Future Tradie Yard / Surf Club variants are not implemented yet.
- Separate red-earth scene, seamless curved asphalt loop, open-sided timber/corrugated shelter, clear scoring ring, benches/esky, servo frontage and awning, two background pumps, gum trees/scrub and distant ridges.
- All Smoko scoring, carry/throw timings, objective radius/centre and supply positions are preserved. Flat dirt and asphalt have identical movement. Existing rectangular play bounds are marked by a stock fence; the road is curved, not the outer collision boundary.
- Removed inherited Backyard scenery and invisible pool/trampoline/clothesline collisions from Smoko. Shared supplies remain visible and interactive. New ground-based solid props use shared render/collision dimensions. Guest worlds reconstruct the same layout. Protocol 17 prevents older clients joining the incompatible map layout.
- Code-generated 512px dirt/road textures and a 1024x128 sign; generator retained in rust/tools/make_servo_textures.py. Existing lighting, sky, character design, animation and materials reused. No AI assets or paid services.

## Verification

673 tests pass (237 app + 436 core; one historical ignored). Native release and browser builds pass. Focused tests verify clear delivery approaches, render/collider dimensions, dry ground at the old pool, removal of old trampoline behaviour, unchanged outer projectile bounds and host/guest collision reconstruction. Existing mode/rule tests remain green.

Actual native captures for Backyard, Teams, Heist and Race all launched without runtime errors. These are smoke checks, not exhaustive manual rounds. Two 75-frame / five-second native clips show delivery and moving-camera views. Each controlled delivery completed with final_score=100 and carries=0. The fixture stages the victim and drives the existing carry/throw systems; it is not a human playtest. Duplicate-target screenshot warning at the shared poster frame did not affect the 75 numbered motion frames.

Local Chrome: selected Bush Servo in Solo game and launched into the actual map; browser-selector.png and browser-gameplay.png saved. No live multi-computer multiplayer test or external Cloudflare publication was performed in this stage. Both peers need the rebuilt version because of protocol 17.

Matched native comparison: same 1280x800 cameras, polished/pop settings, 600 measured frame intervals after warmup; no builds running during final comparison. Wall-clock timings, not GPU timestamps.

| View | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Overview | 8.013 ms | 7.403 ms | 11.762 ms | 13.163 ms |
| Gameplay | 8.250 ms | 7.519 ms | 11.756 ms | 12.345 ms |

Mean frame time improves about 8–9%; p95 increases but remains below 16.67 ms. Frames above 16.67 ms: overview 0→0, gameplay 2→0. No sustained regression in this small local sample. Assets: meshes 2,993→2,771; materials 801→813; total mesh entities (including retained hidden Backyard entities) 7,136→7,338; image CPU bytes 42,667,928→46,163,196 (+3.3 MiB). No browser/device-wide performance claim.

## Limits and next review decision

This is a first playable environment section, not a match to the concept render. Detailed vehicle/rock/foliage assets, richer wear, fuller landscaping and alternate Smoko maps remain later work. The servo building and pumps are background scenery beyond the play fence, not an enterable shop. Shelter roof/signs are visual overhead details in this stage; posts, benches and esky are solid. The generic pickup hint still refers to the lawn. No new lighting system was introduced; existing polished/pop lighting is reused.

Review the shelter, curved route and palette before expanding the map. Do not commit, push or publish without user approval. Full changed-file list and SHA256 manifest accompany this checkpoint. No unrelated pending work existed at start.

Testing the captured visual settings:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko --look polished --style pop
```
Or launch with `--menu`, choose Solo game → Bush Servo → Play Smoko — Bush Servo. Existing With mates setup uses the same map selector.

## Publication approval — 25 October 2026

User approved committing and pushing the reviewed Bush Servo first pass, including rebuilt browser files. All 23 reviewed source/asset/build hashes match the tested state: 673 tests and native/browser builds passed. Earlier review statuses are historical. Blender connection repair and further art work remain separate future work.
