# Australian BBQ / CHUCK IT — characters and expressions

**14 October 2026 · Milestone 2 implemented and verified, awaiting Marcus’s review.**

Nothing committed or pushed. Marcus’s requests to continue/resume authorised this milestone; they did not authorise a commit, push or the next milestone. The previous lighting milestone remains intact.

## Starting point

Active repository: `/Users/marcus/Claude/australian-bbq`, branch `main`, commit `dc75e0bb7089d2358954022aa642dc536921a829`. Starting working tree contained the completed, uncommitted milestone 1. No unrelated changes were found. Legacy CHUCK IT and synced project sources were not modified. Historical conversations/plans were treated as context.

Milestone 1’s recovery patch, hashes and evidence remain in `../2026-10-13-graphics-m1/`. Its old “next action” is superseded by this checkpoint. Initial copies of files changed here are in `baseline-source/`; these are reference copies, not files to copy over later work blindly.

## Agreed scope and result

Order remains: lighting/materials → characters/expressions → hit feedback → backyard detail → lobby polish. One reviewable milestone at a time. Show implemented captures and performance evidence before committing, pushing or continuing.

This pass reuses the existing four silhouettes, appearance options, expression priorities and animation systems. It adds smoother, larger Pop face features and eye expression without modifying collision, throwing, bot decisions, network data or saved customization.

- Thirteen shared Blender-built surfaces replace segmented mouth/brow beads in Pop. Smile/smirk are continuous tapered curves; open mouths have distinct teeth/tongue layers. Surface curvature follows the head, including at side angles.
- Existing throw, hurt, sipping and ready reactions use the new geometry. Laughing and dancing use the cheerful face when no higher-priority action reaction is active.
- Pop characters have staggered brief blinks, focused throw squints, widened hurt eyes and small warm pupil highlights. These eye animations/highlights apply to characters in the yard; menu/lobby face geometry updates through the shared face system, while their eye animation/presentation remains later polish.
- Corrected the yard-character outline traversal: the Body container was outlining every descendant, duplicating shells and drawing stray rings on eyes. Only appropriate named body parts receive outlines now.
- All mouth, brow, hair and hat choices remain. Explicit no-mouth/no-brows choices stay empty, even during reactions. Existing eyes retain their individual movement styles.
- Current/Clay retain their legacy mouth/brow rendering and eye behaviour. Current regression captures differ at only 16 of 1,024,000 pixels and look unchanged.

No new runtime dependency, texture, paid asset, image generation or 3D generation service. No wholesale GLB rebuild. The JSON mesh pack is embedded at compile time; regeneration requires rebuilding the game.

## Files changed in this milestone

- `rust/crates/bbq_app/src/face.rs`: shared face surfaces, expression integration and blink envelope.
- `rust/crates/bbq_app/src/characters.rs`: eye animation/highlights and corrected outline traversal.
- New `rust/crates/bbq_app/src/face_meshes.rs`: shared mesh loading and appearance-to-mesh mapping.
- New `rust/crates/bbq_app/assets/models/face_pop.json`: 13 reusable surfaces, 8,624 triangles total across all variants, 361,921 bytes. Only active facial parts are drawn.
- New `rust/tools/make_faces.py`: deterministic Blender source generator.
- `rust/crates/bbq_app/src/main.rs`: registers the new module.
- `rust/crates/bbq_app/src/visual_review.rs`: optional staged expression selection, only with both `--shot` and `--review`.
- `rust/crates/bbq_app/src/shot.rs`: better portrait framing for the existing `--bot-head` capture flag.
- New `rust/tools/character_review.py`: sequential matching captures, including all five expressions, four body shapes, no-face choice, side view, Current regression and natural 11-bot Heist.

The patch and hashes cover these nine implementation files. Checkpoint documents/evidence are separate review artifacts.

## Blender connection and assets

