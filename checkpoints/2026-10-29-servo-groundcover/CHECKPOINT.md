# Bush Servo outer ground-cover pass — ready for review (29 October 2026)

Continuation of `2026-10-29-servo-detail`. The live Blender landscape builder now
adds low spinifex fans and grouped, partly buried ochre stones in the outer belt.
The centres stay outside the rectangular play fence. The road, route, existing
boulder bounds, gameplay and protocol data are unchanged. The additions are
static opaque geometry in the existing landscape mesh. The grass reuses the
existing foliage palette; grouped stones add one material and one small colour
map. There are no alpha cards, new lights, simulation or extra per-frame work.

## Review

[In-game overview](review.html) · [Updated landscape authoring notes](../../art/bush-servo/LANDSCAPE.md).
The overview is a fresh native Smoko capture from the current GLB. Several later
capture attempts returned black images despite a successful process exit; those
were excluded. Following the user's approval, the browser bundle was rebuilt
with `rust/tools/build_web.sh` and is live at
`https://carpet-venture-charlotte-temple.trycloudflare.com/rust/`. The public
page returned HTTP 200, and the served JS, wasm, and both Smoko GLBs matched the
local build hashes. The browser loaded the menu and entered the Smoko Bush Servo
game screen.

## Build record

The live Blender MCP ran `art/bush-servo/build_landscape.py` in Blender 5.2.2 LTS.
Export reported 40 trees, 8 existing perimeter cover rocks, 61,577 polygons,
12 materials, and 8,431,572 bytes after removal of 1,624,388 unused UV bytes.
The landscape GLB is 8.0 MB. This replaces the 7.70 MB landscape GLB from the
preceding detail checkpoint; the GLB grows by about 0.74 MB. The added geometry
has no new material batch beyond the new stone material/map. These are export
counts, not a measured device performance comparison.

The overview capture completed in the existing native binary. Its frame timing
was inconsistent with the earlier matched baseline (approximately 16.7 ms in
this run versus 8–9 ms in the earlier run), so no performance claim is made.
It is a single visual capture, not a device/guest performance check.

## Changed files

- `art/bush-servo/build_landscape.py`, `LANDSCAPE.md`, `FRONTAGE-DETAIL.md`.
- `art/bush-servo/smoko-landscape.blend` and
  `rust/crates/bbq_app/assets/models/smoko_landscape.glb`.
- `art/bush-servo/detail-textures/desert-stone-edge.png`.
- This checkpoint and the entry at `checkpoints/LATEST.md`.

The PNG already exists as a shared output from Blender's texture-authoring
step; the builder deterministically rewrites it during export.

## Recovery and remaining work

The user approved committing and pushing the broader art pass, including this
browser bundle. Preserve the unrelated `concept art/` directory. A pre-pass
landscape builder, Blender scene and GLB are saved under `recovery/`; the GLB
matches the preceding checkpoint's SHA-256 exactly. To return to the preceding
detail pass, copy those three files back to their corresponding project paths,
remove `art/bush-servo/detail-textures/desert-stone-edge.png`, and remove the
ground-cover sections from the two art notes and `checkpoints/LATEST.md`. Save
any later edits first. The previous detail checkpoint has the broader verified
backup instructions.

The outer belt now has more ground scale, but ground/stone transitions, road
surface variation, distant terrain and atmospheric depth remain open art gaps.
