# Animation, magpies and cartoon finish — ready for review

Starting commit: b7f8b4453bb61f82fc39d3ff89b4725c9e226cb1. Clean working tree before this work. The previous menu/Race update is committed and pushed.

Verified full backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-25-animation-graphics.tar.gz (5375 files, every archived file hash verified). Excludes .git, node_modules and target caches. Extract into a NEW directory for recovery; do not overwrite later changes.

Launch: cd /Users/marcus/Claude/australian-bbq/rust then ~/.cargo/bin/cargo run -p bbq_app --release -- --menu

Scope: tune existing movement/belly springs, charge lean and warning, host-authoritative overcharge fall/recovery; reuse existing impact feedback; cohesive code-driven graphics/UI/props. Backyard solo and Teams magpies: -10 per peck; teammate rescue for normal team attacks, opponent-only rescues during flock events. No unspecified +50 bonus. Preserve other mode objectives and current customisation. No commit/push without review.

## Implemented changes

- Bouncier uneven walking bob, side sway, idle breathing and softer existing belly springs.
- Weight-sensitive charge lean and warning tremble; host-timed overhold cancels the throw, drops the object and recovers after 1.8 seconds. Existing power curve retained.
- Protocol 16: charge start/cancel, host validation and compact charge/recovery/flock state. Existing movement interpolation drives remote animation. Peers must use this build together.
- Backyard FFA/Teams magpies: -10 per peck; normal solo birds retreat, normal Teams birds cling until valid teammate rescue; flock rescues require opponents. Successful live projectile intersection or nearby catch only. Flocks every 65 seconds, capped at 18 seconds and 16 targets. Bots seek eligible rescues. Heist/Smoko/Race excluded.
- Reused existing 70ms hit-stop, squash/stretch, shake and score popups. Added damage-scaled impact sizes and short velocity trails (maximum 64).
- Shared character/UI palette, cached body/head vertex gradients, restrained material highlights, softer wood/building texture contrast and procedural item-icon silhouettes. Existing models, sky, environment arrangement, fonts, shadows, outlines and customisation retained.
- New code is in complete repository files. See changed-files.txt and TUNING.md for exact paths and tuning.

## Verification

668 automated tests pass (237 app, 431 core); one historical ignored test. Host overhold/drop/recovery snapshot, action serialization and rescue eligibility covered. Actual charge/fall/recovery and flock clips use controlled in-engine debug fixtures; they are not generated concepts. The fixture holds the actor stationary to make the charge readable.

Native release and WebAssembly builds pass. Backyard, Teams, Heist, Smoko and Race launched and captured successfully (smoke checks, not full human rounds). Browser main menu → Solo → Backyard → Play verified with actual captures. Browser automation reports a pointer-lock WrongDocumentError, so mouse capture is not claimed verified; game rendering and menu navigation worked. Build logs retain existing warnings.

Matched release measurements: **8.859 → 8.890 ms mean (+0.35%)**, median 7.597 → 7.890 ms, p95 13.659 → 12.748 ms. Neither run had frames over 16.67 ms. 600 wall-clock intervals after 300 warmup frames, 1280×800, AutoNoVsync, same fixed scene/camera. This is a short local comparison, not a GPU benchmark or full multiplayer stress test. Mesh assets 2951 → 2964, materials 785 → 788, mesh entities 6124 → 6851 (includes the bounded precreated flock, mostly hidden outside an event); CPU image bytes unchanged at 41,619,352. No meaningful mean-frame regression observed.

See evidence/*matched.metrics.txt and capture-matched.py. Starting native binary retained at /tmp/bbq-before-animation-graphics. Actual controlled clips are debug captures; performance figures use release binaries. Source and browser hashes recorded separately.

## Limits and review points

Real two-computer latency and human rescue balance need multiplayer playtesting; automated state round-trips do not establish every network condition. Flock frequency/cap and charge tremble should be judged in play. No +50 magpie bonus was added because the clarified rules did not define it. Existing features/faces/clothing were retained; no new character appendages or sounds. The palette centralizes body/UI colours; some environment-specific colours remain local. Asset replacement is documented at the procedural model cache; no new drag-and-drop GLB loader.

## Recovery

Use `python3 checkpoints/2026-10-25-animation-graphics/restore-backup.py /Users/marcus/Claude/australian-bbq-recovered-2026-10-25` to extract into a NEW directory. The helper skips the cached rust/.venv Python environment (its interpreter links point outside the project). The helper refuses an existing destination and leaves this working tree untouched. The archive excludes Git history/build caches; keep the current .git or obtain b7f8b44 separately if a standalone recovered Git checkout is needed. Do not reset or overwrite newer work. Source hashes are in source-sha256.json. No commit or push has been performed.


## Follow-up: settings navigation

Moved the Game settings button from Main menu to With mates in menu.rs. Settings returns to With mates using the existing navigation history. Solo and lobby settings entry points remain available. Existing navigation tests updated for the new parent. This follow-up is uncommitted with the reviewed animation work.

Follow-up verified: all 6 existing menu tests pass; browser build passes; actual browser Main → With mates → Game settings shows Back to with mates. Screenshot: evidence/settings-in-with-mates.png. Native cargo run will rebuild this small menu change.

## Publication approval

User approved commit and push, including the rebuilt browser files and settings navigation follow-up. Source and browser manifests verified before committing. Earlier no-commit notes describe the review stage.
