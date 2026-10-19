# Height customisation and graphics

Started 19 October; verification continued 20 October 2026 (Australia/Melbourne).
Status: **reviewed; commit and push authorised by the user — 20 October 2026**. Verification evidence below describes the approved source state.

## Recovery baseline

- Active repository: `/Users/marcus/Claude/australian-bbq`.
- Starting commit: `6dbfa03a65bfc97dea7947750b8f04419726da22`, main, clean working tree before edits.
- Full pre-edit APFS clone: `/Users/marcus/Claude/australian-bbq-backup-2026-10-19-211233`. Includes Git, assets, prior checkpoints and build output. Verified crate file sizes and Rust source SHA256 against the original before editing.
- Preserve that backup and all previous checkpoint folders. Synced ChatGPT project sources were not edited.
- No new assets, paid services, generated concepts or Blender edits were needed.

Launch the implemented update:

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

## Implemented behaviour and tuning

- **Leg length** is a live 0–100% slider with a reset button. Default/minimum is 0 (the existing legless look), maximum adds 60 cm. Existing Stumpy settings migrate to 33% / 20 cm and Skinny to 50% / 30 cm. All nonzero values use the existing articulated leg rig; feet remain on the ground, with body, camera and held-item attachment height adjusted. Other saved customisation is retained.
- Save files include `leg_length`. Existing appearance packets retain their 14-byte footprint; new continuous values use bounded encoding. **Protocol 12 requires matching updated builds for all players**.
- Clothesline arms and cords now share exact visible geometry with their thin collision volumes. The frame stays stationary, replacing the old cosmetic rotation/lean so collision remains aligned. Fabric remains non-solid. Clearance uses character head height plus leg lift; decorative hats and temporary animation stretch do not enlarge the collision capsule.
- Each of the three stacked crates has its own bounds and landing surface (1.15 m lower tops, 2.30 m upper top). A 15 cm ledge allowance allows the existing jump to land reliably. Movement speed, jump velocity and gravity are unchanged. The allowance is shared with other solid ledges.
- Host derives remote clearance from validated appearance and resolves obstacles, bounds horizontal movement, and sends position corrections. This uses the existing multiplayer movement architecture; it is not a new comprehensive vertical anti-cheat system.
- Graphics changes are deliberately modest: warm timber surfaces, narrow top edging, board seams and contact shading beneath the top crate. Existing cartoon lighting, grass, character assets and palette are preserved. See actual comparison in `review.html`.
- Existing weight-sensitive throws, hit-stop, wobble, score feedback, sounds, weapons, objectives and scoring are preserved.

Tuning: `Legs::MAX_LIFT` / `DEFAULT_PERCENT` in `appearance.rs`; `LEDGE_CLEARANCE` in `movement.rs`; `CRATE_SIZE`, `CRATE_CENTRES` and shared `clothesline_beams()` in `yard.rs`.

## Exact changed source files

| File under `rust/crates/` | Change |
| --- | --- |
| `bbq_core/src/appearance.rs` | Continuous length, legacy migration, save/packet encoding and tests |
| `bbq_app/src/ui.rs` | Slider identifier |
| `bbq_app/src/menu.rs` | Slider, reset, persistence, preview label and opt-in capture length override |
| `bbq_app/src/legs.rs` | Reuse one articulated rig for all nonzero lengths |
| `bbq_app/src/characters.rs` | Expose existing character lookup for height calculation |
| `bbq_app/src/player.rs` | Local collision height, eye and hand lift |
| `bbq_app/src/game.rs` | Bot/remote character height; scripted capture avoids idle-bot return-to-home |
| `bbq_core/src/movement.rs` | Shared obstacle resolution, overhead clearance, ledge allowance and regression tests |
| `bbq_core/src/yard.rs` | Three box colliders, shared clothesline geometry |
| `bbq_core/src/looks_yard.rs` | Matching visible geometry, crate material/edge/seam/contact detail |
| `bbq_app/src/yard_scene.rs` | Stationary clothesline frame |
| `bbq_app/src/online.rs` | Host-derived height and shared obstacle validation/corrections |
| `bbq_core/src/net.rs` | Protocol 12; continuous length message round-trip coverage |
| `bbq_app/src/visual_review.rs` | Opt-in actual climbing replay using scripted normal movement/jumps |
| `bbq_app/src/bots_app.rs` | Preserve input for scripted replay only |

