# Smoko picnic island · 11 November 2026

[Open the before/after review](review.html)

## Scope

Local art and collision pass on top of `615b759`. The screenshot set was captured in the actual Smoko game. This pass only adjusts the east picnic island and its approaches; it does not change game rules or the browser game's JavaScript build.

## Changes

- Reworked the ten large rocks along the front bank to vary their width, depth and height while keeping the bank in the same strip of ground.
- Moved the blue esky and rubbish bin to the shelter-side picnic area. Updated their shared solid definitions, the small arena-depth contact patch, and the ground-wear stamp so the artwork, collision and ground details agree.
- Retuned the picnic shade texture to soften the foliage edge while retaining canopy openings.
- Reproducible source: `art/bush-servo/picnic_island.py`; layout: `art/bush-servo/picnic-island.json`. The script requires separate input and output asset directories and is deliberately guarded against in-place exports.

## Review notes

The matched entrance views show the esky and bin moved out of the near foreground and into the shelter-side grouping. The close bench view shows the more varied stone edge and a more open path beside the table. The browser images are gameplay captures from separate runs, so their player/item poses and camera location are not a strict visual comparison.

The scene counts are unchanged at 2,632 meshes, 862 materials and 5,798 mesh entities. Image CPU memory is unchanged in each matched native view. The browser samples are essentially equal (13.330 ms mean before/after; p95 14.8/14.9 ms). Native timings vary: the entrance pair improved from 18.687 to 9.639 ms mean, while the bench pair changed from 8.169 to 9.858 ms. These one-off runs are noisy and do not establish a performance trend; no meaningful added scene cost is visible in the counts.

## Verification

`cargo test --workspace` passed after rerunning with local network access for the temporary relay integration test. The first sandboxed run could not connect to that relay; the authorized retry completed successfully. The suite reports 452 `bbq_core` tests passing. Existing compiler warnings remain outside this pass.

The captures document a solo Smoko round and the browser gameplay view. A separate two-player online art/gameplay session was not run for this visual-only change.

## Review and recovery

Marcus approved committing and publishing this pass on 12 November 2026. The local source tree already contained earlier pending graphics documentation edits when this pass began; those older edits are kept separate.

To undo only this pass, first confirm the modified assets and Rust source still match this checkpoint's changes, then restore `smoko_props.glb`, `smoko_surroundings.glb`, `smoko_picnic_shade.png`, `arena_depth.rs`, and `bush_servo.rs` from base commit `615b759`. Keep the earlier pending documentation edits. The input art assets remain recoverable from that commit.
