# Chuck It / Backyard Brawl: phase checklist

✅ = done · ⬜ = to do · 🔒 = checkpoint

🔒 **Checkpoint 0.11.1:** `checkpoints/chuck-it-0.11.1-checkpoint.html` (in GitHub and in `CHUCK IT/checkpoints` on the Mac). If a new phase goes wrong, we roll back to this.

---

## Step 1: Bigger yard and the bar ✅
- ✅ Backyard 1.5x bigger, with extra cover
- ✅ Bar table with beer, wine and rum
- ✅ VB renamed to VP: green cans with a VP label
- ✅ Space = jump, Shift = temporary sprint boost, C = dodge
- ✅ Hit streak bonus

## Step 2: Drinking and drunk effects ✅
- ✅ Drunk meter and sobering up over time
- ✅ Wavy vision (light blur) and screen tilting both ways
- ✅ Camera sway and drifting when you walk
- ✅ Visible swaying and crooked walking that other players can see
- ✅ Drunk bonus points for hitting people while drunk

## Step 3: Teams, chest and slaps ✅
- ✅ Teams: Red / Blue / Wildcard (odd player out), with team scores that add up
- ✅ Host option for friendly fire
- ✅ Teams in local games, and moving players between teams by hand
- ✅ Dildo chest at a random spot each game
- ✅ Dildo slap with three knockdown animations: Helicopter, Cartwheel, Timber

## Step 4: Falling over and helping up ✅
- ✅ Random drunk falls (rarer, and at unexpected moments)
- ✅ Teammate help-up (+25), auto get-up, 25s fall immunity
- ✅ "Go help them" banner across the top, with a few variations
- ✅ BBQ with Dazza: he yells gibberish, chases you, and slaps you with the spatula
- ✅ Meat table (take from any side): steak/fish slap = 3s stun, +50

## Step 5: Smoko and extras ✅
- ✅ Smoko area with a chair for every player and a beer in hand; safe while seated
- ✅ Pool and trampoline knock bonus
- ✅ Taunts and emotes (T / G / B)
- ✅ Jump and Boost buttons for phones
- ✅ Best-of-3 matches (host or local chooses the length)
- ✅ Slapping the chef: KO / Berserk / Flip-the-grill outcomes
- ✅ Bigger fish; long, wobbly dildos
- ✅ Shorter menu rules
- ✅ Host picks what's in the yard (bar, BBQ, chest, smoko, falls)
- ✅ 0.11: floppy dildo, slap arc other players can see, deep pool, pool noodles (tap = slap, hold = throw)
- ✅ 0.11.1: characters kept as the original design

---

## Step 6: Pre-game lobby ⬜ (next up; everything after this plugs into it)
*Build on what's already there: the host/join code, warm-up yard, Start button and yard-feature menu.*

