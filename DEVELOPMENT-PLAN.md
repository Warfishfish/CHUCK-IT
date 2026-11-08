# Australian BBQ · five-feature coding plan

8 November — milestones 1–3 are included in starting commit 963e1f8. Milestones 4 and 5 are implemented locally for review at checkpoints/2026-11-08-race-gaze/review.html. Full clockwise guest and counterclockwise host laps, peer-visible gaze, 707 tests and both release builds verified. Commit, push and public browser release await review.

Work in /Users/marcus/Claude/australian-bbq. Scope is the five active backlog additions in NEXT_UP.md. Each milestone can ship independently. Suggested order follows the backlog so Smoko stays first. The two-slot work precedes Race so the full-lap playtest also exercises catching. Keep the shelter-material graphics pass as a separate review.

## Execution and handoff

- Luna handles bounded work such as measuring assets, reproducing current behaviour, focused edits, regression fixtures, capture and checkpoint updates. Give it one milestone and its acceptance tests at a time.
- Use Sol 6.1 High for the decisions that affect shared collision, state ownership, networking or animation layering, and to review each milestone's diff and evidence. Use Astra Medium for the requested gaze research if available. If unavailable, record that and use Sol 6.1 High for the research decision.
- Before edits, read checkpoints/LATEST.md, confirm the active branch and inspect pending work. Save a dated, verified recovery point including the pending road and gum-shade changes. Capture the baseline and record the launch command. Keep source edits scoped to the current milestone.
- For every milestone, test through the normal menu, run affected Rust tests, then verify native and private browser release builds. Record changed files, A/B evidence where visual, known limits and selective rollback. Ask for review before committing, pushing, publishing or starting the next milestone. Batch routine checks.

## 1. Move the Bush Servo inside the Smoko fence

**Decision first.** Compare two full-scale placements before permanent collision or art changes. The shipped frontage GLB already includes the shop, forecourt fixtures and ute. Its current world bounds are approximately x −23.75 to 11.67, z −33 to −24, height 7.3 m. The fence runs at x ±33 and z ±24. A small pump-only mockup would give a misleading impression of the space available.

**Luna:** Load the existing smoko_servo.glb through its root in yard_scene.rs. Make two local comparison builds or use a temporary capture-only placement option. Try the following translations from the original position, in metres, as starting points. Leave the shelter and scoring ring fixed.

- A, north-west: shift x −6 and z +9.5. The unrotated bounds become approximately x −29.75 to 5.67 and z −23.5 to −14.5. Keep the frontage parallel to the north fence, facing the oval. This is the first choice for the concept's shop-left, shelter-centre composition.
- B, north-east: shift x +17 and z +9.5. Unrotated bounds become approximately x −6.75 to 28.67 over the same z range. Check the existing prop near (27,−20). Try a slight inward angle only if the rotated footprint, pump clearance and circulation still fit. Check conflicts with spawns, bots and pickups in both placements.

**Review gate:** Show both options from the same player-height approach and from above with occupied footprints, the oval and two carrying routes marked. Recommend the better site based on actual clearance and sightlines. Obtain the user's location choice before permanent integration.

**Sol 6.1 High:** Define one canonical placement shared by visual roots, fallback geometry and solid bounds. Inspect Blender object bounds in build_servo.py and build_frontage_detail.py. Keep the shop a closed building and the veranda walkable where visible. Add simple shared colliders for reachable walls, posts, pumps and vehicle. Use collision shapes the existing movement and throwing systems actually support. Preserve credits and reuse the current model. Inspect the old exterior for duplicate signs, pumps or floating fixtures.

**Done when:** A host, guest and bot can reach the Smoko objective from two approaches. Carrying and throwing still score once, walls block players consistently, and visible openings and projectile impacts agree with geometry. Check spawning, pickup reach, frame time and non-Smoko maps. Apply the project's peer compatibility policy to changed collision.

## 2. Make menu overviews follow the selected map

**Starting point:** menu.rs::menu_yard_preview currently reacts to feature changes and creates a generic Yard. preview.rs owns the separate rotating character portrait. Trace the actual round-start map resolution and the menu background camera first.

**Sol 6.1 High:** Define a single mapping from mode/settings to environment identity. Solo uses local selection. The host owns online lobby selection, and joined guests display it. Decide how lobby updates reach guests before the round starts.

