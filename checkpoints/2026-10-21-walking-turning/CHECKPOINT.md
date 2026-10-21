# Walking and turning — complete, awaiting review

21 October 2026. No commit or push. Earlier pending Cartoon Backyard, Unarmed Pickup and Natural Graphics work preserved.

## Recovery
Full APFS clone of the project, including uncommitted work, Git metadata and the starting release executable: `/Users/marcus/Claude/australian-bbq-backup-2026-10-21-220805-walking-turning`. Verified all 77 Rust source hashes. Starting commit `8badc1bf4a14b2f82fecd39b7d9851022c6f8871`; see `starting-status.txt` for pending work.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`.

## Changes
- `rust/crates/bbq_app/src/locomotion.rs` (new): presentation-only gait state, eased shortest-path yaw, speed/start/stop blending, planted-foot stance and smooth lift/return, directional footsteps and a restrained pivot shuffle. Named constants control response, stance, cadence, bob and camera amplitude. Supports existing leg-length values.
- `rust/crates/bbq_app/src/characters.rs`: connect gait to visible avatars (bots and replicated players), use the eased facing for turn lean, replace ordinary walking bob, and retain existing throw/hit/down/emote poses. State keyed by player ID; cleared outside active rounds and on large position changes. Attack poses use immediate facing. Add an opt-in `--pose locomotion` review fixture with three leg lengths; it is inactive in ordinary gameplay.
- `rust/crates/bbq_app/src/player.rs`: first-person walking bob is 70% of its former amplitude, easing in/out with speed. Mouse turning remains immediate.
- `rust/crates/bbq_app/src/main.rs`: register the new module.

No movement physics, speed, jump, steering, aim, collision, score or network protocol changes. Existing leg customisation, natural graphics, unarmed pickup and effects preserved. No new assets, lights or draw calls from the animation state.

## Verification
- 650 tests passed: 220 app + 430 core; one pre-existing ignored test.
- New tests check shortest-path angle wrapping, 30/144 Hz turn agreement, stance contact and cycle continuity, lateral footsteps at 0/50/100 leg lengths, stopping, teleport reset and immediate facing for action poses.
- Guarded rollback dry-run and disposable-copy restore/later-edit refusal passed; working files not reverted.
- Release build passed (2m 28s); existing warnings remain. `git diff --check` passed.
- Actual normal gameplay before/after: 120 frames each, same camera/bot setup. Controlled new-animation preview: 200 frames, showing 0/50/100 leg lengths walking a curve, stopping and pivoting. Sampled poses inspected; packaged clips in `review.html`. Runtime capture logs contain no ERROR messages.
- Performance: before 9.165 ms mean / 13.381 ms p95; after 9.201 ms mean / 13.628 ms p95. Mean +0.4%, 109.11 → 108.69 FPS. No frame over 16.67 ms in either 600-interval sample. Same 2,953 meshes, 787 materials and 42,930,584 image CPU bytes. Single paired test on this Mac; normal desktop variability applies. Measurements run without concurrent builds or clip encoding.
- [Actual motion comparison and controlled leg-length preview](review.html).

The review remains open for the user's playtest. No commit/push or unrelated milestone started.

## Limits
This is a modest procedural animation improvement, not a full motion-capture or world-space foot-locking system. Some foot sliding at sprint speed or abrupt direction reversals can remain. Game controls and first-person turning are deliberately responsive. No fresh WAN multiplayer session is claimed; the same presentation system renders replicated players.

## Rollback and resume
Run `python3 checkpoints/2026-10-21-walking-turning/revert.py` first to check. Add `--apply` to restore only this pass's starting sources and remove the new module, then rebuild. The script checks all files before modifying any and refuses if later edits would be lost. It keeps evidence files. Restore newest passes first; never reset to Git HEAD to revert this pass, because earlier work is also pending.

`capture.py before|after`: same normal gameplay camera/bot setup. `controlled.py`: new implementation's scripted walking/turning preview at 0/50/100 leg lengths; it is a test fixture, not an ordinary match. `performance.py before|after`: identical camera, fixed simulation and 600 wall-clock intervals. All graphical capture scripts require an unlocked Mac.
