# Graphics finish backlog — checkpoint and recovery

**Status:** implementation, builds, tests, actual captures and live Low/High selector check complete. Ready for review. No commit, push or browser publication.

## Starting point
- Repository: `/Users/marcus/Claude/australian-bbq`
- Branch: `main`
- Starting commit: `40f20b6ff7359ef3ec7020087a9473e23fc0ab48` (recorded in `starting-commit.txt`)
- Starting status: only this recovery folder existed as untracked work when captured; see `starting-status.txt`. The separately present `rust/checkpoints/2026-11-08-graphics-list-progress/` was preserved.
- Launch: `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --menu`
- Latest review: open `review.html` in this folder.

## Work completed
- D15.2: captured actual Backyard magpie perch and projectile reaction frames; the image sequence shows the projectile approaching and the bird lifting/flapping from the fence.
- D15.3: reviewed the Blender-authored `magpie_pop.glb` in the running game. Current still: `evidence/magpie-motion/magpie-view.png`.
- E16.4: paired the existing one-shot grounded fall flecks with a short, soft dirt puff. Both are in High only and share the per-bot one-shot guard; gameplay and scoring are unchanged. Review fixture logs confirm the bot is down and grounded. Actual post-change capture and a short 75-frame sequence are in `evidence/bot-stack-botcam.png` and `evidence/bot-stack-botcam-*.png`; the focused view clearly shows the grounded bot and added dirt puff. Earlier black captures from the locked session are renamed with a `capture-failed-` prefix and not linked.
- E16.6: compared SMAA and MSAA 4× at the same camera/scene. Kept SMAA as default: visually close edges, and it keeps the established SSAO path. MSAA 4× measured faster in the isolated comparison, where SSAO was omitted for both.
- E16.7: added persistent local Low/High quality selection. Low uses smaller shadows and omits SSAO, particles and decals; High retains the full visual effects. The preference is local and does not affect network/gameplay state.
- E17.6–E17.8: retained bevelled geometry and added deterministic subtle vertex colour variation plus seeded weathered cuboid transforms clamped to authored bounds. Shape tests cover bevel orientation/bounds, repeatability and fit.
- E18.4: reran matched 11-bot Heist/Cheeky wall-clock measurements at 1280×800. Matched initial Low 99.79 mean FPS / 11.396 ms p95; High 92.54 mean FPS / 15.217 ms p95. AA SMAA 91.56 mean FPS / 16.374 ms p95; MSAA 4× 98.03 / 12.423 ms p95, without SSAO. Later Low/High refreshes were 43.93 / 42.20 mean FPS while another game process remained open; retained as contaminated samples and not used for the comparison. Mesh/material/entity counts were unchanged between presets. See `evidence/*.metrics.txt`.
- E19.3: added matched actual starting/current chest and character close-ups. Both objects are intentionally unchanged and the captures confirm preservation. E19.4 local visual check is complete; user approval remains pending. The full comparison pack still needs a matched BBQ before/after view.
- E19: inspected the full-yard palette/readability in the matched Low/High captures and added an approval review page. The review page includes the bot spray capture/sequence and settings-screen capture. Low and High were clicked and verified in a temporary isolated copy of the app; actual saved user settings were not modified. Matched before/current native captures for chest and character are now included in `evidence/approval/` and the review page. They are visually unchanged because this finishing pass intentionally did not alter either asset. The wider E19.3 pack still lacks a matched BBQ before/after view.

## Verification
- `cargo test -p bbq_app`: 261 tests run; 260 passed, 1 existing ignored, 0 failed.
- `cargo build -p bbq_app --release`: passed. Existing project warnings remain.
- `cargo check -p bbq_app --target wasm32-unknown-unknown`: passed after final particle change.
- Captures use the native review path at 1280×800 with matching scene/camera. Frame timings are wall-clock intervals on the Mac, not GPU timestamp measurements. Counts stayed at 3,276 meshes, 984 materials, 7,801 mesh entities and 50,598,652 CPU image bytes in the Heist stress captures.
- The Mac was unlocked for final review. Selector interaction was verified in a temporary app copy under an isolated home directory, with no changes to the user’s saved settings.

## Changed source and planning files
`rust/crates/bbq_app/src/fx.rs`, `graphics_quality.rs` (new), `main.rs`, `menu.rs`, `models.rs`, `shapes.rs`, `shot.rs` (repeatable chest capture fixture and banner suppression for review captures only), `ui.rs`, `visual_review.rs`, `GRAPHICS_2C.md`, and `RUST_REWRITE_PLAN.md`.

## Rollback
Before reverting, inspect `git status`. To discard only this pass’s tracked source/planning edits, restore these paths from the recorded starting commit (do not reset the whole repository):

```sh
git restore --source=40f20b6ff7359ef3ec7020087a9473e23fc0ab48 -- GRAPHICS_2C.md RUST_REWRITE_PLAN.md rust/crates/bbq_app/src/fx.rs rust/crates/bbq_app/src/main.rs rust/crates/bbq_app/src/menu.rs rust/crates/bbq_app/src/models.rs rust/crates/bbq_app/src/shapes.rs rust/crates/bbq_app/src/shot.rs rust/crates/bbq_app/src/ui.rs rust/crates/bbq_app/src/visual_review.rs
```

Then remove the new file `rust/crates/bbq_app/src/graphics_quality.rs` only if you intend to fully abandon this pass. Checkpoint/evidence files are review artifacts; preserve or remove them separately as desired.

## Resume
1. Open `review.html` to inspect the actual captures and image sequences.
2. To retest locally, run `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --menu`; Settings now places Low/High at the top.
3. The matched chest/blob captures are in the review page. The wider E19.3 pack still needs a matched BBQ before/after view.
4. E19.4 local look check is recorded as done; Marcus’s review of the captures is still pending.
5. This pass remains uncommitted and unpublished pending review.
