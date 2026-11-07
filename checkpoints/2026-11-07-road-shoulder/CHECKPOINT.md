# Picnic-side road shoulder · 7 November 2026

Local review on commit 374f0db. [Fixed-camera comparison and browser walk](review.html).

The road uses planar UVs on an opaque elliptical mesh. The generator paints broken dust deposits, exposed charcoal bitumen and embedded aggregate on its inner picnic-side bend. 39,319 of 1,048,576 texels differ. Verified pixel-identical outside the target angular sector, identical opacity, dimensions and unchanged dirt texture. Deterministic rebuild produced the same road file hash. Geometry, routes and game rules are untouched.

Changed runtime asset: rust/crates/bbq_app/assets/textures/servo_road.png. Authoring: rust/tools/make_servo_ground.py. Existing Python tool needs NumPy and Pillow. No executable or public bundle change.

Same native release, fixed 1280×800 camera, three bots, 300 warm-up frames then 600 timed frames per run. Median of three warmed run means 9.45 → 9.23 ms. Median of run p95 values 13.28 → 12.92 ms. Meshes, materials and decoded image cost agree. PNG grows by 25,949 bytes. Timing varies between runs.

The short clip records the actual Chrome WebGPU canvas after verifying it loaded the new 850,950-byte texture. Moving solo player with zero bots, balanced browser graphics. Playback verified at three timestamps. The overlay around the approach showed roughly 75 FPS. Browser before/after spawn positions differ. No matched multiplayer timing claim. Browser preview is private on this Mac at http://localhost:3037/?perf=1 while its server runs.

Recovery: /Users/marcus/Claude/australian-bbq-road-shoulder-backup-20261107/source.tar holds the committed source snapshot and local.tar holds prior untracked references and plan. Archive listings, untracked contents and original road bytes checked. To undo this pass selectively, extract rust/tools/make_servo_ground.py and rust/crates/bbq_app/assets/textures/servo_road.png from source.tar into a fresh folder and compare before restoring them. Existing public build remains at the previous release.

Review before committing or extending the treatment. Local launch from rust: ~/.cargo/bin/cargo run -p bbq_app --release -- --menu then Solo game → Bush Servo → Play Smoko.
