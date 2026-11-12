# Smoko Arena circular road · 12 November 2026

Marcus approved publishing the circular Bush Servo road pass to the Rust desktop and browser build.

## Change

- The Bush Servo arena road is a circular annulus centered at `(0, 6.5)`, with a 16.5 m inner radius and 19 m outer radius.
- Lane dashes and embedded shoulder stones follow the circle. The custom Bevy road mesh reads the same radii and segment count as the map layout.
- The road is visual-only; gameplay ground remains flat. Existing prop positions, collisions, fence and shelter are unchanged.
- The ute jump was already implemented. Its tray landing was checked in the 10 November capture pack; this pass does not change jump behavior.

## Verification

- `cargo test --workspace`: passed (263 `bbq_app` tests; 452 `bbq_core` tests; one ignored app test).
- Rebuilt the Smoko screenshot after the mesh change; the [review page](review.html) compares it with the previous road and the Bush Servo concept.
- WebAssembly browser bundle built with `rust/tools/build_web.sh`; the optimized build completed successfully.
- No online or collision code changed, so no two-player networking check was needed.

## Local browser test

```sh
cd /Users/marcus/Claude/australian-bbq
npm start
```

Open `http://localhost:3000/rust/` to test the Rust browser build. The project also serves the older JavaScript game at `/`.

The bundle was checked locally. No public Cloudflare tunnel was started as part of this publication.
