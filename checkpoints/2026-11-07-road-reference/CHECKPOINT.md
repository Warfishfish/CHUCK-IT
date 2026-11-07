# Bush Servo picnic bend · reference finish

Starting HEAD 374f0db with the prior road shoulder work pending. Verified starting road texture in /Users/marcus/Claude/australian-bbq-road-reference-backup-20261107/working-state.tar. This archive contains the modified and untracked working files on top of the commit recorded in base.txt.

This iteration adds art/bush-servo/picnic_road_finish.py and updates rust/crates/bbq_app/assets/models/smoko_surroundings.glb. The generator takes the original surroundings model from 374f0db and an output path. Rebuild with Python, NumPy and Pillow. It appends a local 1024×512 image, one material and a 288-triangle surface covering the earth and asphalt at their respective heights. Export converts the game palette to glTF colour space. Original geometry, materials and buffer content verified intact. Asset buffer grows by 717,976 bytes. Native and private browser loaded the asset through normal release builds.

Review.html presents actual native fixed-camera images, links the concept and plays the local Chrome WebGPU solo clip. Native 600-frame samples in evidence/before.metrics.txt and after.metrics.txt. Startup timing and bot variation limit precision. Browser was tested with zero bots at balanced quality, roughly 75 FPS on approach. No multiplayer benchmark. No deployment.

Selective rollback: extract just the original surroundings GLB from commit 374f0db to a temporary file, compare before copying it over the edited model. This retains the previous road PNG and generator. The full earlier working state can be reconstructed by extracting the working-state archive over a fresh checkout of the saved commit.
