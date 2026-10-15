# Held motion refinement

14 October 2026. Complete, ready for review. User requested natural holding/running motion with restrained comic flapping, and Q to change weapon. Prior item physics is preserved; no commit or push. HEAD: dc75e0bb7089d2358954022aa642dc536921a829

Recovery: use a separate checkout at that HEAD, overlay starting-work.tar.gz. All modified and new Rust implementation files are archived with hashes. Do not overwrite active work. Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --cheeky

Plan: controlled first-person held-motion baseline, replace random large jolts with movement-driven bounded flex, verify run/stop/turn behaviour, update Q hint (Q already cycles slots), capture and test before review. No model rebuild needed.

Baseline capture adjustment: first diagnostic clips contained bot hits. They will be replaced with isolated captures. Baseline instrumentation also includes the Q controls label; the wobble implementation remains the starting version.

## Implementation

- `rust/crates/bbq_app/src/items_view.rs`: removes random idle nudges and occasional large footstep kicks; uses smoothed movement speed, continuous gait forcing and a small start/stop impulse. Angular input is filtered with lower gain while held. Held springs have extra damping, restrained per-material bend limits and smaller idle droop variation. The spring integrator discards outward energy at its angular limit to avoid stored energy releasing later. The first transform sample no longer creates a false rotation impulse. Existing throw/contact systems and hit/catch geometry are untouched.
- `rust/crates/bbq_app/src/hud.rs`, `menu.rs`, `player.rs`: Q is labelled “Change weapon” in the HUD, menu and developer controls. Q already cycles the two held slots; E, wheel and slot shortcuts are preserved. This does not grant uncollected weapons.
- `rust/crates/bbq_app/src/visual_review.rs`: opt-in held-noodle / held-rubber scripted first-person sequence, isolated from bot attacks. Only runs with shot/review tooling. Camera position is fixed; input speed and look direction are staged.
- `rust/tools/held_motion_review.py`, `encode_held_motion.py`: repeatable captures and actual-frame WebP/contact-sheet encoding.

## Verification

`tests.txt`: 180 app + 411 core tests passed, one existing ignored app test. Added tests verify finite, bounded movement and settling after stopping at 30/60/120 Hz, no random idle kicks, and outward-energy removal at limits. Existing slot-switching test passed. `git diff --check` passed. The initial diagnostic capture included bot interference; all before images/logs were replaced by clean isolated baseline captures. `baseline-build.txt` and `before-captures.txt` confirm the corrected baseline succeeded.

## Recovery

Use a separate checkout at the starting HEAD recorded above; overlay `starting-work.tar.gz`, then apply `implementation.patch` to restore this refinement. Verify from that repository root with `shasum -a 256 -c /path/to/this/checkpoint/FILES.sha256`. Seven-file recovery delta was verified byte-for-byte in an isolated directory (`recovery-verification.txt`). Do not apply the baseline instrumentation patch as well: it is only for rebuilding the old-motion comparison and is already included in the final delta. `candidate-items_view.rs` is the saved refined motion source, also included in the implementation patch.

To rebuild baseline evidence: start from HEAD + starting archive, apply only `baseline-instrumentation.patch`, then build. That patch includes the Q labels so both comparisons share the same UI. Baseline executable is temporarily `/tmp/bbq-held-baseline`; do not rely on /tmp for long-term recovery. Keep this entire checkpoint folder separately; it is not inside the starting archive.

Reproduce after captures from repository root: `python3 rust/tools/held_motion_review.py --label after`. Encode using bundled Pillow Python: `/Users/marcus/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 rust/tools/encode_held_motion.py --label after`. Do not build or encode during performance capture.

This is a visual spring approximation, not full soft-body simulation. Controlled input playback cannot establish subjective feel or live multiplayer behaviour. Playtest holding, running, sudden turns, charging, slapping and Q swapping. Await review before commit/push or another milestone.

## Final result

Release build passed in 2m22s (`build.txt`, existing 16 warnings). Both held-item capture sequences and stress run completed without logged ERROR (`after-captures.txt`). Actual frames, looping WebP previews and contact sheets are in evidence/. Inspected first-person samples cover idle, walk, running turns, stopping and settling. Q hint is readable and does not overlap the inventory at the tested 1280×800 size. Motion is intentionally restrained, with visible residual flex after stopping; subjective comic feel needs user review.

591 tests passed (one existing ignored test); release build passed. Matching 11-bot benchmark: 7.578 → 7.709 ms mean frame time (+1.7%), 131.97 → 129.72 FPS, P95 8.384 → 8.485 ms. Both runs had zero of 600 frames over 16.67 ms. Mesh/material counts and image memory are unchanged. Single runs on this Mac; the small difference does not establish a meaningful regression.

Performance method: fixed 60 Hz simulation, 300 warmup frames then 600 wall-clock intervals, 1280×800, AutoNoVsync. Not GPU timestamps. Builds and image encoding did not run during measurement. Both runs report 2897 mesh assets, 912 materials, 7736 mesh entities and 42,129,816 CPU image bytes. No new runtime assets, Blender meshes, paid services or image generation.

Recovery hashes and local gallery links verified. All requested changes remain uncommitted; no push. Next step: user playtest/review of held motion and Q switching. Do not start another milestone without review.
