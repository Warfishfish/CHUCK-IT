# Prototype 0.23.50 beta readiness

Desktop and the full Rust/WebGPU game now have saved controller button remapping in Settings → Controller, D-pad navigation on the pause and results screens, and distinct item art in the HUD. Start remains available for pausing. The legacy browser has the new prototype label. It already has illustrated item slots.

[Mac controller settings](controller-mac.png) · [Held water balloon HUD](hud-mac.png)

The Mac archive launched from outside the repository. The desktop packaging workflow includes the game assets and credits in both platform archives. The seven recorded Heist clips are credited and have peak headroom of 1.5 to 3.1 dB.

Workspace tests passed with 756 successes and three ignored. The bot hit-recovery regression passed. The crowded solo Heist simulation completed 8, 12 and 16 banks with 20 participants on the two-, three- and four-flag layouts respectively.

A match with friends is still needed to judge the relative value of passes, vaults and Dazza’s tunnel, and to rate carried-camera comfort. Physical-controller and Windows hands-on play also need validation.

## Controller and menus

A simulated standard Gamepad in Chrome changed [Jump to the right face button](pad-rebound.png), swapped the old assignment, and reloaded with the new binding intact. Start opened [Pause](pad-pause.png), the bottom face button resumed, and the same button started another round from [Results](pad-results.png). The game and settings were served by the Cloudflare preview. The D-pad moved focus to [Quit to menu](pad-focus.png) and the bottom face button activated it. Physical controller hardware was unavailable.

## WebGPU water and grass audio

A local QA harness staged the same sprinkler, pool entries and actual balloon throw on both presets. Captures show the [Low sprinkler](water-low-sprinkler-1.8.png), [High sprinkler](water-high-sprinkler-1.8.png), [Low pool ripples](water-low-pool.png), [High pool ripples](water-high-pool.png), [body drips after pool entry](water-high-entry.png) and [balloon-hit wetness](water-high-balloon.png). In both runs the pool set the wet timer and the balloon applied the 80% speed effect. The fixture was kept out of the published build.

Chrome loaded all three Heist grass samples on each layout. The Web Audio probe measured silence while still and nonzero footstep samples while walking on the two-, three- and four-flag maps. Legacy JavaScript syntax and the WebGPU pointer-lock test both passed.

## Mixed-client carry

A controlled native host and WebGPU guest connected through the game relay in 1v1 Heist with zero bots. The guest saw [the first corner](carry-first-corner.png), [the second corner](carry-second-corner.png), [the throw](carry-flight.png) and [the return to first person](carry-first-person.png). A rejoin attempt was refused during play as expected, then [admitted in the lobby](rejoined-lobby.png) after the host ended the round. The production WebGPU build was used as the guest. The desktop host used a local fixture to stage the carry.

The workspace tests also cover host validation of objective passes, normal jumps with a teddy at each crate vault, and exit from every tunnel branch after the hatch closes. Human defenders and human carry-camera comfort are still playtest items.

## Mac frame pacing

High preset, 1280 × 800, 600 wall-clock frame intervals per map after 300 warmup frames, with VSync off. These are fixed-view runs on one Mac.

- Backyard with six bots had a 7.0 ms 95th-percentile frame time.
- Bush Servo with six bots had a 9.2 ms 95th-percentile frame time.
- Four-flag Heist with 19 bots had a 7.7 ms 95th-percentile frame time.

All three runs stayed below 16.7 ms per frame in the sample.

## Desktop beta archives

[Build 44867819649](https://github.com/Warfishfish/CHUCK-IT/actions/runs/44867819649) passed on macOS ARM64, Windows x64 and Ubuntu. The Mac app opened through macOS Launch Services and also loaded gameplay with a held item from outside the repository. The Windows ZIP passed the archive integrity check and contains the Windows executable, runtime assets and credits. The Windows executable was not launched on a Windows PC. These are test builds for sideloading.
