# Step 2c: Sun-Baked Cartoon Realism (graphics checklist)

Added 5 Oct 2026 (Marcus). This is its own list to work from, one item at a time, highest impact first. `RUST_REWRITE_PLAN.md` (Phase 2c) points here. The JavaScript game is not touched: this look is for the Rust version only.

**The look in one line:** simple cartoon geometry + exaggerated, slightly wonky proportions + sun-baked Australian colours + surprisingly rich surface detail. Not photoreal, not clean, not generic low-poly. Readable first.

## What the inspection found (5 Oct 2026)

**Engine and rendering**
- Bevy 0.19.1, `StandardMaterial` (PBR: base colour, roughness, metallic, normal map, occlusion are all available; only base colour and a roughness value are used so far). MSAA 4x (Bevy default). No bloom, SSAO, SMAA, tone mapping or colour grading. Distance fog only (75 to 140 m).
- Lighting copies three.js r149 on purpose: hex colours go in as linear, tone mapping off, a small shader (`assets/shaders/display_raw.wgsl`) undoes the sRGB step, one sun with shadow maps (default cascades) plus a flat ambient and a soft top light. That is why browser and Rust screenshots match. A new look must not break that, so there is a **look switch** (`--look browser|polished`); the compare toolkit uses `browser`.
- Textures: 18 small PNGs (64 to 256 px) in `assets/textures/`, colour only, repeating, hand-made mipmaps, anisotropy 4. No roughness, normal or AO maps.
- No decals, no instancing, no LODs.

**Meshes and Blender**
- Almost everything is built from shape lists in code (`bbq_core/src/looks.rs`, `looks_yard.rs`: cuboid, cylinder, sphere, cone, torus, capsule, ico, half-cylinder, ground...), flat-coloured, no bevels, no wobble. Shapes-as-data was chosen so a Blender script can rebuild them later.
- Blender only makes the four blobs (`tools/make_blob.py` -> `assets/models/blob_*.glb`, about 100 to 125 KB each, flat grey, tinted per player by the game). The blob parts are separate (body, head, eyes, pupils, feet, hands), not a skinned skeleton. `/opt/homebrew/bin/blender` is installed; run scripts with `blender --background --python tools/<script>.py` (the `pip install bpy` route is not set up).

