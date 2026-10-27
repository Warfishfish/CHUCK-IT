# Stage 4 library sources

`servo-library.blend` is a packed, offline Blender library containing the selected Sketchfab pump hierarchy and a staging cube with the Poly Haven material. Textures are capped at 1024px (512px for small pump overlays). `../build_servo.py` appends this library, makes material copies, assembles the scene and exports the runtime GLB. No API keys, downloads or paid tools are needed to rebuild.

- **Vintage Gas Pump**, Rescue3D Assets: https://sketchfab.com/3d-models/vintage-gas-pump-0096f206ce1a4381bea58f9f94fc6b1c — **CC BY 4.0**. Attribution, source link and modification notice ship in `rust/crates/bbq_app/assets/CREDITS.txt`, displayed in the game's How to play panel.
- **Distressed Painted Planks**, Rob Tuytel: https://polyhaven.com/a/distressed_painted_planks — **CC0**. A 1.6m material tile, restrained normal strength, no displacement.

Full-resolution imported source was also preserved outside Git at `/Users/marcus/Claude/australian-bbq-stage4-library-original-2026-10-27.blend`. The compact checked-in library is sufficient to rebuild.

Do not export the material staging cube. The script removes its appended staging objects after using their materials and meshes. Prior Blender scenes are retained.
