# Australian BBQ: Rust rewrite plan

Analysis only. Nothing has been rewritten yet. Based on the browser game at **version 0.23.1** (GitHub commit `9c5ed07`).

---

## 1. How the current game is built

### Files

| File | Size | What it is |
|---|---|---|
| `public/index.html` | 3,095 lines (~290 KB) | The whole game: about 200 lines of CSS, 350 lines of HTML (menus, HUD, overlays) and 2,500 lines of JavaScript in one script |
| `public/room-ws.js` | 67 lines | Connects the game to the server over WebSocket. It fakes the "room" API the game expects when it runs inside a Claude artifact |
| `server.js` | 81 lines | Node.js server: serves the files and relays messages between players in rooms. No game logic |
| `package.json` | | One dependency: `ws` (WebSocket) |
| `CHECKLIST.md`, `PROMPT*.md`, `checkpoints/` | | Feature list, prompts, an old saved version |

### Technology

- **Three.js r149**, loaded from a CDN, for 3D. **Web Audio** for sound. **Canvas 2D** for textures, name tags, speech bubbles and the Heist scoreboard. **HTML/CSS** for all menus and the HUD. Google Fonts: *Bowlby One* and *Figtree*.
- **No asset files at all.** Every model is built from code out of spheres, capsules, boxes and cones. Every texture is drawn in code, and every sound is synthesised in code.
- One big function holds everything, with global state. The only thing saved on the device is the field of view setting.

### The game loop (`frame()`, line ~3042)

One `requestAnimationFrame` loop, with the frame time capped at 50 ms. It is **not** a fixed timestep. Each frame, in order:

