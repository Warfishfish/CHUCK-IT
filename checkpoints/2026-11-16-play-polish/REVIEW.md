# Smoko, Heist routes and character motion · 17 November 2026

## Changes

- Smoko: thin muted shoulder lines follow the existing circular road. Small road dashes and shoulder stones no longer cast tiny shadows. Prop positions, collision and delivery lanes are unchanged.
- Heist: A* lookahead scans the entire route for the furthest visible point. Previously it tried only cells near the destination, then fell back to one-cell steps when the final bend was hidden. The returned route is still sampled against solid walls.
- Bot recovery: intentional pauses, hesitation and windups do not trigger random stuck recovery. Rust checks the actual steered request rather than the original full-speed goal request.
- Rust characters: previous velocity is transformed into the current facing frame before deriving acceleration. This removes false acceleration when the body rotates. Belly acceleration response is scaled by elapsed time; braking lean is slightly clearer.
- JavaScript characters: clearer braking lean, exponential speed settling and a restrained breathing/step belly response. The full browser game uses the same Rust code as desktop.

## Checks

- `cargo test --workspace` passed, including the new long-wall lookahead regression, the 150-second Heist steal-and-bank match and Smoko bot collect/deliver tests. Three pre-existing tests remain ignored.
- Browser script syntax passed; the equivalent JavaScript long-wall lookahead regression also passed.
- Local relay test passed (two room peers and an isolated third peer). This is a relay test, not a rendered two-player playtest.
- Native release build passed.
- Full WebAssembly release build passed.
- Native rendered Smoko delivery sequence finished with +100 and no remaining carry.
- Live rendered browser review: results recorded below.

## Review scope

This is a focused polish pass, not a new environment asset pack. It adds no external assets and changes no controls, scoring, round length, movement-speed constants, character hit sizes or network protocol.

Before/after overview: `smoko-overview-before.png`, `smoko-after.png`.

## Live release review

- Chrome loaded the rebuilt full Rust browser game through the replacement Cloudflare tunnel. The live WASM SHA-256 matches the generated local file (browser-hash.json).
- Started solo Smoko with three bots. After a physical click enabled mouse capture, walked and turned near the road, props and shelter. No abrupt ground-tile patches were seen in those views. This was a partial route check, not a full timed lap.
- Live Chrome Teddy Heist match: at 1:57 remaining, Red and Blue each had one bank. Davo and Kev each showed one bank / 150 points. This verifies actual theft and banking in the rendered release, not just headless movement. The console error query returned no entries.
- The native Heist close-up was black and was discarded; it is not used as evidence for character appearance.
- Rendered two-player online play was not retested. Movement/physics and network messages are unchanged; bot decisions run on the authority. The local two-peer relay check passed.

Existing research notes under reports/ and research_notes/ are included at Marcus's request to commit everything. They remain historical proposals, not a claim that every idea is implemented.

Release source: `80a8bfa`; generated browser build: `67c0fa5`. Both desktop and browser use the same updated Rust gameplay/animation code.
