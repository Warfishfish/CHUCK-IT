# Cooking station · 9 November 2026

Start at main 72113a1 (published game source ac513c9). Prior browser review and planning edits were present and retained. [Compare the four in-game views](review.html).

## Art and scope

The Smoko approach exposes the back of the raised grill hood. A darker red finish and cream thermometer give that side a recognisable face. A eucalyptus-painted apron supports the food warning plate. Two battens support three timber slats between the existing table legs. Both models come from the shared polished BBQ definition, so they also appear in Backyard modes with the BBQ enabled. The redesigned plate occupies the existing 256 × 60 sign texture slot wherever the prep table appears. The changes use the existing table dimensions, food positions and gameplay collision definitions.

## Matched captures

- Backyard E19.3 uses an independent build from a Git archive of 40f20b6 and the local release executable from before this art edit. Runtime assets at 40f20b6 and 72113a1 have identical Git contents. Both runs used the fixed-step native capture flags `--bare --bots 0 --look polished --graphics high --review --big --cam -6.95,1.78,-13.8,2,-12,39 --shot OUTPUT.png`. The images match byte for byte. The historical run used the development build profile, so timings from this pair should be excluded from performance comparisons.
- Smoko uses the saved pre-edit release and the rebuilt release. Both used `--mode smoko --bots 0 --bare --look polished --graphics high --review --big --cam -7.24,1.8,-7.4,1,-12,42 --shot OUTPUT.png`. Each binary loaded its matching assets through BEVY_ASSET_ROOT. The comparison is from the arena facing the shop at 1280 × 800.
- The camera was stationary for 300 warm-up frames and 600 measured intervals. Mean native frame time was 7.853 ms before and 7.855 ms after. The 95th percentile was 10.528 ms before and 10.274 ms after. Scene costs rose by eight meshes, three materials and 24 mesh entities. Image memory stayed at 95,260,720 CPU bytes.
- Native release build, browser-target compile check and the BBQ food/sign test passed. Browser frame time still needs measurement on a live build.

## Files and recovery

- Model source in `rust/crates/bbq_core/src/looks_yard.rs`. Sign artwork in `art/signs/prep-table.svg` and the game texture in `rust/crates/bbq_app/assets/textures/sign_hands.png`. On this Mac the texture can be rebuilt with `sips -s format png art/signs/prep-table.svg --out rust/crates/bbq_app/assets/textures/sign_hands.png` from the repository root.
- To undo just the visuals, inspect the diff and restore the BBQ model hunks and sign PNG from 72113a1. The SVG can be retained for future edits. Keep the earlier planning edits.
- To launch the local Smoko round, run `cd /Users/marcus/Claude/australian-bbq/rust && cargo run -p bbq_app --release -- --mode smoko --look polished`.
