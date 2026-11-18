# Teddy Heist reference maps

One SVG grid cell spans 7.3 m. Each arena measures 116.8 × 116.8 m inside its perimeter, about twice the footprint of the old oval with 54 m and 40 m radii. The square includes the wall footprints. Rust and JavaScript use the same wall coordinates. The import combines adjacent straight segments. An independent edge comparison matched all 272, 268 and 272 reference edges respectively. Each flag centre has the game's three-sided 6 m banking pen.

- Two flags — [reference](square-2.svg) · [browser overhead](maze-2.png)
- Three flags — [reference](square-3.svg) · [browser overhead](maze-3.png)
- Four flags — [reference](square-4.svg) · [browser overhead](maze-4.png) · [player-height approach](approach.png)

## Ground routes

Distances are approximate shortest base-centre walks on a one-metre grid, with diagonals, midpoint collision checks and 0.65 m clearance. Solid walls are bidirectional. A flood test also checked every walkable sample and every spawn, doorway and teddy position.

- Two flags — red–blue 224 m.
- Three flags — red–blue 250 m, red–green 250 m, blue–green 264 m.
- Four flags — red–blue 205 m, red–green 205 m, red–gold 212 m, blue–green 223 m, blue–gold 205 m, green–gold 205 m.

## Participants and bots

A local player and 19 simulated bots played Free for All, Teams, Bush Servo, the developer Race mode, and each flag count. Full Heist teams split 10/10, 7/7/6 and 5/5/5. Simulated three-minute matches recorded 8, 10 and 16 banks respectively. The separate three-bot match also completed a bank. In the legacy browser, all three layouts started with 20 distinct spawn positions.

Bots share a walkability grid sampled from the current yard's colliders. Each runner plans towards its current objective and follows a visible point along the route. This uses the arena geometry. A new mode still supplies the objective its bots should pursue.

The WebSocket relay admitted 20 separate automated connections to one room and refused connection 21. Two separate native games then entered a round through the relay. Both host and guest reported one remote player, 18 bots and the same running round clock. These were automated tests. A match with 20 human players has yet to be tested.

The full Rust browser game opened all three flag counts. The browser gameplay checks reached the running clock and mouse-capture prompt. The overhead and approach images above are from the legacy browser renderer. Final native recaptures were black. A separate native host/guest match also verified 1v1 with zero bots. The workspace passed 741 tests with three ignored.
