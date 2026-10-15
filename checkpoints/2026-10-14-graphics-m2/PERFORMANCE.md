# Milestone 2 performance

14 October 2026. Apple M1 Pro, 14 GPU cores, 16 GB. Release build, Pop polished, 1280 × 800.

Each run advances simulation at 60 Hz, captures at frame 240, warms up for 300 frames and samples 600 real wall-clock frame intervals with AutoNoVsync. One game process at a time. No concurrent builds. This measures frame intervals, not GPU timestamps. Short tests do not establish all gameplay or hardware performance. Portrait moods are staged; stress uses natural gameplay.

| View | Before mean ms | After mean ms | Change | Before P95 ms | After P95 ms | After FPS |
|---|---:|---:|---:|---:|---:|---:|
| cheer | 6.415 | 6.337 | -1.2% | 7.579 | 7.331 | 157.8 |
| current | 6.156 | 6.266 | +1.8% | 7.120 | 7.186 | 159.6 |
| egg | 6.964 | 6.927 | -0.5% | 8.236 | 8.241 | 144.4 |
| grit | 6.329 | 6.388 | +0.9% | 7.347 | 7.419 | 156.6 |
| gumdrop | 6.750 | 6.493 | -3.8% | 7.781 | 7.509 | 154.0 |
| none | 6.442 | 6.338 | -1.6% | 7.618 | 7.425 | 157.8 |
| normal | 6.338 | 6.501 | +2.6% | 7.390 | 7.893 | 153.8 |
| ouch | 6.406 | 6.348 | -0.9% | 7.482 | 7.375 | 157.5 |
| pear | 6.587 | 6.579 | -0.1% | 7.707 | 7.637 | 152.0 |
| profile | 6.476 | 6.566 | +1.4% | 7.681 | 7.656 | 152.3 |
| sip | 6.374 | 6.426 | +0.8% | 7.477 | 7.644 | 155.6 |
| stress | 8.222 | 7.845 | -4.6% | 9.528 | 8.714 | 127.5 |

11-bot Heist: 8.222 → 7.845 ms mean (−4.6%); 121.6 → 127.5 FPS; P95 9.528 → 8.714 ms. No measured stress frames exceeded 16.67 ms. Treat the improvement as a short-run observation, not a general guarantee.

Resident mesh assets: 3411 → 2897; mesh entities: 8414 → 7736; materials: 910 → 911; CPU image bytes unchanged at 42,129,816. These are asset/entity counts, not draw calls or total GPU memory. Thirteen shared face meshes contain 8,624 triangles across all variants, embedded as a 361,921-byte JSON source asset; only the active mouth/brows are drawn. No new textures.

Current-style regression: visually unchanged; 16 of 1,024,000 pixels differ between captures. No new Pop meshes, glints or expressions are enabled for Current. Small run-to-run particle/raster differences remain possible.

Canonical measurements are evidence/*.metrics.txt; measurements.json is the parsed summary. The Egg view uses the wider existing shot camera after the initial framing clipped its taller silhouette. The capture script reproduces the final framing.
