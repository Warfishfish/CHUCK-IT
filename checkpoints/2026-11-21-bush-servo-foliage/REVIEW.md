# Bush Servo foliage and shade review

21 November 2026 · prototype 0.23.49

The landscape now uses fewer, wider leaf shapes, with more of the original detail kept in the two arena gums. Fine foliage no longer casts individual shadows. The broader crowns, trunks and baked picnic shade still give the picnic area and shelter deep, readable patches of shade.

The landscape model went from 8.72 to 5.76 MiB and from 138,475 to 85,437 triangles. The bark, rocks, mesas, materials and embedded images matched the previous export byte for byte. Gameplay and collision data stayed in their existing code. The full WebGPU build uses the same landscape model as desktop. The legacy Three.js page received the prototype version bump.

## Review captures

- **Desktop.** Repeated the scripted walking and turning path on Low and High with the release binary. Each path captured 75 frames. The small silhouettes and shadows were visibly calmer, and the matched fixed view preserved broad shade beside the road and shelter. [High comparison](desktop-high.jpg) · [Low comparison](desktop-low.jpg) · [Matched walk](desktop-walk.webp) · [Picnic shade comparison](picnic-shade.jpg)
- **Chrome WebGPU.** Repeated a solo walking route with pointer lock and the same input on Low and High. The browser runs start from different spawn points. The matched desktop captures give the direct visual comparison. [High comparison](chrome-high.jpg) · [Low comparison](chrome-low.jpg) · [Walking comparison](chrome-walk.webp)

The Rust workspace passed 755 tests with three ignored, including existing tests for the arena-tree collision and route clearance. The legacy JavaScript syntax check and browser pointer-lock test passed. This pass did not exercise the WebGPU water effects or longer two-player carry-camera comfort.
