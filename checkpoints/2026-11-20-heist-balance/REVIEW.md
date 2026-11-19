# Teddy Heist shortcut balance

Prototype 0.23.45 adjusts passes, crate vaults and Dazza’s tunnel across the three flag layouts. Desktop and WebGPU clients use protocol 33. Reload or update each client before joining.

## Passes

Friendly-fire protection previously skipped the catch check for teammates. A teammate can now right-click to catch a stolen teddy during the normal catch window. A missed catch still goes through a teammate safely. The host applies the same horizontal and upward limits to an objective teddy no matter who threw it.

A full-power 45-degree example from 1.4 m previously had a 33.1 m estimated still-air distance to first ground contact. With the 12 m/s horizontal and 3 m/s upward caps it measures 6.7 m in the same calculation. Terrain, bounces and release height affect travel. A timed 4.1 m pass was caught and banked in browser fixtures on all three maps. Catching gives no points in Heist. The receiver gets the normal 150 personal points when they bank.

## Vaults

The old approach-to-landing ground detours measured 13.5–70.1 m on a 0.5 m grid with player-width clearance. Five stair placements moved to shorter detours. The existing short vault on the four-flag map stayed in place. The new ground alternatives measure 13.5–13.7 m over the same approach-to-landing segment. All six vaults still cleared a player carrying a stolen teddy. The browser input fixture crossed each in 1.32 simulated seconds with consecutive grounded jumps.

## Tunnel

The walking route from the hatch to a base doorway measures roughly 100–134 m across the three layouts. The underground alternatives cover roughly 65–76 m including the exit ramp. In the browser tests, carrying a teddy from the bottom of the hatch to an exit now takes 13.4–15.5 seconds instead of 10.4–12.1 seconds. Hatch entry lasts 8 seconds rather than 12. Runners already underground can reach all nine exits after it closes. The original drink and escort requirements still apply.

The Rust workspace passed 751 tests with three ignored. Twenty-participant Rust matches recorded 8, 12 and 16 banks on the two-, three- and four-flag maps. Balance figures are scripted comparisons. Defensive pressure and repeated shortcut use over a full human match still need a playtest.

A controlled two-client legacy-browser test sent a pass from a guest over the public relay. The host caught and banked it, and both clients saw one bank. The fixture positioned the players in a clear lane and assigned them to the same team.

The native release loaded the relocated two-flag vault. Two native clients also started a timed, zero-bot two-flag round over the public relay. The Cloudflare WebAssembly bundle and manifest were compared byte for byte with the local release.
