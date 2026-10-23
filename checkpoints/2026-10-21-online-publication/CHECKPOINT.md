# Online publication — verified 23 October 2026

User approved committing/pushing all reviewed pending updates and selected Cloudflare for browser access.

Scope: Cartoon Backyard, Unarmed Pickup, Natural Graphics and Walking/Turning. All 34 reviewed sources matched their combined checkpoint hashes before publication work. Starting HEAD: `8badc1bf4a14b2f82fecd39b7d9851022c6f8871`; `origin/main` matched, with no incoming changes.

Publication corrections:
- `rust/crates/bbq_app/src/models.rs`: register texture handles in PreStartup before any Startup scene/material creation. The browser smoke check exposed untextured pale scenery from undefined Startup ordering. Before-file saved here as `models-before.rs`.
- `.github/workflows/rust.yml`: add `libfontconfig1-dev` to Linux dependencies. Previous run 39296575934 succeeded on macOS/Windows but failed on Linux because `fontconfig.pc` was missing. Prior workflow saved here. CI run 39603223992 for 04b69f4 passed on macOS, Ubuntu and Windows.
- `HOW_TO_RUN.md`: document rebuilding browser assets and using a temporary Cloudflare link.

Publication verification: 650 tests passed (220 app + 430 core), one existing ignored test. Browser and native release builds passed (web-build.log and native-build.log). The public browser scene rendered with the corrected textures. Subsequent host-menu work recorded 654 distinct passing tests and rebuilt the browser output; see ../2026-10-22-host-menu/README.md.

Cloudflare quick tunnel: `https://effective-friendly-logistics-sharing.trycloudflare.com/rust/`.
The server is the existing `server.js` on port 3000. PID files/logs in `/tmp/australian-bbq-online-server.*` and `/tmp/australian-bbq-cloudflare.*`. Processes started detached, without changing account or DNS configuration. Keep the Mac awake. This temporary link expires when the tunnel stops; restarting creates a new address.

Public HTTPS page returned HTTP 200. Two real WebSocket peers joined through the Cloudflare URL and successfully relayed messages in both directions (`relay-test.log`); both test connections then closed. This is a transport check, not a full remote gameplay session.

Browser assets in `public/rust/` became tracked in f734bac and were rebuilt in 6e7170f. They are served by the running Node server. No paid hosting or permanent deployment configured.

## Resume verification — 23 October 2026

- The reviewed updates and publication fixes were already committed in 1a7de5c. Current game commit 04b69f4 includes the newer host-map settings and persistent navigation. Fresh fetch confirmed main matched origin/main, with a clean working tree before these documentation corrections.
- Current tracked browser output matches the checked-out commit; no redundant rebuild was needed.
- The existing Cloudflare URL returned a healthy response. A fresh two-peer relay smoke test passed in both directions and closed its test room. This verifies transport, not a full remote gameplay match.
- Reloaded the public URL in Chrome: the game menu, persistent navigation, character preview and textured Backyard rendered. No game warnings/errors were captured; a browser-extension content-script error was present.
- GitHub Rust build run 39603223992 passed all three desktop platforms.
- Existing server and tunnel were kept running. The link still depends on this Mac remaining awake and both processes staying alive.
- Corrected HOW_TO_RUN.md to reflect the now-tracked browser output. No gameplay changes made during this resume.

Recovery: existing full backups and checkpoint reverts remain. If reverting the Natural Graphics checkpoint after this publication fix, first account for the one-line models.rs startup-order change; its guard correctly refuses newer edits. Do not blindly apply old checkpoints over later source changes.
