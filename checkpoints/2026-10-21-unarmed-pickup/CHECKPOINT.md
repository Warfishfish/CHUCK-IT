# Unarmed hit pickup — 21 October 2026

Complete and awaiting review. **647 tests and the optimized release build passed.** No commit or push.

## What changed

- Empty-handed hits now stun for **2.5 seconds**, up from 0.6. The old 0.6-second effect was too short to approach and was never above Smoko's >0.6-second pickup threshold.
- Use the existing **F** pickup/drag control after the victim lands and while they are still stunned. Reach, drop/throw controls, carry limits and recovery rules are unchanged.
- The existing Cheeky-mode and empty-inventory requirements remain. Damage/scoring, knockback, weapon effects and stun grace/non-stacking rules are preserved.
- Multiplayer guests can send an explicit unarmed-hit request. The host chooses the target and validates range, facing, active play, empty inventory, attacker status, team restrictions and 0.9-second cooldown. No client-supplied victim, score or stun duration is accepted.
- Unarmed hits on remote victims now send their hit/stun notification, so the victim and host agree on the pickup opportunity.
- Protocol **14**: multiplayer participants must run matching builds.

## Testing

647 passed (217 application + 430 core), one existing ignored test. `tests.log`.

New integration tests exercise actual empty-hand input, wait one second for the victim to land, approach and press F in Backyard and Smoko. They also check expiration and misses, guest hit followed by Smoko carry, cooldown replay rejection, invalid mode/state/range/facing/held-item requests, host-to-guest stun notification, and guest input generation. The new action is included in binary codec round trips. Existing weapon, scoring, stun grace and carrying tests pass.

No rendering/model changes, so no new graphics benchmark or concept generation. Multiplayer changes have codec and host/guest integration coverage; a fresh two-computer/WAN session was not run for this small follow-up. Existing non-Smoko online guest carrying is not added by this change: the game already has local/host dragging and a separate multiplayer carrying system in Smoko.

## Try it

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Enable Cheeky mode as before. With empty hands, face an opponent and left-click to hit them. Approach after the shove and press F while they are stunned. Smoko supports the existing carry/drop/throw and delivery objective; Backyard uses its existing drag/drop/throw system.

## Recovery

Starting Git commit: `8badc1bf4a14b2f82fecd39b7d9851022c6f8871`, with all Cartoon Backyard changes pending. Full pre-edit APFS copy:

`/Users/marcus/Claude/australian-bbq-backup-2026-10-21-unarmed-pickup`

Verified all 77 starting Rust source hashes. This backup includes the pending graphics, Face length and magpie update. `baseline.json` and `starting-status.txt` record the starting state. Only six Rust files changed in this follow-up; all other starting Rust sources are preserved.

Guarded selective rollback (dry-run first):

```sh
cd /Users/marcus/Claude/australian-bbq
python3 checkpoints/2026-10-21-unarmed-pickup/revert.py
```

Add `--apply` only when intending to revert this follow-up, then rebuild. All file hashes are checked before any mutation; later edits trigger refusal. Tested live dry-run and exact restore in a disposable copy, including refusal when a later edit was inserted. This rollback preserves the pending Cartoon Backyard update. To revert both stages, revert this follow-up first, then use the previous checkpoint's guarded rollback. Do not overwrite the working project with the full backup if later work exists.

## Exact changed files

- `rust/crates/bbq_core/src/melee.rs` — stun tuning and core expectation.
- `rust/crates/bbq_core/src/net_act.rs` — explicit BareSlap codec and round trip.
- `rust/crates/bbq_core/src/net.rs` — protocol version 14.
- `rust/crates/bbq_app/src/life.rs` — shared unarmed hit resolution, validated remote request and hit notification.
- `rust/crates/bbq_app/src/online_world.rs` — guest input/host routing and multiplayer integration tests.
- `rust/crates/bbq_app/src/game.rs` — local hit-to-pickup, expiry and miss integration tests.

`changed-files.json`, `before-source/` and `implementation.patch` record the exact change relative to the pending pre-edit state, not just Git HEAD.

Next step: user playtest/review. Do not commit or push without approval.
