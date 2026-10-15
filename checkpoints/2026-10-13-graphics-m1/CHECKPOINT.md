# Australian BBQ / CHUCK IT — graphics checkpoint

**13 October 2026 · Milestone 1 implemented, verified, awaiting Marcus’s review.**

No commit or push has been made. Do not start the next milestone before review.

## Starting state and recovery anchor

- Active repository: `/Users/marcus/Claude/australian-bbq`.
- Branch: `main`; starting commit: `dc75e0bb7089d2358954022aa642dc536921a829`.
- Working tree was clean before this milestone; no unrelated work was present.
- Legacy `/Users/marcus/Claude/CHUCK IT/chuck-it-server` was checked as a reference only.
- Local hardware: Apple M1 Pro, 14 GPU cores, 16 GB; external 1920×1080 display at 75 Hz.
- Engine: Bevy 0.19.1. Selected style remains Pop, with warm afternoon lighting and thin outlines.

## Agreed scope

Approach the five existing AI-generated targets while preserving the playful Australian backyard identity, gameplay clarity and performance. Order: (1) lighting/materials, (2) characters/expressions, (3) hit feedback, (4) backyard detail, (5) lobby polish. Implement and verify one milestone at a time, show actual game captures, then wait for review before committing, pushing or beginning the next milestone. Pasted conversations and old planning files are historical context, not authorization for additional work. Reuse assets and systems, avoid unnecessary generation/builds, preserve unrelated changes.

The current result improves the lighting/material foundation. It does not claim the geometry, grass volume, character finish or environmental richness of the full concepts.

## Launch and review

