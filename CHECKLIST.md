# Chuck It / Backyard Brawl: gameplay checklist

✅ done · ⬜ to do · 🔒 checkpoint

🔒 **Checkpoint 0.11.1:** `checkpoints/chuck-it-0.11.1-checkpoint.html` (GitHub + `CHUCK IT/checkpoints` on the Mac)

---

## Already in the game ✅

**Moving**
- ✅ WASD, Space jump, Shift sprint boost, C dodge, right-click catch
- ✅ 1.5x bigger yard with cover, pool (deep), trampoline

**Chucking and slapping**
- ✅ Teddy, VP can, gnome, and floating pool noodles
- ✅ Dildo chest (random spot): slaps knock people flat with Helicopter / Cartwheel / Timber
- ✅ Steak, fish, noodle slaps stun
- ✅ Slap the chef: KO / Berserk / Flip the grill

**Drinking**
- ✅ Beer, wine, rum at the bar; drunk meter, wobbly vision, drifting
- ✅ Everyone sees you sway; random drunk falls; mates help you up

**Yard life**
- ✅ Dazza at the BBQ yells, chases and spatulas you
- ✅ Smoko chairs: sit, beer, safe
- ✅ Emotes and taunts

**Scoring and matches**
- ✅ Hits, streaks, leader bounty, drunk bonus, pool/tramp bonus
- ✅ Teams (Red / Blue / Wildcard), friendly fire option
- ✅ Best-of-3 matches; host picks what's in the yard

---

## 1. Grab and carry your mates ⬜ (next up)
> Knock someone down, then pick them up and haul them somewhere embarrassing.

- ⬜ You can only grab someone who's **down, fallen, stunned or passed out**, so every slap sets up a carry
- ⬜ **V** near them: grab. **V** again: drop. **Hold V**: chuck them
- ⬜ Carried player hangs over your shoulder, flopping and bouncing with every step
- ⬜ Carrying slows you down, and you can't throw or sprint
- ⬜ Carried player **mashes Space** to wriggle free (max carry ~6s)
- ⬜ Chucked into the pool or onto the tramp scores the existing bonus
- ⬜ You drop them if you get hit, slapped, fall over, or walk into the pool
- ⬜ Can't be re-grabbed for ~8s after getting free
- ⬜ Bots can be carried (they won't grab you yet)
- ⬜ Works online the same for host and guests, with no one getting stuck if someone leaves mid-carry
- ⬜ **Playtest:** is it funny? Is escaping too easy or too hard? Is 6s the right carry time?

## 2. Naughty Corner ⬜
> Dump a mate in the smoko area and they get punished.

- ⬜ Host setting: smoko is **Safe Zone** (like now) or **Naughty Corner** (added to the existing "What's in the yard" menu)
- ⬜ Drop or chuck a carried player into smoko: **+100** and a "SENT TO THE NAUGHTY CORNER!" banner
- ⬜ Victim sits on the naughty chair for ~5s and can't move
- ⬜ Then they're **Naughty** for ~20s: aim sways, wind-up is slower, and throws are weaker
- ⬜ Everyone can see a dunce cap on the Naughty player
- ⬜ No one can carry them while they're being punished
- ⬜ In Naughty Corner mode, you can't sit at smoko for a safe break
- ⬜ **Playtest:** is the punishment annoying enough to fear, but not so bad it's no fun?

## 3. Fair scoring ⬜
> Stop people farming the same mate for points.

- ⬜ No points for dumping or carrying your own teammate
- ⬜ The same victim can't be worth Naughty Corner points again for ~45s
- ⬜ The same attacker and victim pair has a longer cooldown (~90s)
- ⬜ No points for slapping someone who's already down
- ⬜ 3s protection at the start of each round
- ⬜ No points in warm-up or after the round ends
- ⬜ **Playtest:** try to farm points two ways; neither should work

## 4. Lobby and game setup ⬜
> Before the match: hang out, get ready, pick your look.

- ⬜ Waiting yard before the match, with free play and no scoring
- ⬜ Player list with ready ticks
- ⬜ Ready button, with a "3 / 4 ready" count
- ⬜ Host starts when everyone's ready, or can force start
- ⬜ Late joiners jump in next round
- ⬜ Pick your look: colour, one accessory (cap / bucket hat / sunnies / headband), optional shirt pattern (original blob style)
- ⬜ All host settings in one panel (rounds, teams, friendly fire, yard features, smoko mode)
- ⬜ Rules on their own screen, in short pages, only showing what's switched on

## 5. Polish ⬜
> Juice once the gameplay feels right.

- ⬜ Carry animations: grab lunge, wriggle, drop flop
- ⬜ Naughty Corner: dunce cap wobble, sulking pose
- ⬜ Sounds: grab grunt, "OI!", naughty jingle
- ⬜ Marcus's custom sounds for Dazza and other effects
- ⬜ Phone buttons for Grab and Ready

## 6. Engine and Steam ⬜ (later)
> Finish the gameplay here in the browser first (quick to change), then decide on the engine.

- ⬜ Keep building and tuning gameplay in the browser version until it feels right
- ⬜ Then compare engines:
  - ⬜ **Godot 4:** free engine, lighter, similar cartoony look with better lighting
  - ⬜ **Unreal Engine 5:** best looks and real ragdoll physics; a full rebuild using this game as the blueprint
  - ⬜ Option: wrap the current browser version as a desktop app for Steam (keeps today's graphics)
- ⬜ Build a small test scene (yard, one character, a floppy dildo) in the chosen engine before committing
- ⬜ Only use Steam-safe sounds and assets (CC0, CC-BY, Pixabay, ZapSplat; no NC or ND), and keep a licence list

## Ideas parked
- ⬜ Weather or time of day
- ⬜ Sabotage items
- ⬜ Dazza crying or throwing hot snags
