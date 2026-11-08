# Sparse gum crown variation — visually reviewed (8 November 2026)

B5.3 adds a rare near-yard gum with a visibly thinner, drier crown: fewer leaf clumps and a muted sun-dried tint leave more of its existing bare limbs visible. The variation is chosen from the tree's position-derived random stream. It does not change tree positions or consume the world-generation stream, and far trees are not affected.

## Review and verification

The later [Backyard overview capture](../2026-11-08-graphics-list-progress/evidence/after-backyard.png) shows the gum silhouettes in the current build. Their sparse dry crowns remain subtle among the other gums. `cargo build -p bbq_app --release` completed with existing warnings. No tests were run. Local only: no commit, push or browser update.

`GRAPHICS_2C.md` now records B5.3 as checked. B5.4's existing height/width/spread variation and B5.5's existing distance-dusted foliage plus fog are also checked.
