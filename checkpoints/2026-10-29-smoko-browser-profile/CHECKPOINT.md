# Smoko browser frame profiler — starting checkpoint

Starting commit 4a8d070, source tree tracked-clean. Unrelated untracked concept art is preserved. Verified archive: /Users/marcus/Claude/australian-bbq-before-smoko-browser-profiler.tar.

Purpose: show each local browser peer rolling render-loop frame timing and fixed-step backlog while playing Smoko. This gathers same-device evidence to separate local frame stalls from multiplayer movement corrections. It does not send telemetry, affect scoring/physics, or change protocol. Opt in with ?perf=1; normal play has no overlay or rolling sample work.

Implementation: browser-only opt-in panel, active solely in Smoko Play. Five-second rolling local render-loop intervals: approximate FPS, average/p95 frame interval, frames at or above 33 ms, and samples with fixed-step overstep at or above 16.7 ms. Labels local role HOST/GUEST/SOLO. No telemetry or multiplayer changes. Disabled entirely unless query string contains perf=1. Never visible in other maps.

Browser compilation passed after adding a strict visibility check so the panel only appears during Smoko gameplay. The web build reports 19 pre-existing warnings; no profiler-specific warnings. Keep diagnostics local and do not publish until review. To roll back only this feature, remove `browser_perf.rs` and its wasm-only module/plugin registration from `main.rs`; do not reset the tree.

Delivery status: implemented locally, awaiting review. No gameplay capture or live browser interaction is claimed because the compiled bundle has not been published. After review/publication, use the online opt-in URL with `?perf=1` on both host and guest and collect matched readings. This panel measures browser-loop and fixed-step timing only; it cannot directly measure network latency or GPU execution time.

Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu
