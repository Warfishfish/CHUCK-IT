# Step 2c: Sun-Baked Cartoon Realism (graphics checklist)

Added 5 Oct 2026 (Marcus). This is its own list to work from, one item at a time, highest impact first. `RUST_REWRITE_PLAN.md` (Phase 2c) points here. The JavaScript game is not touched: this look is for the Rust version only.

**The look in one line:** simple cartoon geometry + exaggerated, slightly wonky proportions + sun-baked Australian colours + surprisingly rich surface detail. Not photoreal, not clean, not generic low-poly. Readable first.

## How we work (Marcus, 5 Oct 2026)
- ONE item at a time, in phase order. For each: find the current code first, reuse it, change only that item, keep gameplay as is, build and test, send Marcus before/after screenshots, tick only that item, then STOP. Commit and push only after Marcus says it looks right.
- Item numbers (A1.2 and so on) are the handles to use: "do A2.3".
- Direction (Marcus's prompt): **Sun-Baked Cartoon Realism**: a simple cartoon silhouette from a distance, surprising material and surface detail up close. Stylised, not photoreal. Tiny Glade and Foxtrot are inspiration only (environment richness, soft light, grounded materials), never a look to copy. Keep the lawn and playable areas readable; do not fill every space with clutter.

## What exists now (inspection, updated 5 Oct 2026)
- **Engine:** Bevy 0.19.1, `StandardMaterial`. Used: base colour, roughness, texture, vertex colours (the lawn), emissive. Available but not used yet: normal maps, metallic, occlusion maps. Two looks: `--look browser` (matches the browser game for the compare toolkit) and `polished` (everything in this list).
- **Lighting and screen:** one sun with a 4096 shadow map and Gaussian filtering, a weak cool rim light, an ambient light from the sky/ground colours, SSAO with SMAA (not MSAA), colour grading and a vignette in `display_raw.wgsl`, distance fog. Tone mapping is off on purpose. Bloom was tried and removed (it flickered the sky).
- **Meshes:** almost all built in code from shape lists (`bbq_core/src/looks.rs`, `looks_yard.rs`), with `sun_bake` and `texture_pass` giving colour and textures. Blender only makes the four blobs (`tools/make_blob.py`, with the belly shape keys). Models load as GLB; the blob parts are named (Torso, Head, EyeL...) and the game finds them by name.
- **Textures:** 18 browser pictures plus generated ones from `tools/make_textures.py`: wood, metal, plastic, concrete, bark, three decal shapes, a particle dot, and normal maps for wood, metal, plastic, concrete and bark (the normal maps are not used yet). Laid by real size with `Surface::material`.
- **Also in place:** forward decals (`decals.rs`), GPU particles with `bevy_hanabi` (`fx.rs`), a character preview camera (`preview.rs`). Not in place: LODs, instancing, a Blender hero-model path for props.
- **Limits found:** SSAO needs MSAA off; decals need a depth pre-pass; shapes built in code have no bevels; no skeletal animation (characters are animated by moving named parts); frame time on Marcus's Mac has not been measured.

**Decisions made while merging the new brief (change me if wrong)**
1. Game rules and hit sizes do not change. Character looks change only (Marcus's 4 Oct rule: the four characters differ in looks, not speed or hit sizes). The big feet and thongs are drawn bigger than the collision, never used for it.
2. Classic stays the faithful original capsule (decided 4 Oct). The caricature details (big feet, thongs, singlet, tummy, asymmetry) go onto Pear, Egg and Gumdrop first; Classic gets only the eye bob and shading.
3. The decorative eskies change colour so the Chest is unique. The Chest stays Cheeky-mode only.
4. The "veranda" and "worn paths" do not exist yet; they are new geometry/texture and must not block walking (colliders stay as they are or get checked).
5. Colour emoji, the browser look and the JS game are unchanged.

## Checklist

### Style direction (decided by Marcus before the phases go further)
- [x] S1. Two upgrade examples compared (6 Oct 2026): "Cartoon pop" (outlines, vivid colours, big-headed characters) and "Dusty clay" (no outlines, squat clay toys, golden light). Marcus chose Pop with clay's warm light and thinner outlines
- [x] S2. The game's overall style: **Pop with clay's warm golden light** is now the polished look (`style.rs`, `--style pop|clay|current` to switch): outlines on the "hero" things only (characters, Dazza, items, magpie, BBQ and bar area, table, chairs, eskies; not fence, trees, ground or house), vivid colours, a low strong golden sun with cool shade, a warm hazy horizon, big-headed big-handed big-footed characters (models `blob_*_pop.glb`, made by `make_blob.py --style pop`), a stronger belly flop. Outlines thinned twice at Marcus's request. The browser look is unchanged. Dazza's outline is complete (body, head, hat brim, corks, hands, spatula). Still open: a subtle clay skin texture on Pop characters is optional

## PHASE A: Environment foundation

### A1. Ground and grass
- [x] A1.1 Lawn with dry yellow grass, stubborn green patches and uneven coverage, mow stripes still visible (a 1 m grid of vertex colours over the lawn tile, `PatchyLawn` + `lawn_tint`; polished look only)
- [x] A1.2 Worn pathways from the back door to the BBQ, bar and Hills Hoist, and dirt round the BBQ and bar (`dirt_spots`)
- [x] A1.3 Dirt patches placed naturally: clusters of overlapping soft ellipses in different earthy tones, along paths and where people stand, creeping in from the fences
- [x] A1.4 Weeds and dry tufts: 170 tufts along the fences and about
- [x] A1.5 Instance the tufts and weeds: parts with the same shape and surface now share one mesh and one material (cache in `models.rs`), tuft blades are one shared cone scaled per blade. Materials 3,129 to 373, meshes 3,166 to 1,583 (`--stats`); frame time still to measure (E18.4)
- [x] A1.6 Fallen leaves and bark bits under the gum trees and along the fence lines (about 170 leaves and some bark strips along the fences, a light scatter on the lawn, a few under each tree; flat, tiny, no shadow, clear of the pool and smoko pad; Marcus asked for fewer leaves)
- [ ] A1.7 (on hold, Marcus 6 Oct 2026: the ground already has enough going on) A few small bits of backyard debris on the lawn (bottle caps and stubbies are in; add sticks, a crushed can, a lid). Also bring back the four empty yard eskies that open onto nothing: switch them on again with `bbq_core::yard::set_empty_eskies(true)` in `main.rs`; they are red, green, orange and grey, openable with R, and could hold other things later
- [x] A1.8 Paddock and far trees get dry grass and a dusty colour, still very simple (dry khaki paddock with dust patches, far trees dusted, 22 extra far trees; plus a neighbourhood: 12 simple neighbour houses round the yard, visual only)
- [x] A1.9 Check the ground still reads clearly for moving and for seeing items (checked by screenshots and a colour check: every item differs in hue from the lawn, but the teddy, steak, gnome and purple dildo are about as bright as the lawn, so they lean on A3.8 (stronger item colours). Marcus to judge on his Mac)

### A2. Lighting
- [x] A2.1 Warm sun, cooler shadows, a stronger sun and slightly lower ambient for depth (first pass; tune on Marcus's Mac)
- [x] A2.2 Shadows: 4096 shadow map and Gaussian filter in the polished look
- [x] A2.3 A rim or back light so characters separate from the lawn
- [x] A2.4 Ambient occlusion so things sit on the ground (SSAO, subtle, strongest under the BBQ and in creases)
- [x] A2.5 Tune the shadow cascades so near shadows are crisp and far tree shadows still reach (4 cascades, first one ends at 8 m, last reaches 110 m; the difference is small)
- [ ] A2.6 Check tree shadows on the lawn and fence; thicken or soften them if they look patchy
- [ ] A2.7 Under the shed roof and under the veranda get a darker, cooler feel
- [ ] A2.8 Tune the whole afternoon feel on Marcus's Mac (sun angle, warmth, how dark the shadows are), still clearly daytime, not orange

### A3. Materials
- [x] A3.1 Generated textures: wood, worn metal, sun-faded plastic, cracked concrete, laid by real size (`make_textures.py`, `texture_pass`)
- [x] A3.2 Wood on the table, fence posts, crates, bar and veranda; metal on the BBQ and shed roof; plastic on the chairs and eskies; a concrete pad
- [x] A3.3 Use the normal maps that are already made (`wood_n`, `metal_n`, `plastic_n`, `concrete_n`, `bark_n`) on their materials, polished look only
- [x] A3.4 Roughness per material instead of one matt value (polished look: wood, concrete, bark and cloth matt, plastic in between, metal shinier). Metallic stays 0 because the game has no sky reflections for metal to show
- [x] A3.5 Roughness and ambient occlusion maps for wood and metal (`wood_orm.png`, `metal_orm.png`, from `make_textures.py`)
- [x] A3.6 A fabric material (`fabric.png`, `fabric_n.png`): washing, umbrella cloth, trampoline mat, pool towels. The singlet is still to do under C11.3 (the character models need their own UVs)
- [x] A3.7 Scratches and wear: tyres in the plastic material, trampoline mat in fabric and legs in metal (chairs, bar and BBQ already had materials). The shed is still plain corrugated metal
- [x] A3.8 Stronger colours on gameplay objects: items first got 30% more colour, then (6 Oct 2026, Marcus: too much highlighting) 22% LESS colour and a thin, soft brown outline instead of the full black one; the bins a stronger green (the Chest and sauce bottles were already bold)

### A4. Environmental integration
- [x] A4.1 Decals: grease and a scorch mark by the BBQ, a beer spill by the bar, meat drips, a wet edge round the pool, bird droppings under the magpies, grime at the back door, and fading beer stains where a VP can bursts
- [x] A4.2 Fence: dirt and weeds at the base of the posts and boards, and darker ground where the fence shades it
- [x] A4.3 BBQ area: flattened grass, more grease and sauce stains, food scraps and bottle caps close to the BBQ and table
- [x] A4.4 Tyres: compressed grass and dirt under them, a few weeds round the stack, a soft contact shadow
- [x] A4.5 Pool: worn grass at the edges, a wetter darker ground band all round (more than the two decals now), towels and thongs left on the edge
- [x] A4.6 A soft blob shadow under props that SSAO misses (the shed, bins, bar legs)

## PHASE B: World character

### B5. Trees
- [x] B5.1 Gum trees with pale streaky peeling bark, a bent trunk with a root flare, 2 or 3 forking limbs, airy crowns of many small clumps in sun-bleached greens, the odd dead twig; tall-and-spindly to short-and-spreading
- [ ] B5.2 Leaning trunks and a few split or double trunks
- [ ] B5.3 One or two slightly dead trees (bare limbs, a few leaf clumps)
- [ ] B5.4 More variety in trunk width and crown shape (a wide spreading one, a thin whippy one)
- [ ] B5.5 Far trees fade slightly into the haze (ties to B8.3)

### B6. Fence
- [x] B6.1 Fence leans a little, in 3 m sections of slightly different heights and shades with fat posts (visual only; the collider is still straight)
- [ ] B6.2 Variation inside a section: boards of slightly different height, width and shade (subtle, not every board)
- [ ] B6.3 Slight warping and leaning posts
- [ ] B6.4 Stains, knots and weathering on the paling texture
- [ ] B6.5 A few damaged boards: one missing, one broken, one hanging
- [ ] B6.6 A gap or two the magpies can look through (visual only)

### B7. Detail clusters
- [x] B7.1 Small clutter that tells a story: cricket stumps and bat, dog bowl and bone, thongs by the back door, washing basket and sock, stubbies and bottle caps
- [x] B7.2 BBQ area: BBQ, outdoor table, chairs, sauce and mustard bottles, esky, food (the BBQ cluster's rubbish is in A4.3)
- [ ] B7.3 Pool area: towels, a pool umbrella, chairs, pool toys, thongs, wet ground (with A4.5)
- [ ] B7.4 Shed and junk area: tyres, tools, a mower, timber offcuts, oil stains, old junk
- [ ] B7.5 Clothesline area: washing and pegs on the hoist, a peg basket, worn dirt right under it
- [ ] B7.6 Fence areas: the wheelie bins (a green one), old furniture, stacked timber, small rubbish
- [ ] B7.7 A hose and a sprinkler on the lawn
- [ ] B7.8 Keep the middle of the yard open: check each cluster stays out of the walking and throwing space

### B8. Sky and atmosphere
- [x] B8.1 Sky and fog a little warmer near the horizon (fog 0xc9e1e4 in the polished look)
- [ ] B8.2 Sky gradient: deeper blue overhead, paler and warmer at the horizon
- [ ] B8.3 Distance haze: far things lose contrast and fade into the horizon colour
- [ ] B8.4 Cloud variety: different sizes, some thin and streaky, slow drift (clouds stay faceted and simple)
- [ ] B8.5 Separate the foreground from the background (slightly cooler, hazier far trees and fence)
- [ ] B8.6 Keep the stylised sky: no photographic sky box

## PHASE C: Characters

### C9. Proportions
- [x] C9.1 Slightly oversized heads, larger hands, noticeably large feet (visual only, collision unchanged)
- [x] C9.2 Oversized chunky thongs in a colour each, with a strap; slightly exaggerated stomachs
- [x] C9.3 Beer bellies are part of the body (shape keys), full and round, with a "Beer belly" slider in the menu
- [ ] C9.4 Slight asymmetry: one shoulder lower, a wonky grin (head tilt is done)
- [x] C9.4b Classic updated (6 Oct 2026): same caricature as the others (bigger head, hands, feet, thongs, singlet, belly), its Beer belly slider now works; also fixed `--char` and the saved pick not reaching the on-screen characters
- [ ] C9.5 Clearly different silhouettes between the four characters
- [ ] C9.6 Skinny lower legs (the blobs have no legs, so this needs a pose change; decide with Marcus first)
- [ ] C9.7 Walk, throw, stunned and fall poses re-checked with the bigger feet and hands (no clipping)

### C10. Secondary animation
- [x] C10.1 Eyeballs bounce and bob while walking and running, each character in its own style (springs, footstep kicks, jolts), pupils riding on them; strength set to 60%
- [x] C10.2 Belly bounce on every step, squash on landing, swing when speeding up or stopping
- [x] C10.3 Feet step as the blob walks
- [ ] C10.4 Eyes lag a little in fast turns
- [ ] C10.5 Body leans forward when speeding up and back when stopping
- [ ] C10.6 Exaggerated arm swing when walking and running
- [ ] C10.7 Thong flap: the thongs lift and slap with each step
- [ ] C10.8 Controlled squash and stretch on strong impacts (a big hit, a hard landing)
- [ ] C10.9 Simple expressive faces: brows and a mouth that changes (stunned, drinking, throwing, drunk)

### C11. Character materials and looks
- [x] C11.1 Singlet over the tummy with folds and a wavy hem (Pear, Egg, Gumdrop)
- [x] C11.2 Dazza: sunburnt skin, cork hat, sunnies, moustache, belly, apron, stubby shorts, stubby holder, thongs
- [ ] C11.3 Singlet in the fabric material (A3.6), and shorts or stubbies for the blobs
- [ ] C11.4 A cap or hat on some characters
- [ ] C11.5 Skin and body roughness (a soft sheen, a little sunburn on the nose and shoulders)
- [ ] C11.6 Leader crown, stars, sash and name tag still read clearly on the new bodies

## PHASE D: Hero assets

### D12. BBQ
- [x] D12.1 Oversized uneven wheels, big rounded red hood propped open, stubby knobs, slight asymmetry
- [x] D12.2 Surface wear: grease and burn marks on the plate, heat staining, bolts, a weld seam, rust patches
- [ ] D12.3 Grill bars you can see, hood hinges, a proper handle, vents
- [ ] D12.4 Faded stickers and food residue
- [ ] D12.5 A Blender hero version with baked normal and AO (with E17)

### D13. Dildo Chest (the blue esky)
- [x] D13.1 Rebuilt as a big, chunky, bright-blue esky with chunky hinges, a large handle, an exaggerated lid that still opens as before
- [x] D13.2 Every esky in the yard was openable and only one held the toys; the other four (red, green, orange, grey) are now switched OFF in the polished look (picture, collider and R prompt) until A1.7
- [x] D13.3 The toys in the Chest look larger (0.9)
- [ ] D13.4 Surface: scratches, faded plastic, dirt, stickers, worn corners, slight discoloration
- [ ] D13.5 Check it reads against grass, dirt, the shed and the fence, in shadow and in sun, at all 9 spots
- [ ] D13.6 A Blender hero version (with E17)

### D14. Interactive props
- [x] D14.1 Table, chairs, sauce and mustard bottles (see A3 and B7 for the surface work)
- [x] D14.2 Items get a second look: teddy seam, patch and bow tie; gnome face and belt; steak and snag grill marks
- [x] D14.3 Dildos: floppy like the browser, bigger in the chest, more surface texture. Noodles: floppy, longer, slap works
- [ ] D14.4 Cheeky-mode bottles: a glass beer bottle and a spirit bottle (labels, see-through glass with liquid, cap or cork); gameplay is in CHECKLIST.md
- [ ] D14.5 Smash effect for those bottles: glass shards, a splash and a stain decal
- [ ] D14.6 Food: a slightly oversized snag and steak with real-looking texture, the fish
- [ ] D14.7 Shed, bar, trampoline, planters, log pile, tank: bevels, a slight lean, dirt, rust; springs on the trampoline, taps and bottles on the bar
- [ ] D14.8 Hills Hoist: pegs, a towel and washing (with B7.5)

### D15. Animals
- [x] D15.1 A magpie on the fence watching the player: big head, big beak, chunky body, head that tracks you and cocks now and then (two spots)
- [ ] D15.2 Magpie extras: hops along the fence, flaps off if something flies close (visual only)
- [ ] D15.3 A Blender hero magpie
- [ ] D15.4 (Later, optional) a dog, a lizard or a kookaburra on the shed

## PHASE E: Technical polish

### E16. Rendering
- [x] E16.1 The look switch (`--look browser|polished`), colour grading and a vignette, SSAO with SMAA, anisotropy 8, and a `--fx off` switch
- [x] E16.2 GPU particles with `bevy_hanabi`: hit stars and puff, beer foam, whack, pool splash, fall dust, BBQ smoke
- [x] E16.3 Flicker fixes: bloom removed (it flickered the sky), umbrella surfaces separated, smoko pad lifted above dirt
- [ ] E16.4 More particles: sprint dust, trampoline puffs, pool steam, a spray when a bot stacks it, confetti on the results card
- [ ] E16.5 Bloom, only if it comes back with a very strict threshold and a flicker test
- [ ] E16.6 Check SMAA against MSAA edges on Marcus's Mac and pick one
- [ ] E16.7 A Low / High graphics option in the Phase 8 settings (turns off SSAO, particles, decals, the 4096 shadows)

### E17. Blender pipeline
- [ ] E17.1 One script per asset in `tools/` (`make_esky.py`, `make_bbq.py`, `make_magpie.py`...), run with `blender --background --python`, each writing a `.glb` into `assets/models/`; a `tools/README` line for the order
- [ ] E17.2 Bevel modifier, weighted normals, shade smooth and UVs on every export
- [ ] E17.3 High-poly to low-poly bake of normal and AO for the hero props, 512 to 1024 px
- [ ] E17.4 Slight controlled mesh deformation and asymmetry in the scripts (seeded, so a rebuild gives the same wonk)
- [ ] E17.5 The game loads a hero model and falls back to the shape-list version if the file is missing
- [ ] E17.6 Shape builder in code: a bevel option on cuboids and cylinders, and a small seeded tilt and size change so nothing is mathematically straight
- [ ] E17.7 Vertex-colour variation on shapes (a faded top, a darker bottom edge)
- [ ] E17.8 Tests that props still sit inside their colliders after any new shape

### E18. LOD and optimisation
- [ ] E18.1 A low version of each hero model for far away, swapped by distance
- [ ] E18.2 Share materials between parts; merge static background parts into fewer meshes
- [ ] E18.3 Keep textures at 128 to 512 px (1024 only for the BBQ and esky); no 4K
- [ ] E18.4 Measure frame time on Marcus's Mac (11 bots, Heist arena, Cheeky mode, effects on and off); keep 60 fps

### E19. Final visual consistency pass
- [ ] E19.1 Walk the whole yard for anything that looks off-style (too clean, too glossy, too straight, too detailed)
- [ ] E19.2 One palette pass so the sun-baked colours match everywhere
- [ ] E19.3 Before and after screenshots of the yard, the BBQ, the Chest, the magpie and a blob close-up for Marcus to approve
- [ ] E19.4 Tick the Phase 2c look check on Marcus's Mac in `RUST_REWRITE_PLAN.md`

## Notes
- Items done before this reorganisation keep their `[x]` and their wording; nothing was removed.
- Normal maps are made but not used until A3.3. The flicker findings are recorded in E16.3.

## Fixes from play (6 Oct 2026)
- [x] Toys in the chest no longer poke through the lid (lid opens wider, toys sit lower and tilt back).
- [x] G throws away what you are holding (dance moved to H). You cannot grab it straight back for 2.5 s.
- [x] Dildo and noodle flop is back to the browser game's own spring numbers for everyone, first person included (a physics-chain version was tried twice and only squiggled at the tips; a copy is kept outside the repo). More natural flop, if wanted, would start from these numbers
- [x] Dazza walk glitch: his walk cycle was spinning about 19 times a second (30 rad per metre instead of 2.4), and his drawn position now eases after the 60 Hz brain so it does not judder
- [x] The BBQ is turned round so the knobs, hood handle and open side face Dazza (he stands behind it, on the fence side)
- [x] Dildo skins (6 Oct 2026): six random surfaces per dildo (studded, smooth, ribbed, rilled, spiral, veiny); tiny skin details get no outline (`DildoSkin` in `looks.rs`)
