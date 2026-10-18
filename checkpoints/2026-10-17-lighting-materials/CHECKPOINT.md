# Lighting and materials — 17 October 2026

In progress. User approved lighting/material implementation with a revert point. Preserve the race and Smoko changes; no commit/push.

Starting commit/status and full tracked patch saved here. starting-untracked.tar.gz contains untracked Rust sources/tools. before-source contains exact original versions of files selected for this visual change. Recover full start in a separate checkout using the starting commit, patch and archive; never reset the working checkout. Selective visual revert will be provided after verification.

Scope: warmer afternoon lighting, readable shadows, richer lawn and more coherent matte materials. Reuse current assets and geometry. No character redesign, gameplay changes or new generated textures.

## Completed — awaiting user review
Changed only lighting.rs, models.rs and yard_scene.rs. Pop lighting: slightly lower warm sun, increased neutral/cool sky fill, gentler contrast/saturation/vignette. Existing Gaussian shadow filtering and SSAO retained; no expensive soft-shadow feature added. Lawn: greener base tint, matte finish and adjusted texture scale; race ground now shares the lawn tile. No new meshes, textures or lights. No gameplay edits.

Release build passed (existing 17 warnings). 200 application tests passed, one existing ignored. git diff --check passed. Actual before/after captures for Heist, Smoko and Race were visually inspected. Full runtime logs and metrics are in before/ and after/. Final captures use frame 900 (rather than 360) to allow all characters to load; camera, resolution and settings match. Live entities can differ slightly. No new multiplayer session was required for visual-only settings.

### Performance
- Backyard / Heist: 8.405 → 8.095 ms mean (-3.7%). P95 9.730 → 10.284 ms.
- Smoko: 7.956 → 8.034 ms mean (+1.0%). P95 9.134 → 9.420 ms.
- Race: 6.106 → 6.171 ms mean (+1.1%). P95 7.183 → 7.234 ms.
No meaningful regression observed. Same texture memory and loaded mesh/material counts; one transient entity difference in Heist. Local timing samples vary and are not GPU measurements. No claimed speedup.

### Revert just this update
From the repository run:
`python3 checkpoints/2026-10-17-lighting-materials/revert.py --apply`
Then rebuild/play with:
`cd rust && ~/.cargo/bin/cargo run -p bbq_app --release`

The revert checks all three file hashes before changing anything, refusing if later edits would be overwritten. It restores their exact pre-update versions, including the existing race implementation in yard_scene.rs. It does not reset Git, touch gameplay files or remove the race/Smoko work. After reverting, this document and review remain historical records of the trial; their after images no longer describe the running game. Reapplying visual-only.patch restores this trial.

Recovery verification: reverse-applied visual-only.patch in a temporary directory; all three files matched their before-source hashes. `revert.py` dry-run also passed. changed-files.json records before/after hashes. To recover the entire starting state separately: start at starting-commit.txt, apply starting-tracked.patch, then extract starting-untracked.tar.gz. Preserve checkpoint folders separately.

### Limitations and next step
This is not a match to the AI concept’s full finish: dense grass geometry, improved foliage and character models are outside this pass. Shadows retain the existing Gaussian filtering; the change is their light balance and angle. Review review.html or play the updated build before deciding to keep/revert. No commit, push or further milestone without review.
