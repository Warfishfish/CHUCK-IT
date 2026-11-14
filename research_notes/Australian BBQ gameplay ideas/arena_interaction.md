# Environmental gameplay and arena interaction ideas

## What similar games make their environments do

### Takeaway
The strongest transferable lesson is to give a small number of themed objects a clear, readable purpose, then let players use them socially. A distinct arena mechanic can make a familiar map memorable; it does not need to make every prop physical or every round unpredictable.

### Cited Findings
- Totally Reliable Delivery Service describes its setting as an interactive sandbox world, with toys, vehicles and machines that can be used for either work or play. Its official site: [Totally Reliable Delivery Service](https://www.totallyreliable.com/).
- TRDS has a competitive trampoline footrace across hotel rooftops, and its update notes explicitly mention adjusting trampoline physics. It also adds seasonal world dressing and limited-time fireworks. This is evidence for pairing one movement toy with a clear activity, while tuning its behaviour and using visuals to refresh a place: [TRDS PC developer blog, v2.03.03](https://www.totallyreliable.com/developer-blog/categories/pc) and [v2.02.02](https://www.totallyreliable.com/developer-blog/categories/pc).
- Gang Beasts' Trawler stage combines a setting that deteriorates over time, sharks, two controllable crane systems and a detachable shark cage. Crane later added a player-drivable crane and time-escalating bird attacks. The maps make one or two local features the centre of the round: [Gang Beasts official announcements](https://steamcommunity.com/app/285900/announcements/).
- Human: Fall Flat's official Steam description emphasizes several routes through a level and physics puzzles with objects such as a boulder, catapult and breakable wall. This favors spatial choices and collaborative object use over a single prescribed path: [Human: Fall Flat on Steam](https://store.steampowered.com/app/477160/Human_Fall_Flat/).
- Ultimate Chicken Horse turns map construction into a social strategy: players take turns placing platforms and hazards, then attempt the course; players can score for succeeding where rivals fail. That creates counterplay because the same hazards affect the builders too: [Ultimate Chicken Horse, official publisher page](https://noodlecake.com/games/ultimate-chicken-horse/).
- Party Animals has added small interactables to an existing casual map, including beanbags and boats in its Autumn Cabin update. This demonstrates that a map can gain activity through a couple of low-stakes props without rebuilding its main play space: [Party Animals official announcements](https://steamcommunity.com/app/1260320/allnews/).

### Inferences
- For Australian BBQ, start with one or two backyard objects that prompt player choice or comic cooperation; avoid a large inventory of random traps. A recognizable visual cue (lid, handle, spray arc, painted route) can explain the affordance before a player presses a button.
- Use arena-specific interactions, so the same core throw-and-catch game feels different in the yard, the Smoko circuit and Teddy Heist. The interaction should be placed in a symmetric or neutral position and should give players time to react.
- A timed escalation can create a memorable round, but should be optional or gentle: Gang Beasts demonstrates the dramatic payoff, while its official patch list also includes repeated fixes for stage stability, collision and exploits. Complexity and exploits rise with moving machinery and hazards.

### Gaps
- Public sources describe features but do not establish which interactions players enjoy most or how they affect match balance. The ideas below are design hypotheses; Marcus should choose them through local playtests.

## What fits Australian BBQ's existing maps and rules

### Takeaway
The game already has several strong environmental anchors. Build on the pool, trampoline, BBQ, esky, ute and Heist maze, and use visuals to make their effects easier to read before adding more systemic physics.

### Cited Findings
- The local checklist documents Free for All, Teams, Teddy Heist, Smoko and Race; pool and trampoline bonuses; Dazza and the BBQ; decorative eskies; the drivable/jumpable Smoko ute; and an oval arena road. See [CHECKLIST.md](../../CHECKLIST.md) and the current map definition in [bush_servo.rs](../../rust/crates/bbq_core/src/bush_servo.rs).
- The project notes specify: no trick shots or wall-bounce bonuses; Heist only scores when teddies are banked; character speed and hit sizes stay equal; 1v1 with no bots must work; stuns do not stack, with a 1-second post-stun grace; and core browser gameplay remains separate from the Rust desktop rewrite. See [AGENTS.md](../../AGENTS.md) and [heist.rs](../../rust/crates/bbq_core/src/heist.rs).
- Existing Heist rules use team bases and a central maze, with enemy teddy pick-up and banking at the home base; teddies at a home base do not count as recoveries. See [heist.rs](../../rust/crates/bbq_core/src/heist.rs).
- The visuals backlog asks for clear routes, readable players and thrown objects, and deliberately aligned prop collisions. It also says to keep scenery outside the fence low priority and compare real-game captures after changes: [CONCEPT_ART_GRAPHICS_BACKLOG.md](../../CONCEPT_ART_GRAPHICS_BACKLOG.md).

### Inferences
- Most promising low-risk move: turn existing decorative eskies into readable resupply interactions. The code already shows an “R: open the esky” prompt and the current response says it is empty, so a one-can refill or a clear empty/cooldown animation could reuse an existing interaction point instead of adding a new system. Keep refill count limited and visible; do not award points for opening it, especially in Heist.
- Another low-risk improvement is a visual “play language” pass: make pool splashes, trampoline launches, road edges, Heist routes and thrown-item paths readable at match distance. Add small, mode-specific colour shapes and comic feedback to existing actions, without altering stun times, knockback or score rules.
- For Heist, add visually distinct but symmetrical route identities to its existing maze (e.g., red-earth centre lane, gum-shade lane, crate lane). Keep each route the same length/value and openings visible. This could make callouts and team coordination easier without introducing score-changing hazards.
- A higher-risk follow-up could be a backyard sprinkler station: a player can trigger a short, clearly telegraphed spray across one marked patch. First test it as visual-only (mist, droplets, puddles) and preserve movement values; if it later affects play, keep it away from spawn/bank zones, show the full affected area, use fixed timing, and test host plus guest. It is more thematically specific than a generic mine or spike trap.
- A second original set-piece could be a BBQ “service bell” or grill interaction that briefly draws Dazza's attention to the station, creating a readable diversion in FFA/Teams. Keep it out of Heist scoring and avoid voice/speech; test whether it adds choice rather than an easy Dazza exploit.

### Gaps
- I did not verify exact Rust object APIs or networking replication paths for adding a resupply or sprinkler. Those require a code review before turning the concepts into implementation tasks.
- No accessibility review of the current palette/impact effects was performed. Any new route colours should be paired with shape, pattern or icon differences rather than colour alone.

## Recommended shortlist, ranked for a small team

### Takeaway
Prioritize work that makes existing actions more expressive first, then add one new interaction whose payoff is obvious. Leave complex moving hazards and player-built map editing until the core arena has been playtested.

### Cited Findings
- Human: Fall Flat advertises multiple routes and playful physics puzzles; Ultimate Chicken Horse supports strategic placement of hazards that challenge the builders as well as rivals. Together they support readable route options and fair counterplay, not unavoidable traps: [Human: Fall Flat](https://store.steampowered.com/app/477160/Human_Fall_Flat/) · [Ultimate Chicken Horse](https://noodlecake.com/games/ultimate-chicken-horse/).
- Gang Beasts' official Trawler/Crane notes show how a deteriorating stage, controllable machinery and time-based creature attacks can define an arena, but its patch history also includes stability and collision fixes. Use this as a high-effort reference rather than a first implementation: [Gang Beasts announcements](https://steamcommunity.com/app/285900/announcements/).
- TRDS and Party Animals show that interactive toys, vehicles, boats, trampolines and small comfort props can add play to a themed world; these are precedents for grounded backyard interactions: [TRDS official site and developer blog](https://www.totallyreliable.com/) · [Party Animals updates](https://steamcommunity.com/app/1260320/allnews/).

### Inferences
1. **Make existing actions pop.** Add comic item trails, readable landing/splash/orbit feedback, short impact effects, and throwable silhouettes that still read in shade. Fun/originality is high because it improves existing pool/trampoline play and lets Australian backyard slapstick shine. Keep effects behind player/item silhouettes to protect readability. Effort is low–medium; watch visual clutter and online consistency.
2. **Esky resupply.** Open an existing esky for one clearly visible lemonade/VP can, then show an empty or cooldown state. It makes a familiar prop a tactical decision; lid movement and a contents marker should make it clear. Effort is low–medium because an R interaction scaffold appears to exist, but item ownership, refill limits and replication need checking. Do not award Heist points.
3. **Heist route identity.** Add symmetric visual landmarks and callouts to the maze (gum shade, timber stack, servo crates); later test a clearly marked risky shortcut without a score multiplier. It could improve team communication. Distinguish routes with shape as well as colour. Art-only changes are low–medium effort; changed route geometry needs 2–4 team retesting.
4. **One backyard event button.** A sprinkler makes a short spray cycle over a marked area. Start cosmetic-only, then decide whether wet-ground play is worthwhile. This is specific and could create a shared comic moment. Mark the full footprint and telegraph activation. Gameplay effect raises effort and online risk; keep it away from spawns and bases, avoid random timing, and do not affect Heist scoring.
5. **Dazza diversion.** An optional grill-side interaction draws Dazza toward the station, briefly opening an opportunity to steal a snag or escape. It is distinctive and can be communicated with a clear animation or icon. Medium effort; avoid making Dazza trivially exploitable or changing his escalation too much.
6. **Player-built traps, heavy machinery or destructible arena.** Keep these as future research only. They offer high spectacle but can overpower the throwing game and carry very high griefing, fairness, collision stability and online synchronization risks.

### Gaps
- Effort ratings are relative engineering estimates inferred from the repository and public feature descriptions, not measured estimates from an implementation spike.
- Ranked ideas should be treated as a discussion list, not a commitment to implement all of them.
