# Gameplay settings, car traversal and carry camera — recovery checkpoint

**Status:** implementation and automated verification complete; committed and pushed 9 November 2026.

## Starting point

- Repository: `/Users/marcus/Claude/australian-bbq`
- Branch / commit: `main` / `40f20b6ff7359ef3ec7020087a9473e23fc0ab48`
- Launch command: `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --menu`
- Existing uncommitted work: the graphics-finish pass and the additions to `NEXT_UP.md`; preserved as a binary patch plus untracked-file snapshots in `recovery/`.
- E19.3 comparison pack is a separate unfinished review task. No gameplay edits should be mixed into its screenshots or marked as graphic approval.

## Requested scope

1. Let Solo players and multiplayer hosts choose the number of bots. For team modes, allow a clear bot distribution between the active teams; host remains authoritative and guests see the host's configuration.
2. Let players jump and land on the existing car in the relevant map. Inspect its rendered bounds and current solids first; preserve its appearance and avoid changing its location.
3. When carried, smoothly move only the carried player's camera to an exterior third-person view, then return smoothly after release, throw, rescue, respawn or round end.

## Recovery

- `recovery/starting-work.patch` contains the starting tracked modifications relative to the recorded commit.
- `recovery/graphics_quality.rs` preserves the untracked graphics-quality source file.
- The complete pre-trim graphics checkpoint snapshot is preserved locally at `/Users/marcus/Claude/australian-bbq-recovery-2026-11-09/graphics-finish-checkpoint.tar.gz` (607 MB). It is outside the repository because GitHub rejects a single file this large. The review page and linked evidence are included in the repository; redundant, unlinked frame captures were removed from the committed review package.
- `recovery/starting-status.txt` and `recovery/starting-commit.txt` record the exact starting Git state.
- Do not use `git reset` or overwrite the graphics-finish checkpoint. Revert only files listed as changed in this checkpoint if abandoning the gameplay work.

## Work log

- Initial repository inspection confirmed existing bot-count setup is stored in `Settings` and `round::Setup`; bot count sync and team assignment run in `round.rs`; online guests already receive host bot/team state through world snapshots.
- Existing grab-and-carry state is `life::Carry`; local camera is in `player::update_camera`.
- The Smoko ute collision bounds are aligned to the Blender-authored bonnet, cab, tray and lower chassis surfaces. Its existing approach gate is opened and bounded so players can reach the vehicle without leaving the map.

## Implemented and checked

- Solo and host controls now expose the total bot count; Teams and Heist also expose per-team counts. Team counts persist in settings, host lobby adjustments update the preview immediately, and the host remains authoritative when starting a round. Guests cannot change the host's counts.
- Team allocation preserves the odd-player Wildcard rule while respecting configured bot counts. Even-sized Teams matches fold a human Wildcard into the smaller side when appropriate.
- The Smoko ute has walk-blocking / jumpable collider tops and the north fence has a visible bounded access gate. A focused movement test verifies gate access, fence blocking elsewhere, the ute surfaces and a jump onto the tray.
- While the local player is host-authoritatively carried in Smoko or Race, the camera eases to an exterior angle framing the carrier and smoothly returns to first person on release. This uses existing carry state, with no new per-frame network traffic.
- `cargo test -p bbq_core -p bbq_app -- --quiet`: 714 passed, 1 existing ignored, 0 failed.
- `cargo build -p bbq_app --release`: passed.
- `cargo build -p bbq_app --profile web --target wasm32-unknown-unknown`: passed. The public browser bundle was not rebuilt or published.
- The before/after graphics review remains incomplete for E19.3 (no matched BBQ comparison). This does not block the requested source commit; the limitation is still recorded in the graphics checkpoint.
- Manual in-game verification of the new sliders, multiplayer camera view and car climbing was not captured in this pass. Automated menu/team, camera geometry and movement tests pass.

## Changed files

Gameplay: `rust/crates/bbq_app/src/menu.rs`, `round.rs`, `player.rs`; `rust/crates/bbq_core/src/bush_servo.rs`, `movement.rs`, `yard.rs`.

Preserved graphics pass and review: `GRAPHICS_2C.md`, `RUST_REWRITE_PLAN.md`, `rust/crates/bbq_app/src/{fx.rs,graphics_quality.rs,main.rs,models.rs,shapes.rs,shot.rs,ui.rs,visual_review.rs}`, `checkpoints/2026-11-08-graphics-finish/`. `NEXT_UP.md` and `checkpoints/LATEST.md` were updated.

## Verification and next step

Run locally with `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --menu`. To roll back the gameplay portion alone before its commit, restore the gameplay files listed above from `40f20b6ff7359ef3ec7020087a9473e23fc0ab48`; do not reset the repository, because that commit predates the preserved graphics pass too. The pushed commit can be reverted normally if desired.
