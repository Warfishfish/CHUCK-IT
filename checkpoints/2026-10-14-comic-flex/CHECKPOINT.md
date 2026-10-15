# More comic held flex

14 October 2026 — complete, ready for review. User resumed after stopping and asked for more floppy/comical dildo and pool-noodle dynamics. Previous held refinement was too restrained. Active repository /Users/marcus/Claude/australian-bbq, HEAD dc75e0bb7089d2358954022aa642dc536921a829. All existing Rust work is preserved in starting-work.tar.gz and its hash manifest. No commit/push.

Scope: softer held springs, larger movement response and more follow-through, retaining bounded bends, quiet idle, Q switching and unchanged ballistic/hit physics. Reuse the previous after captures as this before baseline: all seven previous implementation hashes verified unchanged. Same scene/camera/input script; no need to rebuild the baseline.

Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky

## Implementation

`rust/crates/bbq_app/src/items_view.rs`: held-only stiffness is 68% of the underlying spring. Held damping changes from material damping +2.0 to +0.25. Turn gain increases 0.24 → 0.38; start/stop gain 0.20 → 0.32; running forces 4/3 → 6.5/5. Held bend limits change from 0.38/0.30 to 0.48/0.40 radians (noodle/rubber spring state, not total tip angle). Static droop increases 12%. No random idle kicks. Flying/ground springs, launch trajectories, collision, catching, Q and other controls are unchanged.

`rust/tools/held_motion_review.py`, `encode_held_motion.py`: accept --output-dir to reuse capture tooling without overwriting earlier milestones. Capture runner also accepts --scene for focused captures/benchmarks.

Existing run/stop tests now use the exact runtime held-response function. Their previous bounds and settling assertions were retained. 180 app + 411 core tests passed, one existing ignored app test. Source changes are presentation tuning, not a new physics solver; no modelling or Blender changes were necessary.

## Recovery

Create a separate clean checkout at the recorded HEAD. Overlay starting-work.tar.gz at its repository root to recover all prior uncommitted implementation. Apply implementation.patch to recover this refinement. From the new root, verify with `shasum -a 256 -c /path/to/checkpoint/FILES.sha256`. The three-file delta has been applied and verified byte-for-byte in an isolated directory. Never overwrite later active changes. Keep this checkpoint folder separately from the starting archive.

## Reproduce

From the repository root: `python3 rust/tools/held_motion_review.py --label after --output-dir checkpoints/2026-10-14-comic-flex/evidence`. Encode after capture/benchmark finishes using `/Users/marcus/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 rust/tools/encode_held_motion.py --label after --output-dir checkpoints/2026-10-14-comic-flex/evidence`.

Before motion clips are reused from the previous verified implementation. The fresh before stress benchmark used /tmp/bbq-comic-before (temporary executable, not a recovery dependency). The full previous implementation is recoverable from this starting archive plus unchanged HEAD files.

Scope boundary: present this refinement for review before commit, push or another milestone. User playtesting remains necessary for the preferred amount of comedy.

## Completed verification

591 tests passed (one existing ignored test); release build passed. Fresh matching stress runs: mean 8.075 → 7.811 ms, 123.84 → 128.02 FPS, P95 11.738 → 8.833 ms. Frames over 16.67 ms: 1 → 0 out of 600. Asset counts and image memory match. These single short runs show no observed regression, but do not establish a real speed improvement.

Build log: build.txt; tests: tests.txt; fresh benchmark and capture logs: before-benchmark.txt, after-captures.txt and evidence/*.log. Final capture runs completed without logged ERROR. A relative output-path issue in the reusable capture runner was corrected by resolving the output directory before launching the game; final captures replaced the failed attempt's logs. Existing checkpoint captures were not overwritten.

Performance: 1280×800, fixed 60 Hz simulation, AutoNoVsync, 300 warmup frames then 600 wall-clock intervals. No build or image encoding during measurement. Not GPU timestamp measurements. Both runs: 2897 mesh assets, 911 materials, 7737 mesh entities, 42,129,816 CPU image bytes. No new runtime assets or paid generation.

Actual 15 fps game sequences (75 frames, 5 seconds) compare idle, walking, running turns and stopping from a fixed position. After contact sheets and full-size samples were inspected: deeper movement bends and a more pronounced rebound are visible; the tested poses stay clear of the crosshair. This is staged input playback, not a live player session. Subjective feel and extreme mouse turns still merit user playtesting.

Open review.html for actual before/after motion, or launch the game using the command above. Final source hashes, recovery patch and gallery links verified. All work remains uncommitted; no push. Next step is user review, not another milestone.
