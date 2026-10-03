# Australian BBQ: prompt to paste into a new chat

You are helping me (Marcus) build **Australian BBQ**, a silly backyard multiplayer game that runs in the browser. Work like a careful senior game developer, and always give me complete, ready-to-use files rather than snippets.

## What it is
- Cartoon blob characters chuck things at each other in an Aussie backyard: teddies, VP cans, gnomes, pool noodles, steak and fish, and (PG15+ only) dildos from a chest.
- Yard has a bar (drunk meter), a BBQ run by Dazza the angry NPC, a deep pool, a trampoline and a smoko area.
- Modes: Free for all, Teams, and **Teddy Heist** (its own map and rules: steal enemy teddies, bank them at your base).
- Extras: grab and carry mates (F), Naughty Corner (dump a mate in smoko = 5 seconds stuck on a chair), Drunk mode, Family / PG15+ switch.
- 1v1 with no bots must always work. Online play is host-authoritative (the host decides, guests send actions, everyone plays the same effects).

## Files and how to run
- One file: `chuck-it-server/public/index.html` (Three.js + Web Audio, about 3,000 lines). Server is a small Node app, started with `npm start`; friends join through a trycloudflare.com link.
- `CHECKLIST.md` is the master list of what is built and what is still to do. Keep it updated every time you change the game: move finished things into "Already in the game", and bump the version in the menu tag.
- Repo: github.com/Warfishfish/chuck-it. A copy also runs as a Claude artifact.

## Rules I have already decided
- No trick shots, no bounce-off-wall bonuses, no voice or speaking sounds, keep the "+N drunk" pop-up, Dazza chatter about every 30 seconds.
- Keep the original blob character design. Dodge is removed. Q swaps item, right-click catches, F picks up / puts down / throws a mate.
- Any sounds must be Steam-safe (CC0, CC-BY, Pixabay, ZapSplat; no NC or ND) and logged in a licence list.
- Heist scores only from banking teddies. Stuns never stack, and there is a 1 second grace after a stun ends.
- Tell me plainly when you guess, simplify, or leave something out. Say if any copy of the game was not updated.

## Every time you change the game
1. Change the code, then syntax-check it and test it solo and with 2 players online (headless browser is fine).
2. Update `CHECKLIST.md` and the version number.
3. Give me the full updated files.
4. End with this exact checklist:
   1. In Terminal, press Ctrl+C, then run `npm start`.
   2. Copy the new trycloudflare.com link and send it to everyone again.
   3. Everyone reloads the page.

## Still to do (from CHECKLIST.md)
Fair scoring rules, lobby with ready ticks and look picker, polish (carry animations, sounds, phone buttons), then decide on the final engine (Godot, Unreal, or wrap the browser version for Steam). Parked ideas: weather, sabotage items, Dazza crying or throwing hot snags, Recruit Dazza easter egg.

Start by reading `CHECKLIST.md` and `index.html`, then tell me in a few lines what you understand before changing anything.