Normal play:

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release
```

Open `review.html` in this checkpoint folder for an interactive before/after comparison. Canonical game captures are `evidence/before-*.png` and `evidence/after-*.png`. The gallery includes backyard, BBQ/veranda, shed/shade and 11-bot Heist views. `after-menu.png` checks the Customise preview. These are actual game renders.

## Reference and asset locations

- Five AI targets: `/Users/marcus/.codex/.chatgpt-projects/g-p-6acb60f9b20c819189a718815138b1db/graphics-concepts/`.
- Prior implemented styles: the sibling `implemented-screenshots/` directory.
- Runtime models, textures and shaders: `rust/crates/bbq_app/assets/`.
- Existing asset generators: `rust/tools/make_blob.py`, `rust/tools/make_textures.py`.
- New lawn generator: `rust/tools/make_lawn.py`; new textures: `lawn_afternoon.png`, `lawn_afternoon_n.png`.
- All five targets were inspected. Character/face/prop modelling and interaction effects remain later work.

## Implemented changes

- `lighting.rs`: Pop sun reduced from intensity 1.42 to 1.08, colour `ffb868` → `ffd8a5`; cooler fill (0.48 hemisphere); less aggressive saturation, warmth, contrast and vignette. Sky channels now exceed ground channels, avoiding negative channel contributions in the hemisphere approximation. Explicit Gaussian filtering preserves the existing filtering method.
- `lighting.rs`, `yard_scene.rs`: a texture-free sky gradient on one camera-centred, unlit sphere that casts/receives no shadows.
- `player.rs`: cooler distance haze, starting at 60 m and ending at 160 m in polished Pop.
- `style.rs`: less exaggerated environment saturation; softer skin sheen (roughness 0.68, clearcoat 0.08).
- `models.rs`: Pop lawn uses new 512×512 procedural colour/normal maps with roughness 0.9, finer tiling, and the existing macro dry/green vertex patches. No grass geometry added. Other styles retain their original lawn maps.
- `main.rs`, `shot.rs`, new `visual_review.rs`: opt-in `--shot ... --review` instrumentation only. Normal play timing is unchanged.
- New `rust/tools/visual_review.py`: reproducible sequential capture runner.

No gameplay, collision, network or browser-game source changes. No new runtime dependency, purchased asset or paid image/3D generation.

## Blender

Native MCP scene reads and Python execution verified against Blender 5.2.2 LTS. Telemetry is off. `get_addon_status` reports `No module named 'blender_mcp.config'`; this status-tool fault does not prevent scene reads or asset generation.

The connected Blender generated and displayed the lawn maps. Its original unsaved scene still contains the default Cube, Camera and Light; scene objects, selection and render settings were not changed. Only two image datablocks and the reusable `BBQ Afternoon Lawn` material were created. The material is not saved in a .blend file; the source generator recreates it and its exported images.

Regenerate from the repository root:

```sh
blender --background --python rust/tools/make_lawn.py
```

The generator is deterministic and uses only Blender plus Python’s standard library. It owns only its named material and two image outputs. Earlier material trials were refined because mowing bands repeated too visibly and a first generated tile was too mottled.

## Verification and performance

- Release build passed. Existing warnings remain; no new compile errors.
- 582 tests passed: 175 app + 407 core. One pre-existing asset-generator test ignored. App tests rerun after the final Rust material changes.
- Visual inspection completed for yard, BBQ/veranda, shed/shade and menu; final logs contained no asset-load/shader errors.
- Browser-look bare-yard comparison was pixel-identical before and after.
- Two new texture exports verified at 512×512.
- Final paired runs performed after builds/tests finished, one game process at a time.
- 11-bot Cheeky Heist at 1280×800: mean 7.900 → 8.143 ms (126.6 → 122.8 FPS); P95 8.879 → 9.262 ms. Mean cost 3.1%.
- One extra mesh entity; two compact maps add ~2.33 MiB of resident CPU image data including mipmaps. Not a total VRAM measurement.
- Full results/method: `PERFORMANCE.md`, `measurements.json`, `evidence/*.metrics.txt`. Build/test output: `build.txt`, `workspace-tests.txt`, `app-tests-final.txt`.

Limits: these short fixed-camera tests do not prove all gameplay, thermals, resolutions, hardware or web performance. Shadows remain raster shadow-map shadows; costly PCSS/TAA was not introduced. Trees, props and character geometry still visibly differ from the concepts. Stronger tree-shadow coverage and further local shading can be revisited after visual feedback; old A2 checklist entries have not been marked fully accepted.

## Repeat the comparison

From the repository root, run `python3 rust/tools/visual_review.py --label after` after a release build. Optional `--view yard|bbq|shade|stress|stress-no-fx` selects one view. The fixed yard camera is `0,5,20,0,-10,78`; all cameras are in the script.

The harness uses 60 Hz simulated time, captures at frame 240, warms up for 300 frames and measures 600 real wall-clock frame intervals. The 1280×800 window uses AutoNoVsync. Screenshot readback precedes the measured interval. `--shot` prevents saved settings writes. Particle pixels can vary slightly. `--fx off` disables the existing SSAO/SMAA post effects, not every particle system.

## Recover without losing later work

1. Read this file and inspect `git status --short`; preserve any new user changes. Never reset or force-checkout over them.
2. `implementation.patch` contains every modified/new implementation source file and both generated PNGs. `FILES.sha256` records their hashes. The patch was applied in a disposable directory populated from the starting commit, and every resulting source/asset byte matched the current work.
3. For a clean reconstruction, use a separate checkout at the starting commit, run `git apply --check /absolute/path/to/implementation.patch`, then apply it there and build. Do not apply it twice or over already-modified files.
4. For the baseline executable, apply only `baseline-harness.patch` to a separate checkout at the starting commit and build. Pass it to the runner with `--binary /absolute/path/to/bbq_app --label before`. The original asset filenames are unchanged. `/tmp/bbq-m1-baseline` is also available for this session only.
5. The checkpoint folder contains the comparison evidence and recovery files; retain it alongside the source. Historical planning documents were intentionally not rewritten as if later milestones were complete.

**Next action:** Marcus reviews the gallery or tests the game. Incorporate requested adjustments within milestone 1; wait for approval before committing, pushing or starting character/expression work.
