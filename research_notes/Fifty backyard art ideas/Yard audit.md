# Australian BBQ visual opportunities

## What does the backyard already show?

### Takeaway
Use the latest backyard-finishing captures to inspect placement and roof access, and the earlier Polished shots to judge the normal art direction. Gutters, grass and gnarly gum trees already exist. The current opportunity is bigger differences in form between neighbours, the visual transition between routes, and a story behind the window glass.

### Cited Findings
- In the recent overhead a long roof fills the rear edge, with the sprinkler, toy rack and compost furnishing small parts of the yard. The rear view has one clear solar collector on a wide slope. [Overhead](../../checkpoints/2026-11-15-backyard-finishing/overall-yard.png) [Rear view](../../checkpoints/2026-11-15-backyard-finishing/rear-roof.png)
- The earlier richer overhead shows four similar rear houses with distinct colours and similar roof silhouettes. The facade close-up shows board lines, corrugated red roofing, chunky gutters, a rust-marked tank, grass blades and a shaded but dark window. [Overhead](../../checkpoints/2026-11-15-environment-detail/backyard-overview.png) [Facade](../../checkpoints/2026-11-15-environment-detail/house-detail.png)
- The source explicitly chooses a simplified Browser comparison look when `--shot` has no `--look` override. Normal native play uses Polished. Neighbours and dense gums are created by `yard_styled`; animated prop details are Polished-specific. The simple captures cannot prove normal play has no neighbours. [Look selection, lines 33–47](../../rust/crates/bbq_app/src/lighting.rs) [Scene assembly, lines 5067–5147](../../rust/crates/bbq_core/src/looks_yard.rs) [Animated props, line 286](../../rust/crates/bbq_app/src/yard_scene.rs)
- The original 20-item list is all marked implemented. Existing style goals include warm light, cooler shadows, airier mature trees, bark, repairs, weathering and mode readability. A1.4 etc. in the older master should not be re-proposed as new without distinguishing a targeted improvement. [Completed list, lines 19–40](../../NEXT_UP.md) [Graphics checklist, especially lines 38–106](../../GRAPHICS_2C.md)

### Inferences
- A strong first batch would pair one change visible from the roof (recognisable neighbour silhouettes), one while crossing open grass (a synchronised narrow front of wind), and one at eye height (a sheltered laundry window with depth). Evaluate from the ladder, the clothesline and the pool rather than only overhead.

### Gaps
- No live play or hosted deployment inspected for this research request. Older captures predate the newest props. The screenshot walkthrough is a visual observation rather than a performance test.

## Which proposals would add a different experience?

### Takeaway
Use existing interactive landmarks as anchors, add broad shapes legible across the map and make close details reward the player's walk between them. The following are original proposals derived from the cited photos and code.

### Cited Findings
- Neighbours use a common roof-and-wall generator, seeded sizes/colours and roughly repeating spots. The main polished house already has a screen door, vents, sill dirt, whirlybirds and a tank. [Neighbour construction, lines 4302–4587; house, lines 872–1113](../../rust/crates/bbq_core/src/looks_yard.rs) [JS neighbours, line 877](../../public/index.html)
- The native backyard already has recognisable foot-worn paths and prop-contact shadows. The Hills Hoist, rotor and sprinkler are existing rather than new landmarks. [Yard art, lines 4196–4260 and 4901–4919](../../rust/crates/bbq_core/src/looks_yard.rs) [Source checklist](../../NEXT_UP.md)
- Heist wall pieces are drawn from shared geometry using broad team, stone, crate and warm maze tints. Emblems and different layouts already exist. [Wall and pad builder, lines 4009–4084](../../rust/crates/bbq_core/src/looks_yard.rs) [Implemented layout and rules](../../CHECKLIST.md)

### Inferences
1. **Neighbour roof profiles.** Make one squat hipped brick veneer, one tall weatherboard gable and one long skillion extension. Their silhouettes should be recognisable from the pool and tank roof.
2. **Laundry window depth.** Place a half-open blind, folded towel and amber inside wall behind a recessed pane so the eye reads a room at normal passing distance. The house remains closed.
3. **Main house brick piers and underfloor grille.** At the low end of the lawn expose two shadowed piers with patterned vent blocks rather than adding another stripe of painted foundation.
4. **Tank-to-roof wear.** A broad zinc-polished patch where a boot lands, one replaced sheet and a restrained ridge fastener strip tell a new player where an uninterrupted jump can take them. Bake its marks to prevent flicker.
5. **Back wall service nook.** An old enamel trough below a louvred laundry sash, one cracked tile and a crooked pipe saddle give the back wall a story from a new angle.
6. **Three neighbouring households.** An orchard crate with a folded pram, an oil-stained restoration-project car silhouette under a lean-to, and a line of strikingly straight potted cuttings suggest three different owners beyond the fence.
7. **Back-lane view.** Place two wheel tracks, one repaired asphalt patch and a distant street number beyond an existing sight gap. They read as part of a real street without suggesting a playable route.
8. **Lemon tree by the kitchen.** Low dense foliage and a few matte lemons contrast with the pale tall gums. Use dark shading and place the fruit above throw height so it doesn't look like a pickup.
9. **Under-gum fern pocket.** Narrow fringe of strap leaves at one shaded fork with bare dry soil between the roots; deliberately retain open ground at blob height.
10. **A wind front.** Propagate one gust across grass tips, a single low branch and hanging fabric at slightly different times. Fixed visual phase keeps hard geometry stable and seasonal movement legible.
11. **Ground under the mower.** Swap hard rectangular stripe cutoffs for curved turn marks, a clippings ridge and two pressed wheel paths connecting the mower to the cut area.
12. **Pool lane footprints.** After getting out of water, the next three or four steps stamp successively smaller wet shoe/foot shapes on the existing pavers. Use the existing wet state, adapt footprint for barefoot blobs and reserve the lane for play.
13. **Recessed Heist wall materials.** One lane gets broad diagonal lattice, one is vertically ribbed, another has wide staggered boards. Embed patterns in the existing collision footprint and use the same patterns on both approaches so layouts still work.
14. **A fixed Heist meeting point.** An oversized upturned tin wash tub on a fixed junction wall creates a 'meet me at the tub' callout independent of team and base choice; keep its top below the emblems.
15. **Teddy reveal on the bank shelf.** Show an empty fabric cradle and pressed-nap silhouette as the teddy leaves. Read the game's existing state and ensure no VFX show a hidden carrier through a wall.
16. **Hoist crank and counterweight.** Add a grip polished at the ends, the slightly darker cover and raised winding arrows; somebody passing close can tell how it works without a mechanical interaction.
17. **Spring response on the trampoline.** Animate a shallow bowl in the mat and a single tightening/releasing spring band at the moment of the existing bounce, tuned to the current bounce arc. More readable than additional particles.
18. **Quiet the busiest high-frequency surfaces.** Give thin roof ribs, flyscreen and fences wider value bands as distance rises; inspect from a moving camera, especially beside Heist's pool, to stop sparkle competing with a flying item.

### Gaps
- Ideas are hypotheses for approval, not verified changes, and were not simulated. Additional street terrain remains outside gameplay bounds.

## Which source and review issues affect priorities?

### Takeaway
There is a specific visible assembly error worth fixing in the JS browser and two distinct browser targets. Share matching eye-height images of the intended release build when evaluating the next pass.

### Cited Findings
- Marcus also reported walking through the slatted bench by the bins. The Polished Rust model is at (31.5, 16.8); its 1.15 m seat has two slats spanning 0.61 m deep with their top at 0.505 m. Its back slats end around 1.30 m. The bins have their own solid at (31.5, 19.5); the bench is absent from yard collision. In Three.js the bins are followed by crates, with no corresponding bin-side bench; the shed workbench is a different prop. [Native bench, line 3012](../../rust/crates/bbq_core/src/looks_yard.rs) [Collision, lines 358 and 456](../../rust/crates/bbq_core/src/yard.rs) [Browser bins, line 1149](../../public/index.html)
- In JS the `rod` helper creates objects under the default scene, but wheelbarrow legs/handles, trowel handle and weeds pass the helper local coordinates. The tub and trowel blade correctly use the barrow group, which sits at x 4.2, z 15.8. This predicts detached geometry by the origin. [Rod helper, line 826; wheelbarrow, lines 1269–1284](../../public/index.html)
- Local instructions identify the HTML/Three.js game at `/` and the Rust/WASM build at `/rust/` as distinct browser entries. The checked-in WASM release manifest still names the 12 Nov Smoko road; it cannot be cited as evidence the newest Rust scenery was deployed there. [Run instructions, lines 11–36](../../HOW_TO_RUN.md) [Release manifest](../../public/rust/release.json) [Build tool](../../rust/tools/build_web.sh)
- A native performance comparison with 11 bots exists on the project tracker. Adding texture stamps/geometry ought to be evaluated by mode and machine against a new baseline rather than citing that number for this version. [Graphics tracker, E18.4](../../GRAPHICS_2C.md)

### Inferences
- Make the bin-side bench solid. Match the visible geometry with a low seat box approximately 1.15 × 0.62 m (0.51 m top) and a narrow 1.14 × 0.15 m back at z 16.48 up to 1.30 m. Mirror the visible mesh and solids in the JS version and keep geometry/collision together wherever the bench is enabled. Try walking into it from the front, back and both ends, jumping onto the seat, and walking the aisle between bench and bins.
- Repair the wheelbarrow parenting and take matching pictures at the item and the lawn centre in the HTML target. This is a specific proposed improvement to a built asset.
- A roof-ridge panorama, eye-level ground route, plus a moving side-pass in the actual builds are better review views than a single overhead. For a scripted Rust photo explicitly select `--look polished`.

### Gaps
- The JS parenting issue is established by code inspection; I did not reproduce it live. No conclusion about actual Cloudflare deployment is implied.
