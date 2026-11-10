# Next up (Marcus, 6 Oct 2026)

Marcus's call: online play (Phase 9) is parked after 9.6 until he has tried it with a friend. The focus now is graphics, gameplay, UI and character customization. One item at a time; screenshots and a yes before anything is committed.

## Active backlog additions · 7 November 2026

8 November — items 1–3 are in the committed history. Items 4 and 5 are implemented and published in source commit 075d0fd and browser publication e36c03d. [Full-lap Race route and visible gaze](checkpoints/2026-11-08-race-gaze/review.html). Smoko vividness pass is committed, pushed and published; [actual captures](checkpoints/2026-11-08-concept-vivid-2/review.html). Continue with the next concept-art graphics item when requested.

10 November — the local shelter approach and cooking station are ready for review. Both gameplay checks below are complete. [Captures and recordings](checkpoints/2026-11-10-shelter-approach/review.html).

9 November — solo and host bot-count controls (including team allocation), Smoko ute jump surfaces and carried-player exterior camera are implemented, tested and published. Browser review confirmed the solo/team controls and that Smoko starts; the live ute landing and carried-player camera still need a clear hands-on visual check. [Review notes](checkpoints/2026-11-09-browser-graphics-review/REVIEW.md). E19.3 now includes independent [BBQ captures and the Smoko cooking station update](checkpoints/2026-11-09-cookstation/review.html). Follow the arena-first [concept-art graphics backlog](CONCEPT_ART_GRAPHICS_BACKLOG.md).

[Five-feature coding plan and acceptance tests](DEVELOPMENT-PLAN.md)

These are queued alongside the Smoko graphics roadmap. Work one item at a time and present it for review. The picnic bend and gum shade are ready for local review. Shelter roof, sign and timber materials are next in the graphics sequence.

1. **Smoko — move the existing Bush Servo shop and pumps inside the fence.** Compare two temporary placements before moving the final assets. First try the north-west frontage, with the back wall parallel to the north fence and the pumps facing the oval. This best matches the shop-left, shelter-centre composition of the concept. Second try the north-east frontage facing south-west across the road. This gives the servo a visible corner and keeps the shelter central, but the eastern gum and perimeter props need clearance checks. These are candidate sites. Measure the entire frontage including awning, signs, pumps and parked props. Show identical playable views and a top-down footprint for both. Preserve the road, spawn access and at least two clear carrying approaches. Move shared collision and bot avoidance with the chosen site and remove its old exterior instance.
2. **Menu — preview the selected map.** Keep the existing overview presentation. On opening setup, show the current selection immediately. When the map or mode changes, show the matching environment. Modes sharing one map may reuse its view. Rapid selection changes must end on the last selection, and launching must load that same map. Check Solo, With mates hosting, and the joining player's view of the host's selection.
3. **Global inventory — enforce a two-item carry limit.** Use the same slot decision for ground pickups and successful catches. Empty-handed players equip the item. Players holding one keep it equipped and put the incoming item in reserve. At capacity, the proposed default is to leave a pickup in place or let a caught object fall at the player's feet, preserving both held items and existing catch scoring. Keep Q switching, throws and drops working. Have the host resolve simultaneous events and verify no duplication or lost items on a guest. Cover empty, one-slot and full cases through both entry points.
4. **Race — follow runners for a full lap.** Working interpretation: a sideline thrower can physically move alongside the runners around both straights and both bends. Build or connect a continuous sideline route, keep sightlines onto the track, and preserve the separation from the racing lane. Verify a human thrower can follow a racer for a complete lap on host and guest without a forced reset. Keep checkpoints, standings and throw scoring intact.
5. **Global character gaze — research with Astra Medium first.** Inspect the character rig and the view angles already available to multiplayer. Recommend the smallest approach that lets another player distinguish looking at their feet, straight ahead and overhead, plus left/right turns driven by existing controls. Include angle limits, smoothing, interaction with throw/hit poses, and whether any additional network data is needed. Use head or upper-body pose to retain the current blob design and aim. Scope is the existing camera controls, with no separate free-look control. Verify the eventual prototype from another player's view, including on the browser.

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
- **Outside camera while being carried (implemented/published 9 Nov 2026):** Camera eases to an exterior view for the carried player and returns on release. A two-player browser session verified pickup, the exterior guest camera and the return to first person on release.
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
