# Teddy Heist celebration and shortcuts

Opponent throws award 10 personal points and banking an enemy teddy awards 150. The team with the most banks wins. Equal bank counts are a draw. The end-of-round reveal shows the team result, and the end-of-match podium ranks players by points, then hits. The modern builds use network protocol 30 for match totals.

Dazza starts taking requests after 30 seconds. The drink handover starts an escort. An abandoned request or escort times out after 30 seconds without progress. An opening lasts 12 seconds, counts against the requesting team's one opening per round, and starts a 45-second cooldown. Any player near the open hatch can follow. Each tunnel is one-way from the hatch to a gate. A runner already below ground can leave after the hatch shuts. In Cheeky mode, beer and cider increase Dazza's wobble. Water and soda reduce it. Extra drinks don't spend or renew an opening.

## Visual review

- [Team result](team-result.png) and [20-participant podium](podium.png)
- [Crate vault](vault.png), [open hatch](hatch.png) and [underground junction](junction.png)
- Dazza's [first drink](first-drink.png), [tipsy stage](tipsy.png), [wobbly stage](wobbly.png) and [water handover](water.png)
- Player-height legacy replays showing a [fork](fork.webm), [corner](corner.webm), [opposing traffic](traffic.webm) and [carrier return](carrier.webm)

## Test coverage

- 744 Rust workspace tests passed with 3 ignored. Collision tests cleared all six vaults with and without a teddy and all nine tunnel exits with the hatch closed behind the carrier. The three 20-participant Heist runs banked 8, 12 and 16 teddies, respectively.
- The legacy browser fixture ran all three layouts and opened the hatch for 12 seconds per successful escort. The results fixture gave a player on the losing team the highest personal score. Separate host and guest sessions plus a late join verified replicated hatch state, a second player handing over cider, and the allowance staying at one use.
- In the two-flag legacy bot replay, both builds sent bots to two different exits from the same junction. The first carrier got home around second 73 in both runs. Neither run registered a contact event of two seconds under the sampler's definition of a bot near a wall, moving less than 0.08 m over 0.2 seconds while trying to move. These observations are from simulated input and camera replays. They do not measure every obstruction in a live match.
- The Rust game currently offers Heist in solo mode. Legacy Heist supports a host and guests. Human playtesting of the full scenario matrix remains outstanding.
