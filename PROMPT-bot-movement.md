# Prompt: make the bots' movement more natural

Paste everything below into a new chat (with `index.html` and `CHECKLIST.md` attached, or the repo connected).

---

You are working on **Australian BBQ**, my single-file browser game (`chuck-it-server/public/index.html`, Three.js). Read `CHECKLIST.md` first, then read the bot code.

## Job
Make the **bots' movement look and feel more natural**, like a mate who is a bit drunk and distracted, not a robot following waypoints. Do not change how hard the bots are to play against unless I say so. Do not touch scoring, items or the networking rules.

## Where the bot code is
- `aiUpdate(ch, dt)` is the main bot brain. It sets `ch.wish` (a unit-ish 2D move direction), then `updateChar` turns that into movement. Bots only run on the host.
- Goals come from `pickTarget`, `botErrand`, `botPickErrand`, `botHelp` and, in Teddy Heist, `heistBotGoal` (with `heistVia`, `heistGate`, `heistLeave`).
- Obstacle help: `blocked(x, z, y)` and `los(a, b)`. Colliders are boxes in the `colliders` array.
- `DIFF` holds the Easy / Fair dinkum / Spicy settings.

## What looks fake right now (check each one yourself first and tell me what you find)
1. Bots set `wish` straight at the goal, so they turn instantly and walk in dead straight lines.
2. They stop and start abruptly, with no slowing down when they arrive or speeding up when they leave.
3. They orbit and flip direction on a timer rather than for a reason.
4. Obstacle avoidance tries a fixed set of angles, so they can twitch along walls, and in Heist they can still shuffle around door gaps.
5. They never look around, hesitate, change their mind, or react at different speeds.
6. Drunk bots do not sway or stumble like drunk players do.

## What I want
- **Smooth steering:** turn toward the goal over a short time (limit turn rate), ease in and out of speed, and no snapping 180s.
- **Natural paths:** gentle curves, rounding corners and obstacles early, a small amount of random wander so two bots never walk exactly the same line.
- **Human pauses:** short hesitation after being hit or after a goal changes, a quick glance toward noise or a thrown item, occasionally stopping to "think". Keep it short so bots do not feel slow.
- **Personality:** each bot gets its own small random traits (reaction time, how much it wanders, how often it pauses), kept for the whole game.
- **Drunk bots** sway and stumble in proportion to their drunk level, the same way players do. In Drunk mode this should be visible on every bot.
- **No glitching:** no jitter on walls, bases, chairs or the pool edge; no walking through other bots; never stuck for more than a second or two. Teddy Heist bases (guarding, raiding, leaving through the doorway) must still work.
- Must work with 1v1 and no bots, with 1 to 3 bots, and online (bots are simulated on the host, guests only see them).
- Keep the cost small: no heavy pathfinding every frame.

## How to work
1. First tell me in a few lines what you found and your plan. Then build it.
2. Add a few quick tests with the headless browser: count direction flips per bot over about 60 seconds (the old 4-team Heist game showed about 70 flips per bot before my last fix, about 5 after), longest stuck time, and a screenshot or two. Show me the before and after numbers.
3. Test solo, Teams, Teddy Heist (2 and 4 teams) and 2 players online.
4. Give me the complete updated `index.html` and `CHECKLIST.md`, with the version bumped and the change listed under "Already in the game".
5. Tell me plainly what you guessed or simplified.
6. End with this exact checklist:
   1. In Terminal, press Ctrl+C, then run `npm start`.
   2. Copy the new trycloudflare.com link and send it to everyone again.
   3. Everyone reloads the page.
