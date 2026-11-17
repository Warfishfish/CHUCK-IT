# Movement and camera review, 17 November

## Implemented
- Host resolves character overlap with four separation passes and obstacle checks. Guests block their local movement against replicated bodies. Vertical separation and carry/drag exceptions support those interactions.
- Backyard uses 32 predefined spawn points, supplemented by a clear grid when starting rounds. Points are filtered against solids before shuffling. Each Heist base now has 25 distinct slots spaced one metre apart, replacing the four-slot loop.
- Two visible service steps lead from the Bush Servo truck cab to the shop roof. The roof collider follows the imported model's measured front and rear slopes. Jump physics were exercised from cab to step, step to step and step to roof.
- Carried camera follows a fixed world direction with exponential position and rotation smoothing and steady field of view. This removes carrier-turn orbiting. Comfort still needs Marcus's hands-on review.
- Local camera eye and held-item release position follow the character's head height plus leg lift.
- Backyard scoring inspected. A thrown character striking another awards 100. Releasing the throw alone has no bonus.

## Checks and limits
- Workspace tests passed, 739 passed and three ignored. Includes existing host/guest replication tests, a new real-tick overlap regression, and roof-route jump tests.
- Full native release and WASM builds completed. Legacy browser prototype version is bumped, while these gameplay updates target the full Rust browser at /rust/.
- Native walking review uses real movement input, collision and continuously changing yaw/pitch. Scripts wait for asset warmup. The first Backyard and Smoko frame sequences were inspected. No dark grass-square patches were seen in those samples. The first Heist attempt faced a base wall. The corrected review route uses a clear lane start and obstacle-aware input, but its final captures were black because the Mac locked. Those black captures were discarded. Final walking verification across all three maps is pending unlock.
- Carry screenshot records the external angle. Short/tall rendered comparison is pending unlock.
- Browser launch and a four-team Heist round succeed. The live feed showed bot steals/banks. Automated mouse capture was rejected, so browser walking awaits a physical click. Computer control subsequently confirmed that the Mac was locked.
- A fresh two-human online match was not run.

## Suggested next graphics work
1. Reduce distant grass and small-leaf shimmer with simpler distant silhouettes and controlled fade.
2. Give Heist turf and maze bases a clearer midtone separation so dark floor detail remains readable.
3. Break up the repeating neighbour rooflines with a few recognisable chimney, verandah and fence silhouettes.
4. Improve servo approach dressing around the new roof route with worn step edges, service markings and roof hardware.
5. Refine contact shadows and grounded feet for close character encounters, then repeat walking review at Low and High settings.
