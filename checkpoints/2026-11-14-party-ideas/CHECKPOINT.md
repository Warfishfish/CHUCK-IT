# Carry camera, Heist bases and Dazza reaction · 14 November 2026

Status: implemented locally and ready for review. The carry camera is verified in a scripted gameplay capture; Dazza's reaction and real two-player online play still need hands-on verification.

## Changes

- The carried-player camera transitions to a wider outside view framed around the carrier and passenger, follows through flight, then blends back to first-person. Existing implementation is in `player.rs`, with carry lock and throw tracking in `smoko_mode.rs`.
- Teddy Heist bases receive a distinct geometric emblem and team colour above each base. Markers are presentation-only and have no collider.
- Dazza raises his spatula when a fast steak or fish passes within reach of the BBQ. This is a visual reaction only and does not stop or redirect the item.
- Browser prototype label is 0.23.7.

## Checks

- `cargo check -p bbq_app` passes. Existing project warnings remain.
- Native Heist solo capture shows the expanded maze and team bases in [the updated map review](../2026-11-14-heist-expansion/review.html).
- The carry motion fixture shows both characters in the outside carried view at frame 50 and first-person restored after delivery at frame 73. Try a real two-player carry and throw to verify it in normal play.
- Two-player online play and the Dazza reaction have not yet been verified hands-on.

## Full-game local test

```sh
cd /Users/marcus/Claude/australian-bbq/rust
cargo run -p bbq_app --release -- --menu
```

Choose Heist to inspect the base emblems. In Smoko, throw a steak or fish near Dazza to trigger the spatula save. Test the carried view with a second player to check its framing and return to first-person after release.
