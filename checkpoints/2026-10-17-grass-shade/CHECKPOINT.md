# Grass and shade — 17 October 2026

**Complete, awaiting user review.** The user requested grass and lighting/shade closer to the AI reference. Implemented geometry and lighting, captured actual results and verified performance/recovery. No commit or push. Preserve all existing race, Smoko and earlier lighting work.

## Starting state and scope
Starting state: completed first lighting/material pass. starting-commit.txt and starting-status.txt identify the Git state. starting-tracked.patch plus starting-untracked.tar.gz preserve the source changes on top of that commit. before-source contains exact original versions of the four modified files. New grass.rs is included in visual-only.patch. No gameplay logic changes.

## Implementation
- New app grass.rs: opaque bent blades batched into 5 m tiles, sharing one material. 1,600 near tufts / 300 far tufts per tile, three blades per tuft; higher detail within 22 m and culling beyond 38 m. Blades are short, with variation in height, bend and root-to-tip colour. No per-blade entities, animation, collision or shadow-casting passes. Render-only mesh buffers avoid retaining the mesh data in CPU assets after upload.
- Pool, fixed prop bounds, Smoko scoring pad and race lane/spectator terrace excluded. Existing pickup markings remain visible but grass can cross the flat pickup discs.
- Three gum trees reused beyond the front/right fence; enlarged and broken into smaller canopy clusters for real shadow gaps. They belong to the standard yard, hidden in Race with the other standard environment props.
- Pop sunlight and ambient balance tuned for warm highlights and more readable shade. Existing Gaussian shadow filtering retained; no expensive PCSS feature or new paid/generated assets.
- main.rs registers the grass module; yard_scene.rs runs map rebuild/LOD visibility systems. Other changed files: lighting.rs and core looks_yard.rs. --grass-off disables only the blade layer.

## Verification
201 application + 414 core tests passed (615 total); one existing application test ignored. Includes grass exclusion checks as well as existing mode/gameplay tests. Release build passed with 17 pre-existing warnings. git diff --check passed. Final source hashes match changed-files.json.

Actual before/after captures for Backyard/Heist, Smoko, grass close-up, race overview and racer view are in before/ and after/. Captures use frame 900, identical scene/camera settings, 1280×800 and 600 subsequent timing samples. All process exits succeeded with no logged ERROR. Images visually inspected. Live bots/effects vary slightly; these are representative comparisons, not pixel-identical state replay. No new network session or motion recording for this visual pass.

## Performance and cost
- Grass at player height: 8.195 → 8.454 ms mean (+3.2%), P95 9.507 → 9.522 ms. Mesh assets 2842 → 3223; mesh entities 6820 → 7369.
- Backyard / Heist: 8.204 → 8.213 ms mean (+0.1%), P95 9.508 → 9.323 ms. Mesh assets 2842 → 3223; mesh entities 6820 → 7369.
- Smoko: 8.124 → 8.087 ms mean (-0.5%), P95 9.360 → 9.471 ms. Mesh assets 2662 → 3049; mesh entities 6234 → 6781.
- Race — player view: 5.415 → 5.492 ms mean (+1.4%), P95 6.413 → 6.493 ms. Mesh assets 2788 → 3333; mesh entities 5794 → 6428.
- Race — overview: 6.238 → 6.228 ms mean (-0.2%), P95 7.443 → 7.395 ms. Mesh assets 2841 → 3386; mesh entities 5889 → 6523.

The closest grass view added approximately 3% mean frame time; other samples stayed within about 1.5%. No substantial frame-time regression on this Mac in these views. One shared material added; texture memory unchanged. Additional grass LOD mesh buffers and tree geometry increase mesh memory and mesh/entity counts. Mesh GPU memory was not directly profiled. Frame times are wall-clock intervals, not GPU timestamps; do not claim a speed improvement from small negative differences or extrapolate to other hardware.

## Test
`cd /Users/marcus/Claude/australian-bbq/rust`
`~/.cargo/bin/cargo run -p bbq_app --release`

Use Maps / Game modes to select a mode. For a lighter grass comparison add `-- --grass-off`.

## Recovery
To restore the previous lighting pass from the repository root:
`python3 checkpoints/2026-10-17-grass-shade/revert.py --apply`
Then rebuild before playing. This restores only lighting.rs, main.rs, yard_scene.rs and core looks_yard.rs, and removes this update's new grass.rs. All pre-existing race/Smoko work is preserved. The script checks every affected file before any changes, refusing to overwrite later edits. `revert.py` without --apply performs a dry run.

To return all the way to the original pre-graphics look, first revert this grass pass, then use the previous checkpoint's revert.py:
`python3 checkpoints/2026-10-17-lighting-materials/revert.py --apply`
Rebuild again. Review pages/checkpoints remain historical records after a revert.

Recovery verification: final visual-only.patch was reverse-applied in a separate temporary folder; every restored file matched its starting hash and grass.rs was removed. Dry-run passed again after interruption/resume. For full source recovery, create a separate checkout at starting-commit.txt, apply starting-tracked.patch and extract starting-untracked.tar.gz. Never reset the active checkout or discard unrelated work. Preserve the checkpoint folders themselves separately.

## Review and limitations
Open review.html for actual comparison sliders. concept-reference.png is explicitly a generated target. The implemented grass and canopy shading are closer to the direction, but simpler foliage and sharper shadow edges remain. No wind animation; distant grass detail can visibly change when crossing LOD/cull boundaries. Review through play before deciding to keep/revert. No further milestone or commit/push without approval.
