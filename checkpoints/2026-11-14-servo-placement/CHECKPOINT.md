# Bush Servo shop placement study · 14 November 2026

Status: visual comparison ready for Marcus to choose the final site. No game code, asset, or collision data changed.

## Finding

The Bush Servo is already a single runtime instance at the north-west frontage. Its complete exported model (building, signs, pumps, and parked ute) is inside the fence. The backlog wording that asked to move an exterior instance inside is stale for the current source.

`yard_scene.rs` already provides capture flags `--servo-nw` and `--servo-ne`. The north-east flag moves only the rendered GLB during a screenshot; the current north-west shared collision and avoidance remain active. It is not a playable north-east build.

## Comparison

See [review.html](review.html) for same-camera east-picnic and shelter views, plus matched top-down views.

- North-west is the current site and most closely matches the concept's shop-left, shelter-centre composition. A foreground gum partly obscures it from the east approach; the fence-side route is tighter.
- North-east is clearer from the east and opens more west-side space, but shifts visual weight right and puts the ute and pumps near the eastern fence.

## Envelope and clearance measurements

Source model bounds from the `POSITION` accessors of `rust/crates/bbq_app/assets/models/smoko_servo.glb`:

```text
local x: -23.75 .. 11.665 m
local z: -33.00 .. -24.021 m
```

With the current candidate offsets, the full bounds are:

| Candidate | Offset (x,z) | World x | World z | Tightest side gap | Road separation |
|---|---:|---:|---:|---:|---:|
| North-west | (-6, 9.5) | -29.75 .. 5.665 m | -23.50 .. -14.521 m | 3.25 m west | ≥2.02 m beyond outer radius |
| North-east | (17, 9.5) | -6.75 .. 28.665 m | -23.50 .. -14.521 m | 4.335 m east | ≥3.08 m beyond outer radius |

Both remain within the 66 × 48 m fence and leave a 0.50 m margin to its north rail. Shared spawn filtering rejects one player spawn and two regular item spawns for either option (11 of 12 player spawns remain, as at the current site). The existing 1.4 m carried-player east-west and north-south approach corridors at the central shelter do not intersect either model envelope. Final site implementation should still add explicit route/collider and bot-avoidance checks for both side approaches.

## Review inputs

- East picnic camera: `22,4,10,38,-7,78`
- South shelter camera: `0,3,22,0,-2,78`
- Top-down camera: `0,100,0,0,-90,34`
- Both candidates captured in Rust Smoko with zero bots and no character/item clutter.
- The northeast view changes its transform only via `--servo-ne`; the north-west capture uses `--servo-nw`.

Choose north-west or north-east before moving shared collision, bot avoidance, and the single runtime asset placement. No tests were added or changed in this visual-only study.
