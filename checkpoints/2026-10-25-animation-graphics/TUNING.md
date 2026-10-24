# Tuning and replacing art

- `rust/crates/bbq_core/src/finish.rs`: one PALETTE constant, DETAIL_LEVEL (0 off, 1 subtle, 2 stronger), body finish and gradient depth. Existing saved colour indices are preserved. Shirt names, existing customisations and team markers remain; colour alone is never the sole identity cue. No claim that every possible duplicate colour choice is distinguishable.
- `rust/crates/bbq_app/src/locomotion.rs`: walking bob, sway, breathing and belly spring stiffness/damping. Existing per-character spring state drives the morph targets; no extra per-frame network traffic.
- `rust/crates/bbq_core/src/feel.rs`: charge lean, weight bias, warning threshold/tremble and existing hit-stop, hit squash, screen shake and score-popup strengths.
- `rust/crates/bbq_core/src/hands.rs`: existing item-dependent charge time and overhold grace; throw power still uses the existing formula.
- `rust/crates/bbq_app/src/charge.rs`: 1.8 second fall recovery. Host measures guest charge from the received start and decides when to drop/fall. World snapshots replicate charge/recovery state, not bones or morph weights.
- `rust/crates/bbq_core/src/magpie.rs`: -10 pecks, 2.5 second peck interval while attached, 65 second flock spacing, 18 second flock cap. No +50 reward is implemented because the clarified request specifies penalties and rescues only.
- `rust/crates/bbq_app/src/items_view.rs`: short velocity trails, bounded to 64 active items; visual only.
- `rust/crates/bbq_app/src/hud.rs`: tweakable code silhouettes for held-item icons; names remain beside icons.
- `rust/crates/bbq_app/src/ui.rs`: shared palette for menu cards, button colours, logo and text. Existing bundled fonts and layout retained.

## Replace prop art

Current prop geometry is defined in `rust/crates/bbq_core/src/looks.rs` and selected centrally by `ModelKey::parts` in `rust/crates/bbq_app/src/models.rs`. Edit the appropriate item branch there to replace the procedural shapes without touching item stats, collision, scoring or flight. `ModelKey::of` preserves variants and bottle choices. Keep model units in metres and the origin consistent with the existing asset; the selected-item view and world pickups reuse this cache.

Images are selected through the `Tex` enum and `ModelCache` texture-loading table in models.rs. Keep the same Tex key to replace an existing texture. Imported GLB scenes would require a scene-loading adapter; this update does not pretend a file-drop GLB loader exists. Current code-built models remain the lightweight default.
