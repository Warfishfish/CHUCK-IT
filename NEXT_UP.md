# Next up (Marcus, 6 Oct 2026)

Marcus's call: online play (Phase 9) is parked after 9.6 until he has tried it with a friend. The focus now is graphics, gameplay, UI and character customization. One item at a time; screenshots and a yes before anything is committed.

## Movement and camera follow-up · 17 November 2026

- Implemented in source. Solid character separation on the host and local guest prediction. Carried and dragged poses have interaction exceptions.
- Expanded Backyard spawn pool and obstacle checks. Heist now has 25 distinct slots per base.
- Bush Servo has two service steps from truck cab to roof and a walkable surface matched to the visible pitched roof.
- Carried-player view uses a fixed world direction, smooth position and rotation follow, and steady field of view. The return to first person blends.
- Eye height follows head height and leg lift.
- Backyard scoring confirmed. A thrown person hitting another person awards 100. The act of releasing them alone has no throw bonus.
- Moving flicker review and publication results are recorded in checkpoints/2026-11-17-movement/REVIEW.md.

## Current priorities · 17 November 2026

### Graphics queue for the three beta maps

1. **Cloud correction (completed 17 Nov):** continuous, softly graded cloud meshes; remove detached-looking underside dots/discs; inspect from player height and overview in desktop/browser.
2. **Implemented — Heist preview.** Paint the scoreboard on creation and remove the duplicate pole.
3. **Implemented — Backyard.** Asymmetric gum and citrus crowns, brick garden edging and dimensional laundry window jambs.
4. **Implemented — Smoko.** Shelter knee braces, picnic tins, pump gauges and smoother shoulder dust.
5. **Implemented — Teddy Heist.** Sector stripe counts, wall footings and metal joins, team entrance beacons.
6. **Shared lighting.** Polished Pop now uses stronger diffuse sky fill and softer direct light. Review movement on Marcus’s usual hardware before further lighting changes.


- Three-map beta focus: Race Oval is temporarily hidden from player-facing menus; Backyard, Bush Servo/Smoko and solo Teddy Heist remain. Old saved Race selections fall back to Backyard. Race source/assets and developer CLI remain for a later return.
- Sky/nature pass: deeper blue sky, softer layered clouds, Blender-authored gum/citrus/paperbark trees, a backyard kite and servo windsock with gentle visual breeze. New roots stay outside playable footprints. The overhead wet marker is removed; body drips, HUD and wet slowdown remain. Review: checkpoints/2026-11-17-beta-nature/REVIEW.md.

The dated entries below are history; older “pending” labels do not override this current status.

- Completed: stable grass coverage, with Chrome walking/turning verification recorded in the grass-square-fix checkpoint.
- Current play-polish pass: clearer Smoko road edges and calmer small shadows; Heist route lookahead and false stuck-recovery fixes; consistent character acceleration, stopping lean and belly settling. See checkpoints/2026-11-16-play-polish/REVIEW.md for the actual checks and limits.
- Next review: Marcus's hands-on feel check in Smoko and Teddy Heist, including whether the bots' cornering and character motion now feel right.
- Eight-item follow-up implemented: Smoko clusters/ground/gums/materials, carry-camera clearance and throw framing, first-person catch/settling, controller menu navigation, and outfit presets. Pool wetness and clearer wet effects are also implemented. See checkpoints/2026-11-17-eight-pass/REVIEW.md for exact verification and remaining manual checks.
- Remaining hands-on checks: real-controller feel, sustained two-person carry/throw camera play, and Marcus’s review of the new art/wetness balance. Low/High graphics and the underlying customization system already exist; do not rebuild them from old backlog text.

## Active backlog additions · 7 November 2026

16 November — Smoko flicker, bot corner motion and environment cohesion follow-up implemented, with a brighter grading pass requested during the work. Native captures and Blender lighting studies are in checkpoints/2026-11-16-three-pass. The pass includes higher sun angles, brighter sky fill, restrained contrast, selective saturation, colour-matched grass blades and calmer distant grass. Native/browser release compilation and live file checks are recorded in the review notes. Hands-on browser/online play remains to review.

12 November — Marcus approved the circular Smoko Arena road and browser publication. The road now follows the near-circular loop in the [Bush Servo concept](concept%20art/exec-6ea4e9b3-70f7-4745-b24d-773f23cbe0ae.png) without pushing through perimeter props or the fence. [Review and captures](checkpoints/2026-11-12-round-arena/review.html). The car jump is already implemented and in the browser build; the tray landing check reached y=0.85 m in [the 10 November capture pack](checkpoints/2026-11-10-shelter-approach/review.html). Next arena graphics focus: compare two placements for the servo shop inside the fence, as detailed below.

