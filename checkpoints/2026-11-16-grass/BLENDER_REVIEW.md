# Blender grass contrast revision

The Blender material study compares the previous olive grass (left) with a new sage grass (right), under one light rig with identical cream walls, timber, a blue esky and coral blob proxies. This is a material study. Gameplay and camera-motion review remains pending.

The original texture is generated in Blender by rust/tools/make_comic_lawn.py. It uses periodic broad colour bands and sparse wrapped tapered strokes. Opposite tile boundaries share the same periodic field. The authoring script creates a separate scene, preserving the existing shelter scene.

The engine's lawn tint was near-neutralised, blade colours were moved toward the sage ground, and elevated cameras now cull fine blade geometry by full camera distance.

References reviewed:
- Chris Peacock, Borderlands 3 Eden 6 plant work: https://chrisp.artstation.com/projects/BmL3G4
- Chris Peacock, Bounty of Blood rock work: https://chrisp.artstation.com/projects/kDQ1vy
- Romain Lambert discusses painted ink detail and cohesion: https://www.exp-points.com/borderlands-3-qa
- Official Eden-6 environment example: https://borderlands.2k.com/news/borderlands-3-music-composers/

Art direction inferred from these references for this game: restrained ground values, grouped forms, selective ink marks, and cooler green ground against warm built props. All shipped grass pixels were authored originally in Blender.

Verification: 730 Rust tests passed, 3 ignored. Browser script syntax and full WASM compilation passed. Live game visual and motion review remain pending.