**Luna:** Reuse the chosen map's environment and give each distinct layout a readable overview camera. Keep the portrait and customisation controls working. Update on setup entry, saved selection, host selection changes and return from a match. Reuse the current scene when only the rules change on a shared map. On fast changes, discard stale loads and remove superseded scene roots. Keep preview simulation passive.

**Done when:** Cycle Backyard → Smoko → Race → Backyard quickly, check Teams and Heist, reopen setup, then start a round. The last preview and launched map agree. Repeat through With mates with a connected guest. Capture the menu transitions and verify repeated switching does not accumulate worlds or cameras.

## 3. Two throwable slots with predictable pickups and catches

**Starting point:** hands.rs::Slots already enforces a two-item capacity. Slots::add selects the arriving item. Full-hand catch handling in game.rs currently evicts the oldest item for both the player and bots. Trace the equivalent host/guest and round-specific paths before editing.

**Sol 6.1 High:** Specify one authoritative acquisition decision. Empty hands equip the incoming item. With one held item, retain the active item and put the new one in reserve. Proposed full-hand rule: refuse ground pickups, and on a valid catch drop the incoming item at the catcher’s feet, awarding normal catch credit once. The dropped item should not immediately re-enter the catcher’s inventory.

**Luna:** Apply that decision to world ownership and slots together. Preserve Q switching, throwing, manual drop, held visuals and HUD order. Keep dedicated Race supply and Heist objective handling consistent with their existing rules. Test duplicate item IDs, two same-tick arrivals, failed pickup, full catch and active-slot promotion after throwing. On disconnect or round restart verify held-item ownership clears.

**Done when:** Exercise pickups and catches with zero, one and two held items, including a browser guest. After every case host and guest agree which item is equipped, which is in reserve and what remains in the world. No third item or duplicate catch score is possible.

## 4. Follow racers for the entire oval from the sideline

**Working interpretation:** The sideline thrower physically runs alongside the racers. The feature is complete when a thrower can follow the field through both straights and both bends and keep aiming.

**Starting point:** race.rs::constrain_throwers clamps the angular position to −0.85…+0.85 radians and the radial position to a band. terrace() supplies positions on an ellipse. Inspect the rendered track, playable fence, runner checkpoints and host validation before choosing the full-lap boundary.

**Sol 6.1 High:** Choose a continuous legal sideline derived from the actual layout. Check clearance around the whole lap, angle wraparound and behaviour at the inner and outer edges. Remove the sector restriction together with the artificial end of the visible route. Keep runner progress, grass penalties and thrower restrictions distinct.

**Luna:** Align terrace visuals, spawns and existing supply with that route. Test joining in progress, recovery and movement across the former sector ends. Check thrower bots can still target and resolve hits, and racer bots can finish. Preserve streaks, upgrades and both standings.

**Done when:** A browser guest can follow a racer for a full lap in either direction around the sideline without snapping back, entering the racing lane or crossing an invisible barrier. Record a short demonstration at the old limit plus a logged full-lap test. Verify a legal scoring throw from the far bend.

## 5. Make a player's gaze readable to other players

**Research gate — Astra Medium:** Trace local camera pitch/yaw, the character hierarchy, existing pose composition and remote interpolation. net.rs::PlayerState includes yaw but no pitch field. Check whether another replicated field already carries usable look elevation. Compare head/face-group rotation with a small upper-body assist on the available blob rigs. Return a short recommendation with affected files, expected wire cost and a three-pose prototype viewed from a second client.

**Sol 6.1 High:** Review the research before implementing. If adding pitch is justified, use a bounded value in the existing pose snapshot cadence, with neutral defaults, version compatibility and smooth interpolation. Keep cosmetic gaze independent of hitboxes, aim and throw authority. Use existing left/right facing and controls.

**Luna:** Implement the agreed visual pose for level, up and down views. Try minimum/default/maximum face and leg sliders. Blend with throw charge/recovery, hit reactions, carrying and get-up, and reset on spawn. Check facing during yaw wraparound and when packets are late.

**Done when:** From another player's browser, a tester can distinguish looking at their feet, looking level and looking overhead without an explanatory HUD. Capture the same avatar at all three angles, then turning and throwing. Verify shirt names, held objects, targeting and first-person camera behaviour.

## Local test entry

From the rust directory run ~/.cargo/bin/cargo run -p bbq_app --release -- --menu. Select Solo game for local checks or With mates for host/guest verification. Keep online publication separate from private browser testing.
