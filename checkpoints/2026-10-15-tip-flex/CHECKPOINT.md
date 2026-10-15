# A little more tip flex

15 October 2026 — complete, ready for user review. User requested a modest increase in the firm-base/progressively softer shaft/delayed tip effect. Starting HEAD dc75e0bb7089d2358954022aa642dc536921a829. Previous implementation hashes verified unchanged. Full uncommitted Rust state preserved in starting-work.tar.gz. No commit/push.

Scope: modest upper-shaft flex increase, slightly softer held spring and longer tip response. Preserve firm base, bend limit, geometry, materials, noodle and controls. Compare actual held motion and performance.

## Changes

Only `rust/crates/bbq_app/src/items_view.rs` changes. Silicone held stiffness multiplier 0.52 → 0.48, damping multiplier 0.85 → 0.80. Joint gain array changes [0, .16, .32, .49, .66, .82, .96] → [0, .16, .34, .54, .75, .94, 1.10]. Tip follow rate decreases from 10.8 to 9.0 per second (approximately 93 → 111 ms time constant); base rate stays 24. Base gain stays zero; held bend limit stays .42. No new assets, materials, runtime systems, collision rules or noodle changes.

182 app tests passed, one existing ignored test. Existing 30/60/120 Hz settling and boundedness checks retained unchanged. Core code is untouched; its preceding 411 tests remain the prior validation, not a fresh run in this milestone. `git diff --check` passed. Recovery patch applied to archived starting source in an isolated directory and verified byte-for-byte.

## Recovery

Use a separate clean checkout at the recorded HEAD. Extract starting-work.tar.gz at that root for the complete prior uncommitted implementation, then apply implementation.patch for this adjustment. Verify from that root with `shasum -a 256 -c /path/to/checkpoint/FILES.sha256`. Never overwrite later active work. Keep this checkpoint folder separately from the starting archive.

Launch:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky
```

Before held clips are the preceding soft-silicone after sequence, whose source hashes were verified unchanged before editing. Baseline executable is temporarily /tmp/bbq-tip-before. Restore baseline durably using HEAD + starting-work archive.

## Completed verification

Release build passed (16 existing app warnings). The 75-frame held sequence and separate 75-frame drop sequence completed without logged errors. Reviewed before/after held contact sheets and drop frames at 1.67 and 4.93 seconds: modest upper-shaft silhouette change, steady base, no visible mesh breakup; dropped items settled. Existing screenshot/window teardown warnings remain. Actual game captures use repeatable scripted movement, not a human playtest.

Fresh performance comparison on this Mac, 1280×800, same 11-bot heist scene, 300 warmup + 600 measured frames, fixed 60 Hz simulation, uncapped presentation:

| Measurement | Before | After |
|---|---:|---:|
| Mean frame interval | 8.114 ms | 8.083 ms |
| Mean FPS | 123.25 | 123.72 |
| 95th percentile | 11.865 ms | 10.597 ms |
| Frames over 16.67 ms | 1 / 600 | 0 / 600 |

No meaningful performance regression in this sample. Meshes 2,881; materials 905; mesh entities 7,389; image CPU bytes 42,129,816 — identical before/after. These are wall-clock intervals, not GPU timestamps; small differences are normal run-to-run variation.

Review: open review.html for actual animated before/after clips and synchronized frame comparison. Evidence includes fresh benchmark logs and all 75 held/drop PNGs. Before held frames were reused from the verified unchanged previous milestone; before performance was rerun with its saved executable. After captures use the new release build.

Limitations: silicone motion is a bounded skeletal deformation approximation, not full soft-body collision. Existing aiming and collision rules are unchanged. User feel and multiplayer playtesting remain outstanding. No additional models or Blender operations were needed for this tuning.

Next step: user review/playtest. Await review before further tuning, committing, pushing, or another milestone. No commit or push performed.

## Publication approval — 15 October 2026

User requested “commit and push” after reviewing the result. This supersedes the review hold above. The accumulated implementation and assets are committed together; numbered raw motion frames remain local and are excluded from Git. Compressed comparison clips are included; synchronized frame controls require the local raw sequences. See ../LATEST.md.
