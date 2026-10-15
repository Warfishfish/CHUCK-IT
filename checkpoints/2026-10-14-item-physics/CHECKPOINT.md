# Item physics checkpoint

14 October 2026 — in progress. Marcus approved the cartoon refinement (“yes thats good”) and authorised this next milestone. No commit or push authorised. Active repository /Users/marcus/Claude/australian-bbq, main, starting commit dc75e0bb7089d2358954022aa642dc536921a829, with prior uncommitted visual work preserved.

Starting-work archive and SHA256 manifest preserve the exact implementation. Recover only in a separate clean checkout at the starting commit; never overwrite later work. The preceding cartoon checkpoint remains unchanged. Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release. Assets: rust/crates/bbq_app/assets. Prior validation: 177 app tests passed, release build passed; stress mean 7.734 ms.

Scope: distinguish rubber-like dildo impacts/inertia from soft foam noodle impacts/wobble, improve stable settling. Preserve pre-impact gravity/launch trajectories, aiming, catch/hit radii and network protocol. Existing gravity already differs (18 vs 15.3 m/s²); adding gravity by mass would be inappropriate and would alter aiming. Reuse lightweight rigid movement plus visual spring chains; no expensive soft-body solver or new meshes.

Plan: first build identical deterministic motion-capture fixtures for before/after; then tune contact response and visual tumble/settling; exercise drops, throws, walls, pool and dt stability with meaningful tests. Capture actual motion and benchmark separately. Present for review before commit/push or next milestone.

## Completed implementation — ready for review

Resumed and completed 14 October 2026. No commit or push. Previous graphics work is preserved. The next step is Marcus's review and playtest, not beginning hit feedback.

Changed files in this milestone:
- `rust/crates/bbq_core/src/items.rs`: rubber bounce 0.60 → 0.34; foam 0.45 → 0.16.
- `rust/crates/bbq_core/src/flight.rs`: variant-dependent rubber response; time-based sliding friction; low-bounce cutoff; tests for energy loss, settling at 30/60/120 Hz, unchanged free flight and hit callbacks, wall response and pool flotation.
- `rust/crates/bbq_core/src/looks.rs`: softer noodle spring, increased damping, steadier rubber spring.
- `rust/crates/bbq_app/src/items_view.rs`: persistent visual spin with size-dependent inertia; impact-driven wobble, stable spring substeps and heading-preserving ground transition; support-height correction for rubber variants. Existing hit/catch colliders and protocol unchanged.
- `rust/crates/bbq_app/src/visual_review.rs`: opt-in deterministic drop/throw capture fixtures, active only in shot/review mode.
- `rust/tools/item_motion_review.py`, `rust/tools/encode_item_motion.py`: repeatable capture and actual-frame WebP encoding.

### Verification and evidence

`tests.txt`: 411 core + 178 app tests passed, 1 existing app test ignored. `build.txt`: release passed, existing warnings remain. `git diff --check` passed. `before-captures.txt` and `after-captures.txt`: drop, throw and 11-bot stress captures all passed without logged ERROR. Actual frame sequences and simulation state logs are in `evidence/`. Contact sheets were visually inspected at release, flight, contact and rest; no persistent ground penetration was apparent in the inspected frames. The centre throw also meets the clothesline obstacle.

Open `review.html` for looping comparisons and synchronized frame selection. Images are implemented game captures, not concepts. 75 frames per sequence at 15 fps, 5 seconds. Original frames are 1280×800; WebP previews are 960×600. Capture output is not used for performance measurements.

Drop first recorded Ground state, before → after (resolution 1/15 second): classic 2.07 → 1.20 s; jumbo 2.07 → 1.07 s; noodle 1.53 → 0.93 s. Visual residual flex can continue briefly after physics rests. Full measurements: `evidence/motion-summary.json`.

Matching 11-bot performance, before → after: mean 7.599 → 7.665 ms (+0.9%); 131.60 → 130.46 FPS; P95 8.508 → 8.498 ms. Zero of 600 sampled frames over 16.67 ms in either run. Fixed 60 Hz simulation, 300 warmup frames, AutoNoVsync, 1280×800. Single wall-clock sample runs, not GPU timestamps; no meaningful regression established. Both runs: 2897 mesh assets, 912 materials, 7736 mesh entities, 42,129,816 CPU image bytes. No new runtime assets or services.

Limitations: simple authoritative collider remains; rotation, deformation and support height are visual approximations. Not shape-accurate rigid-body/soft-body physics or synchronized angular state. No live multiplayer session tested. User playtesting should assess hand wobble, catches, charged throws and comic feel. Blender model changes were unnecessary for this milestone; existing segmented meshes were reused.

### Recovery and continuation

Starting commit: `dc75e0bb7089d2358954022aa642dc536921a829`. Restore into a separate clean checkout only:
1. Check out that commit in a separate directory.
2. Extract `starting-work.tar.gz` at its repository root. This overlays the approved uncommitted cartoon/graphics implementation; unchanged files come from the commit.
3. To restore the completed physics milestone, apply `implementation.patch` from that root with `git apply`.
4. Verify `FILES.sha256` using `shasum -a 256 -c` from the repository root. The seven-file delta was applied in an isolated directory and verified byte-for-byte; see `recovery-verification.txt`.

Do not apply `baseline-instrumentation.patch` as well as the implementation patch: it is only the pre-physics capture fixture used for the baseline executable. The full implementation patch already includes those capture changes. Keep the whole checkpoint directory separately when recovering, as it is not contained in the starting archive.

Launch for playtesting:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky
```

Reproduce captures from repository root: `python3 rust/tools/item_motion_review.py --label after`. Encoding needs Pillow; use the bundled Python at `/Users/marcus/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3` with `rust/tools/encode_item_motion.py --label after`. Avoid running builds/encoding concurrently with performance captures. Baseline binary currently exists at `/tmp/bbq-physics-baseline`, but /tmp is temporary; durable source recovery is described above.

Await review before committing, pushing or beginning hit feedback. After approval, continue hit feedback, then backyard detail and lobby polish.
