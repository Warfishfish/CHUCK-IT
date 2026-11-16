# Smoko flicker, bot movement and environment cohesion

## Changes

- Shared colour textures now generate sRGB mipmaps in linear light. The former filter accepted only linear RGBA textures and skipped the newer lawn, weatherboard, roof and bark maps.
- Smoko's roadside cluster is included in the imported-texture filter.
- The two picnic asphalt/dust overlays have separate vertical ranges above the road and ground.
- The roadside prop GLB now exposes a single valid scene. The previous export included empty scenes that Bevy rejected. Its Blender authoring script now exports only the active scene.
- Goal-bound bots face the direction of their actual route while running. Explicit aim and melee use their existing facing behavior.
- If a smoothed turn points into a nearby obstacle, bots use the checked route angle.
- Weatherboard, roof, fence, wood and bark detail is moderated by the now-active filtering pass. Browser JavaScript has equivalent building/bark contrast treatment.

## Checks

730 Rust workspace tests passed, 3 ignored. Includes the long Heist movement and actual steal/bank simulation. Browser script syntax passed. Native asset loading was checked after repairing the roadside GLB.

Native fixed-camera captures are recorded after the opening banner. They demonstrate rendered appearance. A complete hands-on motion/flicker review and browser gameplay review remain pending. Online gameplay was not manually exercised.

## Vividness and shade follow-up

Blender MCP compared identical proxy walls, timber, an esky and a blob with a higher sun and broad cool fill. The game uses stronger hemisphere fill, a slightly weaker direct sun, a higher sun angle, restrained contrast and a modest saturation increase. The grading shader lifts quiet colours more than already saturated ones.

References:
- Toys for Bob on Dragon Shores colour development: https://blog.playstation.com/2018/11/12/spyro-reignited-trilogy-revisiting-dragon-shores/
- Rare environment/lighting artist Andrew Finch: https://afinchy.artstation.com/projects/bylzr
- Rare's lighting discussion: https://www.youtube.com/watch?v=cPQ0ksLKFyo

The lighting direction is an original adaptation to this yard. Blender renders are lighting studies with proxy objects. Native captures show the actual game.

The first native yard capture showed the sage texture rendering considerably darker than the Blender material study. The procedural texture was recalibrated from that actual game capture to keep blade highlights and the base ground closer in brightness. This is a deliberate brightness adjustment in the original Blender texture script.

The brighter native capture exposed dark blade speckling. Blade base and tip colours were matched more closely to the ground. Near tiles use 900 tufts instead of 1800, far tiles use 90 instead of 375, and detail/draw ranges are 14/24 m instead of 22/38 m. This retains near blade geometry while reducing small distant triangles.

## Final verification and review images

- Final workspace run: 730 passed, 3 ignored.
- Native release and full WebAssembly browser release builds passed.
- Native final captures loaded the game with no asset or shader errors.
- The live Cloudflare WASM, grass texture and grading shader hashes match the generated local files.
- Native yard and Smoko rendered views were inspected. Full hands-on browser/online play and a sustained native flicker/motion review remain unverified.

Images:
- yard-before-lighting.png — earlier native yard capture.
- yard-final_0.png — final native yard after grading and blade adjustment.
- smoko-after_0.png — native Smoko overview.
- blender-light-before.png and blender-light-after.png — matched Blender lighting studies.
