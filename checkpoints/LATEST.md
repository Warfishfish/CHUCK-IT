# Bush Servo outer ground cover — approved for commit and browser (29 October 2026)

[Actual in-game overview and comparison](2026-10-29-servo-groundcover/review.html) · [Checkpoint and rollback](2026-10-29-servo-groundcover/CHECKPOINT.md). Added static spinifex fans and grouped ochre stones outside the play fence; routes and shared bounds are unchanged. Blender export: 61,577 polygons, 12 materials, 8.43 MB GLB. User approved commit, push and browser publication; the bundle is rebuilt and live at https://carpet-venture-charlotte-temple.trycloudflare.com/rust/. Live JS, wasm and both Smoko GLBs match the local build; the browser menu and Smoko game screen loaded. Prior landscape source/asset files are saved under the checkpoint's `recovery/` directory.

# Bush Servo concept-style detail — ready for review (29 October 2026)

[Actual before/after and motion](2026-10-29-servo-detail/review.html) · [Checkpoint and rollback](2026-10-29-servo-detail/CHECKPOINT.md). Live Blender MCP: finer open gum foliage, pale Poly Haven eucalyptus bark, subtle rock textures and stocked/finished servo frontage. Existing Sketchfab fuel pumps and Poly Haven timber retained. Collision bounds unchanged; real native Smoko delivery and Backyard smoke capture pass. Matched mean frame cost +0.08–0.55 ms; changed models +2.43 MB after removing 2.63 MB unused UV data. Verified starting backup includes tracked files and untracked concept art. Local asset-only pass; no commit, push or browser publication. Next art gap: ground cover, stone/soil transitions, road surface and atmospheric depth.

# Smoko browser frame profiler — live for testing (29 October 2026)

[Review and host/guest testing instructions](2026-10-29-smoko-browser-profile/review.html) · [Recovery checkpoint and saved capture](2026-10-29-smoko-browser-profile/CHECKPOINT.md). Optional local-only Smoko overlay shows rolling FPS, average/p95 frame interval, long frames and fixed-step backlog, labelled HOST/GUEST/SOLO. Fixed the wasm timer panic using browser `performance.now()` and placed the panel above the HUD. WebAssembly release-profile build passes; live Chrome Smoko gameplay and overlay verified. Solo samples observed: ~75 FPS, avg 13.3 ms, p95 13.9 ms; later saved paused-overlay sample 60 FPS, avg 16.7 ms, p95 23.3 ms, zero long frames/backlog. Neither is a host/guest comparison; collect active-play samples on both devices. Public `bbq.js` and `.wasm` hashes match the live Cloudflare response (HTTP 200). Live opt-in: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/?perf=1 . Source fix and browser bundle are committed/pushed as `3d71308`; active-play host/guest measurements can be collected when both players are available.

# Smoko-first polish — committed and pushed (29 October 2026)

Smoko / Bush Servo is the reference map for staged performance and gameplay polish; proven improvements can transfer to other maps after review. [Actual before/after captures, delivery sequence and comparison](2026-10-28-smoko-polish/review.html) · [Checkpoint](2026-10-28-smoko-polish/CHECKPOINT.md) · [Development direction](2026-10-28-smoko-polish/SMOKO-ROADMAP.md). Contextual carry prompts and reduced hidden-yard update work. 244 app tests pass; native and browser builds pass. Delivery fixture awarded +100 once with no remaining carry. Matched native timing is within run-to-run variation; browser multiplayer pacing still needs host/guest device measurement. Committed and pushed as recorded in this commit. Not published to the live browser.
# Browser mouse capture — publication approved (28 October 2026)

[Checkpoint](2026-10-28-browser-mouse/CHECKPOINT.md). Explicit click-to-capture/recover, actual pointer-lock state checks, safe release on menus/focus loss, input/charge clearing while unlocked. 242 app tests and JS lifecycle tests pass; browser build and public file hashes pass. Updated menu loaded in Chrome; actual mouse-lock gameplay check remains unverified because the browser window closed during testing. User approved commit/push including browser bundle; live file hashes reverified. Live browser updated: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Refresh, start a game, click capture prompt; Escape releases the mouse.

# Browser rendering budget — 28 October 2026

[Checkpoint](2026-10-28-browser-performance/CHECKPOINT.md). Browser Balanced defaults reduce shadow-map and ambient-occlusion work; native graphics unchanged. Existing delayed-position reconciliation fix retained. App and focused network tests pass, browser build/public file hashes and two-way relay verified. Chrome menu loads without captured errors. Guest-device performance retest still needed; no measured FPS claim. Refresh both peers and create a fresh yard. Live: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Use ?quality=high to compare previous rendering settings.

