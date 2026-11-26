# Australian BBQ (was Chuck It / Backyard Brawl): gameplay checklist

- ✅ 17 November, Rust desktop and full Rust browser. Character bodies separate during movement, with host resolution and guest local blocking. Backyard has 32 predefined starting points plus a checked grid pool. Heist has 25 slots per base. The Bush Servo truck cab reaches two visible service steps and the pitched shop roof. Carried victims use a smoothed world-oriented spectator view with steady FOV. First-person height follows the chosen head height and leg length. Browser prototype 0.23.33. Review and motion recordings are in checkpoints/2026-11-17-movement/.

- ✅ Desktop and full Rust browser, 17 November. The Heist preview opens with a painted scoreboard for the selected teams. One pole supports it. Maze sectors have distinct stripe counts, structural seams and footings, and the selected base entrances have beacon posts. Backyard has revised gum and citrus crowns, salvaged brick garden edging and deeper laundry window trim. Smoko adds shelter braces, picnic tins, pump dials and a wider dust transition. The polished Pop preset has more sky fill and gentler direct sun. Browser prototype 0.23.32.

- ✅ (17 Nov 2026, desktop and browser) Clouds replaced with three original Blender-remeshed continuous silhouettes and a broad baked underside colour grade; each mesh is checked for a single connected island. Shared desktop/full-browser assets and legacy JavaScript prototype 0.23.31. No cloud shadows or gameplay changes. Verification notes: checkpoints/2026-11-17-clouds/REVIEW.md.

- ✅ (17 Nov 2026, desktop and full Rust browser) Three-map beta selection: Backyard, Bush Servo and solo Teddy Heist; Race Oval temporarily parked, including old saved selections. Overhead wet droplet removed, with body drips and wet slowdown retained. Sky/nature pass adds layered clouds, a cooler blue gradient, original Blender gum/citrus/paperbark trees, kite and windsock with gentle breeze. No new route collisions. Browser prototype 0.23.30. See checkpoints/2026-11-17-beta-nature/REVIEW.md for verification and limits.

✅ done · ⬜ to do · 🔒 checkpoint

🔒 **Checkpoint 0.11.1:** `checkpoints/chuck-it-0.11.1-checkpoint.html` (GitHub + `CHUCK IT/checkpoints` on the Mac)

---

## Already in the game ✅

- Prototype 0.23.68 ships the approved Backyard foley. Glass has a distinct shatter, balloons snap and splat, fish and steak have their own slaps, and hard landings use a recorded thud. The legacy browser uses the updated fish, steak and landing cues. Rust online protocol 35 identifies the new effects.

- Prototype 0.23.67 uses InspectorJ’s recorded Pool Jump H for pool entry on Mac and both browser builds. The clip starts at water contact and keeps the short wash that follows. The credits include the author, source and licence.

- Prototype 0.23.66 adds Settings to the Rust main menu and separates audio, display and controller preferences from Match options. Solo puts map selection and Play together. With mates puts hosting and the room code above the server address. Lobby hosts open Match options and return to the same room.

- Prototype 0.23.65 keeps the desktop and WebGPU main menu clear of oversized held items. The menu's background demo leaves items on the ground for its bots.

- Prototype 0.23.64 explains what wins each beta map in How to play and covers Hands, direct item selection and controller defaults on desktop and WebGPU. The Heist kickoff names your team and explains banking. The legacy guide covers its Backyard and Heist rules and Hands.

- Prototype 0.23.63 gives Dazza’s tunnel warm work lights, a hatch ladder, overhead service pipes, coloured floor arrows and labelled team exits. Water drips from a pipe joint in each branch and makes a small ripple. The detail is present in the desktop, WebGPU and legacy browser builds.

- Prototype 0.23.62 tightens the attack of the selected throw whoosh and the local recorded pool and balloon splashes. Pool noodle contact keeps its slap and gains a low thump. The opening item wave plays its impact sound on the first landing.
- The raw steak uses the marbled Blender ribeye with five linked joints that bend in the hand and in the air. Backyard crate trim meets cleanly at the corners. In Bush Servo, two low steps lead onto the ute cab and two more lead from the cab to the shop roof.
- A charged, upward Teddy Heist throw can lob a teddy across an interior wall. Short, level passes remain catchable and a lob only scores if someone retrieves and banks the teddy.

- Prototype 0.23.61 uses the selected Fuller Whoosh for throws and Cartoon Glugs for the local player’s bar drinks. The recorded BBQ sizzle in the Mac-only audio preview is louder and audible farther from the grill.

- Prototype 0.23.60 uses the Natural Palm recording on bare-hand contact and the Hard Slap recording on held-item contact in desktop, WebGPU and legacy browser. Both clips are credited CC0 recordings.

- Prototype 0.23.59 triggers the held-item whoosh from the local release on desktop and WebGPU, including for online guests. The sample is trimmed to put the audible start closer to release.

- Prototype 0.23.58 adds audible item throws, impacts, slaps, balloon pops, pool splashes and can rattles to the public desktop and WebGPU builds. Settings and Pause include a Test sound button. Grass footsteps remain paused in the public build.

- Prototype 0.23.57 adds a favourite character slot to Customise on desktop and WebGPU. It saves the shape, proportions, outfit and colours separately from the last-used look and can be loaded on the next launch.