**Waiting area**
- ⬜ Players who join land in the yard in warm-up (free play, no scoring, can't be carried to smoko)
- ⬜ Lobby panel over the yard, toggled with Tab or a button

**Player list**
- ⬜ Shows name, colour, team, ready tick and a host crown
- ⬜ Updates live as players join and leave
- ⬜ Host can kick a player

**Ready system**
- ⬜ Ready button (plus a key); ready state sent over the network
- ⬜ "3 / 4 ready" counter
- ⬜ Changing your character un-readies you

**Host start control**
- ⬜ Start button lights up when everyone is ready
- ⬜ Host can force start (with a confirm) if someone is AFK
- ⬜ Only the host can start; guests see "Waiting for host…"
- ⬜ Late joiners wait in the yard and join the next round

**Host settings (one tidy panel)**
- ⬜ Move the existing settings into it: rounds, round length, teams, friendly fire, yard features
- ⬜ Add a "Smoko mode" slot, ready for Step 8
- ⬜ Guests can see the settings but not change them

**Character customisation (keep the original blob look)**
- ⬜ Pick your colour (no two players the same)
- ⬜ Pick one accessory: none / cap / bucket hat / sunnies / headband
- ⬜ Optional shirt pattern: plain (default) or the saved patterns (footy hoops, flannel, hi-vis, Hawaiian)
- ⬜ Choices sent to everyone, and remembered for next time on that device

**Rules screen**
- ⬜ Its own screen, opened from the lobby (not crammed into the menu)
- ⬜ Short pages: Basics · Items · Yard stuff · Smoko & carrying
- ⬜ Only lists the features the host has turned on
- ⬜ Shown once to first-timers, with a "Got it" button

- ⬜ **Test:** 2–4 players join, ready up, customise, host starts; a late joiner; host force start

## Step 7: Pick up and carry players ⬜
*The rule that keeps it fun rather than griefy: you can only grab someone who's down, fallen, stunned (meat, noodle, spatula) or passed-out drunk. Every slap and stun becomes a set-up.*

- ⬜ **Detect nearby player:** someone grabbable within about 1.8 m in front shows a "V: grab" prompt
- ⬜ **Grab player:** V picks them up (host checks the distance, their state, that nobody else has them, and that neither of you is seated at smoko)
- ⬜ **Carry player:** over-the-shoulder carry; the carrier moves at ~70% speed, can't throw or sprint, and jumps lower
- ⬜ **Break free:** the carried player mashes Space to fill an escape bar; there's also a max carry time (~6s)
- ⬜ **Drop player:** V again drops them in front of you; they land stunned for a moment
- ⬜ **Chuck player (bonus):** hold V to throw them a short distance; into the pool or tramp counts for the existing pool/tramp bonus
- ⬜ **Floppy behaviour:** the carried blob dangles and bounces with every step and flops on landing (original character style)
- ⬜ **Auto-drops:** carrier gets hit or slapped, falls over, round ends, or the carrier goes into the pool
- ⬜ **Grab immunity:** can't be re-grabbed for ~8s after being dropped or escaping
- ⬜ **Bots:** they can be carried; bots don't grab players (for now)
- ⬜ **Multiplayer:** host decides every grab and drop; the carried player's own screen follows the carrier (camera stays on them)
- ⬜ **Multiplayer:** two people grab the same player at once → first request wins
- ⬜ **Multiplayer:** carrier or carried player disconnects → clean drop, nobody stuck
- ⬜ **Multiplayer:** carry state included in the world snapshot, so late joiners see it correctly
- ⬜ **Test pickup system:** solo with bots, then 2-player online (host carries guest, guest carries host, escape, disconnect mid-carry)

## Step 8: Smoko modes, Naughty Corner and fair scoring ⬜
- ⬜ **Smoko mode setting** in host settings: Safe Zone / Naughty Area (shown in the rules and the lobby)
- ⬜ **Safe Zone mode:** exactly how smoko works now (sit, beer, can't be hit)
- ⬜ **Naughty Area mode:** a "NAUGHTY CORNER" sign; you can't sit down for a break in this mode
- ⬜ **Detect player being carried into smoko:** a player dropped (or chucked) inside the smoko ring counts as "sent to the corner"
- ⬜ **Track who carried them:** store the last carrier, who gets the credit
- ⬜ **Award points:** +100 to the carrier, plus a big "SENT TO THE NAUGHTY CORNER!" banner for everyone
- ⬜ **Prevent point farming:**
  - ⬜ No points for sending your own teammate
  - ⬜ The same victim can't earn anyone points again for ~45s
  - ⬜ The same carrier/victim pair gets a longer cooldown (~90s)
  - ⬜ No points during warm-up or after the round ends
  - ⬜ Grab immunity after release (from Step 7) stops instant re-grabs
- ⬜ **Add punishment:** the victim sits on the naughty chair for ~5s (can't move), then has a "Naughty" debuff for ~20s
- ⬜ **Make punishment affect throwing:** aim sways, wind-up is 1.5x slower, and throws are ~30% weaker while Naughty
- ⬜ **Everyone can see it:** a dunce cap or "NAUGHTY" tag above the victim's head, with a countdown on the victim's screen
- ⬜ Victim is immune to being carried while punished (no chain-bullying)
- ⬜ **General scoring protection:**
  - ⬜ Slapping a player who's already down gives no points
  - ⬜ 3s spawn protection after each round start
  - ⬜ Help-up points only when the player really fell (already mostly done; double-check)
- ⬜ **Test:** both modes, solo and 2-player; farming attempts (same victim, teammate, warm-up) give nothing

## Step 9: Polish (after the main systems work) ⬜
- ⬜ Carry animations: grab lunge, struggle wriggle, drop flop
- ⬜ Naughty corner animations: dunce cap wobble, sulking pose
- ⬜ Sounds: grab grunt, struggle, "OI!", naughty-corner jingle
- ⬜ Custom sounds from Marcus: Dazza yelling and other SFX (swap in when ready)
- ⬜ UI feedback: grab prompt, escape bar, naughty timer, lobby ready tick animations
- ⬜ Phone buttons for Grab / Ready

## Ideas parked (not scheduled)
- ⬜ Weather or time of day
- ⬜ Sabotage items
- ⬜ Dazza crying or throwing hot snags
