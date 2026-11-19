# Teddy Heist movement and wall landmarks

The animation used the bot's aim for its body rotation while the steering and collision systems moved it along a different vector. The gait now follows the displayed bot's displacement in Teddy Heist, easing the body into the direction of travel. Windups and non-running poses keep their action facing. Both the host and browser guest use the same Rust gait logic.

The navigation pass uses tighter body-clearance samples, rechecks a blocked corner waypoint, brakes earlier at bends and gives nearby oncoming runners a consistent side to pass on. In the legacy browser's seeded, 10-second capture, 24 of 3,384 ordinary moving bot samples faced more than about 49 degrees away from travel before the change. Afterward, 0 of 3,373 did. A sample spans 50 ms. The same capture had zero stationary samples with strong steering input in both versions, so it did not reproduce prolonged wall bumps.

[Before clip](before.webm) · [After clip](after.webm) · [In-game screenshots of all seven posters](walls.png)

Each flag count has seven separate wall mounts. The screenshots show the legacy browser's two-flag map at eye height. Art was drawn for Australian BBQ.

[Desktop Dazza poster](desktop-dazza.png) · [Desktop missing teddy](desktop-missing.png)

The Rust workspace passed 749 tests with three ignored. The existing 20-participant Rust match test recorded banks on all three layouts. Seeded legacy browser runs recorded 8, 12 and 16 banks in the 2-, 3- and 4-flag layouts. The longest sampled wall hold was 0.2 seconds across all three layouts in each build. The obstruction check sampled active steering within a half-metre of a wall every 0.2 seconds during the 180-second runs.

Additional clips replay recorded bot positions from the crowded legacy simulation. [Corner](corner.webm) · [Traffic](traffic.webm) · [Banking approach](carrier.webm)

The same public relay was used for both browser guest recordings with a native host and 17 bots. The guest camera stays near its spawn, offering a short check of how replicated characters render. [Guest before](guest-before.webm) · [Guest after](guest-after.webm)

Cloudflare delivered the generated WASM and all seven matching textures. The desktop host and Chrome guest also entered a timed two-flag 1v1 with zero bots. Browser mouse capture worked on the updated build in both online runs. These were short automated sessions. A longer human playtest can check whether crowded turns and stride timing feel right across a full match.
