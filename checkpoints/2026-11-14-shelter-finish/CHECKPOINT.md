# Shelter roof, sign and timber finish · 14 November 2026

Status: implemented and ready for Marcus's visual review. Not committed or published; wait for his yes after reviewing `review.html`.

## Changes

- Added a thin galvanized drip edge just beyond each shelter eave.
- Added an irregular strip of sun-faded teal paint below the upper sign frame. The cream SMOKO letters remain separate and clean.
- Kept the existing roof oxidation maps, timber grain/endgrain and later widened-sign finish. `build_shelter.py` now reapplies `finish_approach.py`'s existing sign stretch after exporting the new wear geometry, so rebuilding the shelter preserves the current sign shape.
- No gameplay, collision or browser runtime code changed in this graphics pass. The browser prototype label was bumped per project convention.

## Review and checks

`review.html` compares actual native-game screenshots using three fixed cameras, 1280×800, polished Pop style, solo Smoko with zero bots, and the same release executable for both asset roots. The screenshot runner uses frame 360. Each timing sample measures 600 intervals after 300 warm-up frames. One before/after sample per camera showed variable machine scheduling and is included only as a smoke check, not a performance conclusion.

The live Blender source was opened through Blender MCP; the updated shelter was exported successfully. The script reports 17 material primitives, 15,894 vertices and 16,032 polygons. The shelter asset increases from 16 to 17 material primitives. The doorways, post bounds, game code and collision source are unchanged.

Commands/workflows used:

- `build_shelter.py` executed in Blender 5.2.2 LTS.
- `python3 -m py_compile art/bush-servo/build_shelter.py art/bush-servo/shelter_materials.py`
- `python3 checkpoints/2026-11-14-shelter-finish/capture.py before after`

## Changed files

- `art/bush-servo/build_shelter.py`
- `art/bush-servo/smoko-shelter.blend`
- `rust/crates/bbq_app/assets/models/smoko_shelter.glb`
- `public/index.html` (prototype label only)
- `CHECKLIST.md`, `NEXT_UP.md`, `CONCEPT_ART_GRAPHICS_BACKLOG.md`
- this checkpoint and its evidence files

## Local testing

```sh
cd /Users/marcus/Claude/australian-bbq/rust
cargo run -p bbq_app --release -- --menu
```

Choose Solo game → Bush Servo → Play Smoko.

## Rollback

The pre-pass Blender file and runtime model were copied to `/tmp/australian-bbq-shelter-backup/` before export. Restore `smoko-shelter.blend` and `smoko_shelter.glb` from that directory to undo the model export; restore `build_shelter.py` from the same directory to undo the builder edits. Compare first so later changes are preserved.
