# Original identity and player-made moments

## Which strengths can make Australian BBQ feel distinct?

### Takeaway
The strongest identity is the social backyard setting plus throw-and-carry play: make ordinary backyard rituals drive the funny choices. Comparable games often use hazardous stages, physics, character customization, and many modes; Australian BBQ can stand apart by making the BBQ, esky, shelter, ute, picnic shade and Dazza form one recognizable place with mode-specific reasons to move through it.

### Cited Findings
- Boneloaf describes *Gang Beasts* as slapstick fighting in hazardous environments and supports both melee and gang modes. Its origin story mentions cartoons and multiplayer arcade games, showing how a simple character identity and a specific comic world can anchor the pitch. [Gang Beasts press pack](https://gangbeasts.game/press-pack)
- Flashbulb's *Rubber Bandits* combines brawling with stealing, scavenging, destructible levels, traps, weapons and several objective variants. Its Steam page frames these together as the central party loop, rather than only as a roster of items. [Rubber Bandits on Steam](https://store.steampowered.com/app/1206610/RubberBandits/)
- *Totally Reliable Delivery Service* explicitly presents an interactive sandbox full of toys, vehicles and machines for both work and play. The world itself is meant to invite distraction, alongside the delivery objective. [Totally Reliable official site](https://www.totallyreliable.com/)

### Inferences
- Treat the backyard as the game's signature character: each arena should tell a local story before the first fight (Dazza cooking, chairs dragged aside, ute parked at the servo, teddies hidden in a maze) and then let throws and carries disrupt that story.
- Prefer a few memorable interactions attached to recognizable backyard objects over adding many weapon types. Example: make the shelter, esky and picnic table matter to routes and scoring, with clear visual cause and effect.
- Preserve the current rule constraints: no trick-shot bonuses, wall-bounce bonuses, new voice lines, or changes to blob size/speed parity. Build comedy from pose, timing, object motion and the existing Dazza chatter schedule.

### Gaps
- Public game descriptions do not reveal which individual features drive retention or player enjoyment. These are design inferences from official feature descriptions, not measured causal claims.

## What kinds of replayable arena moments are worth borrowing in spirit?

### Takeaway
The useful pattern in successful comparators is that a map creates one clear decision loop or interaction that changes the fight. A small, legible set-piece can generate stories without turning the whole arena into an unpredictable hazard course.

### Cited Findings
- *Gang Beasts*' Crane stage adds a controllable crane cockpit and an escalating bird threat that can knock players out of the arena; Boneloaf describes the systems and escalating danger directly. [Crane update](https://gangbeasts.game/posts/2024/11/19/is-it-a-bird-is-it-a-crane-update-125-crane-stage)
- Party Animals' developer map log separated maps by mode and describes round rules with distinct team goals: retrieve special items, take objects to a base, or survive. A blizzard arena uses shrinking safe heat from campfires; a train score map asks teams to carry coal to the engine and avoid pulling the brake. [Party Animals developer log](https://store.steampowered.com/news/posts/?enddate=1656470308&feed=steam_community_announcements)
- *Overcooked! 2*'s official FAQ describes dynamic kitchens that can change layout and environment while a timed co-op task continues. It also added throwing food as a deliberate team mechanic. [Team17 FAQ](https://www.team17.com/news/overcooked-2-faq)
- *Human: Fall Flat* advertises multiple routes and playful puzzles in each themed level; its store page also documents community-built levels through Steam Workshop. [Human: Fall Flat on Steam](https://store.steampowered.com/app/477160/Human_Fall_Flat/)

### Inferences
- Add authored “backyard set-pieces” one at a time: e.g., a sprinkler that visibly sweeps a marked strip of ground and briefly changes footing; or a trolley/ute tray interaction that creates a useful route. Keep each hazard predictable, slow enough to read, and avoid random instant knockouts.
- Build mode-specific navigation: FFA should keep open lanes and quick pickups; Teams should have visibly distinct approach paths; Teddy Heist should have a maze of alternate lines of sight, short-cuts and cover. The requested laser-tag style Heist maze already fits this identity.
- Consider a warm-up or short between-round map vote only after online host flow is solid. Party Animals explicitly uses map voting after matchmaking and separates map collections by objective type; for this small host-led game, a compact host-selected rotation may be simpler and friendlier than a public matchmaking system.
- Use predictable variation rather than random physics: e.g., choose one of two pre-authored BBQ-yard layouts per round, or rotate a clearly signaled environmental phase. Avoid making the players' first defeat depend on a hidden hazard.

### Gaps
- The Party Animals developer log is an official Steam announcement but dates from 2022 and reports planned/then-current design, not current production stats. It is useful as an example of stated design reasoning, not evidence of present player preference.

## Which original Australian BBQ ideas fit the current project and constraints?

### Takeaway
For near-term originality, make existing actions and props tell more of the game's story; for medium-term work, give each map/mode a distinct prop-driven objective beat. Reserve new modes, persistent progression and creator tools for later, because the project is desktop-first and still building out one reviewable graphics/gameplay slice at a time.

### Cited Findings
- The local project instructions describe FFA, Teams and Teddy Heist, thrown objects, two item slots, right-click catching, carrying/throwing downed players, Dazza, a bar/drunk meter, and the Bush Servo as current foundations. They also reserve a one-second stun grace, prohibit stacking stuns, and say Teddy Heist scores only by banking teddies. These requirements are in the repo's `AGENTS.md` and mode implementation (`rust/crates/bbq_core/src/heist.rs`, `rust/crates/bbq_app/src/round.rs`).
- The concept-art backlog prioritizes keeping open play routes, meaningful prop clusters, dusty ground leading toward the shelter, readable filtered gum shade and material details that avoid tiny high-frequency visual noise. (`CONCEPT_ART_GRAPHICS_BACKLOG.md` in the project.)
- The project's asset rules require Steam-safe licenses (CC0, CC-BY, Pixabay, ZapSplat; no NC/ND) and a license list; it prefers existing assets/lightweight authored geometry and the Rust desktop game while the browser game remains separate. (`AGENTS.md` in the project.)

### Inferences

**Near-term, low-risk**
- Add a “backyard tells on you” visual layer: scuffed foot paths toward the BBQ and esky, a few loose chair cushions/thongs/bottle caps at the arena edges, and simple impact dust/wet marks that quickly fade. Keep clutter outside the throw lane and central Heist routes; existing backlog already emphasizes open fight space.
- Make prop reactions reinforce actions: sign or hanging bunting sways after a nearby impact, esky lid visibly settles after interaction, the shelter sign/score marker updates on a bank, the sprinkler gives a bright water arc before its ground patch gets slick. Use compact visual cues and existing objects first; don't add voice or surprise damage.
- Give Dazza a small set of clear silent reactions tied to real events—turning to a big hit, celebrating a Teddy bank, shielding a snag when a brawl reaches the BBQ—while retaining his current spoken-chatter cadence. Keep these cosmetic until tested; don't make the cook an untelegraphed extra attacker.
- Add round-end “party stats” cards with playful, non-competitive labels such as longest carry, most catches, teddy deliveries, or best recovery. Avoid an MVP-only system so new players and low scorers still get a laugh.

**Medium systems**
- Create one original map interaction that uses existing carry/throw rules: an oversize esky/ice run, where teams carry a visible bag of ice to their esky; or a snag relay where a team passes an oversized sausage to the BBQ. This is an optional new mode idea, not a replacement for Teddy Heist. Clearly distinguish it from flag capture by its setting, prop animation and short teamwork loop.
- Add small, selected arena variants instead of random global chaos: “Sprinkler on” changes one safe lane; “Windy clothesline” moves the visual clothes but never steals player control; “Cook's break” briefly changes Dazza's pose or location with clear warning. One variant per round is enough to make rematches feel different.
- Use mode-specific arena dressing and lighting: Teddy Heist gets high-contrast laser-tag-like panels and clear base landmarks; Teams gets color-coded table flags; Smoko shows warm cooking lights and the ute/servo as a readable destination. Keep team colors distinct under shade and avoid relying on color alone (use shape/icon too).

**Bold, later options**
- A short “Backyard Party” series: 3–5 fast rounds mixing FFA, Teams and objective rounds, with host-picked mode/map cards and aggregate party stats. Retain solo/bot and 1v1 viability; don't require matchmaking.
- A modular yard-night event (late-afternoon to dusk) with gradual, predictable lighting changes, fairy lights and BBQ glow. This could make the arena feel like a complete backyard gathering, but only after item/player readability and performance are validated.
- A simple arena editor or prop-placement preset mode could deepen replayability, but it has a much larger UI, collision, online synchronization and support cost. Treat this as a distant option, not the next slice.

### Gaps
- No playtest or telemetry data was available to rank these ideas by actual player retention or social laughter. The priority ranking reflects project fit, scope and comparators, and should be validated with local matches.
- The research did not verify licensing for any proposed third-party sounds, models or textures. The safest suggestions above use authored code, existing assets and Steam-safe sources already listed in project policy.
