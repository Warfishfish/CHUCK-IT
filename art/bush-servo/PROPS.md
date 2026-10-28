# Bush Servo small props — Stage 6

Authored in the connected Blender on 28 October 2026. No paid generation or new library downloads. The local shelter's baked timber materials are reused; other materials and geometry are authored here.

`smoko-props.blend` contains one editable scene with separately named parts. `build_props.py` rebuilds it in a new scene without deleting other open scenes. The runtime GLB joins export copies into ten material batches. The authoring file is saved before creating that runtime duplicate.

`props-layout.json` defines placements in game metres (x,z,width,depth,height). Blender uses (x,-z,height), exported to glTF Y-up. Update the matching `PROP_SOLIDS` in `rust/crates/bbq_core/src/bush_servo.rs` when changing solid placements. Run `python3 art/bush-servo/validate_props.py` to compare authored bounds against shared collision data and report GLB cost.

## Content
- Four eskies with rounded lids, seals, recessed grips and latches. One replaces the old simple shelter-side esky using its existing bounds.
- Five green/galvanised bins with lids and pressed ribs.
- Six stacks, totalling fourteen tyres, with sidewall rings and shallow tread geometry.
- Two picnic tables and four benches, plus four enamel mugs and two trays. Existing rear shelter benches remain, giving three furniture groupings.
- Two roadside signs outside the play fence. Existing fencing and sandstone borders already meet the concept's framing needs.

Props use 42,272 triangles, ten material batches and three reused small wood maps; 2.19 MB GLB. Removing bevels from tiny tread blocks reduced the first export by over 32,000 triangles. No extra lights or simulation are added.

## Gameplay constraints
Twenty new shared static collision boxes; existing esky collider retained. Clear central scoring circle and four 2.8m-wide delivery approaches, no overlapping existing map solids, no blocked player spawns, and no new props on the asphalt annulus. Tests cover these constraints. Rounded corners/tyres use conservative box bounds; tables block beneath their top and are not crawl-through furniture. Mugs/trays are small non-blocking decoration. These eskies do not provide a new open/pickup action.

Protocol 21 prevents mismatched Stage 5 clients playing with Stage 6 collision layouts. Refresh/rebuild every peer together after publication. Other maps retain their existing layout.

Review evidence and rollback instructions: `checkpoints/2026-10-27-servo-props/`. The directory retains its original start date; completion and authoring occurred on 28 October local time.
