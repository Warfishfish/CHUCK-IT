# Browser mouse capture — 28 October 2026

Starting commit: 885b9f1. Tracked tree clean; unrelated concept art/ preserved. Verified tracked archive `/Users/marcus/Claude/australian-bbq-before-browser-mouse.tar` (tar listing passed). No unrelated files modified.

Diagnosis: sync_cursor only ran on Screen changes and read_input trusted CursorOptions.grab_mode rather than the browser's actual pointerLockElement. Browser capture can reject a deferred request, or release capture on Escape/focus loss while the engine still assumes it is locked. This allows the cursor to reach screen boundaries and interrupts turning.

Changes: DOM bridge exposes Playing to the browser and reads actual capture. Browser capture requests now happen directly in an explicit overlay button's trusted click. Loss/error shows a retry prompt; menus release capture; tab hiding/window blur releases capture. Keyboard/mouse input and pending charged throws are cleared while unlocked. Online simulation continues; this does not pause other players. Native cursor behavior unchanged. No forced capture after Escape and no fullscreen requirement.

Files: rust/crates/bbq_app/src/{browser_pointer.rs,main.rs,menu.rs,player.rs}; Cargo.toml web-sys DOM features; rust/web/{index.html,pointer-lock.js,pointer-lock.test.mjs}; build_web.sh copies the helper into the browser bundle.

Verification: 242 app tests pass, one existing ignored. Node pointer-lock lifecycle tests pass (explicit capture, loss, rejection, retry, blur, menu exit). Browser optimized build passed; public HTML/helper/JS/WASM match local hashes. Actual updated Chrome menu loaded. Chrome window became unavailable during the click-through gameplay check; actual trusted-click capture/turn/Escape/recapture remains a user test. User approved commit and push including browser publication. Live browser files reverified against the local bundle before committing. Test doubles cannot establish browser permission behavior; actual manual input remains an important final check.

Recovery: preserve later changes, restore the listed tracked source files from 885b9f1 and remove new helper/module/test files, then rebuild browser. Do not reset unrelated work.

Local launch: cd /Users/marcus/Claude/australian-bbq/rust then ~/.cargo/bin/cargo run -p bbq_app --release -- --menu
