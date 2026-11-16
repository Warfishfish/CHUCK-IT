# Stable grass coverage

The previous grass rendering swapped whole 5 m tiles between 900 and 90 tufts at 14 m, then hid them at 24 m. Those abrupt changes exposed square differences in density and shading.

Each tile now uses a fixed 180 tufts. Camera distance no longer swaps or hides the tile. Normal renderer frustum culling still applies.

Checks completed:
- 730 workspace tests passed, 3 ignored.
- Native and full browser release builds passed.
- Native screenshots were inspected at the same camera position facing forward and turned 45 degrees. Both show consistent coverage. Screenshots are yard_0.png and turned_0.png.
- Native capture logs contained no asset or shader errors.
- The Cloudflare-served WASM hash matches the rebuilt local file.

A sustained hands-on browser camera-motion review remains pending. The lighter uniform density is a deliberate tradeoff to avoid whole-tile visual transitions while bounding rendering cost.