# Stage 6 and browser movement fix — published (28 October 2026)

[Stage 6 visual review](2026-10-27-servo-props/review.html) · [Props checkpoint](2026-10-27-servo-props/CHECKPOINT.md) · [Movement fix](2026-10-28-browser-movement/CHECKPOINT.md). User approved commit/push/browser publication; joining-player stutter report led to acknowledged-position reconciliation instead of snapping to delayed host positions. Protocol 21: refresh every peer, create a fresh yard. Native/browser builds pass, 445 core and 240 app tests pass. Public file hashes and two-way relay verified. Live: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Human browser symptom retest remains requested; device/frame-rate diagnosis not established. Unrelated concept art/ preserved and excluded.

# Bush Servo Stage 5 — browser published

[Actual before/after and motion](2026-10-27-servo-lighting/review.html) · [Recovery checkpoint](2026-10-27-servo-lighting/CHECKPOINT.md). Warm scoped Smoko lighting, brighter shade and imported-texture distance filtering. 239 app tests and 439 core tests pass; native/browser builds pass; four matched captures and Smoko delivery verified. No measured frame-time regression, +3.3 MB texture memory. Source committed as 643a58c. Browser publication subsequently approved and completed; public files verified against build, two-way relay passed. Live: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Refresh to load Stage 5. Stage 6 awaits a further request.

# Bush Servo Stage 4 — browser published (27 October 2026)

User explicitly approved commit/push and browser publication. [Actual before/after and motion](2026-10-27-servo-building/review.html) · [Checkpoint and publication verification](2026-10-27-servo-building/CHECKPOINT.md). Stage 4 building, veranda, library timber/pumps, freezer, signage and ute are included. 439 core tests, native and browser builds pass. Live files matched the bundle; two-way public relay passed. Current browser: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Refresh and choose Smoko Yard. The usage-threshold reminder is paused. Stage 5 has not started.

# Bush Servo Stage 3 — browser published (27 October 2026)

User approved commit/push and browser publication. Stages 2–3 now included in the live browser at https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Built files verified against public responses, two-peer relay verified. [Checkpoint](2026-10-27-servo-landscape/CHECKPOINT.md). Refresh all peers for protocol 19. Stage 4 not started. Earlier review statuses below are historical.

# Bush Servo landscape — ready for review (27 October 2026)

Stage 2 committed and pushed as 058ed37. Stage 3 gum trees, ridges, scrub and eight perimeter boulders are implemented; native release and 439 core tests pass. [Review](2026-10-27-servo-landscape/review.html) · [Recovery checkpoint](2026-10-27-servo-landscape/CHECKPOINT.md). Browser wasm compilation passed. Stage 3 uncommitted; public browser unchanged.

# Bush Servo surroundings — ready for review (27 October 2026)

[Checkpoint](2026-10-27-servo-surroundings/CHECKPOINT.md) · [Actual before/after and motion](2026-10-27-servo-surroundings/review.html). Stage 2 sandstone edging and native planting; four clear approaches. 438 core tests and native/browser builds pass. Matched mean frame time within 2.3% of baseline. Protocol 18 for new shared stone colliders. No commit/push/publication; live browser remains Stage 1.

# Bush Servo browser updated — 27 October 2026

Ground-and-road browser build published at https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Public files match built artifacts; WebSocket relay verified. Temporary Cloudflare link requires the Mac/server/tunnel running. [Details](2026-10-26-servo-ground/CHECKPOINT.md).

# Bush Servo ground and road — commit approved (27 October 2026)

User approved committing and pushing the reviewed stage 1 update. All reviewed hashes match. Browser bundle/publication remains unchanged; earlier review statuses below are historical.

# Bush Servo ground and road — ready for review (27 October 2026)

[Checkpoint](2026-10-26-servo-ground/CHECKPOINT.md) · [Actual comparisons and moving capture](2026-10-26-servo-ground/review.html). Stage 1: red earth, worn asphalt, dusty uneven shoulders, cracks and small stones. Native release, browser wasm compilation and focused tests passed; +100 delivery fixture verified. No measured performance regression in local samples. Published browser files unchanged; no commit/push.

# Blender shelter publication approval — 26 October 2026

User approved committing and pushing the reviewed shelter and rebuilt browser assets. Earlier review statuses below are historical.

# Blender Smoko shelter — ready for review (26 October 2026)

