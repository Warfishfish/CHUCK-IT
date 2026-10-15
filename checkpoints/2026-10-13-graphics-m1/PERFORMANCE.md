# Milestone 1 performance — 13 October 2026

Final paired captures, after all builds and tests had completed. Apple M1 Pro (14 GPU cores), 16 GB, 1280×800, release build, fixed camera. Same settings and seeded gameplay for each pair. 300 warmup frames; 600 wall-clock frame intervals; AutoNoVsync. Screenshot at frame 240, before timing. These are application frame intervals, not GPU timestamps or a guarantee for all gameplay/resolutions.

| View | Mean ms, before → after | P95 ms, before → after | Mean FPS, before → after | Mean time cost |
|---|---:|---:|---:|---:|
| Backyard | 7.378 → 7.476 | 8.511 → 8.601 | 135.5 → 133.8 | 1.3% |
| BBQ / veranda | 6.817 → 6.887 | 7.554 → 7.791 | 146.7 → 145.2 | 1.0% |
| Shed / shade | 6.356 → 6.453 | 7.234 → 7.455 | 157.3 → 155.0 | 1.5% |
| 11-bot Cheeky Heist | 7.900 → 8.143 | 8.879 → 9.262 | 126.6 → 122.8 | 3.1% |
| 11-bot Heist, post FX off | 7.501 → 7.762 | 8.516 → 8.894 | 133.3 → 128.8 | 3.5% |

The busiest tested scenario retained about 123 FPS average and a 9.26 ms P95 frame interval, versus a 16.67 ms budget for 60 FPS. Its measured mean cost was 0.243 ms (3.1%). The small increases are consistent across views and do not warrant expensive new optimisation work at this checkpoint. These short controlled runs do not establish worst-case performance, sustained thermals, networking load or other hardware results.

`--fx off` is the existing SSAO/SMAA post-processing switch; it does **not** disable all particle effects. Particle systems remain present in both stress variants.

## Asset cost

11-bot scene: mesh assets 3410 → 3411; mesh entities 8413 → 8414; materials 912 → 910 (palette quantisation changes material sharing). One sky sphere; no additional lawn geometry. Two 512×512 lawn maps replace one 256×256 colour map for Pop only. Resident CPU image data increased by 2,446,676 bytes (~2.33 MiB, including mipmaps). This is an image-data measure, not total GPU memory. No new runtime dependencies.

## Verification

- Release build successful; existing warnings remain.
- 175 app tests and 407 core tests passed; one existing generator test ignored. App tests rerun after final Rust changes.
- Browser-look bare-yard capture was pixel-identical before and after.
- Visually inspected yard, BBQ/veranda, shed/shade, and Customise menu captures; no shader or asset-load errors in final logs.
- Blender generated and displayed the lawn tile; both exported maps verified at 512×512. Existing scene objects unchanged.
- Each capture has a corresponding `.metrics.txt`; structured values are in `measurements.json`.

## Reproduce

From the repository root, build with `cd rust && ~/.cargo/bin/cargo build -p bbq_app --release`, then return to the repository root and run `python3 rust/tools/visual_review.py --label after`. Do not run builds or other GPU work simultaneously.

To reproduce the baseline in a separate checkout, use starting commit `dc75e0bb7089d2358954022aa642dc536921a829`, apply `baseline-harness.patch`, build release, and pass that executable to the runner with `--binary /absolute/path/to/bbq_app --label before`. The runner uses the active repository's assets; original baseline filenames remain unchanged by this milestone.
