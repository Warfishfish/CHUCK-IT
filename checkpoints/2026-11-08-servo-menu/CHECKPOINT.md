# Servo placement study and map overviews · 8 November 2026

Starting commit 374f0db. Preserve all pending road and gum-shade work. Backup /Users/marcus/Claude/australian-bbq-before-servo-menu-20261108 contains source.tar, working.tar, commit and the previous native executable. The archives reconstruct committed and pending source in a fresh directory. The archived prior arena_depth.rs was compared against the starting working copy.

## Item 1 — north-west site A selected and integrated

The user selected A during implementation. The original whole frontage moves by x −6, z +9.5 through its root transform. It includes shop, pumps, signs, ute and veranda fixtures. There is one frontage instance. The procedural fallback uses the same translation. The alternative NE capture flag only applies to screenshots.

bush_servo.rs owns the canonical placement and 22 reachable solid bounds. yard.rs includes those colliders and moves the grill/meat-table solids six metres toward the road. yard_scene.rs translates the BBQ visual root by that same amount. life.rs and bots.rs use the moved meat supply point. Dazza's configurable home and return path in dazza_brain.rs move with the grill. The character's home field is host logic; the normal world snapshots already carry his rendered position.

round.rs excludes blocked Smoko starting positions using the actual layout. Smoko item placement uses the same clearance query so supplies cannot spawn inside the newly solid shop. Existing delivery respawns remain outside its footprint.

The original two-site study is retained below the final comparison. Its diagram shows pre-adjustment conflicts. A and B were visual candidates at that point.

## Item 2 — selected-map menu overview

menu.rs resolves the displayed mode from local setup or the joined guest's host snapshot. Backyard/Teams share the yard, Heist shows its bases, Smoko creates its imported map and Race creates the oval. Setup refreshes the Yard description for mode-specific props. yard_scene.rs uses the displayed mode for scene lifecycle and visibility. grass.rs uses it for grass generation. servo_finish.rs and arena_depth.rs apply Smoko's look and shade during its overview.

The menu camera frames Smoko and Race differently. Solo setup gives the map more screen space; the avatar portrait appears on Main and Customise. Existing local map/role controls remain the normal entry point.

online.rs includes the host's selected mode in existing lobby world snapshots and applies that selection to the guest's menu controls. online_world.rs exposes its existing mode encoding within the app. Protocol 26 identifies the changed lobby behaviour. The private build uses protocol 26; the public browser bundle remains protocol 25.

## Evidence and verification

review.html contains site comparisons, overhead captures, the diagram and menu-switching recording. Browser evidence includes a real host and guest connected through a private local relay. The host changed Race to Smoko and the guest's environment followed. The final browser pair joined, followed host selection from Race to Smoko, launched Smoko and moved in first person. Guest Race-role preference is available in the lobby for a future Race choice.

The integrated source check passed 247 app tests and 449 core tests, with one existing app test ignored. The integrated suite covers relocated servo bounds, spawn/loot clearance and Dazza’s moved return route. The scripted Smoko delivery finished with +100 and zero remaining carries. Backyard smoke passed. The final identical-camera run after closing browser games measured 8.095 → 7.965 ms mean and 9.886 → 9.662 ms p95, with zero intervals above 16.67 ms in each 600-frame sample. Decoded image bytes and mesh entity counts agree. A single Mac sample does not establish browser multiplayer performance. Release build and capture logs are saved with this checkpoint.

## Recovery

Use the saved archive to extract only files changed by this pass into a staging directory and compare before restoring. In particular arena_depth.rs must come from working.tar because it contains the prior shade work. Restore menu.rs, online.rs, online_world.rs, yard_scene.rs, grass.rs, servo_finish.rs, round.rs, life.rs, bots.rs, bush_servo.rs, dazza_brain.rs, yard.rs and net.rs from source.tar. Preserve all other pending assets and the prior checkpoints.

Local testing from the rust directory uses ~/.cargo/bin/cargo run -p bbq_app --release -- --menu. Private browser preview http://localhost:3040/?perf=1 while its server remains running. Public browser has not been updated. Review before committing, pushing or starting another backlog item.