- Prototype 0.23.56 adds a saved Master Volume slider to settings and pause on desktop, WebGPU and legacy browser. A Mac-only audio preview pack adds balloon impacts, pool entry, sprinkler sweeps, grill sizzle, magpie wingbeats and calls, can rattles, and Dazza’s hatch. The public build uses the existing credited sound pack.

- Prototype 0.23.55 turns off Teddy Heist footsteps for now on desktop and both browser builds. The grass recordings are retained for future sound work.

- Prototype 0.23.54 gives Teddy Heist softer grass footsteps. Three real turf steps were isolated and cleaned using the quiet gaps in the source recording, with the sharp upper frequencies reduced. Desktop and both browser builds load the same clips.

- Prototype 0.23.53 lets players press Q with one item to stow it and equip Hands for a bare-handed slap. Press Q again or 1 to re-equip the item. Applies in the active game modes in solo and online.

- Prototype 0.23.52 replaces the field-recorded Teddy Heist footsteps with three short, isolated grass effects. The steps are trimmed, filtered and faded so outdoor background does not repeat with each footfall. Desktop, WebGPU and legacy share the same credited clips.

- Prototype 0.23.51 trials quieter recorded sneaker footsteps in Teddy Heist and adds local wingbeats and occasional Australian magpie calls to Backyard’s birdbath visitors. Desktop, full WebGPU and legacy use the credited CC0 clips.

- Prototype 0.23.50 adds saved controller button remapping, pause and results controller navigation, and illustrated held-item slots to desktop and full WebGPU. The desktop build also produces self-contained Mac and Windows beta archives with runtime assets and credits. [Beta review](checkpoints/2026-11-21-beta-readiness/REVIEW.md).

- Prototype 0.23.49 calms Bush Servo’s gum foliage and leaf-shadow shimmer. Wider, fewer leaf shapes preserve the crowns, with broad canopy shade around the picnic area and shelter. [Desktop and browser review](checkpoints/2026-11-21-bush-servo-foliage/REVIEW.md).

- Prototype 0.23.48 lifts shaded Heist turf, animates Backyard sprinkler drops and staggered pool ripples, and refines the balloon splash, wet drips, landing stance and carried-player camera. [Review and captures](checkpoints/2026-11-21-turf-water-and-camera/REVIEW.md).

- Prototype 0.23.47 fades fine Backyard grass blades smoothly with distance. The close grass remains visible and the lawn keeps its painted colour and wear. Tested walking and turning on desktop and WebGPU at Low and High. [Matched captures](checkpoints/2026-11-21-backyard-grass/REVIEW.md).

- Prototype 0.23.46 clears stale Heist bot targeting and dodges after hits and adds three recorded first-person grass footsteps. [Recovery and audio review](checkpoints/2026-11-20-heist-recovery/REVIEW.md).

- Prototype 0.23.45 keeps Heist teddy passes short and lets teammates catch them. Vaults now skip shorter detours, and a stolen teddy slows its carrier underground. Dazza opens the hatch for eight seconds. [Route measurements and test notes](checkpoints/2026-11-20-heist-balance/REVIEW.md).

- Prototype 0.23.44 teaches Teddy Heist bots to clear corners, ease into turns and negotiate crowded halls. Bot bodies and strides follow actual travel on desktop and browser. Seven original wall posters mark different junctions on the three maps. [Movement clips and wall-art review](checkpoints/2026-11-20-heist-movement/REVIEW.md).

- Prototype 0.23.43 adds four recorded Teddy Heist cues to desktop and browser: player chuck, landing, Dazza’s hatch opening and teddy banking. All clips are CC0 and credited in the shipped asset credits.

- Prototype 0.23.42 gives Teddy Heist a host-controlled player chuck, opening pickups in all four outer halls, and marked supply bays. Bots retain their route through corners, give up blocked steering directions and spread raids across comparable objectives.

- Prototype 0.23.41 opens Teddy Heist in the Rust With mates lobby. The host owns banking, hatch state and drink handovers. Players join before the round starts. Team start slots now spread out into clear approach lanes. Lighter turf, contrasting walls and painted doorway marks improve readability.

- Prototype 0.23.40 adds overhead podium names, makes Heist guards hold a doorway post, and sends the nearest teammate with a sightline after a stolen teddy. Armed bots break off after a throw, swing, blocked sightline or short pursuit.

- Teddy Heist base flags fly on 14 m poles. A fold in the team-coloured cloth keeps it visible from side passages on all three maps. Desktop and browser prototype 0.23.38.

- 18 November, prototype 0.23.37. Two, three and four flags select three square Teddy Heist arenas from Marcus’s SVGs. The footprint is 116.8 m on each side. Bots sample solid obstacles to build a shared navigation grid and find routes to objectives. All modes support up to 20 participants including bots. See the [map comparisons and verification](checkpoints/2026-11-18-reference-mazes/REVIEW.md).

- Teddy Heist goal passages (18 November, prototype 0.23.36). Continuous walls join each doorway to a three-turn passage. Across the nine layouts, each opposing pair has a connected ground route at least 18 m longer than the direct line on the 1 m audit grid. Two-team centre-to-centre walks measure 104, 118 and 141 m. The solo bot match completed a steal and bank. [Desktop overview](checkpoints/2026-11-18-heist-goal-passages/overview.png).