8 November — items 1–3 are in the committed history. Items 4 and 5 are implemented and published in source commit 075d0fd and browser publication e36c03d. [Full-lap Race route and visible gaze](checkpoints/2026-11-08-race-gaze/review.html). Smoko vividness pass is committed, pushed and published; [actual captures](checkpoints/2026-11-08-concept-vivid-2/review.html). Continue with the next concept-art graphics item when requested.

10 November — the local shelter approach and cooking station are ready for review. Both gameplay checks below are complete. [Captures and recordings](checkpoints/2026-11-10-shelter-approach/review.html).

15 November — the veggie patch (#1), shed workbench (#4) and birdbath visitors (#7) are implemented in the Rust desktop and browser source. The mower is now a solid, climbable prop, and the existing veranda awning can be walked across. Rust workspace tests and browser JavaScript syntax pass. A native screenshot came back black and local browser preview is disabled in this session, so the rendered scene still needs Marcus's visual check.

15 November — backyard details #2 (rainwater overflow trench), #3 (poolside shower) and #5 (side gate shortcut) are built for Rust desktop and browser. The props are authored with Blender's native mesh tools, so no third-party library or generation key is needed. The gate has a 1.5 m opening in both the visible wall and player collision; the shower riser is solid but leaves a walkable path beside the pool. Rust tests, asset loading and browser syntax pass. Native captures: [shower](checkpoints/2026-11-15-backyard-details/pool-shower.png), [gate](checkpoints/2026-11-15-backyard-details/side-gate.png), [overflow channel](checkpoints/2026-11-15-backyard-details/rain-channel.png). Marcus's hands-on desktop and browser review remains.

15 November — the remaining backyard prop pass (#6, #8–10, #12–20) is implemented in the Rust desktop and browser: mismatched gnomes, a patched umbrella with drifting shade, wheelbarrow, compost and blue-tongue lizard, chalk drawings, pool-toy rack, breezier Hills Hoist hangers, detailed hose reel, roof solar collector, spinning pinwheel, possum box, party leftovers and animated sprinkler glints. Both house-roof slopes now support walking; the roof traversal exception is limited to the roof area. The main and neighbouring houses gain weatherboard, fascia, gutters, downpipes, dimensional windows and porch/foundation details, with added gum bark and branch detail. Props do not change gameplay collision. Browser prototype is 0.23.16. [Native review captures](checkpoints/2026-11-15-backyard-finishing/review.html); browser preview unavailable in this session, though syntax checking passed.

16 November — visual pass continuation: a Blender-authored restoration carport adds a neighbouring house story; the sprinkler now has a dense rotating water fan; the trampoline mat compresses beneath grounded players as a visual response; paired mower tracks and subtle roof-edge oxidation add surface history; Heist base signs clarify where teddies bank; teddy ears react to motion; and native pool splashes leave expanding ripples. Browser prototype is 0.23.17. Automated checks pass. Native review captures are in [the enhancement checkpoint](checkpoints/2026-11-16-visual-enhancement/REVIEW.md); browser/online hands-on review remains outstanding.

16 November — Race Oval grass is restored to the original `lawn.png` without changing the Backyard overhaul. Smoko gains a Blender-authored roadside repair cluster (worn tyres, readable TYRES sign, parts crate, esky and braced rail); its collider boxes are checked against spawns, scoring approaches, delivery lanes and road clearance. Teddy Heist now has independent close-cut turf and a red-earth shoulder, no visible or colliding Backyard props or pool behavior, and bots pursue the actual player position while defending. A 150-second automated bot match completed a steal and bank. Rust blobs now have restrained breathing and a more visible belly flop; browser blobs have a small springy running belly. Rust workspace tests (464) and browser JavaScript syntax pass; browser prototype is 0.23.19. Game screenshots remain pending because the native capture is black while the Mac is locked and browser controls are unavailable. See [review notes](checkpoints/2026-11-16-visual-identity/REVIEW.md).

16 November — bot locomotion gets smoother goal approaches in Rust and browser: bots slow down over the final metres instead of reaching a target at full steering input, which prevents circling and abrupt reversals. Heist navigation now holds its route longer while pursuing moving players and replans on waypoint arrival. The Smoko overview ground-wear layer is offset from the arena dirt to prevent flicker. Top walking speed, objective selection and scoring stay unchanged. Browser prototype is 0.23.20.

## Backyard prop ideas (Marcus's 20-item list)

- ✅ **1. Veggie patch:** tomato stakes, herbs, capsicums and “Mum’s Garden” marker.
- ✅ **2. Rainwater overflow trench:** stone-lined channel from the tank outlet into a small rain bed.
- ✅ **3. Poolside outdoor shower:** worn aqua-and-cream tiles, old copper taps, striped towel and drain.
- ✅ **4. Shed workbench:** pegboard tools, seed packets, bolt jar and mower blade mid-repair.
- ✅ **5. Side gate shortcut:** open timber gate with a bright latch through the low brick garden wall.
- ✅ **6. Garden gnome collection:** several mismatched weathered gnomes arranged as spectators.
- ✅ **7. Birdbath and visiting birds:** concrete birdbath and magpies that scatter when approached.
- ✅ **8. Patched patio umbrella:** poolside shade has a visible repair patch and drifts in the breeze.
- ✅ **9. Wheelbarrow left mid-job:** soil, trowel and weeds beside a garden bed.
- ✅ **10. Compost corner:** rough compost bin, clippings and a blue-tongue lizard.
- ✅ **11. Fence repair patches:** weathered, varied palings with bracing and repair details already in the yard pass.
- ✅ **12. Kids’ chalk art:** playful drawings and a chalk tally sit beyond the cricket marks.
- ✅ **13. Pool toy rack:** faded kickboard, noodle and flippers hang beside the pool gate.
- ✅ **14. Hills Hoist details:** empty hangers sway in a stronger breeze around the existing pegged washing.
- ✅ **15. Garden hose reel:** weathered reel and kinked coils sit beside the existing hose and sprinkler.
- ✅ **16. Solar hot-water unit:** roof-mounted collector with exposed pipework.
- ✅ **17. Bird-deterrent pinwheel:** bright homemade pinwheel spins above the veggie patch.
- ✅ **18. Possum box:** claw-marked nesting box in a gum tree with leafy mess below.
- ✅ **19. Backyard party leftovers:** plates, empty eskies and a handwritten snag note.
- ✅ **20. Sprinkler moment:** the existing sprinkler sweeps and leaves brief glistening water flecks.

9 November — solo and host bot-count controls (including team allocation), Smoko ute jump surfaces and carried-player exterior camera are implemented, tested and published. Browser review confirmed the solo/team controls and that Smoko starts; the live ute landing and carried-player camera still need a clear hands-on visual check. [Review notes](checkpoints/2026-11-09-browser-graphics-review/REVIEW.md). E19.3 now includes independent [BBQ captures and the Smoko cooking station update](checkpoints/2026-11-09-cookstation/review.html). Follow the arena-first [concept-art graphics backlog](CONCEPT_ART_GRAPHICS_BACKLOG.md).

[Five-feature coding plan and acceptance tests](DEVELOPMENT-PLAN.md)

These are queued alongside the Smoko graphics roadmap. Work one item at a time and present it for review. The picnic bend and gum shade have local review captures. The shelter roof, sign and timber-material finish was reviewed and pushed on 14 Nov. Arena lighting and colour was reviewed and pushed. The carried-player view, Heist base emblems and Dazza snag reaction are ready for review in [the gameplay checkpoint](checkpoints/2026-11-14-party-ideas/review.html). The expanded Heist map and maze are also in local review.

1. ✅ **Smoko shop placement — north-west site retained.** Marcus selected the north-west frontage in the 8 Nov review; the building, pumps and ute fit inside the fence, and the yard has one shop. The 14 Nov north-west/north-east comparison is in [the placement study](checkpoints/2026-11-14-servo-placement/review.html).
2. ✅ **Menu map preview — completed and published.** Selected-map preview follows the current mode and host settings; joins show the host's map. See [the menu review](checkpoints/2026-11-08-servo-menu/review.html).
3. ✅ **Two-item inventory — completed.** Pickups and catches share the two-slot rules; full catches drop at the player's feet without replacing held items. Focused host/guest cases are covered in [the inventory checkpoint](checkpoints/2026-11-08-secondary-slots/CHECKPOINT.md).
4. ✅ **Race sideline lap — completed and published.** The sideline path follows the oval for a full lap while preserving the track and scoring. See [the race review](checkpoints/2026-11-08-race-gaze/review.html).
5. ✅ **Visible character gaze — completed and published.** Bounded head pitch replicates through the existing camera controls, with down/level/up and turn checks in [the race and gaze review](checkpoints/2026-11-08-race-gaze/review.html).

6. **Teddy Heist map — expanded for review.** The play area is 80 × 58 m, the bases are farther apart, and a 13-panel maze creates staggered passages and loops. Base doors, teddy spawns and scoring rules are preserved. See the [native review capture](checkpoints/2026-11-14-heist-expansion/review.html).

## Graphics (full list in GRAPHICS_2C.md)
- A1.7 lawn debris and the four yard eskies (on hold)
- A2.6 tree shadows, A2.7 darker under the shed and veranda, A2.8 afternoon feel
- B5 trees (lean, dead trees, variety, haze), B6 fence (variety, warping, damage, gaps), B7 clusters (pool, shed, clothesline, bins, hose and sprinkler), B8 sky and haze
- C9 proportions (C9.5 different body shapes and C9.6 legs as a customisation done 8 Oct; next C9.7 clipping check), C10 secondary animation (thong flap, faces, lean, squash), C11 character looks (hats, skin, singlet)
- D12 BBQ, D13 Chest, D14 props and Cheeky-mode bottles, D15 animals
- E16 more particles, Low/High graphics option; E17 Blender pipeline; E18 LOD; E19 final pass
- Street and power poles round the neighbourhood (when asked)
- Dildo and noodle flop for the toys on the ground, in the air and in other people's hands

## Gameplay
- **Bot count and team allocation (implemented/published 9 Nov 2026):** Solo and host bot controls, including per-team counts, are implemented and automated checks pass. Browser controls and total updates were confirmed; settings restored to defaults. See the [browser review](checkpoints/2026-11-09-browser-graphics-review/REVIEW.md).
- **Jump onto the car (implemented/published 9 Nov 2026):** Focused movement tests cover ute surfaces and a jump onto the tray. The 10 November browser check found a misplaced floor. The corrected local build passed the real jump and standing check. [Ute and camera clips](checkpoints/2026-11-10-shelter-approach/review.html).
- ✅ **Backyard roof access (16 Nov 2026):** The rain tank is solid and reachable by a rung ladder with a walkable climb; players can jump from its lid onto a roof surface that follows the visible pitch. That pass thickened the yard grass and added a detailed ride-on mower beside the shed; the follow-up on 15 Nov adds player collision and climbable deck, bonnet and seat surfaces. Native movement checks cover the tank, roof and mower; the browser script parses successfully. [Roof-access review](checkpoints/2026-11-16-roof-access/review.html)
- **Outside camera while being carried (implemented/published 9 Nov 2026):** Camera eases to an exterior view for the carried player and returns on release. A two-player browser session verified pickup, the exterior guest camera and the return to first person on release.
- **Carry camera follow-up:** When another player picks you up, zoom the camera out and show your blob and carrier from outside. Keep the view readable while they move and throw you, and smoothly return to first-person when released. Verify both solo and with two online players. (The existing outside guest camera behavior is already implemented; this item records the requested zoomed-out presentation/follow-up.)
- (done 11 Oct 2026: Splash and Neon Lucky dildos, Cheeky-mode beer and rum bottles)
- Teddies on the trampoline: tune the bounce by feel
- Hand and first-person feel checks (flop, throw, slap) on Marcus's Mac
- Gamepad: try on a real controller, menu navigation, button remapping
- Online (parked): 9.5 rest (bar, chest, smoko, drag, emotes, help up), 9.7 leaving and rejoining, 9.8 Heist online and the lobby list, 9.9 lag tests

## UI
- Gamepad-friendly menus (tabs, tick boxes, sliders)
- Settings: volume, key bindings, Low/High graphics
- Results card and pause card polish; HUD item icons as pictures
- Controls and How to play card kept up to date

## Character customization (new)
- Colours for each blob (body, shorts or singlet, skin tone), picked in the menu
- Hats and caps, sunnies, thongs colours, name tag style
- Beer belly slider is in; add a body-size or height slider only if it keeps hit sizes fair
- Show your choices to others online (needs the online messages extended)
- Save the look with the settings

## Lobby (new, Marcus 6 Oct 2026) — first version built 8 Oct 2026 (lobby.rs, lobby card in menu.rs)
- When players join a yard they land in a lobby (a small yard or the backyard itself) and wait there
- In the lobby you customise your blob; behind the customise card you see the other players who have joined, walking about or standing, wearing their looks as they change them
- Everybody has a Ready button; when everyone is ready the round starts by itself (a short countdown that stops if someone un-readies)
- The host can also start the round at any time with a Start button, ready or not
- Everybody goes back to the lobby after the results, with Ready cleared
- Builds on 9.6 (guests follow the host) and the customization steps

## Platforms (new)
- Steam first: online through Steam (Steam lobbies, friends invites, Steam relay so no ports or links are needed)
- Mobile app (iOS and Android) later: touch controls, a Low graphics setting, and online through our own small server so phone and Steam players can play together
- Cheeky mode on mobile: check App Store and Google Play age ratings before shipping it there

## Sound and shipping (later)
- Phase 10 audio, Phase 11 settings, packaging and Steam, Phase 12 sign-off
