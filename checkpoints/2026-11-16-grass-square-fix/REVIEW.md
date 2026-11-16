# Stable grass coverage

The previous grass rendering swapped whole 5 m tiles between 900 and 90 tufts at 14 m, then hid them at 24 m. Those abrupt changes exposed square differences in density and shading.

Each tile now uses a fixed 180 tufts. Camera distance no longer swaps or hides the tile. Normal renderer frustum culling still applies.

Checks completed:
- 730 workspace tests passed, 3 ignored.
- Native and full browser release builds passed.
- Native screenshots were inspected at the same camera position facing forward and turned 45 degrees. Both show consistent coverage. Screenshots are yard_0.png and turned_0.png.
- Native capture logs contained no asset or shader errors.
- The Cloudflare-served WASM hash matches the rebuilt local file.

Browser movement review completed in Chrome on 16 Nov 2026 using the Cloudflare-served full Rust build. After a physical click enabled pointer capture, tested forward/backward walking, lateral movement and mouse turns through the centre lawn and beside the house/verandah. No abrupt dark square density patches reappeared in the observed views. Browser console error query returned no entries. This was a solo Free for All round with three bots; multiplayer and other browsers were not retested for this rendering-only check.

The lighter uniform density is a deliberate tradeoff to avoid whole-tile visual transitions while bounding rendering cost.
