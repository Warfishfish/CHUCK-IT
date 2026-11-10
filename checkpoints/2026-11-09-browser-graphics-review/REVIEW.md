# Browser gameplay and graphics follow-up — 9 November 2026

## Browser review

Reviewed the currently published browser game at the temporary link recorded in `checkpoints/LATEST.md`.

- Solo setup showed the bot-count slider at 3 bots.
- In host Teams setup, red and blue bot counts were separately adjustable. Changing them updated the total; the settings were restored to the defaults (2 red, 1 blue; 3 total).
- Bush Servo setup selected Smoko deliveries and a live Smoko round started. Bot interactions were visible in the round.
- A short 0-bot round also started successfully during movement review; host settings were restored afterwards.
- The settings screen was left on Bush Servo with the default three-minute round and three bots.

## Limits of the hands-on check

- I did not positively confirm a clean jump and landing on the ute in the live browser. The focused automated movement test recorded in `checkpoints/2026-11-09-gameplay-settings/CHECKPOINT.md` passes, including the jumpable tray fixture.
- I saw a carry-related hit/feed message during the bot match, but did not get a clear view proving the carried player's exterior camera and smooth return. Treat that camera behaviour as still needing a manual visual check.
- No gameplay code was changed in this review. The release commit and browser publication remain untouched.

## BBQ before/after evidence audit

`checkpoints/2026-11-08-graphics-finish/evidence/approval/before-bbq.png` and `after-bbq.png` have the same SHA-256 (`05832f10cc9a5204c57e71fe0c45af0d44705753e90f44d4d37ed9c44da9591a`). They are duplicate files, not a valid before/after comparison. The earlier BBQ close-up is an actual game capture, but has no matched starting-commit view.

I attempted a matched capture from the graphics pass's starting commit (`40f20b6ff7359ef3ec7020087a9473e23fc0ab48`) using the same screenshot camera and the current graphics build. The earlier release's optimized compile remained in LTO for more than six minutes, so I stopped it to avoid more build time. No source files or browser release were changed. An exploratory current-build screenshot did render, but is not used as comparison evidence because there is no matching baseline.

**Graphics status:** the BBQ comparison remains pending. Next, use a lightweight repeatable capture path (or a manually positioned in-game view) to capture the BBQ at the same scene, camera, resolution, graphics quality and time-of-round from both revisions. Only then decide whether the grill/nearby cooking area needs a visual update.

## Recovery

- This review only adds documentation and the concept-art graphics backlog; there are no gameplay or rendering source edits.
- Browser/game source remains at the existing published commits. Do not roll back gameplay because this visual review was incomplete.
- Local launch: `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --menu`.
- Browser link is temporary and only works while its host Mac, server and tunnel remain active; consult `checkpoints/LATEST.md` for the current URL.
