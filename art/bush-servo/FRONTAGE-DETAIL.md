# Concept detail pass — 29 October 2026

`build_frontage_detail.py` runs through the live Blender MCP. It loads the original
`smoko-servo.blend`, creates a new scene, adds named frontage details, saves
`smoko-frontage-detail.blend`, and exports the replacement `smoko_servo.glb`.
The original Stage 4 authoring file is deliberately retained as a repeatable base.

Contents: three stocked shop-window relief displays, five-line service board,
veranda rafters and lamp housings, coiled air hose, three crates, three oil cans,
fascia fixings and freezer vents/latches. All additions are beyond Blender y=24
(game z=-24), outside the northern play fence. No new collider is required.
The window displays are shallow opaque scenery, not a traversable shop interior.

The original Poly Haven painted-plank and Sketchfab fuel-pump resources remain.
New frontage geometry reuses the base's material palette and adds no textures or
material batches. The packed original library sources remain in `library/`.

`build_landscape.py` replaces solid canopy masses with tapered, drooping opaque
leaf sprays and smooth bark normals. It uses the packed CC0 eucalyptus bark
material from `library/eucalyptus-bark.blend`; provenance is in assets/CREDITS.txt.
Only recoloured 512px colour and reduced-strength normal maps enter the game;
source roughness/displacement maps remain unused. Small 256px stone colour maps
add restrained broad variation. Existing runtime mip filtering handles the maps.

Tune named leaf-density/length, bark-resolution/strength/colour and stone-resolution
constants at the top of `build_landscape.py`. World layout and boulder dimensions
are unchanged. Blender's Z-up scene converts to glTF Y-up on export.

Both builders run `compact_glb.py` after export. It removes unused UV attributes
from textureless static primitives and repacks retained data verbatim. It rejects
animated/skinned/sparse assets. No compression extension or extra game decoder is
required. Editable Blender scenes retain full authoring data.

Final changed-model totals: 172,486 triangles, 26 material batches, 15,646,880 bytes.
Prior totals: 145,698 triangles, 27 batches, 13,219,760 bytes. Runtime remains static;
there is no added lighting or vegetation simulation. Validate the boulder bounds
with the current checkpoint's `validate-landscape.py` (the older validator writes
into its historical checkpoint).

Current review, captures, cost measurements and recovery instructions:
`checkpoints/2026-10-29-servo-detail/`. This is a first concept-style refinement,
not a complete match to the reference's terrain, atmosphere and scenery density.

## Follow-up: outer ground cover

The 29 October follow-up adds deterministic low spinifex fans and clustered,
partly buried ochre stones outside the play fence. They use existing landscape
geometry/material batches and leave the road, shared bounds and gameplay data
untouched. Current in-game overview and review notes are in
`checkpoints/2026-10-29-servo-groundcover/`.
