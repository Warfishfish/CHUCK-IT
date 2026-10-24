# Main menu and hosting flow — 23 October 2026

Status: complete, awaiting review. Verified 24 October 2026. Nothing committed or pushed.

Starting commit: `35b7e36c98bbd6b5e57cbf0eeb7da61f3e287b60`. Working tree was clean.
Recovery archive: `/Users/marcus/Claude/australian-bbq-backup-2026-10-23-main-menu.tar.gz` (1419 entries; menu source verified byte-for-byte).
SHA-256: `7848fc86733e58d7f4c532c16c55e4ef1407a748e031fb16fc376338f0359459`.
Archive contains all tracked sources, assets, browser build and checkpoints. Build caches/dependencies are regenerable and excluded. Recover into a separate empty folder with tar -xzf; do not overwrite newer work.

Launch: `cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --menu`

Scope: hierarchical main menu, dedicated online lobby, predictable Back navigation, visible connection failures. Preserve gameplay and existing settings.

Starting verification: prior published native/browser builds and CI passed; new flow checks pending.

## Implemented

- menu.rs: dedicated Main menu with Solo game, With mates, Customise, Game settings and How to play. Child pages keep a fixed Back button with the correct parent destination. Dedicated Yard lobby contains room code, players, Ready, Start, map controls and links to customisation/help/settings. Guests retain personal settings and a race-role choice; host rule restrictions remain. Round return preserves the lobby. Errors stay visible on With mates. Compact main-menu layout and preview fit 800×600.
- online.rs: host id 1 no longer implies a successful connection. A relay acknowledgement is required before the lobby opens. Successful Host/Join opens Yard lobby; disconnects return to With mates with the status retained.
- ui.rs: Main/Lobby destinations, Back action and full-width main-menu buttons.
- public/rust/bbq.js, bbq_bg.wasm, bbq_bg.wasm.gz: rebuilt browser output.

## Verification

- 657 tests passed (227 app + 430 core); one pre-existing ignored test. Includes parent navigation, lobby settings for host and guest, scroll reset, host map replication, connection acknowledgement and disconnect regressions. See tests.log.
- Native release and production WebAssembly builds passed; existing warnings only. See native-build.log and web-build.log.
- Native 800×600 main menu and actual failed connection captured with correct assets. The failure stayed on With mates with its reason visible. The initial direct-executable captures lacked BEVY_ASSET_ROOT; they were rerun and replaced with valid captures.
- Two native clients joined the same test lobby and entered a round together (native-host.log/native-join.log and native-*.png). Both test processes exited.
- Final browser build clicked through Main → With mates → Host → Yard lobby → Start → Escape → Quit to menu → same Yard lobby → Leave → With mates → Back → Main. Customise/Back also retained the room in the preceding browser check. Final captures show the three lobby links, including personal Settings.
- Browser automation reported a pointer-lock WrongDocumentError when starting gameplay. Menu navigation, round start and return succeeded; mouse-look capture under automation was not validated. No new claim of a full remote gameplay playtest or performance benchmark.
- Gameplay rules, scoring, models and movement unchanged. No new gameplay systems or external services.

## Recovery and next step

Review review.html and playtest the desktop/browser menu. Await user approval before commit/push. Backup above predates all changes. Extract it into a separate empty directory to recover the prior version. after-sha256.json records the completed changed sources and browser output; compare hashes before any selective rollback so newer edits are not overwritten. The browser files are served locally; refresh already-open browser tabs to load the new build.
