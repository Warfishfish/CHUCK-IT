# Teddy Heist maze · 14 November 2026

Status: Marcus approved committing and pushing this review on 14 November 2026.

## What changed

- Replaced the center crate cross with eight 3.2 m tall, 0.7 m thick, staggered laser-tag style panels.
- Kept the current oval step perimeter, side bays, 2–4 team bases, base openings, teddy placement and bank rules.
- Reused `Kind::HeistCrate`, so the map uses the existing collision and scene rendering path. The maze panels have two alternating warm orange finishes.
- Added a test for open lane gaps and clearance from every base interior, doorway, teddy spawn and banking radius across 2, 3 and 4 teams.

## Verification

- `cargo test -p bbq_core heist::tests`: 20 passed.
- `cargo test -p bbq_core bots::tests::heist_bots_do_not_run_errands_and_follow_their_objective_goal`: passed.
- `cargo test -p bbq_core --lib`: 452 passed.
- `cargo test -p bbq_app game::tests::heist_bots_play_for_a_while_and_nobody_gets_stuck`: passed.
- Release build succeeds. Captured and visually inspected the running native scene at 1280 × 800. The overhead uses `--mode heist --bots 0 --bare --cam 0,55,0,0,-89,70`; the playable approach uses one bot with `--cam -15,2.2,-18,135,-5,75`. Both are actual captures from the edited build, not the diagram or the old map.

## Review materials

- [Top-down map plan](layout.svg)
- [Native overhead capture](maze-top.png)
- [Native playable capture](maze-playable.png)
- [Review page](review.html)
- Native test command: `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --mode heist --bots 1`

The top-down drawing is a diagram made from the layout coordinates. The two PNG captures are from the native game. Review them before approving the commit.