1. Countdown and round clock (the host ends the round).
2. Player input (`playerInput`), then carry/drag, then the host-only checks (`carryHostTick`, `chuckHitTick`).
3. Bot brains (`aiUpdate`, host only).
4. Character physics (`updateChar` for your own player and bots; `updateProxy` smooths other online players), then pushes overlapping characters apart (`separate`).
5. Leader crown, then the camera (bob, shake, roll, drunk sway, knocked-flat and stacked-it views, FOV).
6. Items (`updateItems`), item spawner and throw tracking (host).
7. Particles, puddles, chest, smoko chairs, Dazza (host brain plus everyone's animation), BBQ.
8. Character meshes and animation (`syncCharMesh`, `updateCharFx`), then item meshes and the throw-path preview.
9. Floating text and the HUD.
10. Network send, 20 times a second.
11. Render, with the drunk post-process shader when you're drunk.

---

## 2. What each system does now

### Player movement and controls
- **Keys:**
  - Move and look: WASD/arrows to move; the mouse looks around, with pointer lock or a drag-to-aim fallback.
  - Throwing: hold click to wind up and release to throw; right-click catches.
  - Movement extras: Space jumps (and wriggles when dragged); Shift is a speed boost.
  - Actions: R interacts (drink, BBQ, chest, smoko, hold to help someone up); F grabs, drags and throws someone who's down, and otherwise winds up a throw.
  - Items, emotes and pause: Q/E/1/2 swap items; T, G and B are emotes; P/Esc pauses.
  - Touch: on-screen buttons on phones. There's no gamepad support.
- **Movement** (`updateChar`, line ~2160):
  - Your input becomes a "wish" direction. Top speed is 6.2 m/s, multiplied up or down by being in the pool, winding up, boosting, drinking, the old snag effects, dragging someone and Drunk mode's walking speed.
  - Speed changes towards the target by a "grip" amount: 12 on grass, 6 in the air, 3.2/2.2 when stunned, 1.1 on a puddle, 0.3 while sliding.
  - Gravity for characters is 24. Jumping is 7.5, with 0.14 s of input buffer and 0.1 s of coyote time.
  - Boost is 1.7× for 1.6 s, then a 5 s cooldown.
  - The trampoline bounces you higher each time in a row, up to 5 times. The pool slows and sinks you.
  - Drunkenness twists your movement direction.
- **Camera:** first person, with walking bob, screen shake, roll, drunk sway, special views for slap outcomes and falls, and an FOV slider (60–105, sprinting adds 7).

### Characters, models and animation
- **The blob** (`createChar`, line ~1494): a capsule body, sphere head, eyes, feet and two floating hands. Extras: crown (leader), stun stars, team sash, a name tag sprite and a beer can for smoko.
- **All animation is code**, driven by time and springs:
  - Walk bob, drunk sway and lean, tilt/spin when hit (`applyTumble`).
  - The throw swing comes from a small keyframe table (`SWK`).
  - Three slap outcomes: Sent Flying, Cartwheel, Timber (`slapPose`/`slapCam`).
  - Knocked flat, stacked it, seated, the drag pose for both people, emotes.
  - The HELP ME sign and the stink cloud.
- **Dazza** (the BBQ NPC) is also built from code. He has 6 states (cook, angry, chase, return, KO, stunned), speech bubbles and a spatula swing.

### Map and world
- **Yard:** 66 × 48 m (`W=33`, `D=24`).
  - Structures: fence, house and shed over the back, a water tank, gum trees and clouds.
  - Middle of the yard: the Hills Hoist, the deep pool (14 × 8 m, cut into the lawn) and the trampoline.
  - Cover: eskies, table, bins, crates, hedge, tyres, woodpile, brick wall and planter.
  - Hangouts: the BBQ and meat table, the bar, smoko chairs (more chairs as more players join) and the chest (moves between 6 spots).
- **Feature switches:** bar, BBQ, chest and smoko can each be turned off. Turning one off hides its meshes and removes its colliders.
- **Teddy Heist arena:**
  - Layout: an oval of stone steps with a half-round bay on each side, and a crate cross in the middle.
  - Bases: 2–4 walled team bases, each with a floor pad, a flag pole, a light beam and teddies.
  - Scoreboard: a big floating 4-sided cube.
- **Spawn points:** fixed lists for items and characters, plus team spawns inside the bases in Heist.

### Physics and collision (all hand-written, no physics engine)
- **Colliders** are boxes on the ground with a height (`{x0,x1,z0,z1,h}`). You can stand on top unless a box is flagged `noTop` (the clothesline pole).
- **Characters** are circles of radius 0.42 pushed out of the boxes. They step onto a box top if they were above it, are kept inside the yard edges, and are pushed apart from each other within 0.9 m. There's no character rotation physics.
- **Items** are spheres, each with its own size:
  - Flight: gravity 18 × the item's own factor. Fast items take several small steps per frame so they don't pass through walls.
  - Bounce and impact: each item has its own bounce amount. VP cans smash and leave a slippery puddle.
  - Special surfaces: the trampoline bounces items, the pool floats them, and items leaving the yard are removed (Heist teddies go home instead).
- **Hits:** each player's own computer checks whether an item touched its own body. The host then checks the throw is real (within 1.8 s).

### Interactions and items
- **8 item types:** teddy, VP can, gnome, dildo, steak, fish, pool noodle, plus the dormant snag.
  - The dildo has 4 size variants.
  - Rare Bum-Out gnomes appear sometimes.
- **Using items:**
  - Carrying and throwing: hold 2 and select/swap between them. Throw power builds as you wind up. A throw at 90%+ knocks the person over. Holding a full wind-up too long drops the item.
  - Slapping: the steak, fish and noodle stun. The dildo knocks people flat, with a 15% critical hit. With Cheeky mode on, you can also slap bare-handed.
  - Catching an item flying at you scores +50.
- **Stun rules:** stuns don't stack, and there's a 1 s grace after a stun ends.
- **Bar:** 3 drinks fill a drunk meter from 0 to 100, through 5 levels.
- **Stacking it and helping up:**
  - Being drunk makes you fall over. Drunk mode adds its own fall chance (4% per 2 s of walking) and the uneven walking speed.
  - Holding R next to a fallen mate helps them up.
- **Smoko and Naughty Corner:** sitting at smoko is safe but you keep drinking. In Naughty Corner mode, dropping someone there gets +100 and sticks them on a chair for 5 s.
- **Grab, drag and throw:** grab someone who's down by the ankles, drag them, throw them, and they wriggle free. A thrown person who hits someone knocks them over (the "human cannonball").
- **BBQ, Dazza and the chest:** steal meat from the BBQ and Dazza gets angrier and chases you. Slapping Dazza can KO him, send him berserk or flip the barbie. The chest holds dildos.
- **Pool/tramp bonuses, emotes, scoring and modes:**
  - Pool/tramp: knocking someone into the pool or onto the tramp scores a bonus.
  - Emotes: there are 3, each with speech lines.
  - Scoring: 100 per hit, streak multipliers, a leader bounty, a drunk bonus, a long-shot bonus and the scores for catches and help-ups.
  - Modes: Free for all, Teams (Red/Blue/Wildcard, with friendly fire) and Teddy Heist (steal, bank for 150, return).
  - Matches: best of 1, 3 or 5.

### UI and menus
- **All HTML/CSS** drawn over the 3D canvas:
  - Main menu (Solo / With mates / How to play tabs) and the options panels.
  - The yard (lobby) card, results, and pause ("Smoko").
  - HUD: scoreboard, clock, item slots, drunk meter, wind-up bar, hints, banners, alerts, the kill-feed-style event feed, the Heist tally and interaction prompts.
  - Floating score text that follows 3D positions, the hit marker, and touch buttons.
- **Text drawn inside the 3D world** uses canvas textures: name tags, speech bubbles, the HELP ME sign and the Heist scoreboard.

### Lobby and waiting room
- **Finding a yard:** everyone first joins a shared room called `lobby`. Hosts advertise their yard there, and the With mates tab lists the open yards. Joining needs a code like `yard-kbzq`.
- **Warm-up:** in a yard, everyone starts in free play with no scoring. The host chooses settings and presses Start round.
- **After the game:** results show, then everyone goes back to the lobby when the match is over.

### Multiplayer and networking
- **Server:** a dumb relay. It keeps rooms (max 16 players), stores each player's latest "presence" message (JSON, up to 8 KB) and forwards it to everyone else. It has a flood limit of 120 messages a second and no game logic.
- **The host is a player's browser.** Whoever hosts runs the bots, items, Dazza, scoring and round flow.
- **Every player** sends presence 20 times a second. It contains:
  - your position, velocity, facing and state flags;
  - your last 14 actions, numbered so none are missed.
- **The host's presence also carries a world snapshot**, cut down to fit ~3.9 KB: items, bots, scores, settings, teams, Heist state, drags, Dazza, puddles and the clock.
- **Guests** move their own player themselves and send actions (about 20 kinds, such as pick up, throw, hit, slap, grab). The host checks and decides, then sends out "fx" events (31 kinds) that everyone plays.
- **Hiding lag:**
  - Other players are smoothed between updates.
  - A guest's own throws show instantly as "ghosts".
  - Online, the person who got hit is the one who reports the hit.
- **No cheating protection:** the host trusts guests about their own position and about being hit.

### Audio and assets
- **About 35 sound effects**, all synthesised with oscillators and filtered noise. Volume drops off with distance (no left/right panning).
- There are no voice clips, music files or recorded sounds.

---

## 3. Recommended Rust architecture and tools

Versions checked October 2026. Check them again when work starts, because Bevy releases breaking changes every few months.

| Need | Recommendation | Why | Alternatives |
|---|---|---|---|
| Engine | **[Bevy](https://bevy.org/news/bevy-0-19/) 0.19** (June 2026) | Popular, open source (MIT/Apache) Rust engine. ECS design, 3D renderer, glTF models with skeletal animation, UI with text input, audio, and desktop plus web (wasm) builds | Fyrox (has an editor, smaller community); Godot with Rust bindings (if you want a visual editor) |
| Physics | **[Avian 3D](https://docs.rs/avian3d) 0.7** (Bevy 0.19) | Bevy-native colliders, ray and shape casts, collision layers, rigid bodies. Use it for queries and thrown objects, but keep the hand-tuned character movement as our own code so it feels the same | Rapier (`bevy_rapier3d`); `bevy-tnua` or `bevy_ahoy` character controllers on top of Avian |
| Networking | **[lightyear](https://github.com/cBournhonesque/lightyear) 0.29** (Bevy 0.19) | Server-authoritative multiplayer with client prediction and rollback, smoothing of other players, and a "host plays and runs the server" mode. Transports: UDP, WebSocket, WebTransport and Steam (via `aeronet`) | `bevy_replicon` + `renet` (simpler, less built in) |
| UI | **`bevy_ui`** for HUD and menus; **[bevy_egui](https://docs.rs/crate/bevy_egui/0.41.0)** for debug and tuning panels | `bevy_ui` can match the current look; egui is fastest for tools | Bevy Feathers widgets |
| Audio | **[bevy_seedling](https://docs.rs/bevy_seedling)** (Firewheel), or built-in `bevy_audio` | Spatial audio and effects. The plan is to record today's synthesised sounds to files once so they sound identical | `bevy_kira_audio` |
| Characters and animation | **Blender → glTF**, played with Bevy's `AnimationGraph` | Keeps the blob design but with a proper skeleton, animation clips and blending | Rebuild the code-made blobs first for quick parity, then swap |
| Steam | **`bevy-steamworks`** / `steamworks` crate | Steam overlay, friends, invites, relay networking | Check compatibility with Bevy 0.19 before relying on it |
| Data | **RON/TOML** asset files | Item stats, drinks, dildo variants, Dazza lines and bot names move out of code so tuning doesn't need a rebuild | |

### Suggested layout (one Cargo workspace)

```
australian-bbq/
  crates/
    bbq_core/     pure rules, no Bevy: item/drink defs, scoring, match flow, drunk maths,
                  carry rules, heist rules. Fast unit tests.
    bbq_sim/      Bevy plugin that runs the game world at a fixed 60 Hz; also runs headless
                  (server / tests): movement, items, bots, Dazza, heist.
    bbq_net/      lightyear protocol: player inputs, replicated components, server events.
    bbq_client/   rendering, models, animation, camera, audio, UI, input.
    bbq_server/   headless dedicated server binary.
    bbq_app/      the game executable (desktop, and web if we keep browser play).
  assets/         glTF models, sounds, fonts, RON data files.
```

### Architecture improvements over the JavaScript version
- **Fixed 60 Hz simulation** separate from rendering. Today, how the game behaves depends slightly on frame rate.
- **The server decides everything,** including hits, with lag compensation, instead of trusting players.
- **ECS components** (Position, Velocity, Stun, Drunk, Carrying, Held items…) instead of one big object per character with 60+ fields.
- **Game rules in a pure crate with unit tests**, so behaviour can be checked without running the game.
- **Typed network messages** instead of position-based arrays like `[id, type, state, x, y, z, …]`.
- **Real model and sound files** with a licence list, ready for Steam.

---

## 4. Major rewrite risks

1. **The feel changes.** Movement, throws and bounces depend on hand-tuned numbers (grip values, two different gravities, frame-time cap). Moving to a fixed timestep and a different engine will shift the feel. *Fix:* write a numbers-and-behaviour spec from the code, port the maths exactly, and compare side by side with video.
2. **Networking is a different model.** Today: a browser host, players report their own position and hits, and a dumb relay. Rust: a server that decides everything, with prediction. Drag-pinning, ghost throws, hit detection and the snapshot trimming were all built around the old model. This is the biggest rework.
3. **Playing in the browser.** The link-in-a-browser setup is what makes it easy for mates. Bevy runs on the web, but needs WebGL2/WebGPU, has slower load times, and a browser connection needs a server with a proper https certificate. Decide early whether browser play stays.
4. **No art to import.** Everything is made in code, so the blobs, Dazza, the yard props and all the canvas-drawn text (name tags, bubbles, scoreboard, signs) must be rebuilt, either in Blender or in Rust code.
5. **The UI is big.** Menus, options, lobby, HUD and floating text are a lot of screens to rebuild in `bevy_ui`. Text boxes (name, yard code) only arrived in Bevy 0.19.
6. **The sounds are made in code.** About 35 synthesised sounds need recording to files or rebuilding.
7. **Bevy and plugin churn.** Bevy breaks things every 3–4 months, and plugins (physics, networking, Steam) lag behind. Pin exact versions and upgrade on purpose.
8. **Small behaviours get lost.** About 3,000 dense lines hide dozens of small rules: feed messages, Dazza's anger build-up, Heist edge cases, stun grace, drunk falls and more. Use CHECKLIST.md plus the behaviour spec as a sign-off list.
9. **Hosting.** A server that decides everything needs to run somewhere: your Mac, a cheap VPS, or Steam's relay with a player hosting.
10. **Build and learning cost.** Rust and Bevy compile slowly, and there's a lot to learn. Without tests, an AI-assisted rewrite drifts. Keep each phase small and testable.
11. **The physics engine might feel different** from the hand-made bounces. Use Avian where it helps (queries, thrown items, ragdolls later) and keep our own code where the feel matters.
12. **Steam.** Steamworks needs a partner account, has SDK terms, and the Steam Deck should be tested.

---

## 5. Phased rewrite checklist

Work top to bottom. Every phase ends with something runnable and checked against the JavaScript game.

### Phase 0: Decisions and freeze
- [x] Decided (4 Oct 2026): **desktop first, browser later**. Build for Mac/Windows/Steam; the JavaScript game stays as the browser version for mates until a web build is wanted
- [x] Decided (4 Oct 2026): **a player hosts**, like today. No dedicated server for now (keep the sim headless-capable so one can be added later)
- [x] Tagged the JavaScript game as the reference (`v0.23.1-js`). Still to do: record short gameplay clips of each feature (needs Marcus's machine)
- [x] Wrote the **behaviour spec** (`BEHAVIOUR_SPEC.md`). It lists the places where the code and the written rules disagree (section 20); Marcus needs to decide those
- [x] Rust 1.99.0 installed on the Mac; Bevy 0.19.1 starter app in `rust/` (compiles); GitHub Actions build for macOS, Windows and Linux in `.github/workflows/rust.yml`. Still to do: run it once on the Mac

### Phase 1: Core rules (no graphics)
- [x] Set up the Cargo workspace and crates
- [x] `bbq_core`: item, drink and dildo-variant data; scoring (streaks, bounty, drunk bonus, long shot); round and match flow; teams; Heist banking; stun rules; drunk maths; drag rules
- [x] Unit tests copied from today's behaviour
- [x] Fixed 60 Hz simulation that also runs without a window (for tests and the server)

Phase 1 result: `bbq_core` has 79 unit tests, clippy is clean. Simplified/left out: the fixed-step `Sim` only runs the round clock (countdown, play, whistle, results panel) and each fighter's body and drunk meter; movement, items and hits join it in Phases 2 and 3. The dildo variant weights add up to 96 (not 100), as in the JavaScript game.

### Phase 2: The yard and moving around (solo, no bots)
- [x] Build the yard layout and colliders from `buildWorld` (simple shapes first), lighting, sky, fog, shadows
- [x] Feature switches (bar, BBQ, chest, smoko) (F1-F4 for now; menu comes later)
- [x] First-person camera, mouse look and pointer lock, FOV slider, sprint FOV, bob, shake (FOV is `[` and `]` for now; a real slider comes with Settings in Phase 11)
- [x] Port the character movement: grip, jump buffer and coyote time, boost, trampoline chain, pool, puddles, slide, yard edges, stepping onto objects (puddles and slide are in the maths as inputs; the puddles themselves arrive with stubbies in Phase 3)
- [ ] Side-by-side feel check against the JavaScript game (needs Marcus: `cd rust && cargo run`)

Phase 2 result: `bbq_core` gained `yard` and `movement` (111 tests in total, clippy clean). `bbq_app` now draws the yard and lets you walk it. Not checked by me: how it looks and feels on screen, because the cloud machine has no graphics card. Simplified: boxes are plain coloured cuboids, the pool is a flat blue slab (no sinking water yet), no sky dome or clouds, and the camera does not yet lie down when stunned.

### Phase 2b: Match the browser look (added 4 Oct 2026, Marcus)
The Rust version still draws items and props as plain shapes and does not look like the browser game. This phase fixes that before any more gameplay. Rule: every object is written as **plain data** (a list of shapes with size, position and colour) so the same list can later drive a Blender script that makes `.glb` models, with no rewrite.
- [x] Items, copied from `MAKERS` (`index.html` line ~935): teddy, VP can, gnome, steak, fish, pool noodle, snag, and the four dildo sizes. Shapes are in `bbq_core/src/looks.rs` (data only), meshes in `bbq_app/src/shapes.rs`. Same shapes and places as the browser; the picture differs by about 11/255 on average, mostly highlights. The dildo and noodle do not flop about yet (the browser game bends them)
- [x] Yard props, copied from `buildWorld`: fence, house, shed, water tank, Hills Hoist (it turns), eskies, table, bins, crates, hedge, tyres, woodpile, brick wall, planter, bar (with drinks and signs), BBQ and meat table, smoko pad, pole, umbrella, esky and chairs, chest (box, hinged lid, toys). Shapes in `bbq_core/src/looks_yard.rs`
- [x] Textures: 17 pictures exported from the browser game as PNG (`bbq_app/assets/textures/`), with smaller copies made for distance
- [x] Sky colour, clouds (they drift), gum trees (same positions as the browser game when `make_ref.py` seeds it), pool water (slides) and shimmer, trampoline, sun and shadows, fog
- [x] Side-by-side check: six camera spots compared with the browser game on Marcus's Mac, average difference 4 to 9 out of 255 (`rust/tools/compare_views.sh`). Lit lawn, fence and house match within about 4%
- Still different (small): the water's green is about 7% low; fog is a straight line where three's is an S-curve; surfaces facing down are a little bright (no real hemisphere light); the smoko pad and chair count follow the number of people (the browser comparison page has none); the chest and the Heist arena are not compared yet
- [ ] Later and optional: make nicer Blender versions from the same data lists

### Phase 2c: Sun-Baked Cartoon Realism (added 5 Oct 2026, Marcus)
Its own list, worked one item at a time, highest impact first: see **`GRAPHICS_2C.md`** (inspection findings, decisions, and the checklist for characters, proportions, environment, backyard details, props, the Dildo Chest, animals, models, textures, materials, lighting, rendering, Blender workflow, high-detail assets and optimisation). Gameplay and the JavaScript game do not change.
- [ ] Work through `GRAPHICS_2C.md` (tick items there; note here when a whole section is done)
- [ ] Phase 2c look check on Marcus's Mac (before and after screenshots)

### Dazza fixes (5 Oct 2026, Marcus)
- [x] Dazza no longer glitches when you grab food from the table: he turns at 13 rad/s instead of snapping up to 180 degrees in one frame, his walk animation speed is smoothed (it stuttered when the frame rate was not a multiple of the 60 Hz tick), a new speech line is not started for every steak, and he no longer dithers at the 5 m edge when he walks out to a thief
- [x] He runs after you for longer. DELIBERATE CHANGE from the browser game (`BEHAVIOUR_SPEC.md`): a chase now lasts 12 + 3 per anger level (12 to 36 s, was 8 + 2 per level, 8 to 26 s); after a spatula hit he gloats for 1.8 s and keeps chasing (at least 6 s more), going home after 3 hits in one chase (he used to go home after the first hit); grabbing more food while he is chasing keeps him coming. The numbers are `chase_dur`, `CHASE_HITS`, `GLOAT`, `KEEP_CHASING` in `bbq_core/src/dazza_brain.rs`

### Phase 3: Items and throwing
- [x] Item spawner, pick up, 2 slots, swap, ground glow
- [x] Wind-up, throw, holding too long, throw-path preview, power throw
- [x] Item flight (small steps, bounce, can smash and puddle, pool float, trampoline, leaving the yard)
- [x] Hits, knockback, stun rules (no stacking, 1 s grace), catching, hit scoring

Phase 3 result: `bbq_core` gained `flight`, `hands`, `hitting`, `itemworld`, `vec` (166 tests in all). `bbq_app` has the items, your hands, a throw-path line, puddles and three practice dummies to hit (blue = fine, red = stunned, yellow = in the 1 s grace). The headless tests in `bbq_app` throw a teddy at a dummy and check the score, the over-hold drop, a miss, a swap and a pick-up. Not checked by me: how it looks and feels on screen. Left out for now: melee slaps (steak, fish, noodle, dildo, bare hands) arrive with Dazza and bots; the late-hit window (1.8 s) belongs to online play in Phase 8; hits on the player come when bots can throw (Phase 5).

### Phase 4: Characters, models and animation
- [x] Blob characters (keep the design): four selectable Blender models in `rust/crates/bbq_app/assets/models/` (`blob_classic`, `blob_pear`, `blob_egg`, `blob_gumdrop`.glb), built by `rust/tools/make_blob.py`. Classic is the original capsule; the other three are Marcus's new body shapes. Looks only: speed and hit sizes are the same for all. They are separate parts, not a skinned skeleton: the game moves the body and hands from the animation code. In the Rust test yard, F9 cycles your pick and the three practice blobs wear the other three. A proper character-select screen comes with the menus (Phase 8).
- [x] Animations (procedural, numbers ported from the JS): idle, walk, throw swing, Sent Flying, Cartwheel, Timber, knocked flat, stacked it, seated, drag, emotes (taunt, laugh, dance), drunk sway. Logic is in `bbq_core/src/pose.rs` with tests.
- [x] Name tags, speech bubbles, leader crown, stun stars, team sash, HELP ME sign, stink cloud (simple Text2d / shape versions)
- [x] Dazza model and his 6 states (look and states only; his behaviour is Phase 5). Logic in `bbq_core/src/dazza.rs`.
- Result: 199 core tests + 5 app tests pass. Left out or simplified: the player's own body is hidden (first person); `slap_cam` camera effects are written but not wired to the camera; the look and animations have NOT been seen on a screen yet (no GPU in the cloud). Marcus: run `cargo run` and use the viewer keys (J/K/L slap, N stacked, M emote, Y drunk, C crown, V sash, X stink; Dazza now runs for real, see Phase 5), then tick "looks right" here.
- [ ] Phase 4 look check on Marcus's Mac

### Phase 5: Yard life
- [x] Bar, drinks, drunk meter and levels, drunk camera and screen shader (R at the bar; rules in `bbq_core/src/drunk_state.rs`, 21 tests, plus 11 game tests. SIMPLIFIED: the screen shader is a camera sway, a breathing view and a warm pulsing tint, not the blur/wobble shader. Bar drinks have no sound yet (Phase 10).)
- [x] Stacking it and helping up; Drunk mode (walking speed, 4% falls) (F5 falls on/off, F6 Drunk mode, P pours a quick +30 drunk for testing. Hold R next to a fallen blob 1.5 s to help them up for +25. In the test yard only you can fall, the three blobs only fall when you press N. Team-only helping is in the rules but untested until Phase 7. Fall alert alarm sound comes with audio.)
- [x] Smoko and Naughty Corner (R inside the pad sits you in the nearest free chair for up to 20 s; nobody can hit you; +4 drunk per second; R or Space stands you up. With the Naughty Corner on (default; F8 toggles) the pad is NOT a safe break: R just tells you so, and setting a downed blob on the pad, or chucking them onto it, sits them in a chair for 5 s and pays you +100. Rules in `bbq_core/src/smoko.rs`. SIMPLIFIED: the three practice blobs never choose smoko themselves; chairs are drawn for 4 people and don't grow with the crowd until real players/bots exist.)
- [x] BBQ meat raid and Dazza (R at the left half of the meat table gives a steak, the right half a fish, max 2 held, 3 uses each. His whole brain is in `bbq_core/src/dazza_brain.rs`: grudges, angry, chase, spatula swing, stun, KO (8 s), berserk, grill flip with 5 bits of meat flying. A steak/fish/noodle slap on him stuns him for +20; a dildo slap knocks him out, sends him berserk or flips the barbie for +75/+25/+50. The picture now follows the real brain: the old Z/H demo keys are gone. SIMPLIFIED: the food on the grill doesn't animate when he flips it; no sounds.)
- [x] Chest, dildo variants and crits, bare-handed slaps, Cheeky mode switching content (F7 switches Cheeky mode: the chest at its spot (R within 2 m, 3 max, refills every 12 s), rude Dazza and emote lines, bare-handed slaps with nothing in your hands (wet willy/noogie/wedgie +40), and a gnome can pop out of Dazza's bum. Steak/fish/noodle slaps stun, the dildo knocks flat in one of three ways with a 15% crit and the four sizes. Rules in `bbq_core/src/melee.rs` and `chest.rs`. SIMPLIFIED: the chest stays at the spot the yard starts with (no new spot each round until rounds exist); your own swing is not drawn (first person), the dummy just reacts; slap camera effects are still not wired.)
- [x] Grab, drag, throw, wriggle, human cannonball (F next to someone who is lying down grabs them by the ankles; tap F puts them down, hold F 0.4 s then let go chucks them; a chucked blob that lands on someone flattens them for +100. SIMPLIFIED: only you can grab and only the practice blobs are grabbed, so the Space-wriggle is not used yet (blobs wriggle by themselves, 1.7x timer); F does not wind up a throw when nobody is down; dragged picture uses the existing drag pose.)
- [x] Pool and trampoline bonuses, emotes (knock a blob into the pool or onto the tramp within 3.5 s of hitting them: +50 "SPLASHDOWN!" / "INTO ORBIT!". T taunt, G dance, B laugh with a 2 s cooldown, the line shows as pop-up text. Rules in `bbq_core/src/emotes.rs`. SIMPLIFIED: your own emote animation isn't visible (first person) and there is no speech bubble over you.)
- Result: 279 core tests + 43 app tests pass; clippy clean. Not checked by me: how any of it looks or feels (no GPU in the cloud). Marcus: `git pull`, `cd rust && cargo run`, then try: steak from the meat table and slap a blob and Dazza; F7 for Cheeky mode; F8 for the Naughty Corner; knock a blob (N makes them fall) then press F to drag and chuck them. Tick "feels right" below when happy.
- [ ] Phase 5 look and feel check on Marcus's Mac

### Phase 6: Bots
- [x] Bot brain (`bbq_core/src/bots.rs`, 33 tests): picks targets (leader x2.8, real players favoured by difficulty, teammates and people at smoko skipped), goes for the nearest free item (never melee items, never ones another bot has claimed), keeps its range and orbits, winds up for 0.55-1 x the item's charge and throws with a ballistic aim that leads the target and gets worse with drunkenness, never power-throws, dodges/hops/catches incoming items by difficulty (easy/fair/spicy table), swings melee items every 0.9 s, helps fallen teammates up in team modes, and runs errands every 14-26 s (bar, smoko 5-9 s, chest, meat; none in Heist)
- [x] Natural steering (`botSteer`): turn-speed limit, easing, personal wobble, drunk sway, giving mates room, slowing near the goal, look-ahead round obstacles on one side, stuck check. Heist goals: the brain takes a goal and a "carrying a stolen teddy" flag; the goal rules themselves arrive with Phase 7
- [x] Bot checks: headless walks with the real movement rules (6 seeds x 90 s, plus beside the shed): longest stuck time under 2 s and under 8 turns right round
- [x] In the game (`bbq_app/src/bots_app.rs`): the three practice blobs are now bots. They pick things up, throw, hit you (-50, knocked back, popup "Bruce got you!"), slap you with steak/fish/noodle/dildo, drink at the bar (never stack it), sit at smoko, take meat (Dazza gets cross at them too), take chest toys in Cheeky mode, catch, and the leader wears the crown. F10 freezes them into practice dummies, F11 cycles easy/fair/spicy. Held items are drawn in their hands
- Result: 335 core tests + 80 app tests pass. Simplified or left out: bots can't yet be hit into the pool for a bonus when they hit the player (the player has no "last hit by" yet); Dazza only chases them (no bot-bot grabbing, as in the browser game); bot count is still fixed at 3 until the menu (Phase 8)
- [ ] Phase 6 look and feel check on Marcus's Mac

### Phase 7: Modes and matches
- [x] Free for all, Teams (with Wildcard), friendly fire (`bbq_app/src/round.rs`; teams are dealt, wear sashes, bots never target teammates, teammates pass through each other unless friendly fire is on)
- [x] Teddy Heist: oval arena, bases with flags and light beams, teddies, stealing, banking for 150, strays sent home, big four-sided scoreboard, bots play the objective (`bbq_core/src/heist.rs`, `bbq_app/src/heist_app.rs`). SIMPLIFIED: the scoreboard shows team-colour bars with plain pixel digits (teddies banked big, points small), no team names or emoji; the teddy drops only when stunned or knocked down; no separate tally bar on the HUD yet (the top-right text line shows the scores)
- [x] Rounds, best of N, results, leader bounty, streaks (countdown 3.2 s, play, whistle, results panel, best of 1/3/5). Until the Phase 8 menus, keys set it up: Enter starts a round, F12 mode, -/= bots, backslash round length, semicolon best-of, quote friendly fire. `--mode heist|teams|ffa` on the command line starts a round straight away
- Result: 351 core tests + 103 app tests pass (round flow, Teams, Heist steal/bank/drop/stray/winner, bots stuck check). Checked on screen with screenshots: arena, bases, flags, scoreboard, HUD countdown
- [ ] Phase 7 look and feel check on Marcus's Mac

### Phase 8: UI and menus
- [x] Main menu (Solo / With mates / How to play), all options, Cheeky mode, FOV (`bbq_app/src/menu.rs`, `ui.rs`, `results.rs`): cream card, yellow outlined title, red Play, segmented rows, tick chips, FOV slider, name field, yard behind it with three wandering bots and the slow orbiting camera. SIMPLIFIED: no emoji on labels (the engine can't draw colour emoji); the fonts are the browser's own, Bowlby One and Figtree (SIL Open Font Licence, files and licences in `rust/crates/bbq_app/assets/fonts/`, built into the program); "With mates" is a placeholder until Phase 9; no touch controls
- [x] Character select: pick Classic, Pear, Egg or Gumdrop (a "Your blob" row in the Solo tab; other players see your pick online comes with Phase 9)
- [x] HUD (`bbq_app/src/hud.rs`): scoreboard (team groups, crown, Heist banked), yellow clock box that turns red and pulses in the last 10 s, round tag, drunk meter, Heist bar, feed, crosshair with hit marker and catch ring, wind-up bar (red at power, blinking when over-held), Space/Shift/Right-click chips with cooldown bars, two item slots, big outlined banner, hint line with the first-time hints and the team line, the R prompt, pool and hit tints. SIMPLIFIED: item icons are coloured discs with the name (the browser draws small pictures); no separate red alert box (those lines go in the feed); the hit flash is a plain red tint, not the edge vignette; floating world text stays the centre pop-ups; the old developer text is behind F3
- [x] Pause ("Smoko" card, Esc), results card (title, win line, match line, table with accuracy and banked), settings saved on the device (`~/Library/Application Support/AustralianBBQ/settings.txt`)
- [x] Gamepad support (new): sticks walk and look (dead zone, smooth curve), A jump, right trigger throw/slap, left trigger catch, bumpers swap, X interact, Y grab, B drop, stick press boost, D-pad emotes, Start pauses, A/B on the menu and cards (`bbq_core/src/gamepad.rs`, `bbq_app/src/gamepad.rs`, listed on the How to play card). Not tried on a real controller yet (none in the cloud or here): Marcus to test. SIMPLIFIED: the menus' tabs and tick boxes still need the mouse; no rumble; no button remapping

### Phase 9: Multiplayer
How it works (decided 6 Oct 2026, close to the browser game): one player hosts and runs the rules (bots, items, Dazza, scores). Everybody moves their own blob so the controls feel instant, and says where it is 20 times a second; the others' blobs are puppets that follow. The relay is the same `server.js` the browser game uses (it now also passes game messages along, `rust/tools/relay_test.js` checks it), so friends join over the internet the way they do today (`npm start`, cloudflared). The messages are in `bbq_core/src/net.rs` (small binary, tested against junk), the conversation is `bbq_app/src/online.rs` (`Session`, no sockets, tested), the WebSocket is `bbq_app/src/net_link.rs` (tested against the real relay).
- [x] 9.1 Network messages: hello / welcome / refused / roster / player state / bye, the host's roster (ids, unique names, 16 players, version check)
- [x] 9.2 Connection: relay passes messages, Rust WebSocket thread, "With mates" tab with server address, room code, Host / Join / Leave and a status line, settings saved
- [x] 9.3 See each other: puppets for everybody else (their name, blob, position, facing, winding up), bots stay out while mates are in; `--net-host ROOM`, `--net-join ROOM`, `--server ADDRESS`, `--net-debug` for testing. Tried with two copies on one computer and a local relay; NOT yet tried over the internet or with a real friend
- [x] 9.4 The host's world reaches the guests (`bbq_core/src/net_world.rs` has the snapshot and its checks, `bbq_app/src/online_world.rs` builds it and applies it): about 15 times a second the host sends the items, bots (with what they hold and how they are doing), Dazza, chest stock, smoko pad, round phase and clock, scores, teams, new feed lines and particle effects. A guest's game is a mirror: it does not run the bots, items or rules, it copies the host and glides things between pictures; your own blob is still yours. Tried with a host in a round and a guest on one computer: item counts, bot positions and the clock matched. SIMPLIFIED / NOT YET: guests cannot pick up, throw or hit anything (that is 9.5); Teddy Heist is not mirrored (9.8); puddles, the Naughty Corner timers and the other eskies are not sent; the host's "You" in feed lines becomes the host's name by swapping the word; guests need the host to press Play after they join (a round in progress refuses newcomers)
- [~] 9.5 Guests' actions. DONE (`bbq_core/src/net_act.rs`, `online_world.rs`): picking things up (the host does it for them as they walk over items, people take everything, bots leave the slapping things alone), choosing which item is in hand, throwing (the guest winds up and aims as usual, sends item, start, speed and charge; the host checks they hold it, are close to where it saw them and the round is on, then flies and scores it), dropping with G, slapping (steak, fish, noodle, dildo with its own reach and swing speed; the host finds who is in front of them), catching, and being hit: the host tells the person hit (thrown item or slap) and their own blob is knocked about the same way. A guest's item leaves their hand at once. NOT YET: drinking at the bar, the chest, smoko and the Naughty Corner, grabbing and dragging, emotes, helping up, bare-handed slaps, slapping Dazza, and "ghost" throws (the guest sees their throw leave the hand at once but the flying item appears with the host's next picture, about 70 ms plus the trip)
- [ ] 9.6 Rounds online: host starts the round for everybody, results for everybody, back to the lobby; late joiners wait
- [ ] 9.7 Leaving and rejoining mid-game (no host migration: when the host leaves the yard ends)
- [x] 9.8a Looks online (8 Oct 2026): everybody's body shape, face, hair, colours and belly travel with the hello and whenever they change (protocol 2); puppets in a round wear them
- [x] 9.8b The lobby: in a yard, not in a round, everybody's blob stands in a row on the lawn with their name; the Customise tab is open; a lobby card shows the room code, who is here and who is ready; Ready (arms up, READY sign, a grin) and the host's Start now; when everybody is ready a 3 s countdown starts the round; after the results the host goes back to the lobby and guests follow
- [ ] 9.8 Teddy Heist online; held items on puppets; a list of open yards
- [ ] 9.9 Tests with 2 to 4 players and simulated lag; speed and bandwidth check
- NOT doing for now: UDP/WebTransport/Steam relay (the WebSocket relay does the job, as in the browser game)

### Phase 10: Audio
- [ ] Record today's ~35 synthesised sounds to files (or rebuild them as synths)
- [ ] Spatial audio and volume settings; Steam-safe licence list

### Phase 11: Polish, packaging and Steam
- [ ] Settings (name, FOV, volume, key bindings)
- [ ] Builds for macOS, Windows, Linux/Steam Deck (+ web if chosen)
- [ ] Steamworks: overlay, invites, relay
- [ ] Performance pass

### Phase 12: Sign-off
- [ ] Go through CHECKLIST.md "Already in the game" line by line in the Rust version
- [ ] Retire the JavaScript version, or keep it as the browser edition
