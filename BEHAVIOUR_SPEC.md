# Australian BBQ: behaviour spec (from the JavaScript game)

Every tuning number and rule in `public/index.html` that affects how the game plays, taken from version **0.23.1** (tag `v0.23.1-js`). The Rust version ports from this file, so if a number changes in Rust it should change here first.

**How to read it:** `file:line` references point at `public/index.html` at the tag. "Host" means the machine that runs the rules (in the browser version, whoever hosts; in solo it's you). Units are metres and seconds. Where the code and the on-screen text disagree, the code wins and the mismatch is listed in section 20.

**What this covers:** the rules, scoring, physics, movement, items, bots, Dazza, Heist and match flow.
**What it doesn't cover:** the network message formats (lines 2445-2672), the 3D models, textures, canvas drawing, the HUD layout and the synthesised sounds. Those are rebuilt, not ported number-for-number; see section 21.

---

## 1. The yard and the clock

| Thing | Value | Source |
|---|---|---|
| Yard half-size | `W=33` (x), `D=24` (z), so 66 x 48 m | 641 |
| Item gravity | `GRAV=18` (times each item's own `grav` factor) | 641 |
| Character gravity | `24` | 2188 |
| Character radius | `0.42` | 2189 |
| Pool | x -27..-13, z 6..14, water surface y -0.16, depth 1.55, sink 1.0 | 642 |
| Trampoline | centre (15, 12), radius 1.8, top height 0.6 | 643 |
| Bar | centre (0, -21.5), 3.6 x 0.9, height 1.05 | 644 |
| Meat table | centre (-8.7, -18.2), 1.5 x 0.8, height 0.8 | 645 |
| BBQ grill | centre (-6, -18), collider 1.7 x 0.8, height 1.25 | 793 |
| Smoko pad | centre (-24, -3) | 1101 |
| Max frame step | 0.05 s (the loop is not fixed-timestep) | 3044 |
| Round length | chosen in the menu (180 s default variable) | 1316 |
| Countdown before a round | 3.2 s (characters frozen) | 2939 |
| "TIME!" to results panel | results panel opens 1.1 s after the whistle | 2948 |
| Online game over, back to lobby | 9 s after the final whistle | 2949 |
| Last-10-seconds beeps | one per second | 3049 |

### Static colliders (box on the ground: x0,x1,z0,z1,height)
Clothesline pole (0,3) 0.3x0.3 h2.3 **noTop**; shed (22.5,-16.5) 4x3 h2.6; eskies at (-7.5,-4.5), (9,-6), (-3,18), (24,6) (rotated, h0.62); outdoor table (6,-16.5) 2x1 h0.8; BBQ (-6,-18) 1.7x0.8 h1.25; wheelie bins (31.5,19.5) 1.7x0.85 h1.1; crates (-25.5,-13.5) 2.4x1.2 h2.3; hedge (-31.6,-3) 1.2x9 h1.3; tyres (12,2) 1.2x1.2 h1.0 and (13.2,2.4) h0.68; woodpile (-12,-3) 2.6x1.05 h1.0; brick wall (20,-4) 4.5x0.35 h1.1; veggie planter (-8,20) 5x0.9 h0.95; bar (0,-21.5) 3.6x0.9 h1.05; meat table. The yard fence is at the edge of W/D (not a collider box).
The bar, BBQ (grill and meat table), chest and smoko each can be switched off; switching off hides them and removes their colliders (lines 650-666).

### Spawn points
- Item spawn spots: 25 fixed (x,z) pairs (line 845-846).
- Character spawn spots: 12 fixed pairs (line 1307): (0,19.5), (-24,-19.5), (27,-9), (24,16.5), (-27,15), (12,-19.5), (-9,-10.5), (30,0), (-18,-6), (18,4), (-6,14), (6,-8). Shuffled each round and handed out in order.
- Chest spots: 9 fixed (x,z,rotation) triples (line 648). Each round picks a new one (never the same as last time).

---

## 2. Player input and movement

### Controls (lines 2694-2730)
| Input | Action |
|---|---|
| W A S D / arrows | move (relative to where you look) |
| Mouse | look. Pitch limited to +-1.45 rad. Sensitivity `0.0022` rad per pixel |
| Left mouse (hold, release) | wind up and throw |
| Right mouse | catch |
| Space | jump; wriggle when being dragged; stand up from a smoko chair |
| Shift | speed boost |
| Q or E, mouse wheel | swap item (also 1 and 2 pick slot 1 or 2) |
| R | interact (drink, sit at smoko, grab meat, grab from chest, eat snag, help someone up when held) |
| F | grab / put down (tap) / throw (hold) someone who is down; otherwise starts a throw wind-up |
| T, G, B | taunt, dance, laugh emotes |
| P | pause (solo); Esc pauses only when mouse capture isn't available |
- Touch: left thumb moves, right thumb aims; on-screen buttons for Chuck, Catch, Swap, Jump, Boost, Emote, Drink.
- There is no gamepad support.
- Fallback when the mouse can't be captured: drag to aim at 1.4x the movement, hold F to wind up.

### Wish direction (line 2754-2762)
`wish = (cos(yaw)*mx - sin(yaw)*my, -sin(yaw)*mx - cos(yaw)*my)`, limited to length 1. You face `yaw + PI`. While sitting at smoko or being dragged, input is zeroed. In a hosted yard with the panel open, input is zeroed.
**Drunk steering:** when drunk (`da>0`) and moving, the wish vector is rotated by `th = da * (sin(t*0.5)*0.5 + sin(t*1.37+1)*0.2)`.

### Speed (updateChar, line 2160-2186)
`maxSp = 6.2 * pool(0.5) * charging(0.72) * sprint(1.7) * drinking(0.55) * snagBuff(1.35) * snagStumble(0.62) * carrying(0.55) * drunkGait`
Each factor applies only while its condition is true. `tx,tz = wish * maxSp`, forced to 0 when stunned or during the countdown.
**Velocity easing:** `vel += (target - vel) * (1 - exp(-grip*dt))` with grip:
| Situation | grip |
|---|---|
| On the ground, normal | 12 |
| In the air | 6 (2.2 if stunned) |
| Stunned on the ground | 3.2 |
| Sliding on a puddle (SLIDE) | 0.3 |
| Standing in a puddle | 1.1 |
| Snag stumble | 2.4 |
(The old dodge code sets velocity to 14.5 m/s along a direction for `dodgeT`, but nothing can start a dodge any more: the dodge is removed.)

### Jump, boost, bounce
- **Jump:** `JUMP_V = 7.5` (5.5 in the pool). Pressing sets a 0.14 s input buffer; there is 0.10 s of "coyote time" after leaving the ground. Needs not stunned and not in the countdown.
- **Boost:** `BOOST_MULT=1.7`, lasts `1.6 s`, then `5 s` cooldown. Not allowed while being carried, at smoko, stunned, or during the countdown. Boosting widens the camera FOV by 7 (eased at `1-exp(-8dt)`).
- **Trampoline:** if landing inside radius 1.8 at height <= 0.62: `vel.y = 11.5 * (1 + 0.2*n)`, where `n` counts bounces in a row without touching the grass, capped at `TRAMP_MAX=5` (so the biggest bounce is 23 m/s). Camera shake `0.03+0.015n`. The chain resets when you touch any floor.
- **Pool:** you're "in the pool" when inside the rectangle and `y < 0.1`. You sink to `POOL_SINK=1.0` (eased at `1-exp(-7dt)`), move at half speed, and jump lower.
- **Landing shake:** `vel.y < -10` gives shake 0.05.

### Collision with the yard (line 2190-2203)
- Boxes: if the character was at or above the box top (`y >= h-0.05` this frame or last), they land on it; otherwise they're pushed out horizontally by the shortest way and the velocity into the box is removed. The pole (`noTop`) can't be stood on: you only collide with it below `h+1.5`.
- Yard edge: `x` clamped to +-(33-0.45), `z` to +-(24-0.45).
- Floor under you = the highest box top you're over, if you're at least `h-0.35` high (or were `h-0.05` high last frame). So you can step up onto things when you're jumping or landing on them.
- **Characters push each other apart:** within 0.9 m (and under 1.5 m apart vertically), each moves half the overlap (all of it if the other is a remote player).

### Pick-ups (line 2205-2208)
Automatic when you're not stunned, not in the countdown, holding fewer than 2, and the item is `state=ground`, rested at least 0.2 s, within **1.1 m** horizontally and within 1.4 m of your chest height (`y+0.4`). Bots never pick up melee items (steak, fish, noodle, dildo). An item you were forced to drop can't be picked by you for 2.5 s (over-hold).

### Wind-up, throw, over-hold (lines 2679-2691, 2213-2217)
- Wind-up fills `charge += dt / item.charge` (see item table) up to 1. While winding up, speed x0.72.
- You can't start a wind-up while stunned (unless knocked flat), drinking, at smoko, or carrying someone.
- **Over-hold:** if charge stays at 1 for more than **1.5 s**, you drop the item ("Held it too long!") and can't pick it up for 2.5 s.
- **Throw velocity:** aim direction (camera forward with `y += 0.07`, normalised) times `item.speed * (0.42 + 0.58*charge)`, plus 0.3 x your horizontal velocity.
- **Power throw:** charge >= **0.90** marks the throw as a power throw. Only players can (bots never pass a charge).
- Throw start point: right-hand position `(0.28, -0.22, -0.55)` in camera space.
- Items with `throwable` melee (the noodle): tapping (under 0.2 s) slaps instead of throwing.
- You can still throw or slap while knocked flat if holding something; bare-handed attacks and catches don't work lying down.
- Switching item cancels a wind-up.

### Catch (line 2129, 2008-2009)
Right-click starts a **0.32 s** catch window, **0.9 s** cooldown. Needs not stunned, down, fallen or at smoko. A thrown item that touches you while the window is open and you're facing it (`dot(forward, toItem) > 0.1`) is caught. A catch gives +50 (not in Heist), resets the thrower's streak, and if you already hold 2 items, drops the oldest.

---

## 3. Items

| Key | Name | speed | knock | charge (s) | radius | bounce | grav | spawn weight | other |
|---|---|---|---|---|---|---|---|---|---|
| teddy | Teddy Bear | 24 | 6 | 0.55 | 0.24 | 0.5 | 1 | 45 | |
| stubby | VP Can | 27 | 8 | 0.7 | 0.14 | 0 | 1 | 35 | smashes; leaves a puddle |
| gnome | Garden Gnome | 19 | 15 | 1.0 | 0.28 | 0.25 | 1.15 | 20 | stun 0.65 s (explicit) |
| dildo | Purple Dildo (Cheeky mode) | 23 | 9 | 0.75 | 0.2 | 0.6 | 1 | 0 | melee "down"; knockdown |
| steak | Raw Steak | 20 | 3 | 0.6 | 0.18 | 0.1 | 1 | 0 | melee "stun"; from the BBQ |
| fish | Raw Fish | 20 | 3 | 0.6 | 0.18 | 0.2 | 1 | 0 | melee "stun"; from the BBQ |
| noodle | Pool Noodle | 21 | 5 | 0.6 | 0.2 | 0.45 | 0.85 | 0 | melee "stun"; throwable; uses 6; slapStun 1.2; slapPts 50 |
| snag | Dodgy Snag | 20 | 3 | 0.6 | 0.18 | 0.1 | 1 | 0 | retired, not spawned. Eat (R) for a speed buff |
Source: lines 573-590. Weights (45/35/20) are used by `randomType()` (line 1725); items with weight 0 never spawn from the random spawner.

### Where each item comes from
- **Teddy, VP, gnome:** random spawner and initial spawn (first three of the initial spawn are exactly teddy, stubby, gnome, then random).
- **Steak, fish:** taken from the meat table (R, within 3.2 m on the host check; left half of the table gives steak, right half gives fish; 0.8 s between grabs; max 2 held). Each has **3 uses** (`SLAP_USES`). Taking meat makes Dazza angrier (section 12).
- **Noodle:** two live in the pool; a new one is made every 4 s if fewer than 2 exist. Spawn inside the pool rect at y 0.6.
- **Dildo:** from the chest (Cheeky mode only), 3 uses each.

### Spawner (line 2120-2126, 2936-2937)
- Target item count: `min(22, 6 + players * 1.6)`.
- Every **1.3 s**, if below target, pick a random spawn spot that has no non-held item within 2.2 m (in Heist also at least 1.2 m clear of walls) and drop a random-type item from a random height 7..10 m, with jitter +-0.4 m.
- Start of a round: all items and puddles cleared, then `8 + floor(players*1.5)` items placed (the first 3 are teddy, stubby, gnome), plus 2 noodles. Noodle timer starts at 4 s.

### Item flight and landing (lines 2018-2073)
- A flying item moves in `ceil(speed*dt/0.12)` sub-steps so it can't tunnel.
- Each sub-step: `vel.y -= 18 * item.grav * h`, move, check for a body hit (if the item is live), then collide with the world.
- **Spin:** rotates about a random axis at `min(18, speed*0.6)` rad/s (gnome x0.6).
- **Hit a wall or box side:** the VP smashes if speed > 4. Otherwise `vel[axis] *= -(bounce + 0.15)` then all velocity x0.8. A "thud" (or "clonk" for the gnome) plays if speed > 5.
- **Land on a surface:** if speed > 5 the VP smashes. Otherwise it stops being "live" (can't hit anyone). If `vel.y < -2.5` it bounces: `vel.y = -vel.y * bounce`, horizontal x0.6. Else `vel.y = 0`, horizontal x0.85, and it comes to rest when horizontal speed < 1.2. Landing on a box top above 1.3 m high with a bounce-off: it's shoved away from the box centre at 4 m/s, up 2.5.
- **Trampoline:** an item falling onto it gets `vel.y = max(7, -vel.y*0.9)` and a random +-1 sideways kick.
- **Pool:** an item landing in the pool floats (sits at the water surface and bobs). A VP landing in the pool is removed (no puddle).
- **Out of the yard** (|x|>33 or |z|>24 when landing, or y<-4, or |x|,|z|>60): removed, and the thrower gets an "over" event. Heist teddies are sent home instead.
- **Wall bounce at the fence:** x or z beyond (edge - r) below 1.8 m high bounces back like a wall.
- **Rest heights:** VP 0.07, gnome 0.28, dildo 0.14, noodle 0.08, steak 0.03, fish 0.17, otherwise the item's radius.
- **Smash:** removes the item, bursts glass, and leaves a **puddle** if it was under 1.4 m high and not in the pool.
- **Puddle:** radius random 1.1..1.5, lasts 14 s, grows to full size in 0.25 s (`grow += dt*4`), fades during the last 2 s. Standing on one gives grip 1.1.
- **Slide:** stepping onto a puddle at speed > 2.4 (not sitting, not down) while grounded and not already sliding starts a **1.1 s slide**: speed boosted to `min(11, speed*1.35)`, spin 7..11 rad/s, can't steer (grip 0.3), cancels a wind-up.

### Hit detection (lines 2000-2015)
Each player's own machine checks whether a live item touches its own character: horizontal distance < `0.45 + item.r`, and item height relative to the character between `-r` and `1.85 + r`. The host validates (below). Things that make a hit pass straight through:
- the victim is at smoko (untouchable);
- the thrower is a teammate and friendly fire is off;
- the thrower themself; or someone this item already passed through (remembered per item).
Catch is checked before the hit.

### What a hit does (authHit, applyKnock; lines 1770-1790, 1839-1850)
- **Validation:** a hit counts once per throw ("flight"). A hit reported after the item has landed must arrive within **1.8 s** (`LATE_HIT`) of landing.
- **Knockback:** `vel += dir * knock` (items; `13` for Sent Flying, `2.5` for Timber; times any multiplier). `vel.y = max(vel.y, knock*0.38)` for plain item hits (5.5 for Sent Flying, 2.5 for the other slaps). Victim becomes airborne.
- **Stun:** `stun = max(stun, def.stun ?? 0.35 + knock*0.035)`. That works out to: teddy 0.56, VP 0.63, dildo 0.665, steak 0.455, fish 0.455, noodle 0.525, gnome 0.65 (explicit). **No stacking:** if the victim is already stunned or in the 1 s grace period, no new stun starts (they are still shoved).
- **Stun grace:** when a stun ends, `stunGrace = 1 s` during which a new stun can't start.
- A hit cancels a wind-up and (for bots) the bot's wind-up.
- **Power throw hit:** if the item was power-thrown and the victim isn't in stun grace, down or fallen, they're **knocked flat for 2 s** (`POWER_DOWN=2`, flagged as knockdown). Otherwise it's a normal hit.
- **Item after a hit:** a VP smashes; anything else loses its "live" state and bounces away at `(-vx*0.2, 3.5, -vz*0.2)`.
- In Heist, a hit (or any knock) on someone carrying a teddy makes them drop it.
- **Knockdown** (power throw, dildo slap, human cannonball, etc.): `downT = stun = duration`, drops the drink they were holding.
- `DOWN_TIME = 3.5` s default knockdown.

### Scoring a hit (free for all and teams; lines 1770-1790)
```
streak = thrower.streak + 1          (thrower's streak resets on any hit they take, and when they're caught)
streakMult = 2 if streak>=5, 1.5 if streak>=3, else 1
base       = round((100 + (victim is leader ? 50 : 0)) * streakMult)
gain       = base + drunkBonus(thrower) + (Bum-Out gnome ? 15 : 0) + longShot
longShot   = round( charge * clamp((dist - 8) / (25 - 8), 0, 1) * 100 / 5 ) * 5      // dist from where the throw started
thrower.score += gain; thrower.hits++
victim.score  -= 50;   victim.taken++; victim.streak = 0
```
- **Friendly fire on:** a hit on a teammate gives the thrower nothing, but the victim still loses 50 and gets a "taken" (lines 1779-1785).
- Scoring only counts while the round state is `play` (not countdown, warm-up or results).
- Hit streak pop-ups at 3, 5 and 10.
- **Drunk bonus** (`drunkBonus`): thrower's drunk meter >= 40 gives +50; >= 15 gives +25; 0 in Drunk mode.
- **Leader:** the player with the top score if it's above 0 and strictly higher than second place (line 2794). The leader wears a crown and is worth a +50 bounty (before the streak multiplier). Bots favour them (x2.8).

### Melee slaps (lines 1020-1062)
- **Attacker check (client):** target within **2.6 m** horizontally and 1.6 m vertically, within a cone (`dot(facing, toTarget) >= 0.4`), not down (`downT > 0.3`), not at smoko, not a teammate (unless friendly fire). The nearest wins. Dazza is also checked.
- **Cooldown:** 0.55 s armed, 0.9 s unarmed (client); the host also enforces **0.45 s**. Swing animation lasts 0.28 s.
- **Host range check:** attacker within **3.6 m** of the victim.
- **Steak / fish / noodle ("stun" slap):** victim gets a dizzy stun `ST` = 2 s (`MEAT_STUN`) for steak/fish or the noodle's `slapStun` 1.2 s; the attacker gets `slapPts` (noodle 50, else `MEAT_PTS`=50) + drunk bonus. No penalty to the victim and no streak change. Uses drop by 1 and the item is removed at 0 ("poof"). Knockdown none. Only scores outside Heist.
  - **Fish smell:** each fish slap: 10% chance "THAT FISH WAS OFF" (bad cloud, 9 s), otherwise 33% chance of a mild cloud (5 s), else none. Pure joke.
- **Dildo slap ("down" melee):** the victim must not already be down. Scores like a hit: `round((100 + bounty)*streakMult) + drunkBonus + variantPoints + (crit ? 60 : 0)`; victim loses 50. Outcome (Sent Flying / Cartwheel / Timber) is picked uniformly at random. **Critical:** 15% chance, flattens for 5 s with knockback x1.9.
- **Dildo variants** (chosen at random each time the chest is used, weights 46/28/18/4):

| Variant | Name | scale | points | knockdown time |
|---|---|---|---|---|
| dildo | Purple Dildo | 1.0 | 0 | max(1.2, 3.5+0) = 3.5 s |
| dildoMini | Pocket Rocket | 0.66 | -25 | max(1.2, 3.5-1) = 2.5 s |
| dildoJumbo | The Unit | 1.38 | +50 | 4.5 s |
| dildoGold | Golden Wonder | 1.12 | +120 | 5.5 s (crit: 5 s) |
  The Golden Wonder comes with a "MYSTERY CHEST" announcement.
- **Bare-handed slap (Cheeky mode only):** wet willy / noogie / wedgie. Target within 2.9 m; attacker not down; victim not down; **+40** (`SILLY_PTS`) + drunk bonus; knockback 8.5, up 3, stun 0.6, no knockdown. The victim loses nothing and no streak. Cooldown 0.9 s.
- **Sent Flying / Cartwheel / Timber:** animation pose is purely a function of time since the slap (lines 1064-1085), so every machine plays it the same.

---

## 4. Drinking, drunk meter, stacking it

- **Drinks** (from the bar, R while standing in front; bar spot: within `w/2+0.8` of the centre in x, 0 to `d/2+1.6` in front in z, on the ground):

| Drink | Where | Adds | Time |
|---|---|---|---|
| VP (beer) | left third | 18 | 1.4 s |
| Wine | middle | 26 | 1.2 s |
| Rum shot | right third | 34 | 0.6 s |
  While drinking, speed x0.55; getting stunned cancels the drink. The meter is 0..100.
- **Sobering:** `-1.6 per second`, but not while sitting at smoko.
- **Tiers:** Sober < 15, Tipsy < 40, Drunk < 70, Maggot < 90, Absolutely maggoted above that.
- **Drunk amount** `da = smoothstep((drunk - 15) / 70)`, 0 below 15, 1 at 85. It drives: steering twist, camera sway, character lean/roll, wobbly vision shader, bot aim error (+160% x da).
- **Falls (normal):** only the local player. Needs: falls feature on, `da >= 0.35`, grounded, not in the pool, not knocked down, not in fall immunity. Chance per second is `da^1.5 * (moving ? 0.028 : 0.008)`.
- **Drunk mode** (`drunkall`): everyone (bots too) is held at **at least 78 drunk** from the countdown on; no drunk bonus points. Falls are `4%` every full 2 s of walking (only real players fall, not bots). Walking speed wanders (below). Fall immunity afterwards is 6 s (25 s normally).
- **Stacked it:** you're down for `fallDur` (10 s by default; host can choose 20 or 30). You can't throw or slap bare-handed, and you hold a "HELP ME" sign. When you get up you get **25 s** immunity (6 s in Drunk mode). If someone helps you up: immune 25 s.
- **Drunk-mode gait** (`drunkGait`): a state machine scaled by `da`. Each state lasts a random time: **lurch** (target speed x1.32, 0.3-0.7 s, picked 42% of the time), **slow** (x0.5, 0.4-1.1 s, 38%), otherwise **steady** (x0.88, 0.6-1.8 s). Normal waiting between changes 1-3 s. Plus a wave `1 + 0.12 sin(2.1t) + 0.07 sin(5.3t)`. Result is `1 + (k-1)*da` clamped to 0.35..1.45. Speeds up with `1-exp(-10dt)` into a lurch, `-5dt` otherwise.
- **Help someone up:** hold R next to a fallen mate for **1.5 s** (`HELP_TIME`), within 1.9 m (client) / 2.8 m (host), 1 s cooldown; **+25** (`HELP_PTS`) outside Heist. In team modes you can only help teammates. Bots in team modes walk over and help.
- **Fall alerts:** when anyone else falls, a message and alarm go to you (help wording if you can help them, gloat wording otherwise). Eight help lines and four gloat lines (lines 1090-1093).

---

## 5. Smoko and the Naughty Corner

- **Smoko pad** at (-24, -3). Number of chairs = `clamp(players, 2, 16)` on a ring of radius `1.9 + n*0.2`; the zone radius is ring + 1.3. Chair `i` sits at angle `i/n * 2PI + 0.4`.
- **Sit:** press R inside the zone (nearest free chair). While seated: untouchable (hits pass through), can't move/throw/drink, and your drunk meter rises at **+4 per second** (`SMOKO.sip`). Smoko ends after **20 s** (`SMOKO.maxT`) or by pressing Space/R, or if stunned.
- Dazza gives up chasing someone who sits down.
- **Naughty Corner mode** (needs smoko on): the pad turns pink, nobody can sit for a safe break, bots stop going there. Dropping or throwing a carried player into the zone sends them to a chair for **5 s** (`NAUGHTY.sit`), can't move; the sender gets **+100** (`NAUGHTY.pts`) unless it's a teammate (or in Heist). A chucked player must land (grounded, >0.15 s after the throw, within 3 s) inside the zone to count. Someone already seated or in the corner can't be picked up or sent again.

---

## 6. Grab, drag and throw people

All numbers from `CARRY` (line 1192): `max 6, reach 1.9, wig 0.8, immune 8, hold 0.4, slow 0.55, drag 1.15, hv 10.5, vy 8.5`.
- **Who can be grabbed:** only someone actually **down** (knocked flat or stacked it), not merely stunned; on the ground (`y < 0.6`); not in the pool; not already carried or carrying; not at smoko or in the naughty corner. The grabber must not be stunned, down, fallen, in the pool, carried, or carrying. Works in play and warm-up only.
- **Range:** 1.9 m to start (client), 2.8 m (host check). The grab target is the nearest valid person within 1.9 m.
- Grabbing drops anything the victim was holding (including a Heist teddy).
- **While dragged:** the victim is pinned `1.15 m` behind the grabber, copies their velocity, can't act, and takes immunity from re-grab for 8 s after release. Dragging slows the grabber to x0.55, and they can't sprint, throw or slap.
- **Wriggle:** every Space press by the victim (at most one per 0.08 s) adds 0.8 s to the drag timer; the drag ends when the timer reaches **6 s**. With about 8 presses that's a "WRIGGLED FREE!". Bots wriggle by themselves (timer advances 1.7x).
- **Release outcomes:** tap F (under 0.4 s) = "put down" (stun 0.5 s); hold F (0.4 s+) = throw; timeout with no wriggles = "dropped" (stun 0.3 s); wriggle/bot timeout = "wriggled free"; grabber stunned/knocked down/falls/enters the pool, or anyone leaves = forced drop.
- **Throw (human cannonball):** velocity `10.5` along the grabber's facing, `8.5` up. The thrown person gets stun 1.2 s and is placed 0.8 m in front of the thrower at +1 m height. If within **1.05 m** horizontally (and 1.5 m vertically) of someone else (not the thrower, not seated, not down, not carried) between 0.08 s and 1.2 s after the throw, that someone is knocked flat for **2.2 s**; the thrower gets **+100** (not for teammates unless friendly fire). The flying person bounces back at -25% velocity. Each thrown person can hit only once.
- Throwing a person into the pool or onto the trampoline still scores the splash/orbit bonus.
- Bots can be grabbed (they wriggle free); bots don't grab anyone.

### Pool and trampoline bonus (lines 1145-1149)
If a character enters the pool or bounces on the trampoline within **3.5 s** (host) / 2 s (client check) of being hit by someone else (not a teammate), that someone gets **+50** (`ENV_PTS`): "SPLASHDOWN!" or "INTO ORBIT!". Not in Heist.

---

## 7. Emotes
Taunt (T), dance (G), laugh (B). Durations: dance 2.4 s, laugh 1.7 s, taunt 1.4 s. Global cooldown **2 s**. Can't emote while stunned or fallen. A random line is chosen (9 taunts, 4 laughs, 3 dances, plus 3 + 2 + 2 more cheeky ones in Cheeky mode; lines 1151-1154). Speech bubble lasts 2.6 s.

---

## 8. Chest (Cheeky mode)

- Max stock **3**, starts each round with **2**, refills 1 every **12 s** (only while below 3).
- R within 2 m of the chest takes one (host: within 3 m); needs fewer than 2 held. Variant picked by weights (see section 3). The item has 3 uses.
- Moves to a new spot each round (9 spots).

---

## 9. Scoring summary (free for all and teams)

| Event | Points |
|---|---|
| Hit with a thrown item | 100 (+50 if victim is leader) x streak multiplier, + drunk bonus (25/50) + long-shot (0-100, multiples of 5) + 15 for a Bum-Out gnome |
| Victim of a hit | -50, streak reset |
| Catch | +50, resets thrower's streak |
| Steak / fish slap | +50 + drunk bonus |
| Noodle slap | +50 + drunk bonus |
| Dildo slap | as a hit, + variant points (-25, 0, +50, +120) + 60 if critical |
| Bare-handed slap (Cheeky) | +40 + drunk bonus |
| Human cannonball | +100 |
| Pool / trampoline bonus | +50 |
| Naughty Corner | +100 |
| Help a mate up | +25 |
| Stun Dazza with steak/fish | +20 |
| KO / berserk / flip Dazza | +75 / +25 / +50 |
| Bum-Out gnome hit | +15 on top of a normal hit |
**Streak multiplier:** 3 hits in a row x1.5, 5 hits x2. Resets when you are hit or your throw is caught. Hits on teammates don't add to your streak.
**Heist:** the stated rule is that only banking scores. The code does not fully follow it: slaps, catches, pool/tramp, help-ups and so on are blocked in Heist, but a thrown-item hit (`authHit`, line 1770) has no Heist check, so it still pays the thrower and still costs the victim 50. See section 20, item 3.

---

## 10. Game modes, teams, matches

### Modes
- **Free for all:** everyone for themselves.
- **Teams:** Red / Blue, plus a Wildcard when the player count is odd. Friendly fire is an option (off by default). Teams are balanced automatically (`balanceTeams`): even numbers split Red/Blue, an odd one out becomes Wildcard (chosen at random when there isn't one yet); new joiners go to whichever of Red/Blue has fewer. The host can click a player to move them (Red -> Blue -> Wildcard -> Red) or shuffle.
  - Team score = sum of its members' scores. The Wildcard wins outright if their score beats both team totals.
  - Hits and throws pass through teammates (unless friendly fire); you can only help teammates up; bots help fallen teammates.
- **Teddy Heist:** 2 to 4 teams (Red, Blue, Green, Yellow). See section 13.

### Rounds and matches
- Round length: menu choice. Match length: best of 1, 3 or 5 (Heist is always 1 in solo). `matchNeed = floor(len/2)+1`.
- Round winner: highest team/player score; a tie (or 0) means nobody wins the round. Heist winner = most teddies banked. Ties are draws.
- Match ends when someone has `matchNeed` round wins or all rounds are played.
- Round start: items/puddles cleared, bots synced to the requested count, teams balanced, stats reset to 0, chest stock 2 and moved, Dazza reset, players teleported to shuffled spawn spots, countdown 3.2 s.
- Each countdown also resets your drunk meter, drink, fall state and smoko seat.
- Online: a warm-up (free play, no scoring) before the first round and after matches end.

---

## 11. Bots

### Difficulty (line 1308-1312)
| | err | react | catchP | cd | lead | spd | bias | juke |
|---|---|---|---|---|---|---|---|---|
| easy | 2.2 | 0.18 | 0.05 | 1.7 | 0.3 | 0.85 | 0.8 | 0.08 |
| fair | 1.3 | 0.42 | 0.14 | 1.15 | 0.75 | 0.93 | 1.0 | 0.15 |
| spicy | 0.6 | 0.7 | 0.28 | 0.8 | 1.0 | 1.0 | 1.25 | 0.22 |
Meaning: `err` aim error added to the throw velocity (x `1 + 1.6*drunk`); `react` chance to react to a thrown item; `catchP` chance to catch (as a share of react); `cd` multiplier on the gap between throws; `lead` how much they lead a moving target; `spd` throw speed multiplier; `bias` how much they favour real players over bots as targets; `juke` chance to dodge when they didn't "react".

### Brain (aiUpdate, lines 2343-2406)
Each bot carries personal habits for the whole game: `react` (0.14-0.32 s hesitation after being hit), `wob` (path wobble 0.1-0.26), `pause` (chance rate of stopping to look 0.02-0.055), `turn` (turn speed 5.5-8), `look` (head turn 6-10), `range` (7-13, preferred distance, re-rolled on retarget), orbit direction flips every 1.8-4.2 s.
- **Countdown / stunned:** do nothing. After a stun, a short hesitation (`react + 0.05..0.2`).
- **Targeting** (`pickTarget`): score `1/(dist+3)` x leader x2.8 x bias (real players) x random 0.6-1.4; stunned x0.7. Skips teammates and people at smoko. Retargets every 3-6 s.
- **With no item:** go to the nearest free item (+1.5 m penalty if it's on a box top; items other bots are heading for are skipped). If nothing to pick, ambles to a random point within x +-7, z +-6.
- **With items:** keep within `range`: advance if farther than `range+2.5`, back off if closer than `range-3`, and always orbit sideways (0.75). Detour to a pickup within 4.5 m if holding fewer than 2.
- **Throw:** if throw cooldown is up, target within 21 m with clear line of sight (no box taller than 1.6 m between), and holding something: wind up for `item.charge * rand(0.55, 1)`, then throw. Cooldown after: `rand(0.8, 1.9) * cd`. Aim is a ballistic solution with target lead, aimed at 1.0 m above the target's feet (lower if they're in the pool); bots never power-throw.
- **Dodging:** for each live incoming item within 9 m that's closing in under 0.5 s, once per item: with chance `react` they try (catch with chance `catchP/react`, otherwise 55% to sidestep, hop 55% of those); otherwise with chance `juke` they hop or sidestep. A glance at the item sets their facing for 0.35 s.
- **Errands** (not in Heist): every 14-26 s (first after 6-16 s), 65% of the time nothing; otherwise pick by weight from: bar (3, only if drunk < 55), chest (2.4, if stock and free hands and no melee held), BBQ meat (1.8, if free hands, no melee, and Dazza isn't chasing), smoko (1.6, if more than 1 free chair and not Naughty Corner mode). Each errand times out after 14 s. Bar: drink once; if still under 45 drunk, 40% to go again. Smoko: sit for 5-9 s.
- **Melee bots:** a bot holding steak/fish/noodle/dildo walks to the target and swings when within 2.1 m, every 0.9 s; won't slap someone already down (dildo).
- **Stuck check** every 1.2 s: if it has moved under 0.4 m while trying to walk, wander in a random direction for 0.9 s.
- **Steering** (`botSteer`): avoids other people within 1.5 m (push factor `(1.5-d)/1.5 * 0.7`); path wobble `wob * (0.6 sin(0.9t) + 0.25 sin(2.3t+...))`; drunk sway; slows to min 45% within 1.4 m of its goal; x0.35 while hesitating, x0.1 while pausing; looks ahead at 11 angles (0, +-0.45, +-0.9, +-1.35, +-1.8, +-2.5 rad) with probes 1.1 m and 0.55 m ahead and sticks to the side that worked; turning limited to `turn` rad/s (x2.4 within 2.2 m, x3 from standstill); speed eases at 4.5/s up, 8/s down; eases off through sharp turns (min 40%). Dodges are instant.
- Bots can be dragged, knocked, stunned like anyone. In team modes, a free bot helps up a fallen teammate (within 22 m).

---

## 12. Dazza (the BBQ cook)

Home (-6, 0, -19.3). States: `cook`, `angry`, `chase`, `return`, `ko`, `stunned`. Radius 0.45 against boxes; kept inside the fence (0.5 m margin).
- **Chat:** every 25-35 s while cooking (not during the countdown); first after 14 s. Speech bubble 2.8 s. Rude lines added in Cheeky mode.
- **Anger level** 0..8 (`ANGER_MAX`): goes up by 1 every time someone takes meat, and on each slap outcome of berserk/flip/stun. Reset at round start. Per-player grudge count: +1 per steal, decays at 1/25 per second; at 2.5 or more he chases.
- `chaseDur = clamp(8 + level*2, 8, 26)` s. `stunDur = max(0.4, 1 - level*0.08)` s.
- **Taking meat:** he turns "angry" (3.5 s) and speaks; if that player's grudge hits 2.5, he chases for `chaseDur`.
- **Speeds:** chase `7.2 + min(2.2, level*0.3)` (berserk: `8.6 + ...`); angry (going home or toward someone near the grill) 3.4; return 4. He stops 1.2 m from the target.
- **Angry:** if the target is within 5 m of home he walks to them, else he goes home route. 3.5 s then returns.
- **Home route:** he is behind the grill; going home from the front he walks round the open right side (x -4.3).
- **Spatula:** when chasing or angry, within 2.1 m, swing cooldown **1.3 s**, not on someone who is down; hit lands 0.22 s later if the victim is still within 2.7 m and not at smoko. Effect: knockback 7, dizzy stun **1.6 s** (**2.8 s** if berserk). After a spatula hit during a chase he goes home ("gotcha").
- **Gives up** and goes home if the target sits at smoko, the target leaves, or the chase time runs out.
- **Your slaps on Dazza** (need to be armed with a melee item, within 3.6 m on the host, within 2.6 m cone on the client; cooldown 0.45 s; not while he's KO'd or stunned):
  - **Steak / fish / noodle:** always a stun for `stunDur`, +20 points, then he chases for `chaseDur` after; level +1. The item loses a use.
  - **Dildo:** random outcome, equally likely: **KO** (down 8 s, grudges cleared, +75; in Cheeky mode a 40% chance a Bum-Out gnome pops out while any are left: max 3 per round), **berserk** (+25; he chases at once with a 1.2 s head start; level +1), **flip the barbie** (+50; 5 steaks/fish fly off the grill; angry 3.5 s; level +1).
- Wakes up after KO, says a line, returns home.
- Dazza points are only awarded in play. The 20-point stun is also blocked in Heist; the KO / berserk / flip points are not.

---

## 13. Teddy Heist

- Constants: base half-size 3 (6 x 6 square with one open side), wall thickness 0.35, wall height 2.1, base radius (banking zone) 4.3, **150 points** per banked teddy.
- **Layouts** (base centre and which side is open):
  - 2 teams: (-20,0) open +x; (20,0) open -x.
  - 3 teams: (-21,-10) open +x; (21,-10) open -x; (0,16) open -z.
  - 4 teams: (-21,-10) open +x; (21,-10) open -x; (-20,18) open +x; (21,18) open -x.
- **Arena:** an oval: half-axes A=40 (x), B=29 (z); the corners outside the ellipse are filled with 1 m stone steps (height 2.6, alternating colours), plus a half-round bay on the middle of each side (radius 6, depth 3.6). A crate cross in the middle (4.4 x 1.3 and 1.3 x 4.4, height 2.1) and two 1.6 m crates at (4.8, 2.8) and (-4.6, -3). Items spawn at least 1.2 m clear of walls; the chest spots are checked clear to 1.4 m.
- **Teddies:** each base gets `clamp(2 + floor((players / teams) / 2), 2, 4)` teddies at corners offset (+-1.3, +-1.3) from the base centre, each with a team flag. They are normal teddy items but flagged to a team.
- **Spawn:** each team's players spawn inside their own base at offsets (0, 2.1), (0, -2.1), (2.1, 0), (-2.1, 0) in turn.
- **Teams:** `assignHeistTeams`: each new player goes to the team with the fewest. Colours: Red, Blue, Green, Yellow.
- **Stealing:** walk over an enemy teddy to pick it up (normal pickup). Walking over your own team's teddy that's not at home sends it home: it jumps back to its base ("A stray teddy scurried back").
- **Home check:** touching your own teddy that's already within 2.6 m of its base does nothing.
- **Banking:** carry an enemy teddy into your own base radius (4.3 m from the centre). It's removed, your team's count goes up, and you score 150 (in play only). You can't bank while stunned or down.
- **Dropping:** being hit, slapped, knocked flat or knocked down while carrying a teddy drops it where you stand. The thief can still throw a stolen teddy to a mate (kept on purpose).
- A teddy that leaves the yard or falls out of the world returns to its base.
- **Win:** most teddies banked at the end of the round. Tied or 0 = draw.
- **Scoreboard:** a 7 m cube at (0, 11, 0), four faces show each team's teddies and points, with a crown on the leader (not on a tie).
- **Bots in Heist** (`heistBotGoal`), in priority order: (1) carrying a stolen teddy: go home (via the doorway if outside); (2) an enemy is carrying one of ours within 22 m: chase them; (3) one of ours is loose more than 2.8 m from home within 20 m: touch it; (4) the first bot on a team of 2 or more guards: chases any enemy within 10 m of home, otherwise patrols four spots 1.9 m from the base centre (corners at 45, 135, 225, 315 degrees); (5) otherwise raid the nearest enemy teddy (if hands aren't full), going via the doorway (point 1.2 m beyond the opening, `HEIST_HALF + 1.2` from centre) when it's inside a base. Bots inside a base leave through the doorway when heading elsewhere.

---

## 14. Smoko, bar and BBQ feature switches
`FEAT` keys (bit order): bar, bbq, chest, smoko, falls, adult (Cheeky mode), naughty, drunkall. Defaults on: bar, bbq, chest, smoko, falls. Off: adult, naughty, drunkall. The chest only exists when Cheeky mode is on (`ADULT_FEATS=['chest']`). `featMask` packs these into a bit mask that's shared with everyone.

---

## 15. Camera
- First person, eye height **1.55** above your feet; standing at smoko lowers by 0.45; knocked flat lowers by up to 1.2; stacked lowers by 1.25.
- Walk bob: `sin(walk*2) * 0.035` when grounded.
- FOV: `fovBase` (60..105, default 85, saved on the device) plus 7 when boosting.
- Screen shake decays at 0.5 per second; applied as random offsets of 0.3 (position) and 0.2 (rotation) times the shake.
- Drunk sway on yaw, pitch and roll scaled by `da`. A full-screen wobble/blur shader runs when `da > 0.01`.
- Dizzy (after steak/fish/noodle/spatula): roll wobble up to 0.09.
- Slap cameras (Sent Flying, Cartwheel, Timber) follow the pose animation (lines 1081-1085).

---

## 16. Character animation timings (ones that gameplay depends on)
- Throw swing: key-frame table `SWK`, visible for 0.54 s.
- Slap swing 0.28 s. Dazza's spatula swing 0.35 s.
- Knocked flat: lying pose eases in over 0.22 s and out over the last 0.45 s of the stun.
- Stacked: falls over in 0.3 s; gets up over 0.45 s.
- Walk cycle advances at `speed * 2.4` per second.
- Remote players are smoothed: position follows the network position plus up to 0.25 s of velocity extrapolation at `1 - exp(-16dt)`; facing at `-14dt`.

---

## 17. HUD numbers the rules depend on
- Hit marker flashes 0.16 s; catch ring 0.32 s.
- Feed shows 5 lines, each fades after 4.2 s.
- Floating text lasts 1.3 s (max 24 on screen).
- Hint text shows for the stated time (usually 2-6 s).
- Alerts last about 4.5 s.

---

## 18. Online rules the gameplay depends on (full detail is in the network code)
- Max 16 players in a room. Player state is sent 20 times a second.
- The host runs rules for everyone; each player decides whether they themselves were hit and reports it; the host checks a hit report arrives within 1.8 s of the landing.
- Guests see their own throws instantly as "ghost" items that disappear after 1.6 s if the host doesn't confirm.
- When the host leaves, the yard ends (no host migration).
(Network behaviour is not specified in detail here; Phase 9 starts with its own reading of lines 2445-2672.)

---

## 19. Behaviour that was left in on purpose
- A thief can throw a stolen teddy to a mate (Heist), flagged "watch in playtest".
- "Dodge" code is still in the file but unreachable (no key starts it).
- The snag item and eating code exist but snags no longer spawn.

---

## 20. Mismatches found while writing this (code vs text)
These are places where what the game says doesn't match what it does. The Rust version should follow the **code** unless Marcus says otherwise.
1. **Steak and fish stun length:** the in-game item list says "Slap: 3s stun" but the code uses `MEAT_STUN = 2` s (line 1018).
2. **Friendly fire:** when it's on, a teammate hit gives the thrower nothing, but the victim still loses 50 and a "taken" (lines 1779-1785).
3. **Thrown-item hits still score in Heist.** CHECKLIST.md and CLAUDE.md say Heist scores only from banking teddies, but `authHit` (line 1770) has no Heist check, so a thrown hit pays the thrower (100 plus bonuses) and costs the victim 50. Melee slaps, catches, pool/tramp, help-ups, Dazza stuns and cannonballs are correctly blocked. **Needs a decision from Marcus** before the Rust port copies it either way.
4. **Fall chance per second** is very low at full drunkenness (`0.028 * da^1.5` per second, roughly one fall every 36 s while walking at max drunkenness), which is much less than the "4% every 2 s" used by Drunk mode.
5. **Cheeky Dazza lines:** "Keep the Dazza chatter about every 30 s" is implemented as a random 25-35 s.
6. **Steak "3s stun" text** appears in the in-game How to play too (same as item 1).
7. **Heist per-round scoring pop-ups:** banking shows "+150" only when `state==='play'`.
8. `PROMPT.md` says "Backyard Brawl" in places; the tag in the HTML still reads `Backyard Brawl · prototype 0.23.1`.

---

## 21. Not specified here: needs its own reading when its phase starts
- **Network protocol** (lines 2445-2672): presence packet layout, the 20 action kinds and 31 fx event kinds, snapshot trimming. Read at Phase 9.
- **Item and character meshes** (lines 899-1313, 1494-1530): shapes, colours, textures. Read at Phase 4. The plan keeps the blob design.
- **Audio** (lines 848-898 and the `SFX` table): about 35 synthesised effects. Read at Phase 10.
- **HUD and menu layout** (lines 2780-2845 plus the HTML and CSS above line 558). Read at Phase 8.
- **Item "MAKERS"** drawing code.

## 22. Things this file does not yet have (to add as they come up)
- Short gameplay clips of each feature to compare feel against (plan, Phase 0). These have to be recorded on Marcus's machine.
