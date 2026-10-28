# Browser joining-player movement fix — 28 October 2026

User reported browser guests stuttering and having trouble moving while Stage 6 publication was authorized. Paused publication to investigate first. Starting Git commit bdbe25c; approved Stage 6 changes remained in the working tree. `pre-fix-tracked.patch.gz` records those tracked changes; Stage 6 art/evidence and its reviewed hashes remain preserved. Unrelated `concept art/` remains untouched.

## Finding and change
`online_world_apply` compared today's local position with the host's delayed position from the all-player wet-state list and snapped x/z whenever the gap exceeded 0.6m, including dry players. At normal 6.2m/s, about 100ms of delay is enough to trigger that, providing a concrete rubber-banding mechanism. This code finding does not establish whether the user's devices also have rendering/frame-rate problems.

Player reports now carry sequence numbers. The host acknowledges the last report consumed by the fixed simulation. Guests compare the host's accepted position with the corresponding stored prediction, then preserve movement performed since that report. Host speed/collision/wet/race restrictions remain. Corrections adjust both camera interpolation endpoints. History is bounded to 80 reports, consumed once, and cleared across phase/mode transitions or forced moves from hits. Unknown, repeated and expired acknowledgements cannot cause a teleport.

Wire protocol 21 includes both Stage 6 collision compatibility and new movement acknowledgements. All peers must refresh/rebuild together, preferably close the old room and create a fresh room. Host browser should remain active during play.

## Files
- bbq_core/src/reconcile.rs: bounded prediction history and focused latency/correction/reset tests.
- bbq_core/src/lib.rs: module registration.
- bbq_core/src/net.rs: sequenced player reports, round-trip coverage and protocol 21.
- bbq_core/src/net_world.rs: movement acknowledgement in per-player world state.
- bbq_app/src/online.rs: history recording, host simulation acknowledgements, guest reconciliation and acknowledgement timing test.
- bbq_app/src/online_world.rs: snapshot defaults, with acknowledgements filled by host sender.
Paths above are beneath rust/crates/.

## Verification
445 core tests and 240 app tests pass (one existing app test ignored). New focused tests cover simulated 100–800ms delay for walk/sprint/wet-speed reports, correction applied once with later prediction preserved, stale/unknown reports and phase resets, and acknowledgements remaining tied to simulation rather than newly arrived reports. Existing wire round-trip and local two-peer relay tests pass. This is automated verification; no fresh human browser playtest or device-specific graphics diagnosis has been completed.

Native/browser release build and public delivery checks are recorded below when complete. Existing Stage 6 captures document the art; no new graphics changes were made for this fix.

## Recovery
Preserve subsequent work first. To roll back only this fix, restore online.rs, online_world.rs, lib.rs and net_world.rs from bdbe25c; restore net.rs from bdbe25c plus the protocol-20 change in pre-fix-tracked.patch.gz; remove only new reconcile.rs. Retain Stage 6 art/layout and its other source changes. Never mix wire versions. Rebuild and republish native/browser together. Do not reset the whole working tree or touch unrelated concept art.

## Published
Native release and browser web-profile builds passed with existing warnings. Public HTML, JavaScript, WebAssembly and small-prop GLB matched the generated files byte-for-byte over verified HTTPS; evidence/publication-http.json records hashes. Two-way public WebSocket transport passed in an isolated room. Live URL: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Everyone must refresh and start/join a fresh room on protocol 21. No fresh human browser session has validated the reported symptom yet. This publication includes the user-approved Stage 6 props.
