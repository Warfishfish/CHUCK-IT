# Cloud silhouette correction — 17 November 2026

Marcus reported round dots beneath some clouds. The previous representation had a separate flat lower ellipsoid and six overlapping upper spheres. That construction could read as discs or detached circular details from oblique/below views.

## Change
- Three original Blender-authored continuous cloud silhouettes replace the multi-piece clouds in Rust desktop, full Rust browser and legacy JavaScript browser.
- Voxel union, smoothing and controlled simplification make each cloud one connected mesh island. The source script asserts connectivity before export.
- A broad vertex-painted blue-grey underside to warm-white top replaces hard individual-puff lighting. The standard glTF unlit material extension preserves that grade in all builds, with no cloud shadow casting/receiving.
- Varied rotation and scale prevent identical profiles. Existing placement seeds, drift speed and wrap distance remain; no gameplay or collision changes.
- Each source variant has roughly 2,000 vertices and 3,000 faces. All nine clouds share three asset variants. No downloaded assets or new licences.

## Graphics queue after clouds
- First correct the existing black central scoreboard in the browser Heist menu overview. This was observed during this pass and is recorded in NEXT_UP.md; it was not changed by this cloud-only update.
1. Backyard hero-tree silhouettes, house material depth and garden/lawn transitions.
2. Smoko shelter/servo detail, coherent picnic/fuel clusters and quieter road shoulders.
3. Teddy Heist junction landmarks, maze panel joins/footings/wear and stronger base entrances.
4. Shared lighting/colour consistency, contact shadows, moving-camera flicker and browser performance review.

## Checks
- Workspace: 737 passed, 3 ignored, 0 failed.
- Legacy JavaScript syntax: passed.
- Blender export: all three variants are single connected islands; glTF contains vertex colours and KHR_materials_unlit.
- Optimized native and full WASM builds passed (existing warnings). Native player-sky.png and smoko-sky.png show the continuous undersides at player height and overview.
- Actual Chrome: legacy 0.23.31 loads the shared clouds; full Rust browser loads them in Heist, Smoko and Backyard previews. Repeated Smoko preview captures show drifting complete silhouettes. browser-smoko.png records the rendered view. Browser walking/mouse-look was not repeated.
- Native player-height timing sample: median 11.226 ms, p95 16.986 ms over 600 intervals (one configuration, not a controlled before/after performance claim).
- blender-undersides.png is a separate Blender asset review, not an in-game capture.

No fresh two-player online test is needed for the asset-only change and none was run. Full-round hands-on movement/performance review remains with Marcus.
