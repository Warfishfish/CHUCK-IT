# Backyard grass distance fade

Prototype 0.23.47 · 21 November 2026

Backyard blends the fine 3D blades into the lawn between 7 and 23 metres from the camera. The fade uses each fragment's horizontal world-space distance. Turning on the spot leaves the distance unchanged, and the transition does not follow tile boundaries. Close blades keep their geometry and colours. The existing terrain texture carries the distant lawn colour and wear. The material applies to Backyard in Free for All and Teams. Race Oval retains its opaque grass, and Smoko and Heist keep their own ground and vegetation. The legacy Three.js build already uses a mipmapped lawn texture without separate 3D blades.

## Visual checks

- Matched player-height walking and turning on the release desktop game. The most visible improvement was the band of bright mid-distance specks stretching toward the fence. Foreground blades remained visible on Low and High.
- Two local full WebGPU solo walks in Chrome with real mouse capture, approximately 10.6 seconds each. Both graphics settings fetched the shader and rendered the grass fade. The sampled lawn stayed steady without the earlier dark squares or a visible fade boundary.
- Two fixed High desktop bench runs per build showed similar frame times. The 95th percentile ranged from 13.86–13.91 ms before and 13.73–14.05 ms after.
- Rust workspace tests recorded 753 passed and 3 ignored. The legacy inline JavaScript syntax check and the browser pointer-lock test passed. Networking was outside this graphics check.

[Matched desktop frame](backyard-before-after.jpg) · [Matched desktop walk](backyard-walk.webp) · [Chrome Low](backyard-browser-low.jpg) · [Chrome High](backyard-browser-high.jpg)
