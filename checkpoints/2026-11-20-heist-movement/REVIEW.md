# Teddy Heist movement and wall landmarks

The animation used the bot's aim for its body rotation while the steering and collision systems moved it along a different vector. The gait now follows the displayed bot's displacement in Teddy Heist, easing the body into the direction of travel. Windups and non-running poses keep their action facing. Both the host and browser guest use the same Rust gait logic.

The navigation pass uses tighter body-clearance samples, rechecks a blocked corner waypoint, brakes earlier at bends and gives nearby oncoming runners a consistent side to pass on. In the legacy browser's seeded, 10-second capture, 24 of 3,384 ordinary moving bot samples faced more than about 49 degrees away from travel before the change. Afterward, 0 of 3,373 did. A sample spans 50 ms. The same capture had zero stationary samples with strong steering input in both versions, so it did not reproduce prolonged wall bumps.

[Before clip](before.webm) · [After clip](after.webm) · [In-game screenshots of all seven posters](walls.png)

Each flag count has seven separate wall mounts. The screenshots show the legacy browser's two-flag map at eye height. Art was drawn for Australian BBQ.
