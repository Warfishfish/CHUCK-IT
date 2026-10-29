# Bush Servo concept detail — 29 October 2026

Starting commit: b10007a. Tracked worktree clean; untracked concept art preserved.
Verified recovery archive: `/Users/marcus/Claude/australian-bbq-before-servo-detail-2026-10-29.tar.gz` includes all tracked files and untracked concept art. Archive listing and key asset byte hashes checked. Extract into a NEW directory only.

Scope: visible servo frontage detail based on the existing Bush Servo concept: stocked window displays, supplies board, forecourt service equipment, crates and veranda detail. Existing Blender materials and original building reused. All additions beyond the northern play fence (Blender y > 24). No gameplay/collision changes. Browser publication and commit await review.

Baseline and after use the same native binary and camera; only runtime GLB changes. Capture script aborts on a locked Mac and rejects stale/black images. Asset costs and matched frame intervals will be recorded below.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust` then `~/.cargo/bin/cargo run -p bbq_app --release -- --menu`; select Smoko Yard.

## Completed first pass — ready for review

The user clarified that the reference's overall art style matters, not only more
objects. This pass therefore includes both the servo frontage and surrounding
gum woodland. Reference: `concept art/exec-6ea4e9b3-70f7-4745-b24d-773f23cbe0ae.png`;
the same Bush Servo image is retained in the older shelter checkpoint for review.

### Implemented
- Forty trees now use finer open leaf sprays instead of solid polygon crown masses;
  pale smooth trunks use Poly Haven's Eucalyptus Bark by Charlotte Baglioni (CC0).
  Imported through the live Blender MCP; original 1K maps packed in the library.
  Runtime uses 512px recoloured colour and softened normal maps.
- Restrained 256px baked rock colour variation on mesas and existing boulders.
- Stocked window relief displays, a supplies/service board, veranda rafters and
  lamp housings, coiled air hose, three crates, three oil cans, fascia fixings and
  freezer vent/latch details. All beyond the northern play fence. The shop is
  decorative and not enterable.
- Existing Poly Haven timber and Sketchfab vintage fuel-pump art retained.
- Unused UV data removed from textureless primitives without changing retained
  geometry/image bytes. This saves 2,628,904 bytes from the new exports.

### Exact implementation files
Modified:
- `art/bush-servo/build_landscape.py`, `smoko-landscape.blend`, `LANDSCAPE.md`.
- `rust/crates/bbq_app/assets/models/smoko_landscape.glb` and `smoko_servo.glb`.
- `rust/crates/bbq_app/assets/CREDITS.txt`.
Added:
- `art/bush-servo/build_frontage_detail.py`, `compact_glb.py`, `FRONTAGE-DETAIL.md`.
- `art/bush-servo/smoko-frontage-detail.blend`.
- `art/bush-servo/library/eucalyptus-bark.blend`.
- `art/bush-servo/detail-textures/`: two 512px bark PNGs, four 256px ochre PNGs,
  one 256px warm-rock PNG.
- This checkpoint, scripts/reports/captures and `checkpoints/LATEST.md`.

### Verification
- Final GLBs loaded in actual native Smoko. Three identical before/after cameras,
  nonblack and fresh-file checks, 75-frame camera-drift animation. `review.html`
  includes actual captures and motion; this is not a Blender beauty render.
- Scripted Smoko delivery ended with final_score=100 and carries=0. Existing
  duplicate poster-screenshot warning did not affect the 75 captured frames.
- Backyard native smoke capture passed; no error in the gameplay/capture logs.
- Eight boulder collision bounds match the existing shared Rust layout exactly.
  Frontage bounds assertion confirms all additions have Blender y > 24.
- GLB lengths, buffer references and embedded PNG resources validate. No external
  texture URIs. Authoring Python syntax and git diff whitespace checks pass.
- No Rust source, gameplay, protocol or public browser file changed. Existing
  native binary was reused; no redundant Rust/browser compile was run for this
  asset-only update. Browser/device visual and frame-pacing validation remain.

### Cost and comparison
Combined changed models: 145,698 → 172,486 triangles; 27 → 26 material batches;
13,219,760 → 15,646,880 bytes. Decoded image data increases 4,543,820 bytes.
No new lights, animated vegetation or per-frame simulation.

Same Mac, same native binary, same 1280×800 camera settings; 600 wall-clock frame
intervals per view after warmup. Baseline assets temporarily restored from the
verified backup, then updated assets restored in a finally block. Initial samples
affected by other work were replaced. Final measurements use the compacted assets.

| View | Mean before → after | p95 before → after |
| --- | --- | --- |
| Overview | 8.206 → 8.751 ms (+6.6%) | 9.815 → 10.883 ms |
| Servo | 8.491 → 8.572 ms (+1.0%) | 10.696 → 10.881 ms |
| Frontage | 7.804 → 7.975 ms (+2.2%) | 12.321 → 9.795 ms |

The measured extra mean cost is 0.08–0.55ms. Native averages are about 114–125 FPS;
these are not GPU timestamps or browser/guest-device claims. The overview cost is
larger because foliage occupies more of the view. Browser testing remains important.

### Recovery and remaining work
No commit, push or publication. Public browser files are unchanged. Preserve the
unrelated `concept art/` directory. `reviewed-sha256.json` identifies the reviewed
implementation. Recover the full starting source/assets into a NEW directory from
the verified archive. For selective rollback, first save any later edits; restore
the six modified implementation files above from b10007a or the archive, then
remove only the listed new art files. Preserve checkpoint evidence. Do not reset
the whole tree or overwrite later work. The original `smoko-servo.blend` remains
the repeatable base of the frontage detail builder.

This improves foliage, bark and shop detail; the reference is not yet matched.
Ground-cover density, irregular stone/soil transitions, road surface variation,
distant terrain and atmospheric lighting remain the largest gaps. Foliage is
static; distant-leaf aliasing needs browser/device review. Next proposed art pass:
ground-cover clusters and richer stone/soil borders within the current routes.
