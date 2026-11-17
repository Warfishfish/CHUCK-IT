# Water grass and stability

## Which visible issues can be traced to existing geometry?

### Takeaway
Replace the sprinkler's row of five switched-on spheres with an emitting nozzle. Treat the wooden bin-side seat and its collider as two independent repairs.

### Cited Findings
- The rotor and five transparent spheres are separate and the Rust trail only toggles visibility. The browser rotates the group of five around with the nozzle. The same droplets therefore stay in formation. [Rust prop](../../rust/crates/bbq_core/src/looks_yard.rs) [Rust animation](../../rust/crates/bbq_app/src/yard_scene.rs) [Three.js prototype](../../public/index.html)
- The polished Rust seat beside the bins is at x 31.5, z 16.8. Its four 0.78 m-high legs end below the back boards, centred at 0.94 and 1.25 m. The collision table specifies the bins at z 19.5 and the separate shed workbench at x 25.5, z -15.8. There is no matching bench entry. The checked Three.js bin section constructs bins, and its separate bench is a tool workbench. [Seat geometry](../../rust/crates/bbq_core/src/looks_yard.rs) [Yard collision table](../../rust/crates/bbq_core/src/yard.rs) [Three.js bin section](../../public/index.html)
- Epic documents launch velocity within a cone, gravity and drag as standard particle controls. These concepts translate to the existing engines. [Epic particle spawn reference](https://dev.epicgames.com/documentation/unreal-engine/particle-spawn-group-reference-for-niagara-effects-in-unreal-engine?lang=en-US)

### Inferences
- Sprinkler: launch narrow streaks from the nozzle in world space as it sweeps. Let each droplet continue on its own arc, break into sparkling smaller drops around the crest and disappear upon hitting lawn. Leave a soft temporary ground sheen. Preserve the old visual-only role.
- Bench supports: extend two rear stiles with a slight backward rake from the frame past the upper board and add visible bolt heads to the two real joints. Retain the missing seat slat as intended wear.
- Bench solidity: use the actual seat footprint and a narrow backrest collider (or a supported equivalent). Leave walkable room beside the wheelie bins. Confirm a player cannot walk through it from front, rear or side, can jump to the seat if intended, bots can get past it, and no invisible collider appears in configurations without the visible bench. Place/model the equivalent before adding collisions to a browser implementation missing this seat.

### Gaps
Main browser build may use Rust/WASM. The repository includes both public/rust and the older public/index.html. Source review is not a live acceptance check.

## What should be checked before increasing lawn detail or changing lights?

### Takeaway
Inspect scale, repetition, and camera motion at normal eye height; separate possible flicker causes by turning one layer off for a diagnostic capture.

### Cited Findings
- Native grass is currently opaque batched triangular blades, a high/distant mesh per tile with a distance threshold. The legacy Three.js lawn uses a repeating procedural tile with parallel mowing stripes and painted fibres. [Native grass](../../rust/crates/bbq_app/src/grass.rs) [Three.js lawn](../../public/index.html)
- NVIDIA discusses clump repetition and visible synchronisation as real-time grass problems, and variation in orientation and phase. Three.js documents texture minification and filtering, object-level transparency ordering limitations and depth offsets for decals. [GPU Gems grass](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-7-rendering-countless-blades-waving-grass) [Texture filtering](https://threejs.org/manual/pages/textures.html) [Transparency](https://threejs.org/manual/pages/transparency.html) [Depth offsets](https://threejs.org/docs/pages/Material.html)

### Inferences
- Grass: use a muted, continuous base tint underneath short, irregular clumps; a few taller tufts at fences; softened lawn-mowing bands; fewer high-contrast pinpricks. For native, inspect near/far changes from a moving viewpoint. For JS, inspect the repeating texture before considering blades. Review from grass height, running height and the ridge.
- Remaining flicker: reproduce the reported spot with a slow strafe and a stationary capture. Isolate shadows, stacked faces, grass distance changes, and blended materials in turn. If duplicate faces compete, remove one or integrate the marking in the surface material. If the pattern crawls at distance, check mip filtering instead. Verify the same camera path on Mac and the live browser and test close and far. No root cause is established from source alone.

### Gaps
No motion captures here identify the user's exact flickering object or establish which grass symptom dominates.

## How can a balloon hit and water entry look different?

### Takeaway
Both impacts already produce particles in native gameplay. The balloon can show deformation and sheets of displaced water; the pool can carry a lingering ripple centred precisely on the entry point.

### Cited Findings
- Native balloon bursts already use radial drops and mist. Player and bot logic emit a separate Splash at pool entry. The browser prototype uses a small burst in its pool-entry branch. There is no separate expanding entry ring apparent in those handlers. [Native FX](../../rust/crates/bbq_app/src/fx.rs) [Native game FX events](../../rust/crates/bbq_app/src/game.rs) [Player entry](../../rust/crates/bbq_app/src/player.rs) [Browser entry](../../public/index.html)
- NVIDIA distinguishes geometric water motion from fine normal-map undulations and explains independent amplitude, wavelength and propagation speed. [GPU Gems water](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-1-effective-water-simulation-physical-models)

### Inferences
- Balloon: introduce a very brief impact-shaped water sheet/crown, orient it against the hit surface, allow large drops to fall before the existing mist fades. Use an asymmetrical sideways fan on a character or wall versus a low outward crown on paving. Preserve existing status, points and puddle radius.
- Pool: emit expanding, fading rings centered where feet first cross the water, giving a walk-in a low ripple and a roof-height drop a stronger one. Clip disturbance at the pool edges. Limit simultaneous events and don't restart particles every frame a swimmer remains in water. Consider a short low splash followed by rings visible after airborne droplets settle.

### Gaps
The two different browser targets must be verified in the release actually served. No live scene was tested for these research notes.
