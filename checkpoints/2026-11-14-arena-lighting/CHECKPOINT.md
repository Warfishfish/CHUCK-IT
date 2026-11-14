# Arena lighting and colour · 14 November 2026

Status: first pass implemented and ready for Marcus's visual review. Local only; not committed, pushed or published.

## Changes

- Kept the low warm afternoon sun, with a slightly less amber tint and lower intensity.
- Cooled and gently lifted the sky fill while keeping ground bounce neutral-warm, to open the shade without flattening it.
- Reduced saturation, contrast and vignette slightly to keep warm dirt and timber distinct from cool gameplay props.
- Applied only to the polished Pop Bush Servo scene through its existing scoped lighting override. Browser-match, Clay/Current looks, gameplay and networking are unchanged.

## Review and checks

`review.html` compares actual native Smoko captures from the same release game, asset set, zero-bot setup, camera and 1280×800 resolution. The capture tool recorded a 600-interval timing sample after 300 warm-up frames. Before/after mean frame times were 7.372 ms and 7.307 ms; this single pair is a smoke check, not a performance claim. Mesh, material and image memory counts were unchanged.

The release build completed. Existing compiler warnings remain. The before image was captured before the lighting edit; the after image uses the rebuilt executable. The capture-only frame-time pair is not a cross-device or browser measurement.

## Changed files

- `rust/crates/bbq_app/src/servo_finish.rs`
- `public/index.html` (prototype label only)
- `CHECKLIST.md`, `NEXT_UP.md`, `CONCEPT_ART_GRAPHICS_BACKLOG.md`
- this checkpoint and its evidence files

Other pending carry-camera changes in `player.rs`, `smoko_mode.rs` and `visual_review.rs` are separate and were not included in this pass.

## Local testing

```sh
cd /Users/marcus/Claude/australian-bbq/rust
cargo run -p bbq_app --release -- --mode smoko --bots 0
```
