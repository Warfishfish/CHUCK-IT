# Two-slot pickups and catches · 8 November 2026

Starting commit b6d4b1f. Backup /Users/marcus/Claude/australian-bbq-before-secondary-slots-20261108 includes source.tar, saved commit, original native binary and verified untracked concept references in references.tar. The original hands.rs was compared byte-for-byte with the archive before editing.

## Rules

The first acquired item is equipped. Acquiring another item fills reserve while preserving the current weapon. Q and numbered slot selection keep working. Two items is the maximum. A ground pickup at capacity stays in the world. A valid catch at capacity awards normal catch credit once and drops the arriving item harmlessly at the catcher's feet. Both existing slots stay intact. The arriving item cannot be picked back up by that catcher for 2.5 seconds.

## Changes

hands.rs keeps selection on acquisition and rejects duplicate item IDs. game.rs handles the exact item in each catch event, inserts it through the shared slots or drops it when full. Repeated events cannot resolve the same catch twice. Player, bot and remote-player catches use the same path.

online_world.rs rebuilds the guest inventory while retaining a valid prior selection even when a new item arrives. Remote puppets still follow the host's selected item. Existing pickup capacity checks, Race replenishment, Heist objectives and Q switching use the same two-slot representation.

Protocol 27 identifies the changed inventory behaviour. Use matching builds for host and guest.

## Verification

699 tests passed, with one pre-existing app test ignored. New checks cover actual automatic pickups at one and two items, actual flight/catch resolution with zero/one/two items, safe overflow, repeat resolution, duplicate item IDs and guest snapshots that add reserve and preserve Q selection. The existing Race, Heist, Smoko, movement and weapon tests also pass.

Native and WebGPU release builds passed. In actual solo Chrome play, walking over a teddy equipped slot 1. Picking up the water balloon filled slot 2 while the teddy stayed selected, then Q equipped the balloon. Three captures show this sequence. The round timer reached zero during the Q capture. The browser overlay showed roughly 75 FPS on this Mac; this is a smoke check rather than a matched performance benchmark. Overflow catches and guest inventory replication were exercised by the focused gameplay tests.

Private browser preview http://localhost:3041/?perf=1 while its local server runs. Public browser files remain at protocol 26 pending review.

## Recovery

Restore rust/crates/bbq_core/src/hands.rs and net.rs plus rust/crates/bbq_app/src/game.rs and online_world.rs from source.tar into a staging directory, compare against later work and copy them back together. Rebuild both clients after rollback. Previous browser deployment remains recoverable from the committed release.

Local test command from /Users/marcus/Claude/australian-bbq/rust is ~/.cargo/bin/cargo run -p bbq_app --release -- --menu.

## Concurrent work observed

Separate lighting edits appeared in lighting.rs, servo_finish.rs and a 2026-11-08-vivid-pop checkpoint during verification. These edits were preserved. The four inventory source hashes are recorded independently in reviewed-sha256.json and verified at completion. Browser captures use the staged inventory build. The shared native executable may be replaced by another build, so the documented cargo launch command is the reliable local entry point.
