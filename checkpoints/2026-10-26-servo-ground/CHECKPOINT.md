# Bush Servo ground and road — 26 October 2026
Start: aee3471, clean worktree. Verified git archive /Users/marcus/Claude/australian-bbq-backup-2026-10-26-servo-ground.tar.gz. Restore into a NEW directory, never over active work.
Scope: stage 1 only: red earth, worn asphalt, dusty irregular edges, fine cracks and small stones. No movement, scoring, collision or other-map changes.
Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko --look polished --style pop
Status: complete, awaiting review (27 October 2026). No commit/push/publication authorized.

Implementation: shared physics untouched. An opaque 192-triangle irregular road annulus (Z scale 0.65) replaces the perfect ring. UVs map the 58m-wide baked image. A 1024px RGBA map supplies worn asphalt, variable dust and dusty shoulders; existing mip generation retained; real edge geometry preserves shadows. Red-earth 512px map with coarse variation and fine embedded grit. Up to 96 deterministic tiny stones (max radius 8.7cm, compressed vertically and partially embedded) on the shoulders, clear of the scoring area. Pale circular delivery pad removed visually; yellow score boundary preserved.
Authoring: rust/tools/make_servo_ground.py (Pillow/NumPy, fixed seed). Original make_servo_textures.py now calls it to avoid overwriting the new surfaces with the old ones.
No Blender model change required for this stage. Browser public files deliberately remain at the published version until review; a wasm compile will validate compatibility without republishing.

## Resume verification — 27 October 2026
Native release and two focused collision/clearance tests passed before interruption. All three saved final stills were found to be black; moved with their timings to evidence/invalid-interrupted-captures and excluded from comparison. The complete 75-frame moving-camera sequence is non-black and has been packaged as moving.webp. Its controlled carry/delivery fixture reported final_score=100, carries=0. Capture script now rejects black screenshots and checks lock status before each view.

## Final results
Native release build passed; web profile wasm compile passed on 27 October (browser-build.log). Two focused Bush Servo collision/clearance tests passed. No gameplay source was changed. Backyard launch smoke capture is valid; other modes were not individually replayed in this visual-only pass.
Final after-overview, after-gameplay and after-close PNGs were retaken with no compiler running, validated non-black and visually inspected. Matching before views remain the valid 26 October baseline. Overview mean 7.434→7.040ms; gameplay 8.261→7.349ms; close 8.180→7.213ms. Final p95 9.678–11.968ms. No measured regression; different sessions/thermal conditions mean this is not evidence of a guaranteed speedup. Total image CPU bytes 47,473,916→51,668,220 (+4MiB); mesh entities 7,331→7,426, meshes 2,777→2,780, materials 825→826.
The complete moving capture has 75 valid frames; +100 delivery verified. No new human multiplayer playtest. Review page: review.html (before/after, moving WebP and timing table).
Rollback: extract the starting archive into a NEW directory. To selectively revert, restore the seven source/texture/generator paths in reviewed-sha256.json from that baseline (the newly introduced make_servo_ground.py may be removed after restoring the old generator). Keep any later changes safe. Rebuild native after reverting. Public browser assets were never changed, committed or published during this stage.
Limits: this is a stylised surface pass, not a complete concept match. Broader landscaping, trees, building improvements, lighting and small props remain stages 2–6. No new obstacles or physics. Await user review before commit/push or next stage.

## Commit approval — 27 October 2026
User approved commit and push after testing. All seven reviewed source/asset/generator hashes match. Native release, browser wasm compilation and focused checks remain valid; no code changes since review. This approval packages the source, native assets and review evidence. Published public/rust browser bundle remains unchanged. Earlier review statuses are historical.
