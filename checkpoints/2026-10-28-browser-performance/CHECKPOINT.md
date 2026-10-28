# Browser performance — 28 October 2026

Starting commit: 1e76d4e. Tracked tree clean; unrelated untracked `concept art/` preserved. Verified tracked recovery archive: `/Users/marcus/Claude/australian-bbq-browser-performance-start.tar` (tar listing passed). Git starting commit also preserves the exact tracked state. Archive does not include unrelated untracked art.

Browser-only Balanced defaults: 2048 shadow map, Medium ambient occlusion (8 versus 18 samples). Native retains original rendering. Browser `?quality=high` restores previous graphics settings for comparison. No simulation, network, collision or scoring changes.

Shadow-map texels reduce by 75%; ambient-occlusion samples reduce from 18 to 8. Canvas resolution remains unchanged: Bevy/Winit CSS resizing makes a simple window scale override unsuitable for a reliable render-resolution cap. These are workload estimates, not measured FPS improvements. Actual guest-device test remains necessary.

Recovery: inspect pending work first; restore only `rust/crates/bbq_app/src/main.rs` and `lighting.rs` from starting commit and remove the new browser_quality.rs module after preserving any later changes. Rebuild browser to roll back deployed bundle. Do not reset unrelated work.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust` then `~/.cargo/bin/cargo run -p bbq_app --release -- --menu`.

Verification: 242 app tests passed (1 existing ignored), including two quality-selection tests; three movement reconciliation tests passed, covering 100–800 ms delay, stale reports and correction-once behavior. Final quality tests repeated after removing the window-scale experiment. Browser optimized build passed; public HTML/JS/WASM match local bytes (evidence/public-files.json). Two-way public relay passed. Final browser menu renders in Chrome with no captured console errors. Native final release build passed (existing warnings only).

Limitations: no guest-device frame-time measurement or real two-person playtest was available. Rendering work is reduced, but this does not establish the cause of that device's stuttering or guarantee a fix. No FPS claim. Existing network correction remains unchanged. Both peers should refresh and create a new yard for testing. Keep the host game tab visible while playing.

Changed source files: browser_quality.rs (profile selection/constants/tests), main.rs (browser shadow budget), lighting.rs (browser SSAO quality). Rebuilt public/rust/bbq.js, bbq_bg.wasm and compressed WASM. No assets or gameplay rules changed.

Live test: https://carpet-venture-charlotte-temple.trycloudflare.com/rust/ . Optional previous-quality comparison: append ?quality=high. Temporary link requires this Mac/server/tunnel to remain running.

