# East picnic gum and shelter shade · 7 November 2026

Starting source 374f0db plus both pending road passes. Backup /Users/marcus/Claude/australian-bbq-before-gum-shade-20261107 contains the base revision, archive of modified/untracked files, original landscape GLB and original native binary. Original landscape bytes and archived pending surroundings bytes verified. For full recovery extract working-state.tar over a fresh checkout of the saved revision. For selective rollback inspect subsequent changes, restore the prior arena_depth.rs from the saved revision and the original landscape GLB from the backup. Remove this pass's shade PNG and authoring script when reverting; retain both road assets.

## Implementation

- art/bush-servo/picnic_canopy.py bakes a soft ground projection from 2,100 quads of eastern gum foliage and the existing shelter roof. It uses the Smoko sun direction. The bake should be repeated if the tree, shelter or sun moves.
- rust/crates/bbq_app/assets/models/smoko_landscape.glb keeps the same geometry counts. East canopy leaves are smaller so their live shadows make finer patterns on props and players. The canopy is visibly more open. Validated all other GLB buffer views and all vertices outside the selected foliage against the original.
- rust/crates/bbq_app/assets/textures/smoko_picnic_shade.png is a 512×512, 17,472-byte ground shade texture. Regenerating the assets from the original returned byte-identical output.
- rust/crates/bbq_app/src/arena_depth.rs draws this static shade on one two-triangle surface during polished Smoko play. It cannot cast a second shadow. The softer ground pattern extends beneath the eastern picnic setting and a lighter penumbra follows the shelter eaves. Prop and player shadows remain live. Existing collision, scoring and carrying logic are untouched.

## Checks

Native release and WebGPU release builds passed. The focused arena contact-shadow test passed. Both fixed-camera pairs in review.html are actual native captures of the previous road build versus this pass at 1280×800. Each sample measures 600 frame intervals after warmup. Picnic mean 9.342 → 9.260 ms and p95 12.578 → 12.904 ms. Shelter mean 9.168 → 9.120 ms and p95 12.106 → 12.476 ms. Small single-run differences.

The short WebM records the local Chrome game walking through the picnic shade to the delivery ring. Checked playback at three timestamps. Balanced browser, solo, zero bots, about 74 FPS in the approach sample. This is a single-device visual check. Multiplayer frame pacing requires a separate playtest.

Local preview http://localhost:3038/?perf=1 while the private server runs. Native: cd /Users/marcus/Claude/australian-bbq/rust then ~/.cargo/bin/cargo run -p bbq_app --release -- --menu. Choose Solo game → Bush Servo → Play Smoko. Public release unchanged.

## Next steps after visual review

Review the corrugated roof, painted sign and timber posts from standing height. Then carry the approved style to the frontage and existing arena props. Check visibility through the active carrying route on a browser host and guest before sharing these techniques with other maps.
