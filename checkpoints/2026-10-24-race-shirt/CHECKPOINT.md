# Shirt names and Race interactions — complete, awaiting review

Starting commit: 35b7e36c98bbd6b5e57cbf0eeb7da61f3e287b60

Verified pre-edit backup: `/Users/marcus/Claude/australian-bbq-backup-2026-10-24-race-shirt.tar.gz` (5288 files; archive contents verified against originals). Includes pending menu work. Excludes Git metadata, node_modules and Cargo target build caches.

Recovery: extract into a NEW directory; do not overlay the active working tree. Restore selected files only after comparing later work.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`

Scope: global rear shirt names; Race carrying/throwing, clothed shorts tug, grass slowdown with ordered checkpoint protection, scenery. No commit/push. Implementation and verification complete; awaiting user playtest.

## Implemented rules

- Rear shirt lettering in all modes, bundled bold font, contrasting keyline, full 16-character names; decal follows the existing shirt and body.
- Race only: grounded nearby racers can be picked up using F. Tap F drops; hold at least 0.25s and release throws. Carry expires after 3s. Released victims have 5s re-grab protection. Host chooses targets. No new scoring.
- R shorts tug: 1.65m front range, 65% pace for 1.25s, 5s actor cooldown, 6s victim immunity. Clothing remains in place with a small lateral waistband spring.
- Grass: 0.6m verge tolerance then 55% pace, without ordinary teleport. A gold ring marks where to return within 3m before progress resumes. Carry/throw also suspends progress until re-entry. Existing sideline impact protection grants grass slowdown grace.
- Lightweight spectator marquees, bunting, potted plants and a distant corrugated shed.
- Network protocol 15: all players need this build. Carry uses existing host pose messages; race snapshots carry tug/re-entry/held-by state.

## Verification

- Workspace tests: **664 passed** (234 app, 430 core), 1 historical ignored asset-generation test. Native release and browser release builds passed. Existing compiler warnings remain.
- Focused coverage: rear shirt placement/name fitting; grass return and gate protection; tug/carry speed bounds; pickup eligibility, throw velocity, carry expiry and re-grab immunity; disconnect/round cleanup; host action validation and snapshot round-trip/application.
- Actual native captures in Backyard, Teams, Heist, Smoko and Race. Race result capture confirms all three bots finished three laps (91.64s, 93.92s, 97.59s); separate sideline results retained.
- Local real host/guest relay test: tug state, pickup owner/victim, drop and re-entry anchor replicated. A second test adds 350ms to Carry release, exercising host-measured hold-to-throw. Both clients clear carried state and retain protected race progress. Test fixtures use explicit --net-race-social; they do not affect ordinary games.
- Browser: main menu → solo → Race → Racer → play; keyboard movement onto grass shows the return marker, and returning clears it. Final concise help text keeps setup controls visible.
- Captures are actual game renders. Carry/tug/grass motion clips use deterministic staged inputs through the real gameplay functions. Grass clip ends after the first successful return, before the scripted input walks across the opposite verge.

## Performance

Matched 1280×800 polished/pop scene and camera, 600 wall-clock frame intervals after warmup; fixed 60Hz simulation, AutoNoVsync, compiler and other test clients stopped. These are application frame intervals, not GPU timestamp measurements.

| Scene | Before mean | After mean | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| Race | 6.919ms | 6.933ms | 10.505ms | 10.826ms |
| Backyard | 8.452ms | 8.321ms | 12.940ms | 12.806ms |

No meaningful regression in this sample. Race adds 92 mesh entities, 12 mesh assets and 15 materials for landmarks. CPU image bytes 41,096,600 → 41,091,480. Treat the small Backyard improvement as measurement noise. Raw metrics are in evidence/*-matched.metrics.txt. The final menu-only wording edit does not affect these gameplay measurements.

## Changed files and tuning

See changed-files.txt for the exact 17 existing files changed by this milestone, plus this checkpoint folder. menu.rs and online.rs already contained pending main-menu work; that work is preserved. ui.rs remains at its pre-update state.

- shirt.rs: rear decal placement, full-name fit, outlined sports lettering and refresh on appearance changes.
- race.rs: grass/re-entry and tug rules, progress validation, bot recovery, HUD, tests. Named constants at the top hold tuning.
- smoko_mode.rs: reused carry controls with Race-only eligibility, duration, pose and throw/recovery rules; Smoko delivery stays mode-specific.
- player.rs / face.rs: shared speed effects and clothed waistband reaction.
- online.rs / online_world.rs / core net.rs, net_act.rs, net_world.rs: protocol 15, authoritative action routing and race-state replication, opt-in network verification.
- yard_scene.rs: lightweight race landmarks and return ring; original track geometry/collision boundary retained.
- menu.rs: concise Race controls/objective help.
- visual_review.rs: opt-in repeatable carry, tug and grass capture fixtures.
- public/rust files: complete rebuilt browser release.

## Limits and next step

Ready for user review; no commit or push. Balance values are an initial tuning pass. Bots follow/recover to the course but do not deliberately initiate the new pranks. The live multiplayer check was local, not an internet latency or capacity soak test. One existing browser-automation pointer-lock WrongDocumentError occurred during testing; keyboard movement and the gameplay checks still worked. No claim of error-free browser pointer lock is made.

Shirt names move to the rear; the existing shirt shape, colours and customisation remain. Characters without the existing shirt receive no new clothing. Existing name length rules remain 16 characters.

## Guarded rollback

Backup: /Users/marcus/Claude/australian-bbq-backup-2026-10-24-race-shirt.tar.gz, verified against 5288 original files before edits. Starting-status.txt and starting.patch preserve the pending menu state.

Run `python3 checkpoints/2026-10-24-race-shirt/rollback.py` from the project to check the guards without changing anything. With `--restore`, it restores only the 17 listed files from the verified backup and refuses if later edits are detected. It preserves the pending main-menu work. Rebuild native afterward. For full recovery, extract the archive into a NEW directory; never overlay later work blindly. Checkpoint evidence is retained after rollback.