[Checkpoint](2026-10-26-blender-shelter/CHECKPOINT.md) · [Actual game comparisons and delivery sequence](2026-10-26-blender-shelter/review.html). Live Blender shelter imported into Smoko; verified starting archive, focused tests, native/browser builds and +100 delivery fixture. Model-only upgrade; other scenery and game rules preserved. No commit/push/publication.

# Bush Servo publication approval — 25 October 2026

User approved commit and push of the reviewed first pass, including browser artifacts. Source/build hashes match the verified state. Earlier review statuses below are historical.

# Bush Servo Smoko — first section ready for review (25 October 2026)

[Checkpoint](2026-10-25-bush-servo/CHECKPOINT.md). Verified 4,969-file backup. Separate Bush Servo scene and browser selector, unchanged delivery rules; 673 tests and native/browser builds pass. [Actual captures and motion](2026-10-25-bush-servo/review.html). Mean native frame time 7.4–7.5 ms. First section only; richer scenery/detail remains pending review. No commit/push.

# Publication approval — 25 October 2026

User approved committing and pushing the charged-throw recovery and optional Backyard graphics preview, including rebuilt browser files. Source hashes match the verified builds (669 tests passed). Earlier review statuses below are historical.

# Backyard graphics preview — ready for review (25 October 2026)

[Checkpoint](2026-10-25-backyard-finish/CHECKPOINT.md). Verified backup; first switchable lighting/materials pass. [Actual before/after and motion](2026-10-25-backyard-finish/review.html). 669 tests and native/browser builds pass; matched mean frame times +0.4% / +2.8%. Use --backyard-finish or ?backyard_finish=1. Default look and pending throw work preserved. No commit/push.

# Charge trajectory and smooth recovery — ready for review (25 October 2026)

[Checkpoint](2026-10-25-throw-recovery/CHECKPOINT.md). Higher charge arcs, smooth character return and gentle camera lean. 669 tests and native/browser builds pass. [Actual clips](2026-10-25-throw-recovery/review.html). Starting from committed c720bf2; no commit/push.

# Animation, magpies and cartoon finish — approved for publication (25 October 2026)

[Checkpoint](2026-10-25-animation-graphics/CHECKPOINT.md) · [Actual clips and comparison](2026-10-25-animation-graphics/review.html). Verified full backup; 668 tests, native/web builds pass. Charge lean/fall/recovery, belly/walk polish, Backyard/Teams magpie rescues and flock, subtle graphics/UI finish. Matched mean frame time 8.859 → 8.890 ms. Browser menu/game loads; automated pointer-lock limitation documented. User approved committing and pushing, including the rebuilt browser version and With mates settings relocation. Previous shirt/Race and menu updates were committed and pushed as b7f8b44.

# Shirt names and Race interactions — complete, awaiting review (24 October 2026)

[Recovery checkpoint](2026-10-24-race-shirt/CHECKPOINT.md). Verified full backup includes the pending main-menu update. Global rear shirt names; Race carry/throw, shorts tug, grass return rules and scenery. 664 tests and native/browser builds passed; actual captures, motion clips and matched performance checks complete. [Review](2026-10-24-race-shirt/review.html). No commit or push.

# Main menu and hosting flow — complete, awaiting review (24 October 2026)

[Checkpoint and recovery](2026-10-23-main-menu/CHECKPOINT.md) · [Actual menu captures](2026-10-23-main-menu/review.html). Dedicated main menu, child pages and lobby; hosting waits for server acknowledgement, and connection errors remain visible. 657 tests plus native/web release builds passed. Nothing committed or pushed.

# Online publication verified — 23 October 2026

Game commit `04b69f4` is on `origin/main`, including the host-map menu update. Cloudflare browser page and two-way WebSocket relay verified; macOS/Linux/Windows CI passed. [Publication details and temporary link](2026-10-21-online-publication/CHECKPOINT.md). [Latest host-menu verification](2026-10-22-host-menu/README.md). Earlier review statuses below are historical.

# Publication approval — 21 October 2026

User approved committing and pushing the reviewed Cartoon Backyard, Unarmed Pickup, Natural Graphics and Walking/Turning updates, and selected Cloudflare for online browser access. [Online publication and verification](2026-10-21-online-publication/CHECKPOINT.md). Historical review statuses below record the time each milestone was completed.

[Walking and turning — complete, awaiting review](2026-10-21-walking-turning/CHECKPOINT.md). [Actual gameplay and controlled animation clips](2026-10-21-walking-turning/review.html). Directional planted steps, eased body turns/start-stop transitions and gentler camera bob. 650 tests and release passed; paired performance 9.165 → 9.201 ms. Full backup and guarded rollback verified. No commit/push.

