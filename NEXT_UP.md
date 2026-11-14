# Next up (Marcus, 6 Oct 2026)

Marcus's call: online play (Phase 9) is parked after 9.6 until he has tried it with a friend. The focus now is graphics, gameplay, UI and character customization. One item at a time; screenshots and a yes before anything is committed.

## Active backlog additions · 7 November 2026

12 November — Marcus approved the circular Smoko Arena road and browser publication. The road now follows the near-circular loop in the [Bush Servo concept](concept%20art/exec-6ea4e9b3-70f7-4745-b24d-773f23cbe0ae.png) without pushing through perimeter props or the fence. [Review and captures](checkpoints/2026-11-12-round-arena/review.html). The car jump is already implemented and in the browser build; the tray landing check reached y=0.85 m in [the 10 November capture pack](checkpoints/2026-11-10-shelter-approach/review.html). Next arena graphics focus: compare two placements for the servo shop inside the fence, as detailed below.

8 November — items 1–3 are in the committed history. Items 4 and 5 are implemented and published in source commit 075d0fd and browser publication e36c03d. [Full-lap Race route and visible gaze](checkpoints/2026-11-08-race-gaze/review.html). Smoko vividness pass is committed, pushed and published; [actual captures](checkpoints/2026-11-08-concept-vivid-2/review.html). Continue with the next concept-art graphics item when requested.

10 November — the local shelter approach and cooking station are ready for review. Both gameplay checks below are complete. [Captures and recordings](checkpoints/2026-11-10-shelter-approach/review.html).

9 November — solo and host bot-count controls (including team allocation), Smoko ute jump surfaces and carried-player exterior camera are implemented, tested and published. Browser review confirmed the solo/team controls and that Smoko starts; the live ute landing and carried-player camera still need a clear hands-on visual check. [Review notes](checkpoints/2026-11-09-browser-graphics-review/REVIEW.md). E19.3 now includes independent [BBQ captures and the Smoko cooking station update](checkpoints/2026-11-09-cookstation/review.html). Follow the arena-first [concept-art graphics backlog](CONCEPT_ART_GRAPHICS_BACKLOG.md).

[Five-feature coding plan and acceptance tests](DEVELOPMENT-PLAN.md)

These are queued alongside the Smoko graphics roadmap. Work one item at a time and present it for review. The picnic bend and gum shade have local review captures. The shelter roof, sign and timber finish is implemented and ready for review in [the 14 November checkpoint](checkpoints/2026-11-14-shelter-finish/review.html); review it before commit or publication.

1. ✅ **Smoko shop placement — north-west site retained.** Marcus selected the north-west frontage in the 8 Nov review; the building, pumps and ute fit inside the fence, and the yard has one shop. The 14 Nov north-west/north-east comparison is in [the placement study](checkpoints/2026-11-14-servo-placement/review.html).
2. ✅ **Menu map preview — completed and published.** Selected-map preview follows the current mode and host settings; joins show the host's map. See [the menu review](checkpoints/2026-11-08-servo-menu/review.html).
3. ✅ **Two-item inventory — completed.** Pickups and catches share the two-slot rules; full catches drop at the player's feet without replacing held items. Focused host/guest cases are covered in [the inventory checkpoint](checkpoints/2026-11-08-secondary-slots/CHECKPOINT.md).
4. ✅ **Race sideline lap — completed and published.** The sideline path follows the oval for a full lap while preserving the track and scoring. See [the race review](checkpoints/2026-11-08-race-gaze/review.html).
5. ✅ **Visible character gaze — completed and published.** Bounded head pitch replicates through the existing camera controls, with down/level/up and turn checks in [the race and gaze review](checkpoints/2026-11-08-race-gaze/review.html).

6. ✅ **Teddy Heist maze — implemented and reviewed.** The center crate cross is now eight tall, offset wall panels with several route openings; bases, doors, teddy spawns and scoring rules are preserved. See the [native captures and review](checkpoints/2026-11-14-heist-maze/review.html).

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
