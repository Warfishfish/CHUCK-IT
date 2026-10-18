# Smoko longer knockdowns — 16 October 2026

User requested more time to pick up enemies and deliver them. Built on the uncommitted triple-length race foundation; preserve that work.

## Change
Fresh melee knockdowns and power-throw knockdowns last at least 8 seconds in Smoko. Applies to player, bots and network hit handling. Melee animation durations match; bot power-throw knockdowns use a matching Timber animation. Ordinary stuns, drunk falls, 8-second carry escape, 4-second throw attribution, scoring and respawn protection remain as before. Repeated hits do not refresh the knockout timer. Other modes keep their original combat durations.

Changed app files: smoko_mode.rs, life.rs, bots_app.rs, game.rs, online_world.rs.

## Verification
200 application tests passed, one existing ignored. Includes late pickup after five seconds, successful delivery, no repeated-hit timer refresh, eventual recovery, existing Smoko scoring/carry protections and race regression tests. Release build passed; details recorded in build.txt. No new graphical capture or live two-client session for this timing-only change.

## Test
From /Users/marcus/Claude/australian-bbq/rust run:
`~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko`
Knock an opponent down, approach and press F to pick up; carry into the marked pad, or hold/release F to throw.

## Recovery
Starting commit in starting-commit.txt; starting tracked working changes in starting-tracked.patch; original status in starting-status.txt. Previous race checkpoint contains its verified implementation patch and untracked race sources. Current modified source copies are in source/. To reconstruct the starting state, use a separate checkout at the starting commit, apply starting-tracked.patch, and restore race.rs and the race review script from the prior race checkpoint. Never reset this working checkout or overwrite the unrelated race changes. No commit/push authorized for this change.
