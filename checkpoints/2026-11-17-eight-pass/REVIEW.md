# Eight-item environment and play pass · 17 November 2026

1. **Smoko clusters:** original Blender-authored picnic plates/cups/sauce bottles, folded napkins, table braces and bolts, bin lids/rims/handles/liners, pump hose loops and nozzles. Positioned on existing tables, bins and pumps; no new ground obstacles.
2. **Ground:** road texture rebuilt against the actual 16.5–19 m ring and its world-space UVs; soft dusty shoulders and two broad tyre-wear bands replace the obsolete 18–26 m texture layout. Picnic foot wear is broader and less speckled.
3. **Trees:** irregular broad opaque gum sprays and crown forks supplement the two existing arena gums above player height. Trunks and collision stay unchanged.
4. **Materials:** beveled enamel, sun-faded timber, rubber, galvanized collars, shelter joint straps and bolts use a shared palette. The entire added kit is ten mesh/material groups, authored through Blender MCP. No third-party assets or licences were added. Rebuild scripts are in rust/tools/.
5. **Carry camera:** longer boom as a thrown victim separates from the carrier; samples solids with camera clearance to stop the view passing through walls. Existing ease back to first person remains. A visual-only local avatar now draws the selected blob/wardrobe during the outside view; the local player previously had no visible model there. It never enters the simulation roster.
6. **First-person feel:** held-item bob settles with speed, catch activation lowers the item briefly, and the held viewmodel is hidden during the exterior carried/thrown view. Aim, throw origin and catch timing are unchanged.
7. **Controller menus:** D-pad/left stick navigate visible widgets, A activates, B returns, and left/right adjust focused sliders. Focus has a blue outline and scrolls into view. Arrow keys/Enter exercise the same menu path. Physical controller handling still needs a real-device feel check; it cannot be inferred from keyboard checks.
8. **Customization:** existing colour, clothing and hat controls are retained; Bush worker, Pool party and Footy arvo outfit recipes were added. Presets retain body colour, face and legs; use existing save/online encoding. Guest cosmetic sliders are available without changing host bot controls.
9. **Pool wetness:** standing/jumping into the active Backyard pool refreshes the existing wet timer. It lasts four seconds after leaving, using 80% running speed (in addition to the existing water movement reduction). Applies to local players, bots and host/guest state. Other maps have no pool wet trigger.
10. **Wet visibility:** fourteen shaped falling drops per wet character, small white highlights, drying fade, an overhead blue droplet marker for other players, and explicit local soaked/drying text. Shared pooled meshes/materials; no collision changes.

## Verification

- Workspace: 735 passed, three pre-existing ignored tests. Includes real pool/drying movement, host-to-guest wet state expiry, outfit identity/encoding, camera wall clearance, actual Heist stealing/banking, Smoko delivery, catches and throws.
- Browser JavaScript syntax passed.
- Native release passed. Added Blender model and road texture loaded in the native overview.
- Native wet-character sequence inspected: shaped drops, highlights, overhead marker and drying status are visible (`wet-character-visible.png`).
- Chrome menu input checked via arrows/Enter: page activation, blue focus, slider increment/decrement and back navigation passed. Bot counts restored after checking. Bush worker wardrobe preset displayed correctly; the original outfit was restored.
- Two actual native clients joined the local relay and entered the same Smoko round. Both logs showed the same members, 11 bots and 33 seconds remaining. This is startup/state replication coverage, not a full human carry/throw or wetness feel test.
- Pool wet replication/expiry and carry rules are covered by the automated host/guest tests. A physical controller was not detected; real-device feel remains unverified.
- Browser and final carry render results appended below after completion.

## Scope and limitations

Character choices retain the same speed and hit sizes. Pool wetness is the deliberate new gameplay change. No scoring, round length, controls or network wire format changes were needed for the Rust game. JavaScript presence gets an optional wet duration field, compatible with older messages.

Original Blender art and procedural road bake are project-authored. Existing artist scenes were preserved; the detail kit was built in its own scene and exported using only that scene.

## Final rendered evidence

- `smoko-overview.png`: actual native arena with the current road texture and crown additions.
- `wet-character-visible.png`: actual native wet character with tapered droplets/highlights and overhead marker.
- `pool-soaked.png`: actual native pool immersion showing the soaked status.
- `carried-exterior.png` and `returned-first-person.png`: actual native carry-camera sequence. The selected local blob is now visible alongside the carrier, and disappears on returning to first person.
- `blender-picnic-assembly.png`: Blender assembly of the exported kit with the existing picnic table and servo backdrop. This is an asset-placement review, not a game screenshot.
- Some native close-up captures returned black and were discarded; they do not count as verification. Native valid captures had no asset/shader errors.
- Final native and full WebAssembly release builds passed. The live Cloudflare WASM hash matches the generated local release (browser-hash.json).

The remaining manual limits are a physical-controller feel check, prolonged two-person carry/throw camera play, and a browser pool-entry walking test. Automated pool gameplay/host-guest checks and native rendered pool/carry checks passed. The two real native clients verified room entry and replicated play; no claim is made that a human online wet/carry session was completed.

Final browser reload passed: complete Smoko environment loaded, no console errors, first Down press highlighted Solo without skipping it. Native and browser releases include the carried local avatar.