- Teddy Heist switchbacks (18 November). The nine layouts have 49–53 thin maze panels and screened approaches outside each base. Tall goal beacons are gone. Banking requires entering the base. The route audit checks every walkable sample for sealed pockets. Desktop and browser prototype 0.23.35.

- Teddy Heist maze pass (18 November): 24–27 panels depending on the base layout, with offset turns and barriers on the outer lanes. All nine layouts pass the base-to-base clearance check, and the solo bot match still completes a steal and bank. Desktop and browser prototype 0.23.34.

- 17 Nov eight-item pass: Blender-authored Smoko picnic/bin/pump/shelter detail kit, broader gum foliage, aligned circular-road dust/wear, wall-aware carry framing with a visible local avatar, catch/viewmodel settling, controller menu focus/sliders, and three saved/replicated wardrobe presets. Pool immersion refreshes four seconds of wet slowdown; shaped drops, highlights and overhead wet markers make wet characters readable. Verification: checkpoints/2026-11-17-eight-pass/REVIEW.md. Browser prototype 0.23.29.


- 17 Nov 2026 play polish: Heist bots look ahead to the furthest visible route point instead of dropping to one-cell maze steps. Intentional pauses/windups no longer trigger random stuck recovery. Character acceleration compares velocities in a consistent facing frame, belly acceleration response is time-scaled, and browser blobs have a clearer stopping lean and settling breath. Smoko road edges have subdued paint; tiny shoulder stones/dashes no longer cast unstable small shadows. Browser prototype 0.23.28. Checks and captures: checkpoints/2026-11-16-play-polish/REVIEW.md.


- Grass square-patch fix: one consistent 180-tuft density per 5 m tile replaces abrupt near/far mesh swaps and camera-distance hiding. The renderer culls off-screen tiles. Applies to desktop and the full Rust browser build. Chrome movement review passed on 16 Nov: forward/backward walking, strafing and mouse turns showed no returning dark tile patches in the observed views; no browser console errors were captured.

- Smoko picnic overlays have separated render heights and the roadside cluster receives distance filtering. Shared sRGB grass, house, roof and bark textures now receive linear-light mipmaps. Building/bark detail is moderated to suit the sage ground. Bots face along goal routes and use a checked direction when a smoothed corner turn points into a wall. Brighter sky fill, shorter building shadows and a controlled colour grade add vividness. Blender lighting studies are saved in checkpoints/2026-11-16-three-pass. Browser prototype 0.23.26. Gameplay motion review is pending.

- Blender grass contrast revision uses cooler sage ground with sparse painted marks, neutral lawn material tint, darker matching blades, and camera-height-aware grass culling. Native and full-browser visual review is pending.

- Painted comic-style Backyard grass uses muted olive patches and sparse angular blade strokes on desktop, the full Rust browser game, and the JavaScript prototype (0.23.23). The texture is loaded from the existing lawn asset path. Live gameplay visual review remains pending.

- Backyard lawn texture has softer blade detail and a more muted green across desktop and browser.

- The browser share opens the full Rust game at `/rust/`. The JavaScript prototype is available at `/index.html`.

- Browser 0.23.21 serves Three.js and its matching GLTFLoader from the game server. Verified the shared link opens the menu and starts Solo in Chrome.

**Moving**
- ✅ (Rust desktop and browser, 22 Oct 2026) Host lobby selectors for Backyard, Smoko yard and Race oval, with Free for all / Teams on Backyard and a direct Game settings button. Game settings includes maps, modes, round and match options, Cheeky mode and yard features. Solo retains Teddy Heist; its existing online restriction is clearly labelled. Race's fixed rules are shown instead of offering ignored options.
- ✅ (Rust desktop and browser, 22 Oct 2026) Permanent menu navigation includes Customise, stays above the scrolling content, and resets content scrolling when changing tabs. Hosts and guests can reopen every menu after switching away.

