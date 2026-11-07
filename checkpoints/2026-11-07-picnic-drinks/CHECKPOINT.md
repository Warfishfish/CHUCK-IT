# East picnic table drinks · 7 November 2026

Local review based on the published tyre and timber release c9e1cf8. Editable scene art/bush-servo/east-picnic-drinks.blend, author script art/bush-servo/refine_table_drinks.py, runtime model rust/crates/bbq_app/assets/models/smoko_props.glb. The east table now seats two with one filled mug on the saucer, an empty green mug, a handled flask and a capped soft-drink bottle. All tabletop accessories are decorative.

Verified pre-edit project archive and starting revision are in starting-state.json. To recover extract the archive into a fresh directory. For an asset-only rollback compare the runtime model against the archive and restore that file after checking for subsequent work. The browser release at c9e1cf8 also contains the previous model.

Both captures used the same release executable, camera and frame. Close mean 8.277 to 8.323 ms, gameplay mean 8.322 to 8.414 ms across 600 measured frames each. Browser stationary solo sample about 75 FPS, 13.3 ms mean, 15.5 ms p95. Prior tyre release sample 75 FPS and 14.5 ms p95. Single-device measurements. GLB decreased from 71,504 to 71,012 triangles and 4,889,840 to 4,872,724 bytes. Both exports have 14 material batches. Asset-only check using the existing native and browser binaries.

Launch from /Users/marcus/Claude/australian-bbq/rust with ~/.cargo/bin/cargo run -p bbq_app --release -- --menu and select Solo game → Bush Servo → Play Smoko. Review the first table before expanding.
