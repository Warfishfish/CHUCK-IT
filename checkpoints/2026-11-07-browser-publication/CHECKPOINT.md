# Graphics committed and browser publication — 7 November 2026

Source commit: 455dbfa. Includes completed shelter timber/roof, both picnic settings and shelter-side esky/bin. Source pushed to origin/main. User explicitly authorised source and browser publication.

Browser update is asset-only: unchanged tested JS/WASM retained, two reviewed GLBs replaced and release.json updated. Prior bundle backed up at `/Users/marcus/Claude/australian-bbq-browser-before-prop-detail-20261107`.

Live URL: https://wooden-cinema-worship-menu.trycloudflare.com/rust/

Previous temporary tunnel expired (tunnel not found). New tunnel PID60165; log /tmp/bbq-cloudflare-20261107.log. Existing localhost:3000 server reused. Mac/server/tunnel must remain awake/running.

Six live hashes verified: HTML, JS, WASM, both GLBs and release metadata. Two-way public WebSocket relay on /ws passed; initial check at root returned400 because the relay lives at /ws. Browser menu and Bush Servo load checked; screenshot included. This is not a full new multiplayer playtest.

Existing source/capture checks remain in the four graphics checkpoints. No Rust changes; no unnecessary compilation. Untracked concept art preserved and not committed.

To roll back browser only, copy the previous bundle into a new staging directory, verify it, then swap public/rust while keeping the current bundle as another backup. Do not overwrite source work.

Local launch:
```
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