- ✅ WASD, Space jump, Shift sprint boost, Q swap item, right-click catch
- ✅ (Rust desktop and browser, 12 Nov 2026) Bush Servo Smoko Arena road is a circular loop around the shelter, with the lane markings and shoulders aligned to the road mesh; existing prop clearance is preserved
- ✅ (Rust desktop, 14 Nov 2026; ready for visual review) Shelter eaves have a narrow galvanized edge, the upper sign has restrained uneven paint wear, and the wide sign lettering remains clear. [Matched captures](checkpoints/2026-11-14-shelter-finish/review.html)
- ✅ (Rust desktop, 14 Nov 2026; ready for visual review) Bush Servo lighting keeps a warm afternoon sun while cooling sky fill and easing colour saturation for clearer shade and prop colours. [Matched captures](checkpoints/2026-11-14-arena-lighting/review.html)
- ✅ (Rust desktop and browser, 15 Nov 2026) Backyard house pass adds a ridge cap, gutters, downpipes, window sills, concrete footing, meter box and vent. Character eyes gain small catchlights across Rust looks and browser; bots show a clearer acceleration lean, corner bank and counter-swing without changing navigation or movement values. [Review images](checkpoints/2026-11-15-visual-richness/review.html)
- 🟡 (Rust desktop and browser, 15 Nov 2026; ready for Marcus's review) Yard realism pass adds roof whirlybirds and rain wear, braced paling rails, flowering gums, pegged striped washing, pool coping grout and skimmer, worn pavers, and BBQ tongs, grate and gas-bottle fittings. These details leave route and collision sizes unchanged. [Review images](checkpoints/2026-11-15-environment-detail/review.html)
- ✅ (Rust desktop and browser source, 15 Nov 2026; movement tests and browser syntax pass, rendered visual check pending) The raised veggie bed now has tomato vines, basil, capsicums and a hand-painted “Mum’s Garden” sign; a shed-side workbench has seed packets, a jar of bolts and a half-finished mower blade; a concrete birdbath has two magpies that scatter when approached. The ride-on mower blocks players and can be climbed across its deck, bonnet and seat. The existing sagging veranda awning is now walkable from above and stops jumps from below. No movement tuning, scoring or map size changed.
- 🟡 (Rust desktop and browser source, 15 Nov 2026; 461 Rust tests, native asset-load captures and browser syntax pass; hands-on play review pending) Blender-authored rainwater overflow channel, tiled poolside shower and an open side-gate shortcut add three lived-in landmarks. The gate collision is split to match its 1.5 m visible opening, and the shower riser is solid while its tile pad stays walkable. Other yard and scoring rules are unchanged. [Shower](checkpoints/2026-11-15-backyard-details/pool-shower.png), [overflow channel](checkpoints/2026-11-15-backyard-details/rain-channel.png), [gate](checkpoints/2026-11-15-backyard-details/side-gate.png)
- ✅ (Rust desktop and browser, 16 Nov 2026) The rain tank is solid and reachable by a rung ladder with a walkable climb; players can jump from its lid onto a roof surface that follows the visible pitch. The yard grass has denser, taller blades, and a detailed ride-on mower is parked beside the shed. The yard physics tests cover climbing, tank collision and a stable roof landing; browser script syntax passes. [Review images](checkpoints/2026-11-16-roof-access/review.html)
- ✅ (Rust desktop and browser source, 15 Nov 2026) Backyard finishing adds a walkable rear house-roof slope, thirteen lived-in details (gnome spectators, patched umbrella and drifting shade, wheelbarrow, compost and blue-tongue lizard, chalk art, pool-toy rack, swaying Hills Hoist hangers, hose reel, solar hot-water collector, pinwheel, possum box, party leftovers and sprinkler spray), and more convincing weatherboard, roof, window and porch details on the house and neighbouring buildings. Gum trees gain bark, branch and nesting detail. Props remain decorative; roof access is the only collision change. [Native review captures](checkpoints/2026-11-15-backyard-finishing/review.html)
- ✅ (Rust desktop and browser, 15 Nov 2026) A fence-mounted, hand-painted backyard-cricket scoreboard and chalk wicket creases add a signature Aussie-yard story beside the stumps and bat. Decoration only; no collision or scoring changes. [Review capture](checkpoints/2026-11-15-cricket-vignette.png)
- 🟡 (Rust desktop and browser source, 16 Nov 2026; 728 tests and browser syntax pass) The current visual enhancement pass adds a Blender-authored neighbouring carport/restoration vignette, mower wheel tracks, restrained roof-edge rain wear, a full moving sprinkler fan, a compressing trampoline mat, readable Heist bank signs, moving teddy ears, and pool-entry ripples in the native build. Browser prototype is 0.23.17. Native review captures are saved; hands-on browser and online play review remain.
- 🟡 (Rust desktop and browser, 16 Nov 2026; 731 Rust tests, browser JavaScript syntax, release asset loading, and local texture/model HTTP checks pass; visual review incomplete because the native screenshot returned black and browser UI access is unavailable) The visual overhaul adds shared high-resolution lawn, weatherboard, corrugated roof, gum-bark and garden-mulch textures with correct color handling and distant filtering. A Blender-authored ribbon gum replaces the simplified boundary tree in both builds without changing collision. Browser prototype is 0.23.18. Blender’s external asset libraries and generators are off, so the model was made and exported directly in Blender.
- 🟡 (Rust desktop and equivalent browser Heist/character content, 16 Nov 2026; 464 workspace tests and browser JavaScript syntax pass; in-game visual review pending because the locked Mac screenshot is black and browser UI is unavailable) Race Oval now uses its original `lawn.png` while Backyard keeps `lawn_overhaul.png`. Smoko gains a Blender-authored roadside tyre/sign/crate/esky/rail cluster with matching solid bounds and clear spawn, delivery and road lanes. Teddy Heist has its own close-cut turf and dry oval shoulder, with Backyard scenery, colliders and pool behavior removed; the defender bot now targets the player's real position. A 150-second Heist bot match completed a steal and bank. Rust and browser blobs gain a restrained belly wobble and settle. Browser prototype is 0.23.19. [Review notes and Blender prop preview status](checkpoints/2026-11-16-visual-identity/REVIEW.md).
- ✅ (Rust desktop and browser, 16 Nov 2026) Bot locomotion now eases down as bots approach a goal and replans Heist paths more steadily when chasing a moving target. The Smoko overview ground-wear mesh is lifted above its base dirt plane to stop depth shimmer. Top speed, objective selection, scoring rules and fairness are unchanged. Browser prototype is 0.23.20.
- ✅ (Rust desktop, 14 Nov 2026; ready for visual review) Heist bases have distinct colour-and-shape emblems, Dazza raises his spatula to protect against a fast flying snag, and the carried-player camera frames both players through the throw. [Gameplay review](checkpoints/2026-11-14-party-ideas/review.html)
- ✅ 1.5x bigger yard with cover, a big deep pool (14 x 8), trampoline

**Chucking and slapping**
- ✅ Teddy, VP can, gnome, and floating pool noodles
- ✅ Power throw: let go near the end of the wind-up bar (90% or more, the bar turns red past a little white mark) and a hit knocks the person over for 2 seconds. Players only, bots don't power-throw. Doesn't work on someone in their 1 second stun grace
- ✅ Dildo chest (Cheeky mode only, random spot): 4 variants (Mini/Classic/Jumbo/rare Golden Wonder), each worth different points and a different knockdown length. Slaps knock people flat with Sent Flying (flung backwards away from the slapper, lands on their back and skids, replaced the old spinning Helicopter) / Cartwheel / Timber
- ✅ (Rust version only, 6 Oct 2026) Six dildo types, each with its own odds: Pocket Rocket crits 22% and never sends anyone flying; The Unit crits 8% but half its hits send people flying; Golden Wonder crits 25%. Two new types: The Quickie (small, teal, swings in 60% of the time so you can slap faster) and Long John (long, orange, reaches 40% further but swings slower). Chest weights are now 46/28/18/4/12/10. The browser game still has the original four
- ✅ (Rust version only, 11 Oct 2026) Two more dildo types in the chest: The Splash (blue, full of water: a slap also soaks everyone within 2.2 m of the one hit, pushing them away and stopping them for 0.7 s, +15 points each) and Neon Lucky (glowing green, rare, crits 45% of the time). The old six keep their weights (now 46/28/18/4/12/10/9/3)
- ✅ (Rust version only, 11 Oct 2026) Family mode (Cheeky off) has no alcohol: the throwable is a can of lemonade (plays like the old VP can, bursts into fizz), the bar is a COLD DRINKS stand (lemonade, orange juice, water) that never makes anyone drunk, there is no drunk meter, no stacking it, no drinking at smoko and no Drunk mode, and the empty stubbies on the lawn are gone. Cheeky mode keeps the bar, beer and rum bottles and all the drinking
- ✅ (Rust version only, 11 Oct 2026) A Settings tab on the menu: round length, rounds, bot skill, friendly fire, what's in the yard, field of view and sound in one place (online, guests see that the host picks the round). The pause card has field of view and sound too
- ✅ (Rust version only, 6 Oct 2026) Six dildo skins, picked at random for each dildo (the same one always has the same skin): studded, smooth, fat ribs, fine grooves up and down, a spiral ridge, and veiny. Size, colour and flop are unchanged
- ✅ (Rust version only, 6 Oct 2026) Teddies bounce on the trampoline like a ball: the first bounce is the trampoline's usual kick, then each is lower and stays on the mat until the teddy settles and sits there. Everything else keeps the original kick
- ✅ (Rust version only, 8 Oct 2026) The four body shapes look clearly different: Classic the straight capsule, Pear with a big round bottom, Egg tall and slim, Gumdrop short and wide. Looks only, hit sizes are the same for everyone. The crown, stun stars, name tags, team sash, team bubble and Heist ring fit each shape
- ✅ (Rust version only, 8 Oct 2026) Legs on the Customise tab: None (the default), Stumpy or Skinny. The legs follow the feet as they step and show online too. Looks only, hit sizes are the same for everyone. Half the bots have legs
- ✅ (Rust version only, 9 Oct 2026) Shorts on the Customise tab: None (the default), Budgies (budgie smugglers, which replaced the Stubbies on 10 Oct) or baggy Boardies, in any of 5 colours, and the singlet and shorts are now real woven cloth. They follow the beer belly and show online. Half the bots wear shorts
- ✅ (Rust version only, 9 Oct 2026) Hats on the Customise tab: None (the default), Cap, Akubra or Beanie, in any of 5 colours (a hat covers the hair). Everyone also gets a soft sheen on the skin, a sunburnt nose and a reddened neck. Half the bots wear a hat
- ✅ (Rust version only, 10 Oct 2026) Everyone's name is printed across the back of their singlet in slightly wonky letters (dark ink on light singlets, white on dark). Singlets and shorts are a little dirty, with a Dirty slider on the Customise tab (Spotless to Feral) that shows online too; bots are randomly grubby
- ✅ (Rust version only, 8 Oct 2026) Swimming: in the pool you glide and bob at the top (slower to start and stop, so you drift), and holding Shift dives. Underwater you swim where you look (look down to go deeper, up to come up), the view goes bluer and darker, and thrown things pass over your head. You have about 9 seconds of breath (a meter shows under the drunk bar); when it runs out you float up, and it comes back at the surface. Other people online see you dive and lie along the water. Shift boosts on land as before
- ✅ (Rust version only, 8 Oct 2026) Easier to drag people to the Naughty Corner: reach 2.6 m (was 1.9), you can drag for 10 s (was 6), you walk at 80% while dragging (was 55%), anyone stunned can be grabbed (not just someone lying down), bots wriggle free slower, and dragging someone onto the smoko pad sends them to a chair by itself. The hints say where to take them. (Needs the Naughty Corner tick box on in Solo.)
- ✅ Dildo critical hit: ~15% chance on any dildo slap to flatten them for a full 5s with a much bigger flying knockback — "💥 CRITICAL!", no stun stacking on top
- ✅ Steak, fish, noodle slaps stun (steak and fish stun for 2s). Stuns never stack: hitting someone who's already stunned shoves them but doesn't start or extend a stun (a dildo can still floor someone who's stunned but standing)
- ✅ Fish slaps sometimes leave a green stink cloud round the victim (about 1 in 3), and occasionally the fish is properly off — bigger, darker, longer-lasting cloud and a "THAT FISH WAS OFF!" pop-up. Pure joke, no gameplay effect
- ✅ Anyone who's fallen over (drunk face-plant) holds up a wobbling HELP ME! sign so mates can spot them across the yard
- ✅ Wet-willy, noogie, wedgie (Cheeky mode): a close-range unarmed slap, no item needed — shove-knockback, sillier label, no knockdown
- ✅ Slap the chef: KO / Berserk / Flip the grill (gnome knockdown-stun eased back slightly so it's not as brutal)
- ✅ Press **Q** to swap your held item
- ✅ While you're knocked flat or have stacked it you can still throw or slap if you're holding something (bare-handed slaps and catches don't work lying down). The lying-down animation lasts exactly as long as you're actually down
- ✅ You can't stand on top of the clothesline pole any more (bouncing off the tramp onto it): you slide off

**Drinking**
- ✅ Beer, wine, rum at the bar; drunk meter, wobbly vision, drifting
- ✅ Everyone sees you sway; random drunk falls; mates help you up

**Yard life**
- ✅ Dazza at the BBQ yells, chases and spatulas you, and gets angrier (faster, longer chases) the more you steal from him
- ✅ (Rust desktop and browser source, 15 Nov 2026) Mum’s veggie patch, the mower-repair workbench and birdbath visitors add distinct backyard stories; the magpies startle, fly to the back fence and return after the player leaves.
- ✅ Steak/fish can stun Dazza for a quick, reliable escape — the stun gets shorter and the chase after it gets longer each time. Getting angrier each time you pinch his meat: chases get longer and he moves faster. Steak or fish now reliably stuns him for a quick escape (shrinks each time), then he chases you after
- ✅ Smoko chairs: sit, beer, safe — but sitting there ramps your drunk meter noticeably faster than standing around drinking normally, so smoko isn't a totally free pass
- ✅ Emotes and taunts, plus a cheeky extra set unlocked in Cheeky mode
- ✅ Bum-Out Gnome (Cheeky mode): looks like a totally normal gnome at a glance — no glow, no effect — but has a tiny buttcrack detail round the back if you look closely. Max 3 a game, worth +15 on a hit, small chance to pop out when you KO Dazza with the dildo. Not announced outright; sharp-eyed mates will just see a "dropped something shiny…" line in the feed
- ✅ Cheeky Dazza lines in Cheeky mode (bigger, ruder bubble set); otherwise the clean lines
- ✅ Ruder bot/mate names in Cheeky mode ("Skidmark", "Mate 77" style) — rides on the same Cheeky mode tick rather than its own toggle, to keep the setup screen simple

**Drunk mode and stuns**
- ✅ **🍻 Drunk mode** tick box (Solo and hosted options): everyone, bots included, is kept properly drunk all game ("Maggot" level), so no sobering up and no drunk bonus points
- ✅ In Drunk mode you have a 4% chance of stacking it every 2 seconds you're walking (only real players fall over, not bots)
- ✅ Stacking it keeps you on the ground for 10 seconds by default (host can still pick 20 or 30)
- ✅ Drunk mode walking speed wanders like real drunk walking: an uneven rhythm, sudden lurches forward (legs hurrying to catch up with the body, up to about 1.4x speed) and slow-downs to get balance back (about half speed). Slower than sober on average. Bots do it too
- ✅ After a stun ends you can't be stunned again for 1 second (you can still be shoved, and a dildo can still floor you)

**Bots**
- ✅ Bots move more like people: they turn smoothly instead of snapping, ease in and out of speed, slow down as they arrive, walk slightly wobbly lines, go round obstacles on one side instead of twitching, and give other people a bit of room
- ✅ Each bot has its own habits (reaction time, wobble, how often it stops for a look around), kept all game. They glance at things thrown their way, pause for a beat after being hit or changing their mind, and drunk bots sway like drunk players
- ✅ Dodges and hops are still instant, so bots are no easier to hit

**Setup screen**
- ✅ Tabs: Solo / With mates / How to play, with one big Play button
- ✅ With mates has its own ⚙️ Game options (Cheeky mode, bots, bot skill, round length, rounds, friendly fire for Teams, what's in the yard including Naughty Corner and Drunk mode, stacked-it time). The yard opens with them, and the host can still change them in the yard before each round
- ✅ 😜 **Cheeky mode** tick (off by default), in its own spot: on the main Solo screen under Game mode, at the top of With mates ⚙️ Game options, and at the top of the yard's host settings. Replaces the old Who's playing / PG15+ section. The How to play item list only shows the Purple Dildo when Cheeky mode is ticked Labelled "Rude jokes, cheeky names". Unlocks the dildo chest, rude jokes and cheeky names and any future adult content, synced to everyone in the yard

**Scoring and matches**
- ✅ Hits, streaks, leader bounty, drunk bonus, pool/tramp bonus
- ✅ Teams (Red / Blue / Wildcard), friendly fire option
- ✅ Best-of-3 matches; host picks what's in the yard
- ✅ Online, when the game (single round or the whole match) is over, the results show for a few seconds and then everyone goes back to the lobby together, ready for the next one

**Teddy Heist (its own mode)**
- ✅ Pick **🧸 Teddy Heist** as the Game mode in either the **Solo** or **With mates** section (no separate tab). Choose 2-4 teams there, then Play or Host. Teams spawn inside their own base
- ✅ Bases are loud: coloured floor pad marking the banking zone, a tall flag pole and a light beam you can spot across the yard, taller walls
- ✅ (14 Nov 2026) Teddy Heist now has its own 108 × 80 m rounded oval arena; the shared 80 × 58 m yard stays the same for other modes. Its visible stone perimeter, player/bot boundary, and thrown-item edge follow the same curve, with no square fence around Heist
- ✅ (15 Nov 2026) Heist hides the shared rectangular lawn while its raised oval playfield is active, so overlapping ground layers no longer shimmer or flicker across the arena
- ✅ 2 to 4 teams, using Red/Blue + Green/Yellow for 3-4 teams. Each team gets a base in its own corner/side of the yard, ringed by a fence/crate wall with a gap or two to get in
- ✅ Each base holds a bundle of that team's teddies — 2 each in a small lobby, rising to a max of 4 per team as more players share each team (formula: 2 + one extra per 2 players per team, capped at 4)
- ✅ Run an enemy teddy back to your own base to bank it for points; stealing it back before it's banked just returns it. Getting slapped or knocked down while carrying drops the teddy on the spot — no auto-return, so there's a scramble
- ✅ Walking up to your own team's teddy auto-returns it home instead of letting you "steal" your own
- Teddy Heist awards 10 personal points per qualifying opponent throw and 150 per bank. Team victory is decided by banks.
- The final whistle reveals the team result. At match end the roster celebrates around a three-place podium ranked by points, then hits.
- Each reference maze has two crate vaults and underground branches to all active base gates. Dazza patrols the middle corridors and escorts a cold-drink donor to the hatch.
- Dazza accepts quests after 30 seconds. The hatch opens for 12 seconds once per team, followed by a 45-second rest. Extra drinks change his wobble and written chatter.
- Heist bots account for nearby teammates when planning routes and need a sightline to pursue an enemy carrier.
- ✅ Big four-sided scoreboard floating over the middle of the yard shows each team's teddies banked and points, with a crown on the leader — readable from anywhere. The sidebar team headings show teddies too
- ✅ Each player's own banked-teddy count is tracked: shown as 🧸 next to their score on the live scoreboard, as a 🧸 Banked column in the results, and the results line names the top banker in each team
- ✅ Win: whoever's banked the most teddies when the clock runs out
- ✅ (14 Nov 2026) Three hand-authored Heist base layouts for each team count (2, 3 and 4), selected once per round without immediate repeats and synchronized from the host to guests and late joiners. Bases, doors, spawns, teddies, banking zones, emblems and bot goals share that layout
- ✅ (14 Nov 2026) A connected 14-panel laser-tag maze creates short contested routes and longer flanks that reconnect; the full pool rim keeps player-width clearance and acts as a landmark. Measured 2-team doorway routes are documented in the Heist map review
- ✅ Live tally bar during the round shows each team's banked count by colour
- ✅ Bots play the objective: they raid the nearest enemy teddy (going round to the doorway first), run it home, chase anyone carrying one of theirs, pick up loose teddies of their own, and the first bot on each team of 2+ guards the base by walking between four spots inside it. Bots caught inside a base leave through its doorway instead of pushing at the walls
- ⚠️ A thief can still throw a stolen teddy to a mate. Left in on purpose, so watch it in playtest (it might be fun, it might be exploitable)


**Grab and drag**
- ✅ Press **F** next to someone who's actually down (knocked flat or stacked it) to grab them by the ankles and drag them along the grass on their back behind you. Tap **F** again to let go, **hold F** to throw them (about 7 metres). With nobody to grab, F still winds up a throw like before. Anyone can be grabbed, teammates included
- ✅ **Human cannonball:** throw a dragged player into someone and the person they hit is knocked flat (+100 if it's the other team, nothing for your own team unless friendly fire is on)
- ✅ Dragging slows you down, and you can't sprint, throw or slap. The dragged player drops anything they were holding (including a stolen teddy) and lies looking up at the sky
- ✅ The dragged player **mashes Space** to wriggle free (about 8 presses). Nobody can be dragged for more than 6 seconds, and a freed player can't be grabbed again for 8 seconds
- ✅ You let go if you get hit, knocked down, fall over or walk into the pool. Throwing someone into the pool or onto the tramp still scores the splash/orbit bonus
- ✅ Bots can be dragged (they wriggle free on their own). Bots don't grab anyone yet
- ✅ Works online for host and guests; if someone leaves mid-drag it just ends

**Naughty Corner**
- ✅ Tick **😈 Smoko is the Naughty Corner** in the yard options (solo and hosted). The smoko pad turns red
- ✅ Drop or throw a carried player into the smoko area: **+100** (not for dumping a teammate) and a "SENT TO THE NAUGHTY CORNER!" banner
- ✅ The punishment is just that: the victim sits on a smoko chair for **5 seconds** and can't move. Nothing after it
- ✅ Someone on the chair can't be picked up. Nobody can sit at smoko for a safe break in this mode, and bots stop going there

---

## 3. Fair scoring ⬜
> Stop people farming the same mate for points.

- ✅ No points for dumping your own teammate in the Naughty Corner
- ⬜ The same attacker and victim pair has a longer cooldown (~90s)
- ⬜ No points for slapping someone who's already down
- ⬜ 3s protection at the start of each round
- ⬜ No points in warm-up or after the round ends
- ⬜ **Playtest:** try to farm points two ways; neither should work

## 4. Lobby and game setup ⬜
> Before the match: hang out, get ready, pick your look.

- ⬜ Waiting yard before the match, with free play and no scoring
- ⬜ Player list with ready ticks
- ⬜ Ready button, with a "3 / 4 ready" count
- ⬜ Host starts when everyone's ready, or can force start
- ⬜ Late joiners jump in next round
- ⬜ Pick your look: colour, one accessory (cap / bucket hat / sunnies / headband), optional shirt pattern (original blob style)
- ⬜ All host settings in one panel (rounds, teams, friendly fire, yard features, smoko mode)
- ✅ Simpler setup screen (tabs, big Play button, More options)
- ✅ 🎥 Field of view slider (60 to 105, default 85) under your name on the menu, and also in the Smoko pause screen and the yard panel. Remembered on each player's own device. Sprinting still widens it a little
- ✅ Ticking or unticking things in "What's in the yard" (and Cheeky mode for the chest) adds or removes them from the yard behind the menu straight away, for whichever tab you're on
- ⬜ Rules on their own screen, in short pages, only showing what's switched on

## Cheeky-mode bottles ✅ (Rust version, 11 Oct 2026)
> Added 5 Oct 2026 (Marcus). Rust version first.

- ✅ In Cheeky mode, the VP cans are swapped for glass bottles: brown beer stubbies, and (1 in 4) square bottles of rum, thrown the same way
- ✅ Bottles smash on a hit or a hard landing: glass shards (brown or clear) plus beer foam or a dark rum splash, a stain on the ground (darker for rum) and the usual slippery puddle
- ✅ Spirit bottles are rarer and hit a little harder (knock 10 instead of 8, stun 0.70 s instead of 0.63 s); beer bottles throw exactly like the VP can. A bottle hit scores 100 more than a teddy (beer 200, rum 225)
- ✅ A Bottles setting (Cheeky mode, Settings tab): None, Few, Normal or Lots
- ✅ Family mode (Cheeky off) keeps the plastic VP cans exactly as now
- ✅ Check: the bar's own bottles and the drinks you drink stay as they are; only the throwable item changes

## 5. Polish ⬜
> Juice once the gameplay feels right.

- ⬜ Carry animations: grab lunge, drop flop (the draped carry and wriggle shake are in)
- ⬜ Naughty Corner: dunce cap wobble, sulking pose
- ⬜ Sounds: grab grunt, "OI!", naughty jingle
- ⬜ Marcus's custom sounds for Dazza and other effects
- ⬜ Phone buttons for Grab/carry (V) and Ready

## 6. Engine and Steam ⬜ (later)
> Finish the gameplay here in the browser first (quick to change), then decide on the engine.

- ⬜ Keep building and tuning gameplay in the browser version until it feels right
- ⬜ Then compare engines:
  - ⬜ **Godot 4:** free engine, lighter, similar cartoony look with better lighting
  - ⬜ **Unreal Engine 5:** best looks and real ragdoll physics; a full rebuild using this game as the blueprint
  - ⬜ Option: wrap the current browser version as a desktop app for Steam (keeps today's graphics)
- ⬜ Build a small test scene (yard, one character, a floppy dildo) in the chosen engine before committing
- ⬜ Only use Steam-safe sounds and assets (CC0, CC-BY, Pixabay, ZapSplat; no NC or ND), and keep a licence list

## Ideas parked
- ⬜ Weather or time of day
- ⬜ Sabotage items
- ⬜ Dazza crying or throwing hot snags
- ⬜ Recruit Dazza (easter egg, idea only — not built): stun him with meat 3 times in one round *without* ever KO'ing/berserking/flipping him (i.e. only ever use the steak/fish quick-stun, never the dildo), then feed him something while he's stunned on the 3rd time (needs a replacement for the Dodgy Snag, which has been removed). He joins your team for the rest of the round (follows you, slaps your rivals for free, no more escalating chase). No on-screen instructions — the clue is the "he's not as angry as he should be" dialogue line already in his escalating-anger chatter if he's never been dildo-KO'd. Needs: a small tracker for stuns-without-KO, feeding logic while NPC.state==='stunned', and a "recruited" NPC state. Confirm before building