**The yard today (and how primitive it is)**
- Lawn: one repeating `lawn.png` over a flat ground plane with mow stripes; a flat green paddock outside. No dry grass, dirt, weeds or paths.
- House: a single weatherboard box with two roof slabs, five windows, a door, a water tank. No veranda, perfectly straight. Fence: four straight paling panels.
- Hills Hoist: pole plus turning head (works), perfectly upright.
- Table, chairs (blue seat, dark frame), bar, BBQ (box with a lid and knobs), shed (box with corrugated texture), trampoline, pool (rectangle with coping), log pile, planters, tyres, cricket-free.
- **Dildo Chest:** a brown wooden box with gold bands and a half-cylinder lid, only shown in Cheeky mode, at one of 9 spots (`CHEST_SPOTS`), lid eases open when stocked. **The four decorative eskies are blue boxes with white lids**, so a blue chest would be confused with them: they must change to other colours (red, white, green) so the bright-blue Chest stays unique.
- No animals at all (no magpie yet). The sky has faceted clouds and faceted gum trees.
- Characters: four selectable blobs (Classic = original capsule; Pear, Egg, Gumdrop = Marcus's shapes). Eyes are static white balls with black pupils; feet are flat dark discs; no clothes, no thongs.

**Stays simple:** sky, clouds, far trees, paddock, pool tiles, signs, the house interior-facing details, bar stools, planter boxes.
**Needs more surface detail (texture, not geometry):** lawn, fence, house, table, chairs, shed, tyres, concrete paving, bar, trampoline.
**Worth a higher-detail model (still cartoon proportions):** the BBQ, the Dildo Chest esky, the Hills Hoist, the character blobs (close in every match), the table and chairs, the bin and sauce bottles, the magpie, the held items (teddy, stubby, gnome, steak, fish, noodle, snag).

**Decisions made while merging the new brief (change me if wrong)**
1. Game rules and hit sizes do not change. Character looks change only (Marcus's 4 Oct rule: the four characters differ in looks, not speed or hit sizes). The big feet and thongs are drawn bigger than the collision, never used for it.
2. Classic stays the faithful original capsule (decided 4 Oct). The caricature details (big feet, thongs, singlet, tummy, asymmetry) go onto Pear, Egg and Gumdrop first; Classic gets only the eye bob and shading.
3. The decorative eskies change colour so the Chest is unique. The Chest stays Cheeky-mode only.
4. The "veranda" and "worn paths" do not exist yet; they are new geometry/texture and must not block walking (colliders stay as they are or get checked).
5. Colour emoji, the browser look and the JS game are unchanged.

## Checklist

### Characters
- [ ] Eyes that bob and slide as you walk and run (pupils and eyeballs move on their own springs, a little late and a little overdone), separate for each eye
- [ ] Simple expressive faces: brows, a mouth that changes (stunned, drinking, throwing, drunk)
- [ ] Clothing with folds: a singlet that wrinkles round the tummy, board shorts or stubbies, a cap or hat on some characters
- [ ] Slight asymmetry per character (one shoulder lower, a wonky grin) and clearly different silhouettes between the four
- [ ] Dazza: more caricature (big hat, belly, apron, stubby holder, sunnies)
- [ ] Leader crown, stars, sash and tag still read clearly on the new bodies

### Character proportions
- [ ] Slightly oversized heads, larger hands, noticeably large feet (visual only, collision unchanged)
- [ ] Oversized thongs (flat, chunky, a colour each), skinny lower legs, slightly exaggerated stomachs
- [ ] Walk, throw, stunned and fall poses re-checked with the bigger feet and hands (no clipping)

### Environment
- [x] Lawn: dry yellow grass, stubborn bright-green patches, bare dirt, uneven coverage, mow stripes still visible (done as a 1 m grid of vertex colours over the lawn tile, `PatchyLawn` + `lawn_tint`; soft dirt decals on top; polished look only)
- [x] Worn pathways from the back door to the BBQ, bar and Hills Hoist; dirt round the BBQ and bar (`dirt_spots`, soft `soft_dot.png` discs)
- [ ] Sun-baked palette pass on every flat colour (dry yellow, dusty orange, faded green, weathered brown, washed-out blue, faded red, cream, galvanised grey)
- [ ] Sky and fog a little warmer near the horizon, a hazy heat feel
- [ ] Fence leans a little, boards of slightly different heights and shades, some gaps
- [ ] Paddock and far trees get dry grass and a dusty colour, still very simple

### Australian backyard details
- [ ] The house: slightly crooked weatherboard in faded cream, uneven roofline, a veranda that sags in the middle, flyscreen door, a letterbox
- [ ] Hills Hoist leaning a few degrees, rusty arms, a few pegs and a towel
- [ ] Weeds and dry tufts (instanced), cracked concrete paving by the back door, a green wheelie bin, a hose and a sprinkler
- [ ] Small clutter that tells a story: a cricket bat and stumps, a dog bowl, thongs by the door, a washing basket, bottle caps and stubbies on the ground

### Props
- [ ] Outdoor table: thick timber planks, slightly warped, nails and bolts, beer rings, scratches, bevelled edges
- [ ] Plastic chairs: chunky, slightly warped, sun-faded, scratched, a little asymmetric
- [ ] BBQ: oversized wheels, big rounded lid, stubby knobs, slight asymmetry (see High-detail assets)
- [ ] Shed, bar, trampoline, planters, log pile, tyres, tank: bevels, a slight lean, dirt, rust
- [ ] Items in hand and on the ground get a second look: teddy seams, gnome face, stubby label, steak marbling, fish scales, noodle ends, snag
- [ ] Red sauce bottle and yellow mustard bottle as bright readable props

### Dildo Chest
- [ ] Rebuild it as a big, chunky, bright-blue esky (bigger than a normal esky, strong silhouette, easy to see from far away at all 9 spots)
- [ ] Chunky hinges, a large handle, an exaggerated lid that still opens and eases shut as now (same lid pivot)
- [ ] Surface: scratches, faded plastic, dirt, stickers, worn corners, slight discoloration
- [ ] Change the four decorative eskies to red, white and green variants so only the Chest is bright blue
- [ ] Check it reads against grass, dirt, the shed and the fence, in shadow and in sun

### Animals
- [ ] A magpie on the fence watching the player: absurdly large head, big beak, small chunky body, simple wings, large eyes, head that tracks you
- [ ] Magpie extras: hops along the fence, flaps off if something flies close (visual only, no gameplay)
- [ ] (Later, optional) a dog, a lizard or a kookaburra on the shed

### Models
- [ ] Shape builder: bevel option on cuboids and cylinders, a `wobble` (small per-part random tilt and size change from a fixed seed) so nothing is mathematically straight
- [ ] Vertex-colour variation on shapes (a faded top and a darker bottom edge)
- [ ] Keep each prop's collider as it is unless the new shape genuinely needs a new one; add tests that props still sit inside their colliders

### Textures
- [ ] `tools/make_textures.py` writes stylised timber, galvanised metal, plastic, concrete, dirt, dry grass and rust textures (128 to 512 px; base colour plus roughness and normal maps)
- [ ] Wood: grain, dark edges, nail heads, scratches, chips; fence and table share it
- [ ] Metal: roughness variation, scratches, heat staining, grease, rust round bolts
- [ ] Plastic: sun fade, scratches, dirt in seams
- [ ] Concrete: cracks, stains, aggregate, dark patches, dirt at edges
- [ ] Decals: stickers, beer rings, burn marks, stains (small textured quads, no engine decal system needed at first)

### Materials
- [ ] Roughness and metal per material instead of one matt value (cans and BBQ metal shiny, wood and fabric matt, plastic in between)
- [ ] Use the normal, roughness and AO maps the engine already supports; keep the browser-look path unchanged
- [ ] Stronger, cleaner colours on gameplay objects (Chest, bin, sauce bottles, items) than on the background

### Lighting
- [x] Warm sun, cooler shadows (a blue-ish ambient), a stronger sun and slightly lower ambient for depth (first pass; tune on Marcus's Mac)
- [ ] Softer shadow edges (filter and cascade tuning), shadow range that keeps characters crisp
- [ ] A rim or back light so characters separate from the lawn
- [ ] Contact shadows or ambient occlusion so things sit on the ground
- [ ] Inside-the-shed and under-the-veranda get a darker feel

### Rendering
- [x] The look switch: `--look browser|polished` (screenshots default to browser, playing to polished; a menu option comes with the Phase 8 settings); `browser` keeps today's maths for the compare toolkit
- [x] Colour grading in `display_raw.wgsl`: gentle warm curve, a little more saturation, more contrast, a soft vignette (polished look only)
- [ ] Screen-space ambient occlusion and either SMAA or the existing MSAA
- [ ] Mild bloom on emissive and very bright things only
- [ ] Texture filtering: anisotropy 8 for ground and fences

### Blender workflow
- [ ] One script per asset in `tools/` (`make_esky.py`, `make_bbq.py`, `make_magpie.py`, `make_table.py`, `make_blob.py` updated...), all run with `blender --background --python`, each writes a `.glb` into `assets/models/`; a `tools/README` line says the order
- [ ] Bevel modifier, weighted normals, shade smooth and UVs on every export
- [ ] High-poly to low-poly bake of normal and AO for the hero props (BBQ, esky, magpie), 512 to 1024 px
- [ ] Slight controlled mesh deformation and asymmetry in the scripts (seeded, so a rebuild gives the same wonk)
- [ ] The game loads these models and falls back to the shape-list version if a file is missing

### High-detail assets
- [ ] BBQ: oversized wheels, big rounded lid, stubby knobs, grease, heat staining, burn marks, dirty grill plates, bolts, welds, a little rust; cartoon shape, real surface
- [ ] Dildo Chest esky (see above) as a hero model
- [ ] Magpie as a hero model
- [ ] Hills Hoist and the bar's taps and bottles
- [ ] Food: a slightly oversized snag and steak with real-looking texture, the fish, the teddy, the gnome

### Optimization
- [ ] LOD: a low version of each hero model for far away; swap by distance
- [ ] Instance weeds, tufts, bottle caps; share materials between parts; merge static background parts into fewer meshes
- [ ] Keep textures at 128 to 512 px (1024 only for BBQ and esky); no 4K
- [ ] Measure frame time before and after on Marcus's Mac (11 bots, Heist arena, Cheeky mode); keep 60 fps; add a Low / High graphics option to the Phase 8 menu
- [ ] Before and after screenshots of the yard, the BBQ, the Chest, the magpie and a blob close-up for Marcus to approve

## Order of work (highest impact first)
1. The look switch and warm/cool lighting plus grading (everything else shows up better under it)
2. Lawn colour variety, dirt and paths
3. The Dildo Chest esky and the four decor eskies changing colour
4. Character feet, thongs, eye bob
5. Magpie
6. Wonky shape language (fence, house, hoist, table, chairs) and textures
7. BBQ and other hero models, then LODs and performance
