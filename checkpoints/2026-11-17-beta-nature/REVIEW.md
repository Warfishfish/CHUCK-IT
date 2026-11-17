# Three-map beta sky and nature pass — 17 November 2026

## Changes
- Removed the overhead wet-status droplet. Falling body droplets, highlights, drying HUD and existing pool/balloon slowdown remain.
- Temporarily parked Race Oval in player-facing map menus, lobby role controls and saved preferences (old Race preference falls back to Backyard). The race implementation, assets, protocol and explicit developer CLI remain for later use. Backyard, Bush Servo/Smoko and solo Teddy Heist remain selectable; Heist retains its existing online restriction.
- Deeper blue zenith with a pale horizon and less broad sun haze. Shared procedural clouds now have a shaded base and irregular warm-white upper lobes, plus occasional thin wisps. They do not cast shadows.
- Original gum, citrus and paperbark trees, kite and striped windsock authored/exported using live Blender MCP. Forked trunks, bark scars, root flares, lemons and irregular crowns add variety. Crown/fabric motion is visual only. Replaced the two old foreground Backyard shade trees rather than overlapping them.
- New roots are beyond playable footprints; no route, collision, scoring, speed or timing changes. Backyard gets a kite/citrus story; Smoko gets roadside gums and a windsock; Heist gets an independent boundary grove.
- The full Rust browser uses the same code/assets. Legacy JavaScript prototype 0.23.30 gets a camera-centred gradient sky, new cloud shapes and smoother distant crowns; it has no Race picker.

## Verification
- cargo test --workspace: 737 passed, 3 ignored, 0 failed.
- Native optimized release and full WASM browser build compiled successfully (existing warnings).
- Actual Chrome/Cloudflare check: new menu shows Backyard, Bush Servo and Teddy Heist with no Race option; switched all three map previews and launched a solo Heist round. browser-map-menu.png records the selectable maps. Browser walking/turning was not checked in this pass because mouse capture requires a physical click.
- Browser JavaScript extracted scripts: node --check passed.
- New regression checks: no Race map option; every new tree trunk outside playable boundary. Existing host/guest selection snapshot tests pass for retained maps.
- Native Polished/Pop captures: backyard.png, smoko.png, heist.png. These are actual game renders; blender-tree-library.png is a separate Blender asset study.
- Scripted water-balloon motion capture wet-motion-030.png shows body drips and existing 80% movement status without an overhead marker.
- No movement or networking simulation code changed. A fresh two-person online match and hands-on long-session/performance review were not run for this pass.

All five new GLBs were created from procedural original geometry with no downloaded or third-party assets. Source: rust/tools/make_beta_nature.py. This is an art/menu pass, not a Steam readiness certification.
