# Polished Pop colour and light pass — local review (8 November 2026)

The first graphics-list pass tunes the existing polished Pop light for richer warm highlights and more separation in shade. It affects the Backyard/Teams and Heist polished Pop scenes through `lighting.rs`; the existing Smoko-specific rig in `servo_finish.rs` gets a matching, slightly softened version. Browser-match lighting and Clay/Current styles are unchanged. No assets, geometry, collisions, controls, or networking changed.

## Capture review

`review.html` shows the actual same-camera Smoko captures before/after and the latest overview and Backyard/Teams shed view. The same-camera comparison uses a 1280×800 native release capture with the same mode, camera, assets, and UI. Its fixed 600-interval readings were 16.668 ms mean before and 13.590 ms after; this single pair varies too much to treat as a performance result. The wider Smoko overview and Backyard shed view are after captures only. Shadows remain readable, and the grade does not push the daytime palette toward orange.

## Implementation

- `lighting.rs`: the polished Pop rig has slightly less sky fill, a warmer directional sun, and a small saturation/contrast lift.
- `servo_finish.rs`: the Smoko override follows that light balance while retaining its map-specific sky, fog, and material filtering.

## Build and scope

`cargo build -p bbq_app --release` completed. Existing compiler warnings remain. No tests were run. The screenshots are review captures, not a browser or guest-device claim. Local only: no commit, push, or browser update. Preserve the user's unrelated pending edits and `concept art/`.

The Backyard/Teams shed capture also confirms the existing roof shadows read cooler and darker than the sunlit lawn under this light balance. Together with the verified gum/shelter shade checkpoint, A2.6–A2.8 are now checked in `GRAPHICS_2C.md`.

## Next graphics-list item

B5.2: strengthen the natural lean of a few gum trunks and add a small number of visually split trunks. Their footprints remain outside the playable yard.
