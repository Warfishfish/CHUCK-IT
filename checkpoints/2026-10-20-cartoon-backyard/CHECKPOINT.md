# Cartoon Backyard update — 20 October 2026

Complete and awaiting user review. Final interactive verification passed on **21 October 2026**. No commit or push. [Actual captures and comparisons](review.html).

## Recovery checkpoint

- Starting commit: `8badc1bf4a14b2f82fecd39b7d9851022c6f8871` (`main`); tree was clean.
- Full pre-edit APFS backup: `/Users/marcus/Claude/australian-bbq-backup-2026-10-20-044909`.
- Backup verified against Git HEAD and all 75 Rust source hashes, including all project files and builds. No unrelated work overwritten.
- Active repository: `/Users/marcus/Claude/australian-bbq`.

Launch:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

## Implemented scope

1. **Lighting and materials:** slightly gentler warm sun, more ambient fill, reduced vignette and saturation; existing soft shadows retained. Pop style omits noisy wood/bark/plastic normal maps. No new lights, post-processing or generated texture assets.
2. **Sky and buildings:** warmer horizon gradient with subtle sunward haze; smoother shaded cloud puffs; existing weatherboard/corrugated textures reused on neighbouring houses and roofs; broad fascia, window crossbars and sills on the main house.
3. **Balloon and throw:** softer highlight, fuller sagging balloon body, smaller tie and first-person presentation, gentle movement flex and wind-up squash. Slight release reach and longer damped follow-through. Throw trajectory, wet slowdown, damage and scoring preserved.
4. **Face length:** saved and networked slider 0–100%, original shape at 50%. Vertical scale 0.75–1.60, width/depth reciprocal square-root to preserve volume. Existing eyes and accessories deform with the head. Preview framing and standing clearance account for the shape; weapon hit volumes unchanged. Existing character colours, leg slider and animation retained.
5. **Backyard cover:** three low wooden crates with matching visible/collision sizes at (-8,4.5), (7.5,11), (17,-9). No spawn overlap. Only Backyard FreeForAll receives the added cover; no other mode receives it.
6. **Backyard magpie:** reuses one fence bird. Active only in Backyard FreeForAll while playing; disabled in Teams/Heist/Smoko/Race, menus, countdown and results. One peck deducts exactly 10, then the bird retreats. User clarified Backyard only, so teammate rescue mechanics are deferred.

The shared sky and materials naturally also affect reused scenery in other modes. No broader map redesign, extra character features, new sound, new weapon or unrelated scoring change.

## Magpie rules and tuning

`bbq_core/src/magpie.rs`: 16 seconds resting between visits; 1.25-second warning; 19 m/s committed swoop; contact radius 0.70; -10 per successful peck. Standing, non-stunned, non-swimming targets cycle by ID. The aim follows the target during warning, then commits so a moving player can dodge. A close open catch or intersecting live thrown object repels it; a missed throw does not. A successful projectile loses live-hit status, preventing a second hit from the same projectile. Missing/disconnected targets trigger retreat. Round resets clear attack state.

The host resolves penalties and bird motion. Guests draw replicated phase/position/target; flight heading derives from movement. Protocol **13** requires matching builds (Face length adds one appearance byte; bird state adds 17 bytes per world snapshot).

Art tuning: `lighting.rs` Pop rig and sky gradient; `looks_yard.rs` house/cloud/cover builders; `looks.rs` balloon shape; `items_view.rs` held balloon transform; `feel.rs` THROW_SETTLE and THROW_FOLLOW_THROUGH; `appearance.rs` face_scale and FACE_DEFAULT.

## Verification completed

- **642 tests passed:** 212 application + 430 core; one existing ignored test. `tests.log`.
- Optimized release build passed, 17 existing warnings. `build.log`.
- `git diff --check` passed.
- Focused tests cover all 101 Face length values through saved appearance and network serialization, invalid-value clamping, mode-specific cover, single peck/no duplicate penalty, dodge/disconnect/disabled state, repel cancellation, live projectile intersection versus miss, host score/state replication, and countdown/results/menu cleanup.
- Five same-camera before/after views: gameplay, overview, close boxes, buildings and character animation preview. Face length 0/50/100 captured; the long-head hat now fits in the preview.
- Actual 75-frame held-balloon and magpie motion fixtures; 60-frame controlled throw-animation preview. The magpie recording logs one score change from 0 to -10, retaining -10 through retreat/rest. These are real renderer/simulation captures with scripted setup, not ordinary unscripted play or concept images.
- Actual Teams, Heist, Smoko and Race smoke captures; no runtime ERROR in capture logs.
- Two release clients connected through the actual relay and entered Play. Observed host roster contains Face lengths 0 and 100. Guest received warning, swoop and retreat bird states. `network/observed-verification.txt`.
- Full backup and selective rollback validated. Live dry-run passed. Applying rollback in a disposable copy restored all changed source hashes and removed new files. A later edit deliberately caused refusal before any changes. `rollback-verification.txt`.

## Final verification — 21 October 2026

