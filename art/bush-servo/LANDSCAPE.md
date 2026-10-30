# Bush Servo landscape

Run `build_landscape.py` in connected Blender to author a separate landscape scene and export `smoko_landscape.glb`. The existing shelter and surroundings files are untouched. Native application loads this asset only for Smoko. `parts_with_shelter(true)` retains the lightweight procedural background for fallback renderers.

Tuning: material palette, near-tree layout, leaf density, distant-tree count and ridge proportions are named in the author script. Seed and eight solid cover bounds are in `landscape-layout.json`. Geometry uses pale segmented trunks, irregular branches, tapered opaque leaf quads and terraced ridge rings. No transparent foliage, external images or per-frame vegetation simulation.

All trees and scrub lie beyond the rectangular play fence. Eight perimeter boulders inside the fence use shared Rust bounds, checked by `validate_landscape.py`; their rounded corners are a small (9cm maximum) approximation to rectangular collision. Protocol 19 prevents mixing clients with Stage 2 physics. Keep Rust NATURE_SOLIDS and layout JSON synchronized when editing cover. Run core tests for road, fence and delivery clearance.

Reference density and remaining prop targets are recorded in the Stage 3 checkpoint's CONCEPT-PLAN.md. These landscape changes do not claim to finish the building, lighting or prop stages.

## Concept refinement, 29 October 2026

Solid crown masses have been replaced by finer, open eucalyptus sprays. Trunks
are smooth-shaded and use recoloured, downsampled CC0 Poly Haven eucalyptus bark.
Mesas and the existing boulders now have subtle baked surface maps; shared solid
bounds remain unchanged. The author script now saves only its own scene and
compacts unused runtime UV streams. See `FRONTAGE-DETAIL.md` for tuning, costs,
packed library resources and the current review checkpoint.

## Ground-cover pass, 29 October 2026

Sparse spinifex fans and partly buried, grouped ochre stones add near-ground
scale in the outer belt. Their centres are placed outside the rectangular play
fence; no collision, route, road or gameplay data changes. `build_landscape.py`
keeps the additions deterministic and combines them into the existing landscape
mesh/material batches. See `checkpoints/2026-10-29-servo-groundcover/` for an
actual in-game overview and review notes.

## First playable-arena gum — 30 October 2026
One new tree at game x=12, z=1.5, 8.4m tall, grows inside the road loop. Existing exterior scenery is unchanged. `arena-trees.json` defines the shared 0.7m-wide, 4m-high solid lower trunk; core collision matches it and marks it no-top. High branches and foliage are visual only. Local seeded branching and 21 hanging sprays use existing Poly Haven bark and foliage materials; nine small grass/saltbush clusters surround its base. `ARENA_TREE_HEIGHT` and `ARENA_SPRAY_LEAVES` tune this specimen. Do not expand outside the arena for the current stage.

## Two arena gums — 30 October 2026
A second 7.7m specimen at (-11.5, 2) balances the original 8.4m tree at (12, 1.5). Heights are now per-tree values in arena-trees.json, and local seeds vary canopy shape without changing existing scenery. Both trunks have shared 0.7m × 0.7m × 4m collision, with no standing surface on top. Protocol 23 prevents mixed collision versions. The arena remains the focus; no distant scenery edits.
