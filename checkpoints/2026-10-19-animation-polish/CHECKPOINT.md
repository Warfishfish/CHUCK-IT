# Animation and game-feel polish — 19 October 2026

Scope: small animation/presentation pass. No model, material, palette, sound, collision, scoring, unlock or physical-knockback rules changed. Existing uncommitted Race completion and splash work preserved. No commit or push.

## Backup first

Full project copy made before editing:
`/Users/marcus/Claude/australian-bbq-backup-2026-10-19-072641`

This is an APFS clone copy, including Git data, uncommitted changes, assets, checkpoints and build output. Source and backup become independent on later writes. Verified 3,930 non-build/non-Git file sizes and every Rust source hash. See BACKUP.md. Starting published commit remains 39fad79476fc4f2bed240d2c50dc33d6f580b4c0, with the pending Race completion changes captured in the backup.

## Exactly what changed

- `bbq_core/src/feel.rs` (new): presentation tuning constants, artistic item-weight mapping, throw squash/stretch, directional impact squash, small visual shove and 2.5-cycle damped settling; focused tests. The game has no shared physical mass/damage statistic, so animation weight is an explicit item mapping and impact intensity uses existing knock strength. No item definitions changed.
- `bbq_core/src/lib.rs`: exports feel module.
- `bbq_core/src/pose.rs`: integrates presentation state into the existing animator, including a 70 ms pose freeze and depth-axis scaling. Wind-up takes 0.10–0.20 seconds by item weight, approximately 0.15 seconds for medium items. Charge/release mechanics remain immediate and unchanged; a very quick tap does not delay the projectile to complete the animation.
- `bbq_app/src/characters.rs`: applies directional body scaling and freezes the rendered root during hit-stop; reads held-item weight; observes authoritative remote status transitions for guest reactions; adds an opt-in `--pose polish` preview fixture. Does not change character assets or add body parts, features or clothing.
- `bbq_app/src/bots_app.rs`: retains released-item weight and scales existing local hit shake/popup intensity using knock strength.
- `bbq_app/src/race.rs`: retains released-item weight and adds visual hit response without Race knockback, new slowdown or scoring changes.
- `bbq_app/src/game.rs`: optional impact metadata for popups and a floating copy of the existing successful-hit score, including existing long-shot bonuses. No new trick-shot rewards.
- `bbq_app/src/hud_fx.rs`: damage-proxy-sized floating impact text; uses loaded UI font. Non-impact notifications keep their sizing and position.
- `bbq_app/src/online_world.rs`: weights validated remote throws and scales existing guest hit shake.
- `bbq_app/src/player.rs`: 70 ms local camera pause on new authoritative stun/wet/down reactions. Input, simulation and network clocks continue. This is visual hit-stop, not global time dilation.

All tuning strengths are at the top of `rust/crates/bbq_core/src/feel.rs`. The visual shove is at most 9 cm and returns to the true position; physical knockback remains exactly as before. Wobble settles over 0.65 seconds. Existing knockdown, seated and carried animations take precedence.

## Verification

629 tests passed (208 app, 421 core); one existing ignored. Existing gameplay/scoring/multiplayer tests remain passing. See tests.log. Release/capture results recorded at completion below.

The rendered preview is a controlled animation fixture, not a recording of competitive play. The three existing characters demonstrate light throw, heavy throw and directional impact. No character redesign was performed. Host/offline impact directions use the collision direction; guest remote status-only reactions use the replicated facing as an approximation, without a network protocol change. Exact remote direction and repeat hits that do not change authoritative status are not newly replicated by this small pass.

## Run

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

## Selective recovery

The full backup is kept outside the project. `changed-files.json` and `before-source/` describe only this polish pass, relative to that backup; `animation-only.patch` contains the full changes. Full updated source files are installed directly in the repository.

Check without modifying files:

```sh
python3 checkpoints/2026-10-19-animation-polish/revert.py
```

Only when a revert is wanted, add `--apply` and rebuild. The guard refuses later-edited files before writing anything. It restores the nine modified sources and removes the new feel.rs, preserving the pending Race completion/splash changes. Do not overwrite the working project with the full backup after making later edits; merge selectively instead.

### Completion results

Optimized release build passed (2m28s; existing warnings only). Actual water-balloon collision replay confirmed unchanged 6.20 → 4.96 → 6.20 m/s movement, wet expiry and recovery. Captured a controlled three-character renderer preview plus the actual collision/wet/recovery replay. Raw frames remain local and Git-ignored; compressed clips and selected stills are retained. No new WAN session was run for this presentation-only pass; existing network tests passed.

Selective recovery passed in a disposable copy: later edits refused before writes, all nine modified sources restored to backup hashes and the new module removed. Real project received only a dry-run check. Ready for user review; no commit/push.
