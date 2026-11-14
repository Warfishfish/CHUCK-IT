# Teddy Heist map review — 14 Nov 2026

## Browser captures

- [Layout 0 overhead](browser-layout-0-overhead.png)
- [Layout 1 overhead](browser-layout-1-overhead.png)
- [Pool rim and nearby walls](browser-pool-height.png)

The browser build was opened in Chromium from `public/`, started as a solo 1v1 Heist round, and rendered at 1440 × 1000. The first-person-height view was framed from above the pool edge so both the whole rim and neighboring maze panels are visible.

## Route checks

Distances use a 0.25 m grid and a 0.45 m player radius around solid obstacles. Old doorway route was measured from the pre-pass `c0d1756` yard and 13-panel maze.

| Two-team layout | Door to door | Change from old 47.23 m |
|---|---:|---:|
| Old 13-panel map | 47.23 m | — |
| New 0 | 74.33 m | +57.4% |
| New 1 | 103.15 m | +118.4% |
| New 2 | 87.67 m | +85.6% |

A second 0.5 m grid check with all base walls and the complete pool rim found every door-to-door pair reachable in all nine layouts (2, 3, and 4 teams). Doorways, spawn offsets, and bank circles were checked clear of colliders. Pool-rim clearance is checked across the full rectangle, including the 0.3 m coping.

## Verification

- `cargo test --workspace`: 716 passed, 0 failed, 1 ignored.
- Heist bot simulation runs 150 game-seconds and asserts an actual teddy bank.
- Browser JavaScript syntax check passed; both the layout rotation and oval boundary rendered in Chromium.
- The host snapshot carries the chosen layout and guest reconstruction uses that same layout. A dedicated late-joiner test sends three-team layout 2 to a guest and verifies the rebuilt yard matches. Round snapshot encode/decode is also covered by Rust tests.
- Native debug and optimized release builds both launched directly into Heist and stayed running without startup errors. I verified launch and stability, but did not manually play through a full round or inspect native controls. A live host + guest session was not run; the host-to-guest layout path is covered by the late-joiner test.

Scoring, movement speed, and round length are unchanged. Other modes retain their existing shared-yard dimensions.
