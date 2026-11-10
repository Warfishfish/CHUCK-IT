# Concept-art graphics backlog

Aim: bring the playable arena closer to the Bush Servo concept's warm, detailed, stylised look while keeping the current game's identity, unobstructed play routes, readable characters and browser performance. Work one reviewable slice at a time; capture the real game from matched cameras and compare performance before expanding.

## Priority order

1. **BBQ baseline and first cooking-station detail pass completed.** The grill gained a dial on its arena-facing side and the prep table gained a slatted shelf and painted sign. [Matched views](checkpoints/2026-11-09-cookstation/review.html).
2. **Strengthen the arena composition.** The first entrance pass widens the shelter sign and softens the outer road edge. [Compare the fixed views and walk the approach](checkpoints/2026-11-10-shelter-approach/review.html). Keep the shelter, picnic area and playable road/oval as the focal point. Use the concept's strong foreground-to-centre path, readable curved road edges and a clear visual destination at the Smoko shelter. Do not spend this pass on scenery outside the fence.
3. **Add concept-led arena props in useful clusters.** Prioritise eskies, tyre stacks, timber rails/barriers, crates, bins, roadside signs, fuel pumps and picnic clutter. Place a few coherent groups at arena edges and corners; keep spawn points, carrying approaches, race paths and fight space open. Preserve or deliberately align their collision shapes.
4. **Make the ground tell a clearer story.** Tune the red-earth shoulder, dusty transitions, worn asphalt and subtle wheel/foot traffic so surfaces lead the eye toward the shelter. Keep ground variation broad and quiet enough to avoid visual noise and browser shimmer.
5. **Strengthen gum-tree shade inside the arena.** Use irregular canopies and filtered warm shade around the picnic and shelter areas, with readable soft contact shadows under characters and props. Keep important silhouettes clear in shade.
6. **Unify material detail on hero structures and props.** Add restrained weathering to corrugated roofing, painted timber, sandstone, rubber and metal. Use signs, edge bevels and shape changes for readability; avoid tiny high-frequency textures.
7. **Tune the arena lighting and colour together.** Warm sun, gentle fill in shaded areas, richer red earth and timber, and distinct cool accents on eskies and gameplay objects. Keep saturation controlled so the blobs and thrown items remain easy to track.
8. **Revisit sky and distant landscape last.** The concept has layered ranges and a broad Australian sky, but the user's priority is the arena. Only extend beyond the fence after the in-arena work is reviewed.

## Asset and performance rules

- Use the live Blender connection for targeted model adjustments or layout studies, not as a reason to rebuild existing props.
- Use Poly Haven or Sketchfab as visual reference or for a specific suitable asset when its license, style, scale and runtime cost fit. Prefer existing game assets and lightweight authored geometry when they do.
- Keep prop routes and collisions testable; avoid decorative geometry that looks walkable but is not, or vice versa.
- Review with actual game captures, including a gameplay view and a close-up; use a short moving view when checking shadows or shimmer.
- Compare the same browser scene and camera before/after. Track frame time and mesh/material costs; reduce detail or use a lower-cost representation if a change creates a meaningful regression.
