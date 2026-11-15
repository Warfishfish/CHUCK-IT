# Visual and environment enhancement pass

## What is in this pass

- Added an original Blender-authored rear-lane carport vignette with a weathered restoration sedan, removed bonnet on trestles, garage fronts, tool chest, and a streetlight. The Blender export was made in a temporary scene; the existing Smoko shelter artist scene was restored afterward.
- Continued the yard surface pass with faint mower tracks and restrained roof-edge rain wear.
- Replaced the five-point Rust sprinkler trail with a broad droplet fan and animated it with the turning rotor. The browser sprinkler already has a denser animated spray fan.
- Added a small visual response to the trampoline mat when a grounded character stands on it. It does not affect jump physics.
- Added bank-purpose signs at Heist base entrances in both builds, preserving the open doorway and collision layout.
- Animated the browser teddy's ears in flight and added expanding pool-entry ripples to the native game (the browser already has entry rings).
- Added separate bench seat and backrest colliders, with visible back supports, in both builds.
- Bumped the browser prototype to 0.23.17.

The larger list also contains work already present in the checkout: detailed neighbouring house variants, yard stories and wildlife, existing character face/clothing detail, responsive bots, wheelbarrow/compost/garden props, detailed pool and BBQ areas, roof access, and other motion. This pass builds on those systems rather than duplicating them.

## Review images

![Backyard overhead](/Users/marcus/Claude/australian-bbq/checkpoints/2026-11-16-visual-enhancement/yard-overhead.png)

![Pool-side yard view](/Users/marcus/Claude/australian-bbq/checkpoints/2026-11-16-visual-enhancement/pool-height.png)

![Neighbour restoration carport](/Users/marcus/Claude/australian-bbq/checkpoints/2026-11-16-visual-enhancement/carport-detail.png)

![Heist map overview](/Users/marcus/Claude/australian-bbq/checkpoints/2026-11-16-visual-enhancement/heist-bank-signs.png)

## Verification

- `cargo test --workspace`: passed, 265 `bbq_app` tests plus 463 `bbq_core` tests; 3 app tests are intentionally ignored.
- Browser inline script: `node --check` passed.
- `git diff --check`: passed.
- Native screenshots were captured from the Bevy game using its `--shot` harness. They are scene captures, not hands-on bot matches.
- Browser visual inspection and host/guest play were not available in this session: Codex desktop reported local browser use disabled in Work settings. Browser source syntax was checked.
- An interactive solo Free-for-All / Teddy Heist bot match was not played manually. Automated Heist/objective tests are included in the passing workspace suite.
- The changes do not tune scoring, movement speed, or round length; the bench adds collision to match its visible seat/back, and the trampoline response is presentation-only.
