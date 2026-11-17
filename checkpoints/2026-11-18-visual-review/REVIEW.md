# Native visual review, 18 November 2026

Reviewed the published 0.23.33 source at 5b15544 after the Mac was unlocked. The optimized desktop binary matches that release.

## Walking and turning
Each of Backyard, Bush Servo and Teddy Heist ran the scripted walking review with the real movement system and continuously changing yaw and pitch. Asset warmup preceded recording. Each GIF contains 75 frames over approximately five seconds, with contact sheets and selected full-size frames retained.

The inspected native frame sequences showed no recurrence of the large dark grass-square patches and no large surfaces flashing between materials. Fine grass and small foliage detail remains visually busy at distance. This is a short sampled review at Polished/Pop on this Mac. Longer routes, Low/High comparisons and browser mouse-look remain separate hands-on checks.

Heist's corrected capture moves alongside a maze panel and around its end into a clear lane. Its turf is dark relative to the walls, making floor detail difficult to read. This contrast is a priority for the next graphics pass.

## Character view and traversal
- short-view.png and tall-view.png use the same held-item review position, yaw and pitch, with leg length at zero and 100 respectively. The taller setting raises the camera by the configured 0.60 m and changes the view over nearby objects.
- carry-view.png shows the external carried-player angle framing both people. The camera uses the previously implemented fixed world direction and smoothed follow. Marcus's comfort assessment during longer play is still needed.
- roof-route.png shows the service steps linking the truck cab and servo roof. The real-physics jump regression passed in the previous 739-test run.

No gameplay source changed during this review. No fresh two-person online match or browser walking check was performed during this follow-up.

## Next graphics priorities
1. Simplify distant grass and small-leaf silhouettes to reduce movement shimmer.
2. Lift Heist turf midtones and distinguish floor wear from maze wall shadows.
3. Add variation to neighbouring rooflines, verandahs and fence silhouettes.
4. Detail the servo roof approach with service markings, supported steps, worn edges and roof hardware.
5. Refine character foot contact and compare moving shadows on Low and High settings.