[Natural graphics — complete, awaiting review](2026-10-21-natural-graphics/CHECKPOINT.md). [Actual before/after and moving-camera clip](2026-10-21-natural-graphics/review.html). Calmer greens, softer sunlight/texture detail and smooth foliage/clouds. 647 tests, release build and five-mode launch/capture checks passed. Performance comparable in repeated gameplay checks; full backup and guarded rollback verified. No commit/push.

[Unarmed hit pickup — complete, awaiting review](2026-10-21-unarmed-pickup/CHECKPOINT.md). Empty-hand hits now stun for 2.5 seconds, allowing existing F pickup; host-validated guest unarmed hits and victim replication included. 647 tests and release build passed. Full backup and guarded rollback preserve pending Cartoon Backyard work. Protocol 14; no commit/push.


[Cartoon Backyard — complete, awaiting review](2026-10-20-cartoon-backyard/CHECKPOINT.md). [Actual comparisons and motion](2026-10-20-cartoon-backyard/review.html). Verified 21 October 2026: 642 tests, release build, real local multiplayer guest throw and appearance replication, direct Face length slider interaction. Graphics, balloon/throw polish, cover and Backyard-only magpie ready. Full backup and guarded rollback verified. No commit/push.



# Publication approval — 20 October 2026

User approved committing and pushing the height slider, clothesline clearance, stacked-box collision and graphics update. All 15 source files match the reviewed state: 633 tests and release build passed, with actual captures and local multiplayer verification. Historical review notes below retain their original status. Full backup and guarded rollback remain available.

[Height slider, clearance and graphics — complete, awaiting review](2026-10-19-height-graphics/CHECKPOINT.md). [Actual graphics comparison, previews and climbing clip](2026-10-19-height-graphics/review.html). Verified 20 October 2026: 633 tests, release build, all-mode smoke captures and local host/guest appearance replication passed. Full pre-edit backup and guarded rollback available. Nothing committed or pushed.

# Publication approval — 19 October 2026

User approved committing and pushing Race completion, stronger balloon splashes and animation polish. Final verification: 629 tests passed and optimized release build passed. Earlier entries below retain milestone-time review and recovery notes. Full pre-animation backup remains outside the repository.

[Animation polish — complete, awaiting review](2026-10-19-animation-polish/CHECKPOINT.md). [Actual animation preview](2026-10-19-animation-polish/review.html). Full project backup made first; 629 tests and release passed. Gameplay rules and assets preserved. No commit/push.

[Race completion and stronger balloon splash — complete, awaiting review](2026-10-18-race-completion/CHECKPOINT.md). [Actual captures and clips](2026-10-18-race-completion/review.html). 626 tests and release build passed; local host/guest verified. Guarded rollback available. Starting from published commit 39fad79; no new commit/push.

# Publication approval — 18 October 2026

User approved committing and pushing the integrated pending update: Race foundation, Smoko recovery timing, lighting/grass and water balloons. Earlier entries below record milestone-time status.

# Latest recovery checkpoint

[Water balloons — complete, awaiting review](2026-10-17-water-balloons/CHECKPOINT.md). [Actual captures and replay](2026-10-17-water-balloons/review.html). Four-second wet effect at 80% speed; 621 tests and release build passed. Guarded rollback verified; prior pending work preserved. No commit/push.


[Grass and shade — complete, awaiting review](2026-10-17-grass-shade/CHECKPOINT.md). [Actual comparisons](2026-10-17-grass-shade/review.html). Denser grass, warm light and canopy shade; 615 tests and release build passed. Verified selective revert; no commit/push.


[Lighting and materials — completed, awaiting review](2026-10-17-lighting-materials/CHECKPOINT.md). [Actual before/after comparison](2026-10-17-lighting-materials/review.html). Verified selective revert available. Release build and 200 app tests passed. No commit/push.


[Smoko longer knockdowns — 8-second pickup window](2026-10-16-smoko-recovery/CHECKPOINT.md). 200 app tests passed; release build log in that folder. Awaiting playtest/review. No commit or push.

[Race foundation — triple-length oval complete, awaiting review](2026-10-16-race-foundation/CHECKPOINT.md). [Race captures](2026-10-16-race-foundation/review.html). Preserve the uncommitted race work. Its saved patch is the pre-Smoko-timing revision; use the latest Smoko checkpoint for current changes. Sideline scoring/upgrades remain later milestones.

[Planned official update and Solana project direction](2026-10-16-official-update/UPDATE.md) — saved locally; release and token scope pending.
