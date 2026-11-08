# Concept-led graphics pass — visually reviewed (8 November 2026)

This pass continues the polished Pop graphics list toward the concept art's warm light, richer tree silhouettes, and lived-in environment. Current native screenshots are in [review.html](review.html). The Mac was unlocked for this review, and the previous visually unreviewed additions are now checked in `GRAPHICS_2C.md`.

## Changes and visual review

- `lighting.rs` and `servo_finish.rs`: warmer sun, lower fill, and a modest saturation/contrast lift for polished Pop. Browser-match lighting and Clay/Current styles are unchanged. The same-camera Smoko comparison is in [the lighting review](../2026-11-08-vivid-pop/review.html).
- `looks_yard.rs`: position-seeded gum leans, occasional twin trunks and sparse dry crowns; existing per-tree size/crown variation retained. Backyard details include a pool umbrella, two chairs and inflatable toys; a mower and oil marks by the shed; a coiled hose and sprinkler; and a discarded bench by the bins. Pool, mower, hose, bench and open-lawn placement were reviewed in the linked captures.
- `looks.rs` and `shapes.rs`: each fence section is one shared mesh with varied palings, subtle knots, leaning posts and rare missing, broken or loose boards. The close-up confirms the detail and sight gaps; the wide view keeps their damage restrained. Gameplay collision remains straight.
- `GRAPHICS_2C.md`: B5.3, B6.2–B6.6, B7.3–B7.4, B7.6–B7.8 and B8.4 checked after in-game review. A1.7 remains on hold as requested.
- `looks_yard.rs`: the BBQ now has five visible grate bars, twin hood hinges with supports, a crossbar handle and five crown vents (D12.3). Faded cabinet stickers, small label marks and cooked-on drips complete D12.4. Both were checked in the open-grill close-up in `evidence/after-bbq-cooking-side.png`; the camera is pitched down enough to keep the cabinet details clear of the HUD hint.
- `looks_yard.rs`: D13.4 adds body-color-derived sun fading, corner abrasion, fine scratches and dirt along the lower seam to the blue chest and the yard's other eskies. Small wear marks do not get their own outline, so they read as surface scuffs. The matched close-up keeps the blue body and existing stickers clear; see `evidence/before-esky-wear.png` and `evidence/after-esky-wear.png`.
- D13.5 is checked: all nine chest positions were captured and reviewed. The blue cooler stays distinct from grass and fencing in both shade and direct sun; adjacent shed walls do not wash it out. Captures are `evidence/esky-spot-0.png` through `evidence/esky-spot-8.png`.
- `looks.rs`: the Cheeky beer and rum bottles now use translucent amber glass shells around separate liquid fills; their cream/red labels and gold/black caps remain clear. The close gallery view confirms both bottle profiles and liquid levels; see `evidence/after-cheeky-bottles.png` (D14.4).
- D14.5 was already wired into gameplay: bottle impacts choose brown or clear fragments, beer foam or rum splash, and a fading drink stain. The shot showcase had left its dropped bottles non-live; it now marks them as flying, so the capture runs through the actual smash handler. Beer and rum impact captures are in `evidence/after-bottle-smash-beer-late.png` and `evidence/after-bottle-smash-rum.png`.
- `looks.rs` and `looks_yard.rs`: D14.6 gives a snag its own browned casing and char marks, and separates it from the wider steak with its pale fat edge, marbling and grill marks. The fish retains its scale texture. The close food gallery and in-yard grill are in `evidence/after-food-items.png` and `evidence/after-bbq-cooking-side.png`.
- D14.7 is complete: rounded edges on key shed, planter, log, trampoline, tank and bar shapes; weathered marks on exposed timber and tank metal; planter posts have repeatable slight lean. The tank bands now wrap its wall, with staining set against its facets. Checked in `evidence/after-tank-bevel.png`, `evidence/after-props-yard.png`, `evidence/after-shed-tank-detail.png`, `evidence/after-trampoline-springs.png` and `evidence/after-bar-taps.png`.
- D14.8 is complete: two striped towels, a T-shirt, socks and wooden pegs now hang from the Hills Hoist outer cords. The same clearance frame supplies their attachment points. Visually checked in `evidence/after-hoist-washing.png`.
- D15.2 remains in progress: the perched magpie hops along the fence and lifts/flaps when a live projectile passes nearby. Captured as a 75-frame sequence in `/tmp/bbq-magpie-review/` and reviewed for landing and hop contact.
- E16.4 is in progress: a light, low dust puff trails the local player during a grounded boost, trampoline landings emit a small dust puff, sparse pale wisps rise at two pool corners, and a short confetti fall appears inside the winner's results card. The new pool mist and results-card effect are awaiting an in-game visual capture.

## Performance

The latest valid 11-bot Heist/Cheeky run after trampoline puffs measured 111.72 fps mean and 11.232 ms p95 with effects enabled, and 123.31 fps mean and 9.399 ms p95 with effects disabled. Twenty and six of 600 sampled intervals exceeded 16.67 ms respectively. The local player does not actively boost in the stress scene. These are wall-clock intervals after 300 warm-up frames at 1280×800 with AutoNoVsync, not GPU timestamp measurements. Reports and captures are under `checkpoints/2026-10-13-graphics-m1/evidence/after-graphics-20261108-props-fx-stress*`.

A later follow-up capture attempt found the Mac console locked; its black screenshots and 60 fps capped intervals are quarantined under `checkpoints/2026-10-13-graphics-m1/evidence/console-locked/` and are excluded from the review and benchmark results.

Separate 0-bot inspection captures ranged from 5.678 to 6.531 ms mean per frame. Those are scene-review measurements, not the stress result.

An earlier attempt happened while the Mac console was locked and returned black screenshots and about 60 fps. Those outputs remain quarantined in `evidence/console-locked/`; the unlocked rerun above supersedes them.

## Build and scope

`cargo build -p bbq_app --release` completed with the repository's existing warnings after the scenery, FX and magpie-animation changes. No tests were run. New props add no gameplay colliders or protocol changes; trampoline FX reuses the existing dust event, and magpie hops/flaps do not alter game state. The yard centre and movement/throwing rules are unchanged. D14.7 and D14.8 are included in commit `40b7a8b` on `main`. The earlier commit also contains unrelated gameplay changes and concept art.
