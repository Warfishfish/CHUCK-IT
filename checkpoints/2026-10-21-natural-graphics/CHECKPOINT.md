# Natural graphics — complete, awaiting review

21 October 2026. Review required before commit/push. No gameplay or character-design edits in this pass.

## Recovery checkpoint
- Starting commit: `8badc1bf4a14b2f82fecd39b7d9851022c6f8871` on `main`.
- Full APFS clone, including uncommitted work, Git metadata and release executable: `/Users/marcus/Claude/australian-bbq-backup-2026-10-21-035503-natural-graphics`.
- All 141 starting Rust source/asset files matched SHA-256 hashes in the backup. `baseline.json` records those hashes; `starting-status.txt` records the pending work.
- Preserves the pending Cartoon Backyard and Unarmed Pickup updates. Those changes are not part of this pass's patch.
- Launch: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`.

## Exact implementation changes
1. `rust/crates/bbq_app/src/lighting.rs`: less yellow sunlight, slightly stronger neutral ambient fill, gentler colour grading and lighter blue sky. Existing lights, shadow resolution, SSAO and antialiasing unchanged.
2. `rust/crates/bbq_app/src/grass.rs`: muted green blade roots/tips. Density, distance culling, geometry, positions and materials unchanged.
3. `rust/crates/bbq_app/src/models.rs`: cooler lawn tint; reduce wood grain contrast to 45% and paling/corrugated colour-map contrast to 65% around each texture's original mean. Applied once during loading, before mip generation, only for polished Pop. Original files, normal maps and alpha are unchanged; no new textures or per-frame texture processing.
4. `rust/crates/bbq_core/src/looks_yard.rs`: muted gum foliage and smooth low-resolution canopy meshes; fuller overlapping cloud puffs with gentle underside fill. Same tree/cloud counts and seeded placement; decorative geometry only.

No changes to scoring, weapons, magpie logic, unarmed pickup, movement, collisions, player appearance/customisation or networking. Shared visual changes naturally appear in modes that reuse these assets. Plain browser/other styles keep their lighting and texture processing; shared cloud geometry is updated wherever used.

## Verification
- `cargo test --workspace`: **647 passed**, one pre-existing ignored test (217 app + 430 core).
- `cargo build -p bbq_app --release`: passed; existing warnings remain.
- `git diff --check`: passed.
- Selective recovery dry-run plus disposable-copy restoration and later-edit refusal: passed. Live files were not rolled back.
- Actual before/after gameplay, overview, building and character captures: complete. Identical camera, seed, resolution and requested simulation frame. Overview recaptured to match the round state; initial evidence retained.
- Teams, Heist, Smoko and Race launch/render smoke checks with bots: passed without runtime ERROR logs. This is not a new full multiplayer or complete-round playtest; networking/gameplay code is unchanged.
- 60-frame moving-camera capture packaged in `drift.webp`; sampled start/middle/end frames and close/gameplay images inspected. Existing thin foliage and distant shadow edges can still alias; no claim of completely eliminating shimmer.
- Review: [actual comparisons and camera clip](review.html).

## Performance and asset cost

| View | Before mean ms | After mean ms | Change |
|---|---:|---:|---:|
| Gameplay | 9.655 | 9.453 | -2.1% |
| Overview | 8.979 | 9.408 | +4.8% |
| Building | 8.220 | 8.392 | +2.1% |
| Character | 8.856 | 8.816 | -0.5% |

600 wall-clock intervals per run; fixed 60 Hz simulation, 1280×800, AutoNoVsync, same camera/scene. Overview's 95th-percentile time rose 11.93 → 13.89 ms; building 12.76 → 13.04 ms. These short CPU/wall-clock samples are not GPU profiling or other-hardware guarantees.

Initial gameplay timing was 8.998 → 12.997 ms. Two additional alternating pairs yielded 9.577 → 9.262 and 9.655 → 9.453 ms, so that large slowdown did not reproduce. The table uses the second pair; initial captures and metrics remain in `initial-captures/`, with all repeats in `timing-repeats/`. No repeat measurements ran alongside compilation/encoding. Ordinary desktop activity cannot be eliminated as a source of variation.

Representative gameplay: 2,953 mesh assets unchanged; materials 783 → 787; image CPU bytes unchanged at 42,930,584. No added texture files, lights or post-processing. Canopy sphere clumps use 100 triangles rather than 80, with smooth shared vertices; total decorative part count unchanged. Runtime mesh entity counts also include dynamic effects and are not a static asset budget.

## Limitations
This is a focused natural-colour/cartoon polish, not photorealism or a full environment rebuild. Tree silhouettes remain stylised, clouds remain inexpensive opaque forms, and existing sharp shadow edges remain. Texture processing occurs once during loading; building detail is intentionally quieter. User playtest and artistic review remain.

## Recovery
First run `python3 checkpoints/2026-10-21-natural-graphics/revert.py` from the repository. Add `--apply` only when you want to restore the four files to this pass's starting state. The script checks every current file and recovery source before making any change, and refuses if later edits would be overwritten. Rebuild after restoring. It deliberately retains checkpoint evidence.

For multiple-pass rollback, restore newest first. To inspect the whole starting project without disturbing current work, use the full backup in its own directory. Do not reset to Git HEAD: that would discard earlier pending updates.

## Resume / review
`capture.py before|after` repeats four controlled views; `smoke.py` captures Teams, Heist, Smoko and Race; `drift.py` captures moving-camera frames. Run captures only on an unlocked Mac. Do not commit, push or extend this pass before review.
