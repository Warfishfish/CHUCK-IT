# Host settings and persistent navigation — 22 October 2026

The host lobby now offers Backyard (Free for all / Teams), Smoko yard and Race oval, plus a direct link to the complete Game settings panel. Maps use the existing mode-specific world setup and network snapshots. Teddy Heist retains its existing solo-only restriction. Race's fixed time limit and disabled yard features are explained instead of offering ignored controls.

Customise previously had only a button inside the preview card. Hiding that preview when changing tabs or entering a lobby removed the route back. All five menus now have permanent navigation above a separate scrolling content area; changing tabs resets the content scroll.

## Verification

- `cargo test --workspace`: 223 app tests and 430 core tests passed; 1 existing app test ignored.
- `cargo test -p bbq_app menu_tests`: all 4 passed after adding the scroll-navigation regression (654 distinct passing tests overall).
- The navigation regression failed before the fix: `Cannot reopen Look from Mates (online=false, host=false)`.
- Map tests exercise menu clicks, round setup, world message encoding/decoding and guest application, including switching from Race back to Backyard.
- `node rust/tools/relay_test.js`: passed. Fixed the test's ordering so its intentional oversized-message disconnect happens after the expected direct reply arrives.
- Native build and production WebAssembly build passed. Rebuilt `public/rust/bbq.js`, `bbq_bg.wasm` and its gzip copy.
- Browser check at `/rust/`: navigation remains visible while settings scroll; With mates → Customise reopens at the top.
- Two native clients connected to a local relay: Host and Guest appeared in the same lobby, then both entered the host's selected Smoko round. Guest screenshot showed the Smoko delivery instructions and both players on the scoreboard.

`host-lobby.png` shows the updated lobby selectors and permanent navigation.
