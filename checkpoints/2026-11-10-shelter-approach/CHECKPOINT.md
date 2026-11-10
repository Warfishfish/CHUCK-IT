# Smoko entrance and live browser gameplay · 10 November 2026

[Open comparison and clips](review.html)

## Starting point and scope

Local main at 72113a152f82db55813fd327ca59eb99370f7c74. The cooking station, sign artwork, graphics backlog and earlier review documents were already in progress. The reference is the Bush Servo concept. This pass covers the path from the tyre and timber rail to the shelter, the cooking-station browser review, the carried-player camera, and jumping onto the parked ute.

## Art and browser review

- The two end signs on the shelter gain 38% width and 18% height above their base. The timber frame, board and lettering scale together. The existing tyre, rail and picnic furniture frame the route.
- A thin patch follows the outer bitumen edge seen from the entrance, reusing the existing weathered shoulder material. The two model files together grew by 3,956 bytes. The native scene gained one mesh and one mesh instance, with 862 materials and 93,163,568 image CPU bytes in both captures.
- Matched 1280 × 800 native shots use the Smoko High polished scene and camera 18.8,2.05,-9.3,136,1,57. Native timings are archived but the earlier run overlapped compilation. Use the browser comparisons to assess performance.
- The Chrome cooking-station comparison used a locally served copy of the shipped bundle and the local art build, High graphics, same computer, 1100 × 713, zero bots, looking toward the servo from the asphalt. Each sampled 450 screen frame intervals. Both means were 13.33 ms and both 95th percentiles were 13.90 ms. At walking speed the dial reads as a cream instrument against the red hood and the prep-table plate reads “HANDS OFF” at close range. See the stills and gameplay clip. This is a short, display-limited observation.
- Two browser players joined the same local room. The host knocked down and picked up the guest. The carried guest saw the characters from outside and the carried HUD. After release the guest returned to first person. Stills and the guest's continuous clip are in the review. This local relay check covers the camera transition, not remote internet latency.

At the entrance the two High browser runs faced the shelter from positions 0.30 m apart. Over six seconds with zero bots, mean frame time was 13.33 ms in each build, and the 95th percentile was 14.70 ms in each. The exact-camera art comparison is the native pair. [Before sample](entrance-browser-before.json) · [Corrected build sample](entrance-browser-after.json).

## Ute correction

The old separate jumpable surfaces were at z=40.9. The rendered ute sits beside the servo at z=-21.9, where an earlier solid represented the whole tray at rail height. In the previous build a browser host attempted a side jump and landed back on the ground at y=0, z=-20.42. The corrected shared collision places the bonnet, cab, chassis and 0.85 m tray floor relative to the same servo offset as the rendered vehicle. The simplified playable tray keeps the lip traversable. The usual jump and fence access rules still apply. A regression test jumps beside the servo, settles for 60 simulation steps on the tray and checks that the old location has no hovering floor.

## Files and recovery

- Art source: `art/bush-servo/finish_approach.py`. Outputs: `rust/crates/bbq_app/assets/models/smoko_shelter.glb` and `smoko_surroundings.glb` in that directory. To reproduce, extract the two original GLBs from 72113a1 into a temporary folder and pass that folder and a different output directory to the script. Re-export was checked byte for byte.
- Gameplay and tests: `rust/crates/bbq_core/src/bush_servo.rs`, `yard.rs` and `movement.rs`.
- Review and backlog: this directory, `checkpoints/LATEST.md`, `NEXT_UP.md` and `CONCEPT_ART_GRAPHICS_BACKLOG.md`.
- To undo this pass, compare the two GLBs and three gameplay source files against 72113a1. They were clean before this pass. Restore just those changes after checking for subsequent edits. All prior cooking-station and planning work can be kept.
- Local browser preview runs at http://localhost:3056/after/ while its server is running. Native launch from the repository: `cd rust && cargo run -p bbq_app --release -- --mode smoko --look polished`.

## Gameplay verification

All 452 core library tests passed. The optimized WebAssembly build and browser packaging succeeded. The rebuilt browser host repeated the jump beside the actual ute, starting near (2.39, 0, -20.00), and settled at (2.39, 0.85, -21.31). Three samples over 2.13 seconds confirmed the standing height. See [landing positions](ute-landing.json) and the jump clip. The actual side-jump check used the rebuilt browser executable. The matched native graphics captures predate the gameplay correction.