Checkpoint scripts, captures, logs and `checkpoints/LATEST.md` are the other changes. `implementation.patch` and `changed-files.json` enumerate the exact source state.

## Verification

- `cargo test --workspace`: **633 passed** (208 app, 425 core); one existing ignored test. See `tests.log`.
- New checks cover all 101 slider values through save/packet round-trip, legacy migration and invalid input, short/tall clearance, no blocking outside the frame, upward head contact, all three reachable crate tops and stable standing.
- Optimized release build passed (2m 32s); see `build.log`. Seventeen existing unrelated compiler warnings are retained.
- Actual before/after captures use identical cameras and 1280×800 resolution. `after/menu.png`, `short.png`, `medium.png`, `tall.png` show live previews at migrated/default/middle/maximum length.
- `motion.py` drives normal movement and jumps, asserting a grounded landing at 2.30 m. `climb.webp` is a five-second actual simulation replay, not a generated concept. The fixture positions its actor initially, then climbs through physics without teleporting onto the boxes.
- `smoke.py` launches Teams, Heist, Smoko and Race with bots and captures actual play. Backyard is covered by the environment and climbing captures. These are smoke checks, alongside existing mode tests, not a full human playtest of every round.
- `network.py` uses two release instances and the real local relay. Appearance packets are observed without modification; the host roster must retain 0% and 100% lengths. It also checks connected play and a guest throw. Local networking and codec coverage do not establish behaviour under WAN latency or packet loss.

## Performance

Same camera, scene, resolution and warmup, 600 wall-clock frame intervals per sample, no compiler running. Desktop frame timing, not GPU timestamps; these scenes use no moving bots.

| Scene | Before mean / FPS | After mean / FPS | Before / after p95 |
| --- | --- | --- | --- |
| Boxes | 7.654 ms / 130.65 | 7.367 ms / 135.75 | 13.979 / 13.949 ms |
| Clothesline | 8.246 ms / 121.27 | 8.021 ms / 124.68 | 13.059 / 12.324 ms |

No meaningful regression observed. Small timing differences are not proof of a speedup. Mesh assets 2930→2934, materials 742→745, mesh entities 6207→6232 in the box scene. Retained image bytes unchanged (35,752,344 boxes; 38,898,072 clothesline). Final replay-only fixture correction does not affect these normal scene captures.

## Safe rollback

First run a read-only check from the repository:

```sh
python3 checkpoints/2026-10-19-height-graphics/revert.py
```

If the user decides to revert, add `--apply`. This restores only the 15 changed Rust sources from `before-source/`, leaves evidence and unrelated work intact, and **refuses to write anything if any source has later edits**. Rebuild afterwards. Appearance settings are not reset by this source rollback; the full backup remains available for original settings and other files.

To recover the entire starting project, run or clone the dated full backup into a **new sibling directory**; never overwrite the current checkout. Its `rust/` directory uses the same launch command. For work after this checkpoint, preserve current changes before restoring anything.

`recovery-check.txt` records dry-run and isolated restore verification. Never apply the rollback to the live project merely to test it.

## Review boundary

Implementation, tests, release, actual captures, local relay verification and guarded recovery verification are complete. Actual replay reaches and stays grounded on the 2.30 m top box. Local host-accepted roster retains both 0% and 100% leg lengths, and the guest performs a throw. See `network/verification.txt`.

User approved committing and pushing this update on 20 October 2026. Source hashes were rechecked against the tested state before publication. Further milestones require a new request. Previous published animation work remains intact.
