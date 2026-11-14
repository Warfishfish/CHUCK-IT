# Expanded Teddy Heist map · 14 November 2026

Status: implemented in the Rust desktop game and the browser game; local review capture included. Ready to review, not yet committed.

## Changes

- The playable yard bounds expand from 66 × 48 m to 80 × 58 m. These shared bounds apply to all modes; Free-for-all and Teams keep their existing object layouts.
- Two-team bases move from x ±20 m to x ±26 m. Three- and four-team layouts also move out to use the larger space.
- The old center cross becomes a 13-panel, 3.2 m high laser-tag maze. Staggered openings and a short center divider make runners turn, with more than one route around the main lanes.
- Both the Rust and browser maps use the same maze positions, team base locations and yard bounds.
- Browser prototype label is 0.23.7.

## Verification

- `cargo test -p bbq_core`: 452 passed.
- `cargo test -p bbq_app game::tests::heist_bots_play_for_a_while_and_nobody_gets_stuck`: passed with the expanded map.
- `node --check` on the browser game's inline script: passed.
- Release build and native Heist overhead capture: passed. The capture shows the expanded arena, both bases and maze panels.
- A two-player online match has not been checked hands-on.

## Full-game local test

```sh
cd /Users/marcus/Claude/australian-bbq/rust
cargo run -p bbq_app --release -- --menu
```

Choose Teddy Heist and try a run with a few bots. The native map and maze capture is in [review.html](review.html).
