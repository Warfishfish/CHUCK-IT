# East arena contact shading · 7 November 2026

Local review on c9e1cf84f0cd84b0b316e05df9dd7335f4aa0818, including pending picnic drinks. A single mesh reuses the existing feathered dust material. Twelve footprints mark compressed earth beneath the east gum tyre stack, two barrier posts, the nearby esky and eight table/bench feet. The soft dark centres represent static contact shading alongside the game's directional shadows.

Changed runtime asset: rust/crates/bbq_app/assets/models/smoko_surroundings.glb. Generation: art/bush-servo/ground_prop_bases.py. Use before-surroundings.glb as input and the runtime surroundings model as output. Archive /Users/marcus/Claude/australian-bbq-before-prop-footing-20261107.tar.gz contains the starting project, verified across 2128 files. Extract into a fresh directory for recovery. For selective rollback, compare subsequent work before restoring only the original surroundings GLB. This preserves the earlier picnic model.

Matched cameras, 600 native frames each. Mean ms before → after: tree 8.698 → 8.577, picnic 8.677 → 8.632, reverse tyre view 8.437 → 8.653. Last view p95 increased 10.735 → 11.146 ms. Stationary zero-bot browser sample ~75 FPS, 13.3 ms mean, 14.3 ms p95, compared with 75 FPS in the earlier local picnic preview. This checks rendering on one Mac. Asset cost: 2,688 triangles, 59,472 buffer bytes, one additional primitive sharing the original material. Existing native and browser binaries loaded the asset; gameplay and collision code were outside this edit.

Local launch: cd /Users/marcus/Claude/australian-bbq/rust then ~/.cargo/bin/cargo run -p bbq_app --release -- --menu. Choose Solo game → Bush Servo. Awaiting visual review.