- Corrected `network.py` rerun **passed**: actual host and guest connected, entered Play, host supplied a Teddy, guest released the throw, and host roster replicated Face lengths 0% and 100%. See `network/verification.txt` and fresh logs/captures.
- Direct native UI check **passed**: Face length 100% displayed a long head, a click at 0% displayed a squat head, and dragging to 50% restored the original shape. Existing eyes/accessories followed it. Tested in a temporary macOS app bundle cloned from the verified binary. Original saved settings restored byte-for-byte and hash verified. `slider-ui-verification.txt`.
- Final 29 source hashes still match the tests, release build and rollback manifest. No implementation changes were needed to finish verification; no redundant rebuild performed.
- The earlier failed throw assertion was a test setup issue (Backyard supplied no item), fixed with `--net-give teddy`. The Mac lock was respected; the rerun occurred after unlock.
- **Next step: user review/playtest.** Do not commit, push or begin another stage without approval. Nothing remains blocked on unlock.

WAN latency, packet-loss testing and extended magpie balance testing have not been performed. Existing scoreboard and gameplay tests remain green; smoke captures are not full multiplayer playthroughs. No claim of improved engagement.

## Performance

Same 1280×800 cameras, fixed 60 Hz simulation, uncapped rendering; 600 wall-clock intervals after 300 warmup frames. No compiler ran during these captures. Desktop intervals include system noise and are not GPU timestamps. New cover can change bot paths slightly.

| View | Before mean / FPS | After mean / FPS | Mean change | p95 before → after |
|---|---|---|---|---|
| gameplay | 8.878 ms / 112.6 | 9.427 ms / 106.1 | +6.2% | 12.797 → 13.671 ms |
| overview | 8.960 ms / 111.6 | 9.399 ms / 106.4 | +4.9% | 12.878 → 14.168 ms |
| close | 8.235 ms / 121.4 | 8.867 ms / 112.8 | +7.7% | 12.769 → 13.431 ms |
| building | 8.223 ms / 121.6 | 8.718 ms / 114.7 | +6.0% | 13.419 → 13.706 ms |
| character | 8.854 ms / 112.9 | 9.135 ms / 109.5 | +3.2% | 13.316 → 13.689 ms |

The environment views cost approximately 5–8% more mean frame time in these samples. Asset inspection: gameplay meshes 2939→2953, materials 764→783, retained CPU image storage 41,882,008→42,930,584 bytes (+1 MiB); mesh entities 6513→6500 includes dynamic item effects/despawns and is not a static scene-only count. Added smooth-cloud geometry, materials and cover are bounded, with no new simulation-heavy renderer. These samples remain above 100 FPS, but the cost is reported rather than claimed as a free improvement. Individual contributions were not GPU-profiled.

## Safe rollback

First check without changing anything:
```sh
cd /Users/marcus/Claude/australian-bbq
python3 checkpoints/2026-10-20-cartoon-backyard/revert.py
```
Add `--apply` only when you intend to revert this update, then rebuild. It validates every current source against recorded before/after hashes before modifying any file. Later edits cause refusal. It restores only these 29 sources, removes the two newly added magpie modules, and retains checkpoint evidence. It does not alter Git history or reset unrelated files. `implementation.patch` includes new files; `before-source/` contains the exact prior tracked sources. The full backup is a second recovery option: work from it in a separate folder instead of overwriting later work.

## Exact changed source files

- `rust/crates/bbq_app/src/characters.rs`
- `rust/crates/bbq_app/src/face.rs`
- `rust/crates/bbq_app/src/game.rs`
- `rust/crates/bbq_app/src/hud.rs`
- `rust/crates/bbq_app/src/items_view.rs`
- `rust/crates/bbq_app/src/legs.rs`
- `rust/crates/bbq_app/src/lighting.rs`
- `rust/crates/bbq_app/src/magpie.rs` (new)
- `rust/crates/bbq_app/src/main.rs`
- `rust/crates/bbq_app/src/menu.rs`
- `rust/crates/bbq_app/src/models.rs`
- `rust/crates/bbq_app/src/online.rs`
- `rust/crates/bbq_app/src/online_world.rs`
- `rust/crates/bbq_app/src/player.rs`
- `rust/crates/bbq_app/src/preview.rs`
- `rust/crates/bbq_app/src/round.rs`
- `rust/crates/bbq_app/src/style.rs`
- `rust/crates/bbq_app/src/ui.rs`
- `rust/crates/bbq_app/src/visual_review.rs`
- `rust/crates/bbq_app/src/yard_scene.rs`
- `rust/crates/bbq_core/src/appearance.rs`
- `rust/crates/bbq_core/src/feel.rs`
- `rust/crates/bbq_core/src/lib.rs`
- `rust/crates/bbq_core/src/looks.rs`
- `rust/crates/bbq_core/src/looks_yard.rs`
- `rust/crates/bbq_core/src/magpie.rs` (new)
- `rust/crates/bbq_core/src/net.rs`
- `rust/crates/bbq_core/src/net_world.rs`
- `rust/crates/bbq_core/src/yard.rs`
