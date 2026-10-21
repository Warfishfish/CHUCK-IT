# Online publication — verification in progress

User approved committing/pushing all reviewed pending updates and selected Cloudflare for browser access.

Scope: Cartoon Backyard, Unarmed Pickup, Natural Graphics and Walking/Turning. All 34 reviewed sources matched their combined checkpoint hashes before publication work. Starting HEAD: `8badc1bf4a14b2f82fecd39b7d9851022c6f8871`; `origin/main` matched, with no incoming changes.

Publication corrections:
- `rust/crates/bbq_app/src/models.rs`: register texture handles in PreStartup before any Startup scene/material creation. The browser smoke check exposed untextured pale scenery from undefined Startup ordering. Before-file saved here as `models-before.rs`.
- `.github/workflows/rust.yml`: add `libfontconfig1-dev` to Linux dependencies. Previous run 39296575934 succeeded on macOS/Windows but failed on Linux because `fontconfig.pc` was missing. Prior workflow saved here. Fresh CI result remains to be checked after push.
- `HOW_TO_RUN.md`: document rebuilding browser assets and using a temporary Cloudflare link.

Native tests: 650 passed (220 app + 430 core), one existing ignored test. Browser/native builds and visual check pending.

Cloudflare quick tunnel: `https://effective-friendly-logistics-sharing.trycloudflare.com/rust/`.
The server is the existing `server.js` on port 3000. PID files/logs in `/tmp/australian-bbq-online-server.*` and `/tmp/australian-bbq-cloudflare.*`. Processes started detached, without changing account or DNS configuration. Keep the Mac awake. This temporary link expires when the tunnel stops; restarting creates a new address.

Public HTTPS page returned HTTP 200. Two real WebSocket peers joined through the Cloudflare URL and successfully relayed messages in both directions (`relay-test.log`); both test connections then closed. This is a transport check, not a full remote gameplay session.

Generated browser assets live in ignored `public/rust/`; they are served by the running Node server. Source changes are what GitHub stores. No paid hosting or permanent deployment configured.

Recovery: existing full backups and checkpoint reverts remain. If reverting the Natural Graphics checkpoint after this publication fix, first account for the one-line models.rs startup-order change; its guard correctly refuses newer edits. Do not blindly apply old checkpoints over later source changes.