Live scene access and generation succeeded with Blender 5.2.2 LTS. The generator created named mesh datablocks, exported their positions/normals/triangles, and preserved the original scene’s Cube, Light and Camera, selection and render settings. The Blender `look` tool also displayed an actual implemented game capture. Generated datablocks are not a saved .blend project; the source script recreates them.

Regenerate in a separate background Blender process from the repository root:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --python rust/tools/make_faces.py
```

Visual references remain in the ChatGPT project mirror’s `graphics-concepts/`, especially `02-character-expressions.png`. These are concept targets. Evidence here is from the real running game.

## Launch, captures and verification

Normal play:

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release
```

Repeat final captures from the repository root, after builds finish:

```sh
python3 rust/tools/character_review.py --label after
```

`--view normal|grit|ouch|sip|cheer|pear|egg|gumdrop|profile|none|stress|current` selects one view. `--binary /absolute/path/to/bbq_app` selects a baseline executable. `--shot` prevents settings writes. Portrait expressions are deliberately staged using the actual renderer; the stress view uses ordinary gameplay. Camera and fixed simulation timing match within each before/after pair. The wider Egg capture keeps its tall silhouette in frame.

- Release build passed, with existing warnings (`build.txt`).
- 177 app tests passed, one existing ignored test (`app-tests.txt`). New checks cover embedded mesh integrity, required choices, no-face behaviour, blink timing and reaction aperture bounds. Core gameplay was unchanged; its tests were not redundantly rerun in this milestone.
- All 12 capture pairs completed, with no logged asset/shader errors. Front/side faces, all silhouettes and no-face choices were visually inspected.
- 11-bot Heist: **121.6 → 127.5 FPS**, mean **8.222 → 7.845 ms** (−4.6%), P95 **9.528 → 8.714 ms**. Zero measured stress frames above 16.67 ms.
- Resident mesh assets **3411 → 2897**; mesh entities **8414 → 7736**; materials **910 → 911**; CPU image bytes unchanged. These are not draw-call or total VRAM measurements.
- Full timings/limits: `PERFORMANCE.md`, `measurements.json`, `evidence/*.metrics.txt`. Short tests on this M1 Pro do not guarantee every match, hardware configuration or thermal condition.
- Recovery patch applied successfully in a disposable reconstruction of milestone 1; all nine resulting files matched byte for byte (`recovery-verification.txt`).

`review.html` is the before/after gallery. Its local assets and script syntax were checked. Automatic browser opening was blocked because the browser tool disallows file URLs; no workaround was used and no new browser UI verification is claimed. Open the file manually to use the comparison controls. Actual PNG captures were inspected directly.

## Recovery after interruption

1. Read this checkpoint and inspect Git status; preserve any newer user work. Never reset/force-checkout over it.
2. For a clean reconstruction, use a separate checkout at `dc75e0bb7089d2358954022aa642dc536921a829`.
3. Apply milestone 1’s `implementation.patch` there (first `git apply --check`, then `git apply`).
4. Apply this checkpoint’s `implementation.patch` in the same way. It is a **delta on top of milestone 1**, not an independent patch against the starting commit.
5. Verify `FILES.sha256` from the repository root, then build. Keep both checkpoint directories and their evidence alongside the source.
6. To reproduce the M2 baseline, start from milestone 1 plus this folder’s `baseline-instrumentation.patch` only. That adds staged expressions and matching portrait framing without any visual changes. `/tmp/bbq-m2-baseline` is a temporary convenience copy of that executable, not a durable backup.

## Review boundary and remaining limitations

The new faces are cleaner and more readable; they do not reproduce the concept’s detailed skin, hair, footwear or cinematic finish. Body silhouettes and materials are reused. Blinks are driven by existing character animation, and the comparison gallery contains stills rather than a full animation QA reel. Lobby connection flows and multiplayer sessions were not exercised in this visual milestone.

**Next action:** Marcus reviews the captures or plays the game. Make any requested adjustments within this milestone. Wait for his review before committing/pushing or beginning milestone 3 (throw/impact/scoring feedback). Do not add backyard or lobby work silently.
